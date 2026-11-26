use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::Notify;

use crate::adapters::registry::AdapterRegistry;
use crate::domain::console_profile::{ConsoleSyncReport, SyncTrigger};
use crate::domain::environment::{
    RuntimeVerificationFeature, RuntimeVerificationRecord, RuntimeVerificationStatus,
};
use crate::domain::error::DomainError;
use crate::domain::player::now_utc_rfc3339;
use crate::domain::session::{
    compute_elapsed_seconds_utc, ActiveSessionPhase, ActiveSessionSnapshot, LaunchTarget,
    EXIT_STATUS_CLEAN, EXIT_STATUS_NON_ZERO, EXIT_STATUS_USER_STOPPED,
};
use crate::domain::trophy::{NewlyUnlockedTrophy, TrophySyncReport};
use crate::infrastructure::db::console_env_repo::{ConsoleEnvRepository, GameDetailsRepository};
use crate::infrastructure::db::console_profile_repo::ConsoleProfileRepository;
use crate::infrastructure::db::manager::DatabaseManager;
use crate::infrastructure::db::repositories::SqliteRepositories;
use crate::infrastructure::system::live_runner::{LiveEmulatorRunner, LiveProcessStopHandle};
use crate::infrastructure::system::window_manager::{NoopWindowHook, WindowLifecycleHook};

use super::console_profile_sync_service::ConsoleProfileSyncService;
use super::session_service::SessionService;
use super::trophy_sync_service::TrophySyncService;

pub const DEFAULT_HEARTBEAT_INTERVAL_SECS: u64 = 60;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionExitedPayload {
    pub session_id: Option<String>,
    pub console_slug: String,
    pub game_id: Option<String>,
    pub game_title: Option<String>,
    pub emulator_slug: String,
    pub exit_code: Option<i32>,
    pub exit_status: String,
    pub duration_seconds: i64,
    pub diagnostic: String,
    pub ended_at: String,
}

pub trait SessionEventSink: Send + Sync {
    fn on_session_started(&self, snapshot: &ActiveSessionSnapshot);
    fn on_session_heartbeat(&self, snapshot: &ActiveSessionSnapshot);
    fn on_session_exited(&self, payload: &SessionExitedPayload);
    fn on_profile_synced(&self, report: &ConsoleSyncReport);
    fn on_trophies_synced(&self, _report: &TrophySyncReport) {}
    fn on_trophies_unlocked(&self, _unlocked: &[NewlyUnlockedTrophy]) {}
}

#[derive(Debug, Default, Clone)]
pub struct NoopSessionEventSink;

impl SessionEventSink for NoopSessionEventSink {
    fn on_session_started(&self, _snapshot: &ActiveSessionSnapshot) {}
    fn on_session_heartbeat(&self, _snapshot: &ActiveSessionSnapshot) {}
    fn on_session_exited(&self, _payload: &SessionExitedPayload) {}
    fn on_profile_synced(&self, _report: &ConsoleSyncReport) {}
}

#[derive(Clone)]
pub struct TauriSessionEventSink {
    app_handle: AppHandle,
}

impl TauriSessionEventSink {
    pub fn new(app_handle: AppHandle) -> Self {
        Self { app_handle }
    }
}

impl SessionEventSink for TauriSessionEventSink {
    fn on_session_started(&self, snapshot: &ActiveSessionSnapshot) {
        let _ = self.app_handle.emit("session:started", snapshot);
    }

    fn on_session_heartbeat(&self, snapshot: &ActiveSessionSnapshot) {
        let _ = self.app_handle.emit("session:heartbeat", snapshot);
    }

    fn on_session_exited(&self, payload: &SessionExitedPayload) {
        let _ = self.app_handle.emit("session:exited", payload);
    }

    fn on_profile_synced(&self, report: &ConsoleSyncReport) {
        let _ = self.app_handle.emit("profile:synced", report);
    }

    fn on_trophies_synced(&self, report: &TrophySyncReport) {
        let _ = self.app_handle.emit("trophies:synced", report);
    }

    fn on_trophies_unlocked(&self, unlocked: &[NewlyUnlockedTrophy]) {
        if !unlocked.is_empty() {
            let _ = self.app_handle.emit("trophies:unlocked", unlocked);
        }
    }
}

struct LiveSessionRuntime {
    snapshot: ActiveSessionSnapshot,
    stop_handle: Option<LiveProcessStopHandle>,
}

