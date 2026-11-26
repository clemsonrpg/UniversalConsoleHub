import { expect, test } from '@playwright/test';

/**
 * LAYER 3 — UI E2E TESTS (Playwright + Injected Test Transport) — PHASE 4
 *
 * Scope disclaimer:
 * Validates browser UI flows for the Phase 4 Universal Trophy Center (overall/per-game completion,
 * Platinum/Gold/Silver/Bronze counts, manual RPCS3 trophy sync + toast notification, hidden trophy
 * spoiler masking and explicit reveal toggle, honest "Local / Offline" rarity, filters, timeline,
 * GameDetailsPage trophies section, and PS3 Console Profile trophy summary) using
 * `window.__UCH_TEST_TRANSPORT__`.
 * Does NOT replace Layer 2 real Rust/SQLite/RPCS3 parser integration tests or the standalone `.exe` audit.
 */

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    const state = {
      appName: 'Universal Console Hub',
      version: '0.1.0',
      phase: 1,
      profile: {
        id: 'profile-default',
        userId: 'user-default',
        username: 'player1',
        handle: 'player1',
        displayName: 'Player 1',
        avatarPath: null,
        bannerPath: null,
        bio: 'Perfil offline local.',
        privacyLevel: 'private',
        createdAt: '2026-11-25T21:00:00Z',
        updatedAt: '2026-11-25T21:00:00Z',
      },
      statistics: {
        gamesRegistered: 1,
        gamesStarted: 1,
        gamesCompleted: 0,
        totalPlaytimeSeconds: 3600,
        totalTrophies: 0,
        unlockedTrophies: 0,
        rareTrophies: 0,
      },
      consoles: [
        {
          id: 'console-ps3',
          slug: 'ps3',
          name: 'PlayStation 3',
          manufacturer: 'Sony Interactive Entertainment',
          generation: 7,
          releaseYear: 2006,
          phaseStatus: 'phase_3_playable',
          notes: 'PlayStation 3 jogável via RPCS3.',
          capabilities: {
            supportedStates: ['FULL_SYSTEM_BOOT', 'ORIGINAL_DASHBOARD', 'DIRECT_GAME_ONLY'],
            features: [],
          },
        },
      ],
      emulators: [
        {
          id: 'emu-rpcs3',
          slug: 'rpcs3',
          name: 'RPCS3',
          consoleId: 'console-ps3',
          consoleSlug: 'ps3',
          executablePath: 'D:\\Emulators\\RPCS3\\rpcs3.exe',
          configPath: null,
          detectedVersion: 'RPCS3 v0.0.34-17105',
          integrationPhase: 'phase_3_playable',
          capabilities: {
            supportedStates: ['FULL_SYSTEM_BOOT', 'ORIGINAL_DASHBOARD', 'DIRECT_GAME_ONLY'],
            features: [],
          },
        },
      ],
      games: [
        {
          id: 'game-ps3-1',
          consoleId: 'console-ps3',
          consoleSlug: 'ps3',
          canonicalKey: 'serial:BLUS30443',
          title: "Demon's Souls",
          description: '',
          developer: 'FromSoftware',
          publisher: 'Atlus',
          releaseYear: 2009,
          genre: 'Action RPG',
          coverPath: null,
          metadataSource: 'ps3_param_sfo',
          installState: 'installed',
          associatedEmulatorId: 'emu-rpcs3',
          totalPlaytimeSeconds: 3600,
          lastPlayedAt: '2026-11-25T20:00:00Z',
          completionPercent: 0,
          isFavorite: false,
          versionCount: 1,
        },
      ],
      achievementProviders: [
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
          notes: 'PSN oficial indisponível sem API autorizada.',
        },
      ],
      trophies: [],
      firmwareRecords: [],
      recentActivity: [],
      settings: {},
      rpcs3Inspection: {
        configured: true,
        installation: {
          emulatorSlug: 'rpcs3',
          executablePath: 'D:\\Emulators\\RPCS3\\rpcs3.exe',
          rootDir: 'D:\\Emulators\\RPCS3',
          detectedVersion: 'RPCS3 v0.0.34-17105',
          isValid: true,
          issues: [],
        },
        readiness: {
          firmwarePackageValidated: true,
          firmwareSha256: 'a'.repeat(64),
          systemFilesPresent: true,
          commandBuildable: true,
          xmbRuntimeConfirmed: false,
          generatedCommand: null,
          notes: [],
        },
        diagnostic: 'RPCS3 pronto.',
      },
    };

    let synced = false;

    const syncedSets = [
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
    ];

    const syncedTrophies = [
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
        description: 'Defeat the hidden boss cleanly',
        iconPath: null,
        category: 'gold',
        rarityPercent: null,
        origin: 'emulator',
        isHidden: true,
        isUnlocked: false,
        unlockedAt: null,
        consoleLocalUserName: null,
      },
    ];

    function buildOverview() {
      return {
        providers: structuredClone(state.achievementProviders),
        sets: synced ? structuredClone(syncedSets) : [],
        trophies: synced ? structuredClone(syncedTrophies) : [],
        recentUnlocks: synced ? [structuredClone(syncedTrophies[1])] : [],
        totals: synced
          ? {
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
            }
          : {
              totalSets: 0,
              completedSets: 0,
              totalTrophies: 0,
              unlockedTrophies: 0,
              bronzeTotal: 0,
              bronzeUnlocked: 0,
              silverTotal: 0,
              silverUnlocked: 0,
              goldTotal: 0,
              goldUnlocked: 0,
              platinumTotal: 0,
              platinumUnlocked: 0,
              completionPercent: 0,
            },
      };
    }

    (window as unknown as Record<string, unknown>).__UCH_TEST_TRANSPORT__ = {
      kind: 'injected_test',
      async invoke(command: string, args?: Record<string, unknown>) {
        if (command === 'get_hub_bootstrap') return structuredClone(state);
        if (command === 'get_active_session') {
          return {
            phase: 'idle',
            sessionId: null,
            target: null,
            consoleSlug: null,
            gameId: null,
            gameTitle: null,
            emulatorSlug: null,
            pid: null,
            startedAt: null,
            lastHeartbeatAt: null,
            elapsedSeconds: 0,
          };
        }
        if (command === 'get_trophy_center_overview') return buildOverview();
        if (command === 'sync_console_trophies') {
          const wasSynced = synced;
          synced = true;
          if (state.games[0]) {
            state.games[0].completionPercent = 33;
          }
          state.statistics.totalTrophies = 3;
          state.statistics.unlockedTrophies = 1;
          return {
            consoleSlug: String(args?.consoleSlug ?? 'ps3'),
            setsSynced: 1,
            trophiesSynced: 3,
            unlocksRecorded: 1,
            newlyUnlocked: wasSynced
              ? []
              : [
                  {
                    trophyId: 'tr-2',
                    externalTrophyId: '001',
                    gameId: 'game-ps3-1',
                    gameTitle: "Demon's Souls",
                    name: "Phalanx's Trophy",
                    category: 'bronze',
                    unlockedAt: '2026-11-25T20:00:00Z',
                    iconPath: null,
                  },
                ],
            iconsCached: 2,
            diagnostics: [],
            syncedAt: '2026-11-25T23:10:00Z',
          };
        }
        if (command === 'list_game_trophies') {
          return synced ? structuredClone(syncedTrophies) : [];
        }
        if (command === 'get_game_details') {
          return {
            game: structuredClone(state.games[0]),
            versions: [],
            sessions: [],
          };
        }
        if (command === 'get_console_environment') {
          return {
            console: structuredClone(state.consoles[0]),
            emulator: structuredClone(state.emulators[0]),
            environmentMode: 'authentic_system_capable',
            games: structuredClone(state.games),
            scanPaths: [],
            settings: [],
            firmwareRecords: [],
            runtimeVerifications: [],
            rpcs3Inspection: structuredClone(state.rpcs3Inspection),
          };
        }
        if (command === 'get_console_profile') {
          return {
            consoleSlug: 'ps3',
            consoleName: 'PlayStation 3',
            environmentMode: 'authentic_system_capable',
            profile: {
              id: 'cp-ps3',
              consoleId: 'console-ps3',
              consoleSlug: 'ps3',
              emulatorId: 'emu-rpcs3',
              emulatorSlug: 'rpcs3',
              emulatorStatus: 'available',
              detectedEmulatorVersion: 'RPCS3 v0.0.34-17105',
              detectedFirmwareVersion: '4.91',
              emulatorRootDir: 'D:\\Emulators\\RPCS3',
              nativeSaveCount: 2,
              nativeTrophyDirPresent: true,
              trophySyncReadiness: 'ready',
              nativeMetadataJson: '{}',
              syncStatus: 'synced',
              syncDiagnostics: '',
              lastSyncedAt: '2026-11-25T23:00:00Z',
              updatedAt: '2026-11-25T23:00:00Z',
            },
            localUsers: [],
            emulatorStatus: { value: 'available', origin: 'native_platform', notes: null },
            emulatorVersion: { value: 'RPCS3 v0.0.34-17105', origin: 'native_platform', notes: null },
            firmwareVersion: { value: '4.91', origin: 'native_platform', notes: null },
            emulatorRootDir: { value: 'D:\\Emulators\\RPCS3', origin: 'native_platform', notes: null },
            nativeSaveCount: { value: 2, origin: 'native_platform', notes: null },
            trophyReadiness: {
              value: 'ready',
              origin: 'native_platform',
              notes: 'Leitor local RPCS3 pronto (TROPCONF.SFM / TROPUSR.DAT somente leitura).',
            },
            hubStats: {
              totalPlaytimeSeconds: 3600,
              totalSessions: 1,
              gamesTotal: 1,
              gamesAvailable: 1,
              gamesMissingOnDisk: 0,
              mostPlayed: [],
              recentlyPlayed: [],
            },
            recentSessions: [],
          };
        }
        throw new Error(`Unhandled Phase 4 E2E command: ${command}`);
      },
    };
  });
});

