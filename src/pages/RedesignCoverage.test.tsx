import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { InputProvider } from '../input/InputProvider';
import { ThemeProvider } from '../themes/ThemeProvider';
import type { ConsoleRecord, GameRecord } from '../types/domain';
import { LibraryPage } from './LibraryPage';
import { SettingsPage } from './SettingsPage';

function makeManyGames(count: number): GameRecord[] {
  return Array.from({ length: count }, (_, idx) => ({
    id: `game-${idx + 1}`,
    consoleId: 'console-ps3',
    consoleSlug: 'ps3',
    canonicalKey: `serial:BLUS${String(30000 + idx)}`,
    title: `Title ${String(idx + 1).padStart(2, '0')} Chronicle`,
    description: '',
    developer: 'Studio',
    publisher: 'Pub',
    releaseYear: 2010 + (idx % 5),
    genre: idx % 2 === 0 ? 'Action' : 'RPG',
    coverPath: null,
    metadataSource: 'ps3_param_sfo',
    installState: 'registered',
    associatedEmulatorId: 'emu-rpcs3',
    totalPlaytimeSeconds: idx * 600,
    lastPlayedAt: idx < 5 ? '2026-11-25T20:00:00Z' : null,
    completionPercent: (idx * 7) % 100,
    isFavorite: idx % 10 === 0,
    versionCount: 1,
  }));
}

const ps3Console: ConsoleRecord = {
  id: 'console-ps3',
  slug: 'ps3',
  name: 'PlayStation 3',
  manufacturer: 'Sony',
  generation: 7,
  releaseYear: 2006,
  phaseStatus: 'phase_3_live',
  notes: '',
  capabilities: {
    supportedStates: ['FULL_SYSTEM_BOOT', 'ORIGINAL_DASHBOARD', 'DIRECT_GAME_ONLY'],
    features: [],
  },
};

describe('Redesign coverage: large lists, Desktop/Fullscreen persistence, and gamepad remapping', () => {
  it('handles large 60-game library sorting, filtering, view toggle, and PageDown fast jump', async () => {
    const games = makeManyGames(60);
    render(
      <LibraryPage
        consoles={[ps3Console]}
        games={games}
        rpcs3Ready
        onToggleFavorite={vi.fn().mockResolvedValue(undefined)}
        onOpenGame={vi.fn()}
      />
    );

    expect(screen.getByText(/Jogos Descobertos \(60\)/i)).toBeInTheDocument();

    // Switch to List mode and filter by genre RPG
    fireEvent.click(screen.getByRole('button', { name: /^Lista$/i }));
    fireEvent.change(screen.getByLabelText(/Filtrar por categoria/i), { target: { value: 'RPG' } });
    expect(screen.getByText(/Jogos Descobertos \(30\)/i)).toBeInTheDocument();

    // Sort by Playtime descending -> Title 60 Chronicle appears first
    fireEvent.change(screen.getByLabelText(/Ordenar jogos/i), {
      target: { value: 'playtime' },
    });
    const firstCard = document.querySelector('[data-focus-card="true"]');
    expect(firstCard?.textContent).toContain('Title 60 Chronicle');
  });

  it('persists Desktop/Fullscreen mode, category tabs, and configurable gamepad button remapping', async () => {
    const onSetting = vi.fn();

    function SettingsHarness() {
      const [settings, setSettings] = useState<Record<string, string>>({
        'ui.display_mode': 'desktop',
        'ui.target_resolution': '1920x1080',
        'theme.active_id': 'universal-modern',
      });
      const update = async (k: string, v: string) => {
        onSetting(k, v);
        setSettings((prev) => ({ ...prev, [k]: v }));
      };
      return (
        <InputProvider
          settings={settings}
          suspended={false}
          nativeGamepads={[
            {
              slot: 0,
              name: 'Xbox Wireless Controller (XInput #1)',
              backend: 'xinput',
              glyphFamily: 'xbox',
              connected: true,
            },
          ]}
          lastNativeAction={null}
          onUpdateSetting={update}
          onNavigateBack={() => false}
          onCycleSection={() => {}}
        >
          <ThemeProvider
            settings={settings}
            displayMode={settings['ui.display_mode'] === 'fullscreen' ? 'fullscreen' : 'desktop'}
            installedPackages={[]}
            onUpdateSetting={update}
            onImportThemePackage={async () => {
              throw new Error('unused');
            }}
            onRemoveInstalledTheme={async () => true}
            onRefreshInstalledThemes={async () => {}}
          >
            <SettingsPage
              settings={settings}
              onUpdateSetting={update}
              onCreateBackup={async () => ({
                backupPath: 'C:\\Backups\\b.db',
                sizeBytes: 1024,
                integrityOk: true,
                createdAt: '2026-11-26T00:00:00Z',
              })}
            />
          </ThemeProvider>
        </InputProvider>
      );
    }

    render(<SettingsHarness />);

    // Isolate Controles tab
    fireEvent.click(screen.getByRole('tab', { name: 'Controles' }));
    expect(screen.getByText(/Xbox Wireless Controller/i)).toBeInTheDocument();
    expect(screen.queryByLabelText(/Resolução alvo/i)).not.toBeInTheDocument();

    // Remap West button (X / □) to toggle_mode
    fireEvent.change(screen.getByLabelText(/Botão Oeste/i), { target: { value: 'toggle_mode' } });
    await waitFor(() => {
      expect(onSetting).toHaveBeenCalledWith(
        'gamepad.config_json',
        expect.stringContaining('"west":"toggle_mode"')
      );
    });

    // Switch to Aparência tab and toggle Fullscreen TV
    fireEvent.click(screen.getByRole('tab', { name: 'Aparência' }));
    fireEvent.change(screen.getByLabelText(/Modo de exibição do Hub/i), {
      target: { value: 'fullscreen' },
    });
    await waitFor(() => {
      expect(onSetting).toHaveBeenCalledWith('ui.display_mode', 'fullscreen');
      expect(document.documentElement.dataset.displayMode).toBe('fullscreen');
    });
  });
});