struct RuntimeHooks {
    window_hook: Arc<dyn WindowLifecycleHook>,
    event_sink: Arc<dyn SessionEventSink>,
}

#[derive(Clone)]
pub struct LiveSessionOrchestrator {
    repos: SqliteRepositories,
    env_repo: ConsoleEnvRepository,
    details_repo: GameDetailsRepository,
    profile_repo: ConsoleProfileRepository,
    session_service: SessionService,
    profile_sync: ConsoleProfileSyncService,
    trophy_sync: TrophySyncService,
    adapters: AdapterRegistry,
    heartbeat_interval: Duration,
    runtime: Arc<Mutex<LiveSessionRuntime>>,
    hooks: Arc<Mutex<RuntimeHooks>>,
    idle_notify: Arc<Notify>,
}

impl LiveSessionOrchestrator {
    pub fn new(db: DatabaseManager, adapters: AdapterRegistry, cover_cache_dir: PathBuf) -> Self {
        Self::with_hooks(
            db,
            adapters,
            cover_cache_dir,
            Duration::from_secs(DEFAULT_HEARTBEAT_INTERVAL_SECS),
            Arc::new(NoopWindowHook),
            Arc::new(NoopSessionEventSink),
        )
    }

    pub fn with_hooks(
        db: DatabaseManager,
        adapters: AdapterRegistry,
        cover_cache_dir: PathBuf,
        heartbeat_interval: Duration,
        window_hook: Arc<dyn WindowLifecycleHook>,
        event_sink: Arc<dyn SessionEventSink>,
    ) -> Self {
        Self {
            repos: SqliteRepositories::new(db.clone()),
            env_repo: ConsoleEnvRepository::new(db.clone()),
            details_repo: GameDetailsRepository::new(db.clone()),
            profile_repo: ConsoleProfileRepository::new(db.clone()),
            session_service: SessionService::new(db.clone()),
            profile_sync: ConsoleProfileSyncService::new(db.clone(), cover_cache_dir.clone()),
            trophy_sync: TrophySyncService::new(db, cover_cache_dir),
            adapters,
            heartbeat_interval,
            runtime: Arc::new(Mutex::new(LiveSessionRuntime {
                snapshot: ActiveSessionSnapshot::idle(),
                stop_handle: None,
            })),
            hooks: Arc::new(Mutex::new(RuntimeHooks {
                window_hook,
                event_sink,
            })),
            idle_notify: Arc::new(Notify::new()),
        }
    }

    /// Attaches live desktop hooks (TauriWindowManager + TauriSessionEventSink) in `lib.rs` `setup()`.
    pub fn attach_runtime_hooks(
        &self,
        window_hook: Arc<dyn WindowLifecycleHook>,
        event_sink: Arc<dyn SessionEventSink>,
    ) {
        if let Ok(mut guard) = self.hooks.lock() {
            guard.window_hook = window_hook;
            guard.event_sink = event_sink;
        }
    }

    fn current_hooks(&self) -> (Arc<dyn WindowLifecycleHook>, Arc<dyn SessionEventSink>) {
        if let Ok(guard) = self.hooks.lock() {
            (
                Arc::clone(&guard.window_hook),
                Arc::clone(&guard.event_sink),
            )
        } else {
            (Arc::new(NoopWindowHook), Arc::new(NoopSessionEventSink))
        }
    }

    pub fn get_active_session(&self) -> ActiveSessionSnapshot {
        let mut snap = self
            .runtime
            .lock()
            .map(|r| r.snapshot.clone())
            .unwrap_or_else(|_| ActiveSessionSnapshot::idle());
        if snap.phase == ActiveSessionPhase::Running {
            if let Some(started) = snap.started_at.as_deref() {
                let now = now_utc_rfc3339();
                if let Ok(elapsed) = compute_elapsed_seconds_utc(started, &now) {
                    snap.elapsed_seconds = elapsed;
                }
            }
        }
        snap
    }

    pub fn stop_active_session(&self) -> Result<bool, DomainError> {
        let handle = self
            .runtime
            .lock()
            .map_err(|_| DomainError::Process("session runtime mutex poisoned".to_string()))?
            .stop_handle
            .clone();
        Ok(handle.is_some_and(|h| h.request_stop(Duration::from_secs(3))))
    }

