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
});
