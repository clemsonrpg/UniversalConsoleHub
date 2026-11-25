use std::collections::VecDeque;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;

use crate::domain::emulator::LaunchCommand;
use crate::domain::error::DomainError;

const MAX_TAIL_LINES: usize = 200;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    #[default]
    Idle,
    Launching,
    Running,
    Exiting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitoredProcessOutput {
    pub exit_code: Option<i32>,
    pub stdout_tail: Vec<String>,
    pub stderr_tail: Vec<String>,
    pub timed_out: bool,
}

#[derive(Clone, Default)]
pub struct ProcessManager {
    state: Arc<Mutex<SessionState>>,
}

struct SessionResetGuard {
    state: Arc<Mutex<SessionState>>,
}

impl Drop for SessionResetGuard {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.state.lock() {
            *guard = SessionState::Idle;
        }
    }
}

async fn drain_lines<R: AsyncRead + Unpin>(reader: R) -> Vec<String> {
    let mut buf_reader = BufReader::new(reader);
    let mut ring = VecDeque::with_capacity(MAX_TAIL_LINES);
    let mut raw = Vec::new();
    loop {
        raw.clear();
        match buf_reader.read_until(b'\n', &mut raw).await {
            Ok(0) => break,
            Ok(_) => {
                let line = String::from_utf8_lossy(&raw)
                    .trim_end_matches(['\r', '\n'])
                    .to_string();
                if ring.len() == MAX_TAIL_LINES {
                    ring.pop_front();
                }
                ring.push_back(line);
            }
            Err(_) => break,
        }
    }
    ring.into_iter().collect()
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(SessionState::Idle)),
        }
    }

    pub fn current_state(&self) -> SessionState {
        self.state
            .lock()
            .map(|guard| *guard)
            .unwrap_or(SessionState::Idle)
    }

    pub fn try_acquire_session(&self) -> Result<(), DomainError> {
        let mut guard = self
            .state
            .lock()
            .map_err(|_| DomainError::Process("process state mutex poisoned".to_string()))?;
        if *guard != SessionState::Idle {
            return Err(DomainError::Conflict(format!(
                "another process session is already active (state: {:?})",
                *guard
            )));
        }
        *guard = SessionState::Launching;
        Ok(())
    }

    pub fn release_session(&self) {
        if let Ok(mut guard) = self.state.lock() {
            *guard = SessionState::Idle;
        }
    }

    fn set_state(&self, next: SessionState) {
        if let Ok(mut guard) = self.state.lock() {
            *guard = next;
        }
    }

    pub async fn run_monitored(
        &self,
        cmd: &LaunchCommand,
        timeout: Duration,
    ) -> Result<MonitoredProcessOutput, DomainError> {
        if !cmd.executable.is_file() {
            return Err(DomainError::NotFound(format!(
                "executable not found: '{}'",
                cmd.executable.display()
            )));
        }

        self.try_acquire_session()?;
        let _reset_guard = SessionResetGuard {
            state: Arc::clone(&self.state),
        };

        let mut command = Command::new(&cmd.executable);
        command
            .args(&cmd.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        if let Some(dir) = &cmd.working_dir {
            command.current_dir(dir);
        }

        let mut child = command.spawn().map_err(|e| {
            DomainError::Process(format!(
                "failed to spawn '{}': {e}",
                cmd.executable.display()
            ))
        })?;
        self.set_state(SessionState::Running);

        let stdout_task = child
            .stdout
            .take()
            .map(|out| tokio::spawn(drain_lines(out)));
        let stderr_task = child
            .stderr
            .take()
            .map(|err| tokio::spawn(drain_lines(err)));

        let (exit_code, timed_out) = match tokio::time::timeout(timeout, child.wait()).await {
            Ok(Ok(status)) => {
                self.set_state(SessionState::Exiting);
                (status.code(), false)
            }
            Ok(Err(err)) => {
                return Err(DomainError::Process(format!(
                    "failed waiting on '{}': {err}",
                    cmd.executable.display()
                )));
            }
            Err(_) => {
                self.set_state(SessionState::Exiting);
                let _ = child.kill().await;
                let _ = child.wait().await;
                (None, true)
            }
        };

        let stdout_tail = match stdout_task {
            Some(handle) => handle.await.unwrap_or_default(),
            None => Vec::new(),
        };
        let stderr_tail = match stderr_task {
            Some(handle) => handle.await.unwrap_or_default(),
            None => Vec::new(),
        };

        Ok(MonitoredProcessOutput {
            exit_code,
            stdout_tail,
            stderr_tail,
            timed_out,
        })
    }
}