    pub async fn wait_until_idle(&self, timeout: Duration) -> Result<(), DomainError> {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if self.get_active_session().phase == ActiveSessionPhase::Idle {
                return Ok(());
            }
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Err(DomainError::Process(
                    "timed out waiting for live session to return to Idle".to_string(),
                ));
            }
            let _ = tokio::time::timeout(remaining, self.idle_notify.notified()).await;
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_runtime_verification(
        &self,
        console_slug: &str,
        emulator_slug: &str,
        game_id: Option<&str>,
        feature_key: RuntimeVerificationFeature,
        emulator_version: &str,
        status: RuntimeVerificationStatus,
        evidence_notes: &str,
    ) -> Result<RuntimeVerificationRecord, DomainError> {
        self.env_repo.upsert_runtime_verification(
            console_slug,
            emulator_slug,
            game_id,
            feature_key,
            emulator_version,
            status,
            evidence_notes,
        )
    }

    fn acquire_launching_slot(&self, target: LaunchTarget) -> Result<(), DomainError> {
        let mut guard = self
            .runtime
            .lock()
            .map_err(|_| DomainError::Process("session runtime mutex poisoned".to_string()))?;
        if !guard
            .snapshot
            .phase
            .can_transition_to(ActiveSessionPhase::Launching)
        {
            return Err(DomainError::Conflict(format!(
                "já existe uma sessão ativa em andamento ({:?}); encerre-a antes de iniciar outra",
                guard.snapshot.phase
            )));
        }
        guard.snapshot = ActiveSessionSnapshot {
            phase: ActiveSessionPhase::Launching,
            target: Some(target),
            ..ActiveSessionSnapshot::idle()
        };
        guard.stop_handle = None;
        Ok(())
    }

    fn reset_to_idle(&self) {
        if let Ok(mut guard) = self.runtime.lock() {
            guard.snapshot = ActiveSessionSnapshot::idle();
            guard.stop_handle = None;
        }
        self.idle_notify.notify_waiters();
    }

    fn resolve_active_local_user_id(&self, console_slug: &str) -> Option<String> {
        let users = self.profile_repo.list_local_users(console_slug).ok()?;
        let available: Vec<_> = users
            .into_iter()
            .filter(|u| u.availability_status == "available")
            .collect();
        if available.len() == 1 {
            return Some(available[0].id.clone());
        }
        if let Ok(active_profile) = self.repos.get_active_profile() {
            if let Some(linked) = available
                .into_iter()
                .find(|u| u.linked_profile_id.as_deref() == Some(active_profile.id.as_str()))
            {
                return Some(linked.id);
            }
        }
        None
    }

    pub async fn launch_console_system(
        &self,
        console_slug: &str,
    ) -> Result<ActiveSessionSnapshot, DomainError> {
        let target = LaunchTarget::SystemBoot {
            console_slug: console_slug.to_string(),
        };
        self.acquire_launching_slot(target.clone())?;

        let launch_result = self.prepare_and_spawn_system(console_slug, target);
        match launch_result {
            Ok(snapshot) => Ok(snapshot),
            Err(err) => {
                self.reset_to_idle();
                Err(err)
            }
        }
    }