test('Phase 4 UI flows: sync RPCS3 trophies, toast notification, spoiler toggle, filters, GameDetails trophies, and PS3 profile summary', async ({
  page,
}) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Visão Geral do Hub' })).toBeVisible();

  // 1. Open Central de Troféus
  await page.getByRole('button', { name: 'Central de Troféus', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Central de Troféus' })).toBeVisible();

  // 2. Trigger manual sync -> shows newly unlocked toast + updates completion & sets
  await page.getByRole('button', { name: 'Sincronizar Troféus Agora' }).click();
  await expect(page.getByLabel('Novos troféus desbloqueados')).toBeVisible();
  await expect(page.getByText("Novo troféu desbloqueado: Phalanx's Trophy")).toBeVisible();
  await expect(page.getByText('NPWR00637_00')).toBeVisible();
  await expect(page.getByText('Local / Offline').first()).toBeVisible();
  await expect(page.getByText(/Usuário local: ClemsonPS3/).first()).toBeVisible();

  // 3. Hidden locked trophy masked by default -> revealed explicitly via toggle
  await expect(page.getByText('Defeat the hidden boss cleanly')).toHaveCount(0);
  await expect(page.getByText('Troféu Oculto')).toBeVisible();
  await page.getByLabel(/Revelar troféus ocultos/i).check();
  await expect(page.getByText('Secret Boss Trophy')).toBeVisible();
  await expect(page.getByText('Defeat the hidden boss cleanly')).toBeVisible();

  // 4. Filter by status = Desbloqueados hides locked Platinum
  await page.getByLabel('Filtrar por status').selectOption('unlocked');
  await expect(page.getByText('Toughest Soul Trophy')).toHaveCount(0);

  // 5. Open GameDetailsPage directly from trophy set and confirm per-game trophies section
  await page.getByRole('button', { name: 'Abrir Jogo' }).click();
  await expect(page.getByRole('heading', { name: "Demon's Souls" })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Troféus do Jogo (1 / 3)' })).toBeVisible();

  // 6. Navigate to PS3 Console Profile and confirm live PS3 trophy summary
  await page.getByRole('button', { name: 'Voltar' }).click();
  await page.getByRole('button', { name: 'Consoles', exact: true }).click();
  await page.getByRole('button', { name: 'Abrir detalhes de PlayStation 3' }).click();
  await page.getByRole('tab', { name: /Perfil & Usuários Locais/i }).click();
  await expect(page.getByText(/Troféus PS3 \(RPCS3 local\): 1 \/ 3 \(33%\)/)).toBeVisible();
});
