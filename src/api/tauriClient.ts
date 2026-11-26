import { Channel, convertFileSrc, invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  ActiveSessionSnapshot,
  ActivityEntryDto,
  BackupFileEntry,
  BackupInspectionReport,
  BackupReport,
  ConsoleDetailsDto,
  ConsoleEnvironmentDto,
  ConsoleProfileOverviewDto,
  ConsoleScanPath,
  ConsoleSetting,
  ConsoleSyncReport,
  ConnectedGamepadInfo,
  DetailedTrophyRow,
  EmulatorRecord,
  FirmwareRecordDto,
  GameDetailsDto,
  GamepadActionEvent,
  GameRecord,
  HubBootstrapDto,
  InstalledThemePackage,
  LiveRestoreReport,
  NewlyUnlockedTrophy,
  PlayerProfile,
  RecordRuntimeVerificationRequest,
  RegisterGameRequest,
  Rpcs3InspectionDto,
  RuntimeVerificationRecord,
  ScanProgressEvent,
  ScanSummaryDto,
  SessionExitedPayload,
  StatisticsOverview,
  SyncTrigger,
  TrophyCenterOverviewDto,
  TrophySyncReport,
  UpdateGameMetadataRequest,
  UpdateProfileRequest,
} from '../types/domain';

export interface HubTransport {
  readonly kind: 'tauri' | 'injected_test';
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
}

export class DesktopRuntimeRequiredError extends Error {
  constructor() {
    super(
      'Universal Console Hub requer o runtime desktop Tauri 2 para acessar o backend Rust/SQLite.'
    );
    this.name = 'DesktopRuntimeRequiredError';
  }
}

declare global {
  interface Window {
    __UCH_TEST_TRANSPORT__?: HubTransport;
    __TAURI_INTERNALS__?: unknown;
  }
}

export function isTauriDesktopRuntime(): boolean {
  return (
    typeof window !== 'undefined' &&
    '__TAURI_INTERNALS__' in window &&
    Boolean(window.__TAURI_INTERNALS__)
  );
}

export function resolveCoverAssetUrl(coverPath: string | null): string | null {
  if (!coverPath || coverPath.trim().length === 0) return null;
  if (coverPath.startsWith('data:') || coverPath.startsWith('http://') || coverPath.startsWith('https://')) {
    return coverPath;
  }
  if (isTauriDesktopRuntime()) {
    return convertFileSrc(coverPath);
  }
  return null;
}

export function resolveHubTransport(): HubTransport {
  if (typeof window !== 'undefined' && window.__UCH_TEST_TRANSPORT__) {
    return window.__UCH_TEST_TRANSPORT__;
  }

  if (isTauriDesktopRuntime()) {
    return {
      kind: 'tauri',
      invoke: <T>(command: string, args?: Record<string, unknown>) => invoke<T>(command, args),
    };
  }

  throw new DesktopRuntimeRequiredError();
}

export class HubApiClient {
  constructor(private readonly transport: HubTransport) {}

  get transportKind(): 'tauri' | 'injected_test' {
    return this.transport.kind;
  }

  getBootstrap(): Promise<HubBootstrapDto> {
    return this.transport.invoke('get_hub_bootstrap');
  }

  getConsoleDetails(slug: string): Promise<ConsoleDetailsDto> {
    return this.transport.invoke('get_console_details', { slug });
  }

  configureEmulatorPath(slug: string, exePath: string): Promise<EmulatorRecord> {
    return this.transport.invoke('configure_emulator_path', { slug, exePath });
  }

  inspectRpcs3(): Promise<Rpcs3InspectionDto> {
    return this.transport.invoke('inspect_rpcs3');
  }

  registerFirmwareFile(
    consoleSlug: string,
    emulatorSlug: string,
    filePath: string
  ): Promise<FirmwareRecordDto> {
    return this.transport.invoke('register_firmware_file', {
      consoleSlug,
      emulatorSlug,
      filePath,
    });
  }

  updateActiveProfile(request: UpdateProfileRequest): Promise<PlayerProfile> {
    return this.transport.invoke('update_active_profile', { request });
  }

  registerGame(request: RegisterGameRequest): Promise<GameRecord> {
    return this.transport.invoke('register_game', { request });
  }

  toggleGameFavorite(gameId: string): Promise<boolean> {
    return this.transport.invoke('toggle_game_favorite', { gameId });
  }