    fn prepare_and_spawn_system(
        &self,
        console_slug: &str,
        target: LaunchTarget,
    ) -> Result<ActiveSessionSnapshot, DomainError> {
        let _ = self
            .profile_sync
            .sync_console_profile(console_slug, SyncTrigger::PreLaunch);

        let emu = self
            .repos
            .list_emulators()?
            .into_iter()
            .find(|e| e.console_slug == console_slug)
            .ok_or_else(|| {
                DomainError::NotFound(format!(
                    "nenhum emulador registrado para o console '{console_slug}'"
                ))
            })?;

        let adapter = self.adapters.get(&emu.slug).ok_or_else(|| {
            DomainError::Validation(format!(
                "execução ao vivo para '{}' ({}) ainda não está implementada (planejada para a Fase 5)",
                emu.name, console_slug
            ))
        })?;

        let exe_path = emu.executable_path.as_deref().ok_or_else(|| {
            DomainError::Validation(format!(
                "configure o executável de '{}' antes de iniciar o sistema original",
                emu.name
            ))
        })?;

        let install = adapter.detect_installation(Some(Path::new(exe_path)))?;
        let emu_version = install
            .detected_version
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let cmd = adapter.build_system_boot_command(&install)?;

        let (window_hook, event_sink) = self.current_hooks();
        let _ = window_hook.on_emulator_launched();

        let handle = match LiveEmulatorRunner::spawn(&cmd) {
            Ok(h) => h,
            Err(err) => {
                let _ = window_hook.on_emulator_exited();
                return Err(err);
            }
        };

        let pid = handle.pid();
        let stop_handle = handle.stop_handle();
        let started_at = now_utc_rfc3339();

        // Record independent stage: OS process spawned (`emulator_launch`), NOT `original_dashboard`!
        let _ = self.env_repo.upsert_runtime_verification(
            console_slug,
            &emu.slug,
            None,
            RuntimeVerificationFeature::EmulatorLaunch,
            &emu_version,
            RuntimeVerificationStatus::RuntimeConfirmed,
            &format!(
                "Processo {} iniciado diretamente sem shell (PID {:?}).",
                emu.name, pid
            ),
        );

        let snapshot = ActiveSessionSnapshot {
            phase: ActiveSessionPhase::Running,
            session_id: None,
            target: Some(target),
            console_slug: Some(console_slug.to_string()),
            game_id: None,
            game_title: Some("Sistema Original do Console (XMB)".to_string()),
            emulator_slug: Some(emu.slug.clone()),
            pid,
            started_at: Some(started_at.clone()),
            last_heartbeat_at: Some(started_at.clone()),
            elapsed_seconds: 0,
        };

        {
            let mut guard = self
                .runtime
                .lock()
                .map_err(|_| DomainError::Process("session runtime mutex poisoned".to_string()))?;
            guard.snapshot = snapshot.clone();
            guard.stop_handle = Some(stop_handle);
        }

        event_sink.on_session_started(&snapshot);

        // Spawn background Tokio watcher
        let orchestrator = self.clone();
        let console_slug_owned = console_slug.to_string();
        let emu_slug_owned = emu.slug.clone();
        tokio::spawn(async move {
            orchestrator
                .monitor_live_process(
                    handle,
                    None,
                    console_slug_owned,
                    None,
                    Some("Sistema Original do Console (XMB)".to_string()),
                    emu_slug_owned,
                    emu_version,
                    started_at,
                    false,
                )
                .await;
        });

        Ok(snapshot)
    }

    pub async fn launch_game(
        &self,
        game_id: &str,
        game_version_id: Option<&str>,
    ) -> Result<ActiveSessionSnapshot, DomainError> {
        let active_profile = self.repos.get_active_profile()?;
        let game = self
            .details_repo
            .get_game_record(&active_profile.id, game_id, None)?;
        let target = LaunchTarget::DirectGameBoot {
            console_slug: game.console_slug.clone(),
            game_id: game.id.clone(),
            game_version_id: game_version_id.map(str::to_string),
        };
        self.acquire_launching_slot(target.clone())?;

        match self.prepare_and_spawn_game(
            &game.console_slug,
            &game.id,
            &game.title,
            game_version_id,
            target,
        ) {
            Ok(snapshot) => Ok(snapshot),
            Err(err) => {
                self.reset_to_idle();
                Err(err)
            }
        }
    }

