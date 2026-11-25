import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { ConsoleEnvironmentDto, ScanSummaryDto } from '../types/domain';
import { ConsoleEnvironmentPage } from './ConsoleEnvironmentPage';

function makePs3Env(): ConsoleEnvironmentDto {
  return {
    console: {
      id: 'console-ps3',
      slug: 'ps3',
      name: 'PlayStation 3',
      manufacturer: 'Sony Interactive Entertainment',
      generation: 7,
      releaseYear: 2006,
      phaseStatus: 'phase_1_prepared_phase_4_full',
      notes: 'Prioridade 1 via RPCS3.',
      capabilities: {
        supportedStates: ['FULL_SYSTEM_BOOT', 'ORIGINAL_DASHBOARD', 'DIRECT_GAME_ONLY'],
        features: [
          {
            feature: 'system_boot',
            states: ['FULL_SYSTEM_BOOT'],
            availability: 'requires_user_files',
            verification: 'documented_upstream',
            notes: 'Requer PS3UPDAT.PUP instalada.',
          },
        ],
      },
    },
    emulator: {
      id: 'emu-rpcs3',
      slug: 'rpcs3',
      name: 'RPCS3',
      consoleId: 'console-ps3',
      consoleSlug: 'ps3',
      executablePath: null,
      configPath: null,
      detectedVersion: null,
      integrationPhase: 'phase_1_preparation',
      capabilities: {
        supportedStates: ['FULL_SYSTEM_BOOT', 'ORIGINAL_DASHBOARD', 'DIRECT_GAME_ONLY'],
        features: [],
      },
    },
    environmentMode: 'authentic_system_capable',
    games: [
      {
        id: 'game-ps3-1',
        consoleId: 'console-ps3',
        consoleSlug: 'ps3',
        canonicalKey: 'serial:BLUS30443',
        title: "Demon's Souls",
        description: 'RPG clássico',
        developer: 'FromSoftware',
        publisher: 'Atlus',
        releaseYear: 2009,
        genre: 'Action RPG',
        coverPath: null,
        metadataSource: 'ps3_param_sfo',
        installState: 'registered',
        associatedEmulatorId: 'emu-rpcs3',
        totalPlaytimeSeconds: 3600,
        lastPlayedAt: null,
        completionPercent: 50,
        isFavorite: false,
        versionCount: 1,
      },
    ],
    scanPaths: [
      {
        id: 'sp-1',
        consoleId: 'console-ps3',
        consoleSlug: 'ps3',
        directoryPath: 'D:\\Jogos PS3',
        recursive: true,
        enabled: true,
        lastScannedAt: null,
        createdAt: '2026-11-25T21:00:00Z',
      },
    ],
    settings: [],
    firmwareRecords: [],
    runtimeVerifications: [],
    rpcs3Inspection: {
      configured: false,
      installation: null,
      readiness: null,
      diagnostic: 'RPCS3 ainda não configurado.',
    },
  };
}

function makePs4Env(): ConsoleEnvironmentDto {
  return {
    console: {
      id: 'console-ps4',
      slug: 'ps4',
      name: 'PlayStation 4',
      manufacturer: 'Sony Interactive Entertainment',
      generation: 8,
      releaseYear: 2013,
      phaseStatus: 'planned_phase_5_experimental',
      notes: 'Integração experimental via shadPS4.',
      capabilities: {
        supportedStates: ['EXPERIMENTAL', 'DIRECT_GAME_ONLY'],
        features: [
          {
            feature: 'original_dashboard',
            states: ['UNSUPPORTED'],
            availability: 'unsupported',
            verification: 'documented_upstream',
            notes: 'Orbis OS não é suportado.',
          },
        ],
      },
    },
    emulator: null,
    environmentMode: 'integrated_alternative_only',
    games: [],
    scanPaths: [],
    settings: [],
    firmwareRecords: [],
    runtimeVerifications: [],
    rpcs3Inspection: null,
  };
}

describe('ConsoleEnvironmentPage (independent console experience)', () => {
  it('shows isolated PS3 games, triggers real scanner summary, and never shows fake boot button in Phase 2', async () => {
    const summary: ScanSummaryDto = {
      consoleSlug: 'ps3',
      scannedDirectories: 1,
      scannedFiles: 5,
      gamesAdded: 1,
      versionsAdded: 1,
      skippedExisting: 0,
      errors: [],
    };
    const onScan = vi.fn().mockImplementation(async (_slug, onProgress) => {
      onProgress?.({ event: 'started', data: { consoleSlug: 'ps3', totalDirectories: 1 } });
      onProgress?.({
        event: 'completed',
        data: { scannedFiles: 5, gamesAdded: 1, versionsAdded: 1, skippedExisting: 0 },
      });
      return summary;
    });

    render(
      <ConsoleEnvironmentPage
        environment={makePs3Env()}
        onBack={() => {}}
        onOpenEmulators={() => {}}
        onOpenGame={() => {}}
        onToggleFavorite={async () => {}}
        onAddScanPath={async () => {}}
        onRemoveScanPath={async () => {}}
        onScanLibrary={onScan}
        onUpdateConsoleSetting={async () => {}}
      />
    );

    expect(screen.getByRole('heading', { name: 'PlayStation 3' })).toBeInTheDocument();
    expect(screen.getByText("Demon's Souls")).toBeInTheDocument();
    expect(
      screen.queryByRole('button', { name: /Iniciar Sistema Original|Iniciar XMB/i })
    ).not.toBeInTheDocument();
    expect(
      screen.queryByText(/Este não é o dashboard original do console/i)
    ).not.toBeInTheDocument();

    // Switch to Pastas & Descoberta tab and run scan
    fireEvent.click(screen.getByRole('tab', { name: /Pastas & Descoberta/i }));
    fireEvent.click(screen.getByRole('button', { name: /Escanear Biblioteca/i }));
    await waitFor(() => {
      expect(onScan).toHaveBeenCalledWith('ps3', expect.any(Function));
      expect(screen.getByText(/1 novos jogos/i)).toBeInTheDocument();
    });
  });

  it('displays permanent Integrated Alternative banner for PS4/Xbox 360 consoles', () => {
    render(
      <ConsoleEnvironmentPage
        environment={makePs4Env()}
        onBack={() => {}}
        onOpenEmulators={() => {}}
        onOpenGame={() => {}}
        onToggleFavorite={async () => {}}
        onAddScanPath={async () => {}}
        onRemoveScanPath={async () => {}}
        onScanLibrary={async () => ({
          consoleSlug: 'ps4',
          scannedDirectories: 0,
          scannedFiles: 0,
          gamesAdded: 0,
          versionsAdded: 0,
          skippedExisting: 0,
          errors: [],
        })}
        onUpdateConsoleSetting={async () => {}}
      />
    );

    expect(
      screen.getByText(/Ambiente Integrado do Hub — Este não é o dashboard original do console/i)
    ).toBeInTheDocument();
  });
});
