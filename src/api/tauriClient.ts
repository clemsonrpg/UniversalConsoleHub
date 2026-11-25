import { invoke } from '@tauri-apps/api/core';
import type {
  BackupReport,
  ConsoleDetailsDto,
  EmulatorRecord,
  FirmwareRecordDto,
  GameRecord,
  HubBootstrapDto,
  PlayerProfile,
  RegisterGameRequest,
  Rpcs3InspectionDto,
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

export function resolveHubTransport(): HubTransport {
  if (typeof window !== 'undefined' && window.__UCH_TEST_TRANSPORT__) {
    return window.__UCH_TEST_TRANSPORT__;
  }

  if (typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window && window.__TAURI_INTERNALS__) {
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
}
