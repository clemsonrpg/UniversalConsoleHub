import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import type { TrophyCenterOverviewDto, TrophySyncReport } from '../types/domain';
import { TrophiesPage } from './TrophiesPage';

function makeOverviewFixture(): TrophyCenterOverviewDto {
  return {
    providers: [
      {
        id: 'prov-emulator',
        slug: 'emulator',
        name: 'EmulatorAchievementProvider',
        origin: 'emulator',
        availabilityStatus: 'ready_offline',
        supportsAuth: false,
        supportsSync: true,
        notes: 'Leitura local somente leitura de TROPCONF.SFM e TROPUSR.DAT do RPCS3.',
      },
      {
        id: 'prov-playstation',
        slug: 'playstation',
        name: 'PlayStationTrophyProvider',
        origin: 'official',
        availabilityStatus: 'unavailable_no_official_api',
        supportsAuth: true,
        supportsSync: true,
        notes: 'Sem API oficial autorizada.',
      },
    ],
    sets: [
      {
        id: 'set-1',
        consoleSlug: 'ps3',
        gameId: 'game-ps3-1',
        gameTitle: "Demon's Souls",
        providerSlug: 'emulator',
        externalSetId: 'NPWR00637_00',
        title: "Demon's Souls",
        detail: 'PS3 Trophy Set',
        iconPath: null,
        totalCount: 3,
        unlockedCount: 1,
        bronzeUnlocked: 1,
        silverUnlocked: 0,
        goldUnlocked: 0,
        platinumUnlocked: 0,
        supportsPlatinum: true,
        completionPercent: 33,
        updatedAt: '2026-11-25T23:00:00Z',
      },
    ],
    trophies: [
      {
        id: 'tr-1',
        trophySetId: 'set-1',
        gameId: 'game-ps3-1',
        gameTitle: "Demon's Souls",
        consoleId: 'console-ps3',
        consoleSlug: 'ps3',
        providerId: 'prov-emulator',
        providerSlug: 'emulator',
        externalTrophyId: '000',
        name: 'Toughest Soul Trophy',
        description: 'All Trophies Obtained',
        iconPath: null,
        category: 'platinum',
        rarityPercent: null,
        origin: 'emulator',
        isHidden: false,
        isUnlocked: false,
        unlockedAt: null,
        consoleLocalUserName: null,
      },
      {
        id: 'tr-2',
        trophySetId: 'set-1',
        gameId: 'game-ps3-1',
        gameTitle: "Demon's Souls",
        consoleId: 'console-ps3',
        consoleSlug: 'ps3',
        providerId: 'prov-emulator',
        providerSlug: 'emulator',
        externalTrophyId: '001',
        name: "Phalanx's Trophy",
        description: 'Slayer of Demon Phalanx',
        iconPath: null,
        category: 'bronze',
        rarityPercent: null,
        origin: 'emulator',
        isHidden: true,
        isUnlocked: true,
        unlockedAt: '2026-11-25T20:00:00Z',
        consoleLocalUserName: 'ClemsonPS3',
      },
      {
        id: 'tr-3',
        trophySetId: 'set-1',
        gameId: 'game-ps3-1',
        gameTitle: "Demon's Souls",
        consoleId: 'console-ps3',
        consoleSlug: 'ps3',
        providerId: 'prov-emulator',
        providerSlug: 'emulator',
        externalTrophyId: '002',
        name: 'Secret Boss Trophy',
        description: 'Defeat the hidden final boss without taking damage',
        iconPath: null,
        category: 'gold',
        rarityPercent: null,
        origin: 'emulator',
        isHidden: true,
        isUnlocked: false,
        unlockedAt: null,
        consoleLocalUserName: null,
      },
    ],
    recentUnlocks: [
      {
        id: 'tr-2',
        trophySetId: 'set-1',
        gameId: 'game-ps3-1',
        gameTitle: "Demon's Souls",
        consoleId: 'console-ps3',
        consoleSlug: 'ps3',
        providerId: 'prov-emulator',
        providerSlug: 'emulator',
        externalTrophyId: '001',
        name: "Phalanx's Trophy",
        description: 'Slayer of Demon Phalanx',
        iconPath: null,
        category: 'bronze',
        rarityPercent: null,
        origin: 'emulator',
        isHidden: true,
        isUnlocked: true,
        unlockedAt: '2026-11-25T20:00:00Z',
        consoleLocalUserName: 'ClemsonPS3',
      },
    ],
    totals: {
      totalSets: 1,
      completedSets: 0,
      totalTrophies: 3,
      unlockedTrophies: 1,
      bronzeTotal: 1,
      bronzeUnlocked: 1,
      silverTotal: 0,
      silverUnlocked: 0,
      goldTotal: 1,
      goldUnlocked: 0,
      platinumTotal: 1,
      platinumUnlocked: 0,
      completionPercent: 33,
    },
  };
}

describe('TrophiesPage (Phase 4 Universal Trophy Center)', () => {
  it('renders completion totals, honest Local/Offline rarity, masks locked hidden trophies until spoiler toggle, filters, and triggers sync', async () => {
    const syncReport: TrophySyncReport = {
      consoleSlug: 'ps3',
      setsSynced: 1,
      trophiesSynced: 3,
      unlocksRecorded: 1,
      newlyUnlocked: [],
      iconsCached: 2,
      diagnostics: [],
      syncedAt: '2026-11-25T23:10:00Z',
    };
    const onSync = vi.fn().mockResolvedValue(syncReport);

    render(
      <TrophiesPage
        overview={makeOverviewFixture()}
        onSyncConsoleTrophies={onSync}
      />
    );

    // Overall completion & metal counts
    expect(screen.getAllByText(/33%/).length).toBeGreaterThan(0);
    expect(screen.getByText('NPWR00637_00')).toBeInTheDocument();

    // Unlocked hidden trophy is visible with real local user attribution & honest Local / Offline rarity
    expect(screen.getAllByText("Phalanx's Trophy").length).toBeGreaterThan(0);
    expect(screen.getAllByText(/ClemsonPS3/).length).toBeGreaterThan(0);
    expect(screen.getAllByText(/Local \/ Offline/i).length).toBeGreaterThan(0);

    // Locked hidden trophy description MUST be masked by default
    expect(
      screen.queryByText('Defeat the hidden final boss without taking damage')
    ).not.toBeInTheDocument();
    expect(screen.getByText(/Troféu Oculto/i)).toBeInTheDocument();

    // Toggling explicit spoiler reveal uncovers locked hidden trophy name & description
    fireEvent.click(screen.getByLabelText(/Revelar troféus ocultos/i));
    expect(screen.getByText('Secret Boss Trophy')).toBeInTheDocument();
    expect(
      screen.getByText('Defeat the hidden final boss without taking damage')
    ).toBeInTheDocument();

    // Filter by status = Desbloqueados hides locked trophies
    fireEvent.change(screen.getByLabelText(/Filtrar por status/i), {
      target: { value: 'unlocked' },
    });
    expect(screen.queryByText('Toughest Soul Trophy')).not.toBeInTheDocument();

    // Trigger manual RPCS3 trophy sync
    fireEvent.click(screen.getByRole('button', { name: /Sincronizar Troféus Agora/i }));
    await waitFor(() => {
      expect(onSync).toHaveBeenCalledWith('ps3');
      expect(screen.getByText(/1 conjunto\(s\), 3 troféu\(s\)/i)).toBeInTheDocument();
    });
  });
});