  updateAppSetting(key: string, value: string): Promise<Record<string, string>> {
    return this.transport.invoke('update_app_setting', { key, value });
  }

  createDatabaseBackup(fileName?: string): Promise<BackupReport> {
    return this.transport.invoke('create_database_backup', { fileName: fileName ?? null });
  }

  // ===== Phase 2 IPC Client Methods (13 new commands) =====

  getConsoleEnvironment(slug: string): Promise<ConsoleEnvironmentDto> {
    return this.transport.invoke('get_console_environment', { slug });
  }

  addConsoleScanPath(
    consoleSlug: string,
    directoryPath: string,
    recursive: boolean
  ): Promise<ConsoleScanPath> {
    return this.transport.invoke('add_console_scan_path', {
      consoleSlug,
      directoryPath,
      recursive,
    });
  }

  removeConsoleScanPath(pathId: string): Promise<boolean> {
    return this.transport.invoke('remove_console_scan_path', { pathId });
  }

  scanConsoleLibrary(
    consoleSlug: string,
    onProgress?: (event: ScanProgressEvent) => void
  ): Promise<ScanSummaryDto> {
    if (this.transport.kind === 'tauri') {
      const channel = new Channel<ScanProgressEvent>();
      if (onProgress) {
        channel.onmessage = onProgress;
      }
      return this.transport.invoke('scan_console_library', {
        consoleSlug,
        onProgress: channel,
      });
    }
    return this.transport.invoke('scan_console_library', {
      consoleSlug,
      onProgress,
    });
  }

  updateConsoleSetting(
    consoleSlug: string,
    key: string,
    value: string
  ): Promise<ConsoleSetting> {
    return this.transport.invoke('update_console_setting', { consoleSlug, key, value });
  }

  getGameDetails(gameId: string): Promise<GameDetailsDto> {
    return this.transport.invoke('get_game_details', { gameId });
  }

  updateGameMetadata(request: UpdateGameMetadataRequest): Promise<GameRecord> {
    return this.transport.invoke('update_game_metadata', { request });
  }

  setGameCover(gameId: string, sourceImagePath: string): Promise<GameRecord> {
    return this.transport.invoke('set_game_cover', { gameId, sourceImagePath });
  }

  getStatisticsOverview(): Promise<StatisticsOverview> {
    return this.transport.invoke('get_statistics_overview');
  }

  listActivityHistory(
    eventTypeFilter?: string | null,
    limit = 100
  ): Promise<ActivityEntryDto[]> {
    return this.transport.invoke('list_activity_history', {
      eventTypeFilter: eventTypeFilter ?? null,
      limit,
    });
  }

  listDatabaseBackups(): Promise<BackupFileEntry[]> {
    return this.transport.invoke('list_database_backups');
  }

  inspectDatabaseBackup(backupPath: string): Promise<BackupInspectionReport> {
    return this.transport.invoke('inspect_database_backup', { backupPath });
  }

  restoreDatabaseBackup(backupPath: string): Promise<LiveRestoreReport> {
    return this.transport.invoke('restore_database_backup', { backupPath });
  }

  // ===== Phase 3 IPC Client Methods (7 new commands) =====

  getConsoleProfile(consoleSlug: string): Promise<ConsoleProfileOverviewDto> {
    return this.transport.invoke('get_console_profile', { consoleSlug });
  }

  syncConsoleProfile(
    consoleSlug: string,
    trigger: SyncTrigger = 'manual'
  ): Promise<ConsoleSyncReport> {
    return this.transport.invoke('sync_console_profile', { consoleSlug, trigger });
  }

  launchConsoleSystem(consoleSlug: string): Promise<ActiveSessionSnapshot> {
    return this.transport.invoke('launch_console_system', { consoleSlug });
  }

  launchGame(gameId: string, gameVersionId?: string | null): Promise<ActiveSessionSnapshot> {
    return this.transport.invoke('launch_game', {
      gameId,
      gameVersionId: gameVersionId ?? null,
    });
  }

  getActiveSession(): Promise<ActiveSessionSnapshot> {
    return this.transport.invoke('get_active_session');
  }

  stopActiveSession(): Promise<boolean> {
    return this.transport.invoke('stop_active_session');
  }