    fn prepare_and_spawn_game(
        &self,
        console_slug: &str,
        game_id: &str,
        game_title: &str,
        requested_version_id: Option<&str>,
        target: LaunchTarget,
    ) -> Result<ActiveSessionSnapshot, DomainError> {
        let versions = self.details_repo.list_game_versions(game_id)?;
        let selected_version = if let Some(vid) = requested_version_id {
            versions.into_iter().find(|v| v.id == vid).ok_or_else(|| {
                DomainError::NotFound(format!("versão '{vid}' não encontrada para '{game_title}'"))
            })?
        } else {
            versions
                .into_iter()
                .find(|v| Path::new(&v.file_path).is_file())
                .ok_or_else(|| {
                    DomainError::NotFound(format!(
                        "nenhum arquivo disponível no disco para '{game_title}'"
                    ))
                })?
        };

        let target_file = Path::new(&selected_version.file_path);
        if !target_file.is_file() {
            return Err(DomainError::NotFound(format!(
                "arquivo do jogo ausente no disco: '{}'",
                selected_version.file_path
            )));
        }

        let emu = self
            .repos
            .list_emulators()?
            .into_iter()
            .find(|e| e.console_slug == console_slug)
            .ok_or_else(|| {
                DomainError::NotFound(format!(
                    "nenhum emulador registrado para o console '{console_slug}'"
                ))
            })?;

        let adapter = self.adapters.get(&emu.slug).ok_or_else(|| {
            DomainError::Validation(format!(
                "execução ao vivo para '{}' ({}) ainda não está implementada (planejada para a Fase 5)",
                emu.name, console_slug
            ))
        })?;

        let exe_path = emu.executable_path.as_deref().ok_or_else(|| {
            DomainError::Validation(format!(
                "configure o executável de '{}' antes de iniciar '{}'",
                emu.name, game_title
            ))
        })?;

        let install = adapter.detect_installation(Some(Path::new(exe_path)))?;
        let emu_version = install
            .detected_version
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        let cmd = adapter.build_game_boot_command(&install, target_file)?;

        let local_user_id = self.resolve_active_local_user_id(console_slug);
        let started_at = now_utc_rfc3339();
        let session = self.session_service.start_recorded_session_for_local_user(
            game_id,
            Some(&selected_version.id),
            &emu.id,
            local_user_id.as_deref(),
            Some(&started_at),
        )?;

        let (window_hook, event_sink) = self.current_hooks();
        let _ = window_hook.on_emulator_launched();

        let handle = match LiveEmulatorRunner::spawn(&cmd) {
            Ok(h) => h,
            Err(err) => {
                let _ = window_hook.on_emulator_exited();
                let _ = self
                    .session_service
                    .finish_session(&session.id, None, "launch_failed");
                return Err(err);
            }
        };

        let pid = handle.pid();
        let stop_handle = handle.stop_handle();

        // Record independent runtime stages: emulator_launch + game_boot ONLY (never original_dashboard!)
        let _ = self.env_repo.upsert_runtime_verification(
            console_slug,
            &emu.slug,
            None,
            RuntimeVerificationFeature::EmulatorLaunch,
            &emu_version,
            RuntimeVerificationStatus::RuntimeConfirmed,
            &format!(
                "Processo {} iniciado diretamente sem shell (PID {:?}).",
                emu.name, pid
            ),
        );
        let _ = self.env_repo.upsert_runtime_verification(
            console_slug,
            &emu.slug,
            None,
            RuntimeVerificationFeature::GameBoot,
            &emu_version,
            RuntimeVerificationStatus::RuntimeConfirmed,
            &format!(
                "Inicialização direta de '{}' (PID {:?}) iniciada sem shell.",
                game_title, pid
            ),
        );

        let snapshot = ActiveSessionSnapshot {
            phase: ActiveSessionPhase::Running,
            session_id: Some(session.id.clone()),
            target: Some(target),
            console_slug: Some(console_slug.to_string()),
            game_id: Some(game_id.to_string()),
            game_title: Some(game_title.to_string()),
            emulator_slug: Some(emu.slug.clone()),
            pid,
            started_at: Some(started_at.clone()),
            last_heartbeat_at: Some(started_at.clone()),
            elapsed_seconds: 0,
        };

        {
            let mut guard = self
                .runtime
                .lock()
                .map_err(|_| DomainError::Process("session runtime mutex poisoned".to_string()))?;
            guard.snapshot = snapshot.clone();
            guard.stop_handle = Some(stop_handle);
        }

        event_sink.on_session_started(&snapshot);

        let orchestrator = self.clone();
        let console_slug_owned = console_slug.to_string();
        let game_id_owned = game_id.to_string();
        let game_title_owned = game_title.to_string();
        let emu_slug_owned = emu.slug.clone();
        tokio::spawn(async move {
            orchestrator
                .monitor_live_process(
                    handle,
                    Some(session.id),
                    console_slug_owned,
                    Some(game_id_owned),
                    Some(game_title_owned),
                    emu_slug_owned,
                    emu_version,
                    started_at,
                    true,
                )
                .await;
        });

        Ok(snapshot)
    }

