import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { ConsoleRecord, GameRecord } from '../types/domain';
import { LibraryPage } from './LibraryPage';

const sampleConsoles: ConsoleRecord[] = [
  {
    id: 'console-ps3',
    slug: 'ps3',
    name: 'PlayStation 3',
    manufacturer: 'Sony Interactive Entertainment',
    generation: 7,
    releaseYear: 2006,
    phaseStatus: 'phase_3_playable',
    notes: 'Prioridade 1 via RPCS3.',
    capabilities: {
      supportedStates: ['FULL_SYSTEM_BOOT', 'ORIGINAL_DASHBOARD', 'DIRECT_GAME_ONLY'],
      features: [],
    },
  },
];

const sampleGames: GameRecord[] = [
  {
    id: 'game-1',
    consoleId: 'console-ps3',
    consoleSlug: 'ps3',
    canonicalKey: 'serial:BLUS30443',
    title: "Demon's Souls",
    description: '',
    developer: 'FromSoftware',
    publisher: 'Atlus',
    releaseYear: 2009,
    genre: 'RPG',
    coverPath: null,
    metadataSource: 'ps3_param_sfo',
    installState: 'installed',
    associatedEmulatorId: 'emu-rpcs3',
    totalPlaytimeSeconds: 3600,
    lastPlayedAt: '2026-11-25T20:00:00Z',
    completionPercent: 40,
    isFavorite: false,
    versionCount: 1,
  },
  {
    id: 'game-2',
    consoleId: 'console-ps3',
    consoleSlug: 'ps3',
    canonicalKey: 'serial:NPUB30001',
    title: 'Journey',
    description: '',
    developer: 'thatgamecompany',
    publisher: 'Sony',
    releaseYear: 2012,
    genre: 'Adventure',
    coverPath: null,
    metadataSource: 'ps3_param_sfo',
    installState: 'missing_files',
    associatedEmulatorId: 'emu-rpcs3',
    totalPlaytimeSeconds: 1800,
    lastPlayedAt: '2026-11-24T20:00:00Z',
    completionPercent: 100,
    isFavorite: true,
    versionCount: 1,
  },
];

describe('LibraryPage (automatic discovery only — zero manual registration form)', () => {
  it('completely removes manual game registration form and exposes console discovery shortcuts + missing_files badge', () => {
    const onSelectConsole = vi.fn();
    render(
      <LibraryPage
        consoles={sampleConsoles}
        games={sampleGames}
        onToggleFavorite={async () => {}}
        onSelectConsole={onSelectConsole}
      />
    );

    // Manual registration form MUST NOT exist
    expect(screen.queryByText(/Registrar Jogo Local no Banco/i)).not.toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /Registrar Jogo/i })).not.toBeInTheDocument();
    expect(screen.queryByLabelText(/Título do jogo/i)).not.toBeInTheDocument();

    // Auto-discovery guidance & console shortcut
    expect(screen.getByText(/Descoberta e Sincronização Automática/i)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: /Abrir Perfil PlayStation 3/i }));
    expect(onSelectConsole).toHaveBeenCalledWith('ps3');

    // Preserved missing game shows honest availability status
    expect(screen.getByText('Journey')).toBeInTheDocument();
    expect(screen.getByText(/Ausente no disco/i)).toBeInTheDocument();
  });
});