  recordRuntimeVerification(
    request: RecordRuntimeVerificationRequest
  ): Promise<RuntimeVerificationRecord> {
    return this.transport.invoke('record_runtime_verification', { request });
  }

  // ===== Phase 4 IPC Client Methods (3 new commands) =====

  syncConsoleTrophies(consoleSlug: string): Promise<TrophySyncReport> {
    return this.transport.invoke('sync_console_trophies', { consoleSlug });
  }

  getTrophyCenterOverview(consoleSlug?: string | null): Promise<TrophyCenterOverviewDto> {
    return this.transport.invoke('get_trophy_center_overview', {
      consoleSlug: consoleSlug ?? null,
    });
  }

  listGameTrophies(gameId: string): Promise<DetailedTrophyRow[]> {
    return this.transport.invoke('list_game_trophies', { gameId });
  }

  // ===== Theme Packages & Native Windows Gamepad IPC (4 new commands) =====

  listInstalledThemes(): Promise<InstalledThemePackage[]> {
    return this.transport.invoke('list_installed_themes');
  }

  importThemePackage(sourcePath: string): Promise<InstalledThemePackage> {
    return this.transport.invoke('import_theme_package', { sourcePath });
  }

  removeInstalledTheme(themeId: string): Promise<boolean> {
    return this.transport.invoke('remove_installed_theme', { themeId });
  }

  listConnectedGamepads(): Promise<ConnectedGamepadInfo[]> {
    return this.transport.invoke('list_connected_gamepads');
  }
}

export interface HubRuntimeEventHandlers {
  onSessionStarted?: (snapshot: ActiveSessionSnapshot) => void;
  onSessionHeartbeat?: (snapshot: ActiveSessionSnapshot) => void;
  onSessionExited?: (payload: SessionExitedPayload) => void;
  onProfileSynced?: (report: ConsoleSyncReport) => void;
  onTrophiesSynced?: (report: TrophySyncReport) => void;
  onTrophiesUnlocked?: (unlocked: NewlyUnlockedTrophy[]) => void;
  onGamepadDevices?: (devices: ConnectedGamepadInfo[]) => void;
  onGamepadAction?: (event: GamepadActionEvent) => void;
}

export async function subscribeHubRuntimeEvents(
  handlers: HubRuntimeEventHandlers
): Promise<() => void> {
  if (!isTauriDesktopRuntime()) {
    return () => {};
  }

  const unlisteners: UnlistenFn[] = [];
  if (handlers.onSessionStarted) {
    unlisteners.push(
      await listen<ActiveSessionSnapshot>('session:started', (evt) =>
        handlers.onSessionStarted?.(evt.payload)
      )
    );
  }
  if (handlers.onSessionHeartbeat) {
    unlisteners.push(
      await listen<ActiveSessionSnapshot>('session:heartbeat', (evt) =>
        handlers.onSessionHeartbeat?.(evt.payload)
      )
    );
  }
  if (handlers.onSessionExited) {
    unlisteners.push(
      await listen<SessionExitedPayload>('session:exited', (evt) =>
        handlers.onSessionExited?.(evt.payload)
      )
    );
  }
  if (handlers.onProfileSynced) {
    unlisteners.push(
      await listen<ConsoleSyncReport>('profile:synced', (evt) =>
        handlers.onProfileSynced?.(evt.payload)
      )
    );
  }
  if (handlers.onTrophiesSynced) {
    unlisteners.push(
      await listen<TrophySyncReport>('trophies:synced', (evt) =>
        handlers.onTrophiesSynced?.(evt.payload)
      )
    );
  }
  if (handlers.onTrophiesUnlocked) {
    unlisteners.push(
      await listen<NewlyUnlockedTrophy[]>('trophies:unlocked', (evt) =>
        handlers.onTrophiesUnlocked?.(evt.payload)
      )
    );
  }
  if (handlers.onGamepadDevices) {
    unlisteners.push(
      await listen<ConnectedGamepadInfo[]>('gamepad:devices', (evt) =>
        handlers.onGamepadDevices?.(evt.payload)
      )
    );
  }
  if (handlers.onGamepadAction) {
    unlisteners.push(
      await listen<GamepadActionEvent>('gamepad:action', (evt) =>
        handlers.onGamepadAction?.(evt.payload)
      )
    );
  }

  return () => {
    for (const unlisten of unlisteners) {
      unlisten();
    }
  };
}
