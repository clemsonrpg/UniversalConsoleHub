import { Channel, convertFileSrc, invoke } from '@tauri-apps/api/core';
import type {
  ActivityEntryDto,
  BackupFileEntry,
  BackupInspectionReport,
  BackupReport,
  ConsoleDetailsDto,
  ConsoleEnvironmentDto,
  ConsoleScanPath,
  ConsoleSetting,
  EmulatorRecord,
  FirmwareRecordDto,
  GameDetailsDto,
  GameRecord,
  HubBootstrapDto,
  LiveRestoreReport,
  PlayerProfile,
  RegisterGameRequest,
  Rpcs3InspectionDto,
  ScanProgressEvent,
  ScanSummaryDto,
  StatisticsOverview,
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
}
