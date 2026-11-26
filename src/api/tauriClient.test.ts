import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  DesktopRuntimeRequiredError,
  HubApiClient,
  resolveHubTransport,
  type HubTransport,
} from './tauriClient';

declare global {
  interface Window {
    __UCH_TEST_TRANSPORT__?: HubTransport;
  }
}

describe('tauriClient transport resolution and typed dispatch', () => {
  afterEach(() => {
    delete window.__UCH_TEST_TRANSPORT__;
  });

  it('throws DesktopRuntimeRequiredError when outside Tauri runtime without explicit test transport', () => {
    expect(() => resolveHubTransport()).toThrow(DesktopRuntimeRequiredError);
  });

  it('uses explicit injected test transport only when registered for UI tests', async () => {
    const invokeMock = vi.fn().mockResolvedValue(true);
    window.__UCH_TEST_TRANSPORT__ = {
      kind: 'injected_test',
      invoke: invokeMock,
    };

    const transport = resolveHubTransport();
    expect(transport.kind).toBe('injected_test');

    const client = new HubApiClient(transport);
    const fav = await client.toggleGameFavorite('game-1');
    expect(fav).toBe(true);
    expect(invokeMock).toHaveBeenCalledWith('toggle_game_favorite', { gameId: 'game-1' });
  });

  it('dispatches Phase 2 console environment, scanner progress, stats, and backup restore commands', async () => {
    const invokeMock = vi.fn().mockImplementation(async (cmd: string, args?: Record<string, unknown>) => {
      if (cmd === 'scan_console_library') {
        const cb = args?.onProgress as ((evt: unknown) => void) | undefined;
        cb?.({ event: 'started', data: { consoleSlug: 'ps3', totalDirectories: 1 } });
        return {
          consoleSlug: 'ps3',
          scannedDirectories: 1,
          scannedFiles: 4,
          gamesAdded: 1,
          versionsAdded: 1,
          skippedExisting: 0,
          errors: [],
        };
      }
      return { ok: true };
    });

    const client = new HubApiClient({ kind: 'injected_test', invoke: invokeMock });
    const received: unknown[] = [];

    await client.getConsoleEnvironment('ps3');
    await client.addConsoleScanPath('ps3', 'D:\\PS3', true);
    const summary = await client.scanConsoleLibrary('ps3', (evt) => received.push(evt));
    await client.inspectDatabaseBackup('D:\\backups\\snap.db');
    await client.restoreDatabaseBackup('D:\\backups\\snap.db');

    expect(summary.gamesAdded).toBe(1);
    expect(received).toHaveLength(1);
    expect(invokeMock).toHaveBeenCalledWith('get_console_environment', { slug: 'ps3' });
    expect(invokeMock).toHaveBeenCalledWith('add_console_scan_path', {
      consoleSlug: 'ps3',
      directoryPath: 'D:\\PS3',
      recursive: true,
    });
    expect(invokeMock).toHaveBeenCalledWith('inspect_database_backup', {
      backupPath: 'D:\\backups\\snap.db',
    });
    expect(invokeMock).toHaveBeenCalledWith('restore_database_backup', {
      backupPath: 'D:\\backups\\snap.db',
    });
  });

  it('dispatches Phase 3 console profile, sync, live launch, stop, and runtime verification commands', async () => {
    const invokeMock = vi.fn().mockResolvedValue({ ok: true });
    const client = new HubApiClient({ kind: 'injected_test', invoke: invokeMock });

    await client.getConsoleProfile('ps3');
    await client.syncConsoleProfile('ps3', 'manual');
    await client.launchConsoleSystem('ps3');
    await client.launchGame('game-1', 'gv-1');
    await client.getActiveSession();
    await client.stopActiveSession();
    await client.recordRuntimeVerification({
      consoleSlug: 'ps3',
      emulatorSlug: 'rpcs3',
      gameId: null,
      featureKey: 'original_dashboard',
      emulatorVersion: 'RPCS3 v0.0.34',
      status: 'runtime_confirmed',
      evidenceNotes: 'XMB booted and navigated on Windows 11.',
    });

    expect(invokeMock).toHaveBeenCalledWith('get_console_profile', { consoleSlug: 'ps3' });
    expect(invokeMock).toHaveBeenCalledWith('sync_console_profile', {
      consoleSlug: 'ps3',
      trigger: 'manual',
    });
    expect(invokeMock).toHaveBeenCalledWith('launch_console_system', { consoleSlug: 'ps3' });
    expect(invokeMock).toHaveBeenCalledWith('launch_game', {
      gameId: 'game-1',
      gameVersionId: 'gv-1',
    });
    expect(invokeMock).toHaveBeenCalledWith('get_active_session');
    expect(invokeMock).toHaveBeenCalledWith('stop_active_session');
    expect(invokeMock).toHaveBeenCalledWith('record_runtime_verification', {
      request: {
        consoleSlug: 'ps3',
        emulatorSlug: 'rpcs3',
        gameId: null,
        featureKey: 'original_dashboard',
        emulatorVersion: 'RPCS3 v0.0.34',
        status: 'runtime_confirmed',
        evidenceNotes: 'XMB booted and navigated on Windows 11.',
      },
    });
  });
});
