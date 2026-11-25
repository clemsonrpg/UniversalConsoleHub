import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type {
  BackupFileEntry,
  BackupInspectionReport,
  GameDetailsDto,
  LiveRestoreReport,
  StatisticsOverview,
} from '../types/domain';
import { GameDetailsPage } from './GameDetailsPage';
import { SettingsPage } from './SettingsPage';
import { StatisticsPage } from './StatisticsPage';

const sampleDetails: GameDetailsDto = {
  game: {
    id: 'game-1',
    consoleId: 'console-ps3',
    consoleSlug: 'ps3',
    canonicalKey: 'serial:BLUS30443',
    title: "Demon's Souls",
    description: 'RPG original',
    developer: 'FromSoftware',
    publisher: 'Atlus',
    releaseYear: 2009,
    genre: 'Action RPG',
    coverPath: null,
    metadataSource: 'ps3_param_sfo',
    installState: 'registered',
    associatedEmulatorId: 'emu-rpcs3',
    totalPlaytimeSeconds: 5400,
    lastPlayedAt: '2026-11-25T20:00:00Z',
    completionPercent: 45,
    isFavorite: false,
    versionCount: 1,
  },
  versions: [
    {
      id: 'gv-1',
      gameId: 'game-1',
      versionLabel: '01.00',
      region: 'USA',
      serialCode: 'BLUS30443',
      filePath: 'D:\\PS3\\DemonsSouls\\PS3_GAME\\USRDIR\\EBOOT.BIN',
      installState: 'registered',
      createdAt: '2026-11-25T19:00:00Z',
    },
  ],
  sessions: [
    {
      id: 's-1',
      profileId: 'profile-default',
      gameId: 'game-1',
      gameTitle: "Demon's Souls",
      consoleSlug: 'ps3',
      gameVersionId: 'gv-1',
      emulatorId: 'emu-rpcs3',
      startedAt: '2026-11-25T18:30:00Z',
      lastHeartbeatAt: '2026-11-25T20:00:00Z',
      endedAt: '2026-11-25T20:00:00Z',
      durationSeconds: 5400,
      exitStatus: 'recovered_after_crash',
    },
  ],
};

describe('Task 9 pages (GameDetailsPage, StatisticsPage, SettingsPage backup restore)', () => {
  it('GameDetailsPage renders versions, sessions, saves metadata, and has no fake play button', async () => {
    const onUpdateMetadata = vi.fn().mockResolvedValue(undefined);
    const onSetCover = vi.fn().mockResolvedValue(undefined);

    render(
      <GameDetailsPage
        details={sampleDetails}
        onBack={() => {}}
        onToggleFavorite={async () => {}}
        onUpdateMetadata={onUpdateMetadata}
        onSetCover={onSetCover}
      />
    );

    expect(screen.getByRole('heading', { name: "Demon's Souls" })).toBeInTheDocument();
    expect(screen.getByText(/BLUS30443/i)).toBeInTheDocument();
    expect(screen.getByText(/Recuperada após queda/i)).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: /^Jogar$/i })).not.toBeInTheDocument();

    fireEvent.change(screen.getByLabelText(/Progresso de conclusão/i), { target: { value: '80' } });
    fireEvent.click(screen.getByRole('button', { name: /Salvar Metadados/i }));

    await waitFor(() => {
      expect(onUpdateMetadata).toHaveBeenCalledWith(
        expect.objectContaining({ gameId: 'game-1', completionPercent: 80 })
      );
    });
  });

  it('StatisticsPage displays real database metrics and filterable activity history', async () => {
    const overview: StatisticsOverview = {
      totalPlaytimeSeconds: 0,
      totalSessions: 0,
      gamesRegistered: 1,
      gamesStarted: 0,
      gamesCompleted: 0,
      completionRatePercent: 0,
      byConsole: [
        {
          consoleSlug: 'ps3',
          consoleName: 'PlayStation 3',
          gamesCount: 1,
          sessionsCount: 0,
          totalPlaytimeSeconds: 0,
        },
      ],
      topGames: [],
      monthlyPlaytime: [],
    };
    const onFilter = vi.fn().mockResolvedValue([]);

    render(
      <StatisticsPage
        overview={overview}
        activity={[
          {
            id: 'a-1',
            eventType: 'library_scanned',
            entityType: 'console',
            entityId: 'console-ps3',
            summary: 'Scan de PlayStation 3 concluído.',
            occurredAt: '2026-11-25T21:00:00Z',
          },
        ]}
        onFilterActivity={onFilter}
      />
    );

    expect(screen.getByRole('heading', { name: /Estatísticas & Histórico/i })).toBeInTheDocument();
    expect(screen.getByText('PlayStation 3')).toBeInTheDocument();
    expect(screen.getByText(/Scan de PlayStation 3 concluído/i)).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText(/Filtrar histórico por tipo/i), {
      target: { value: 'library_scanned' },
    });
    await waitFor(() => {
      expect(onFilter).toHaveBeenCalledWith('library_scanned');
    });
  });

  it('SettingsPage requires inspection + ConfirmDialog confirmation before restoring SQLite backup', async () => {
    const backups: BackupFileEntry[] = [
      {
        fileName: 'snapshot_1.db',
        backupPath: 'C:\\Backups\\snapshot_1.db',
        sizeBytes: 131072,
        modifiedAt: '2026-11-25T21:00:00Z',
      },
    ];
    const inspection: BackupInspectionReport = {
      backupPath: 'C:\\Backups\\snapshot_1.db',
      sizeBytes: 131072,
      integrityOk: true,
      schemaVersion: 2,
      profilesCount: 1,
      gamesCount: 3,
      inspectedAt: '2026-11-25T21:05:00Z',
    };
    const liveRestore: LiveRestoreReport = {
      sourceBackupPath: 'C:\\Backups\\snapshot_1.db',
      restoredDbPath: 'C:\\App\\universal_console_hub.db',
      preRestoreSnapshotPath: 'C:\\Backups\\pre_restore\\pre_restore_abc.db',
      integrityOk: true,
      schemaVersion: 2,
      restoredAt: '2026-11-25T21:06:00Z',
    };

    const onInspect = vi.fn().mockResolvedValue(inspection);
    const onRestore = vi.fn().mockResolvedValue(liveRestore);

    render(
      <SettingsPage
        settings={{ 'ui.fullscreen_default': 'false', 'ui.target_resolution': '1920x1080' }}
        backups={backups}
        onUpdateSetting={async () => {}}
        onCreateBackup={async () => ({
          backupPath: 'C:\\Backups\\new.db',
          sizeBytes: 90000,
          integrityOk: true,
          createdAt: '2026-11-25T21:00:00Z',
        })}
        onInspectBackup={onInspect}
        onRestoreBackup={onRestore}
      />
    );

    // 1. Inspect opens ConfirmDialog; Cancel MUST NOT call onRestoreBackup
    fireEvent.click(screen.getByRole('button', { name: /Inspecionar e Restaurar/i }));
    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeInTheDocument();
      expect(screen.getByText(/3 jogo\(s\)/i)).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole('button', { name: 'Cancelar' }));
    expect(onRestore).not.toHaveBeenCalled();

    // 2. Inspect again and Confirm executes live restore
    fireEvent.click(screen.getByRole('button', { name: /Inspecionar e Restaurar/i }));
    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole('button', { name: /Confirmar Restauração/i }));
    await waitFor(() => {
      expect(onRestore).toHaveBeenCalledWith('C:\\Backups\\snapshot_1.db');
      expect(screen.getByText(/Banco SQLite restaurado com sucesso/i)).toBeInTheDocument();
    });
  });
});