    #[allow(clippy::too_many_arguments)]
    async fn monitor_live_process(
        self,
        handle: crate::infrastructure::system::live_runner::LiveProcessHandle,
        session_id: Option<String>,
        console_slug: String,
        game_id: Option<String>,
        game_title: Option<String>,
        emulator_slug: String,
        emulator_version: String,
        started_at: String,
        is_game_session: bool,
    ) {
        let mut wait_fut = Box::pin(handle.wait());
        let mut heartbeat_tick = tokio::time::interval(self.heartbeat_interval);
        // Consume immediate first tick
        heartbeat_tick.tick().await;

        let exit_result = loop {
            tokio::select! {
                res = &mut wait_fut => break res,
                _ = heartbeat_tick.tick() => {
                    let hb_ts = now_utc_rfc3339();
                    if let Some(sid) = session_id.as_deref() {
                        let _ = self.session_service.heartbeat_session(sid, Some(&hb_ts));
                    }
                    let updated_snap = if let Ok(mut guard) = self.runtime.lock() {
                        guard.snapshot.last_heartbeat_at = Some(hb_ts.clone());
                        if let Ok(elapsed) = compute_elapsed_seconds_utc(&started_at, &hb_ts) {
                            guard.snapshot.elapsed_seconds = elapsed;
                        }
                        Some(guard.snapshot.clone())
                    } else {
                        None
                    };
                    if let Some(snap) = updated_snap {
                        let (_, sink) = self.current_hooks();
                        sink.on_session_heartbeat(&snap);
                    }
                }
            }
        };

        // Transition Running -> Exiting
        if let Ok(mut guard) = self.runtime.lock() {
            guard.snapshot.phase = ActiveSessionPhase::Exiting;
            guard.stop_handle = None;
        }

        let ended_at = now_utc_rfc3339();
        let (exit_code, exit_status, diagnostic) = match exit_result {
            Ok(exit) => {
                let status = if exit.stopped_by_user {
                    EXIT_STATUS_USER_STOPPED.to_string()
                } else if exit.exit_code == Some(0) {
                    EXIT_STATUS_CLEAN.to_string()
                } else {
                    EXIT_STATUS_NON_ZERO.to_string()
                };
                let stderr_summary = exit.stderr_tail.join(" | ");
                let diag = if let Some(adapter) = self.adapters.get(&emulator_slug) {
                    adapter
                        .parse_exit_status(exit.exit_code, &stderr_summary)
                        .diagnostic
                } else {
                    format!(
                        "Processo encerrado ({status}, código {:?}).",
                        exit.exit_code
                    )
                };
                (exit.exit_code, status, diag)
            }
            Err(err) => (
                None,
                EXIT_STATUS_NON_ZERO.to_string(),
                format!("Erro ao monitorar processo do emulador: {err}"),
            ),
        };

        let duration_seconds = if let Some(sid) = session_id.as_deref() {
            match self
                .session_service
                .finish_session(sid, Some(&ended_at), &exit_status)
            {
                Ok(rec) => rec.duration_seconds,
                Err(_) => compute_elapsed_seconds_utc(&started_at, &ended_at).unwrap_or(0),
            }
        } else {
            compute_elapsed_seconds_utc(&started_at, &ended_at).unwrap_or(0)
        };

        let (window_hook, event_sink) = self.current_hooks();
        let hub_return_ok = window_hook.on_emulator_exited().is_ok();

        // Record independent monitoring & hub return stages truthfully
        if is_game_session {
            let _ = self.env_repo.upsert_runtime_verification(
                &console_slug,
                &emulator_slug,
                None,
                RuntimeVerificationFeature::SessionMonitoring,
                &emulator_version,
                RuntimeVerificationStatus::RuntimeConfirmed,
                &format!(
                    "Sessão monitorada do início ao fim ({duration_seconds}s, status: {exit_status})."
                ),
            );
        }
        if hub_return_ok {
            let _ = self.env_repo.upsert_runtime_verification(
                &console_slug,
                &emulator_slug,
                None,
                RuntimeVerificationFeature::HubReturn,
                &emulator_version,
                RuntimeVerificationStatus::RuntimeConfirmed,
                "Retorno ao Universal Console Hub concluído após encerramento do processo.",
            );
        }

        // Automatically re-synchronize console profile and local emulator trophies after session end
        let post_sync = self
            .profile_sync
            .sync_console_profile(&console_slug, SyncTrigger::SessionEnded);
        let post_trophies = self.trophy_sync.sync_console_trophies(&console_slug);

        self.reset_to_idle();

        event_sink.on_session_exited(&SessionExitedPayload {
            session_id,
            console_slug,
            game_id,
            game_title,
            emulator_slug,
            exit_code,
            exit_status,
            duration_seconds,
            diagnostic,
            ended_at,
        });

        if let Ok(report) = post_sync {
            event_sink.on_profile_synced(&report);
        }
        if let Ok(trophy_report) = post_trophies {
            event_sink.on_trophies_synced(&trophy_report);
            if !trophy_report.newly_unlocked.is_empty() {
                event_sink.on_trophies_unlocked(&trophy_report.newly_unlocked);
            }
        }
    }
}
