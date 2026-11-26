import { expect, test } from '@playwright/test';

/**
 * LAYER 3 — UI E2E TESTS (Playwright + Injected Test Transport) — PHASE 2
 *
 * Scope disclaimer:
 * Validates browser UI flows, independent ConsoleEnvironment views (PS3 Mode A vs PS4 Mode B),
 * scanner progress UX, GameDetailsPage, StatisticsPage, sticky sidebar CSS, and ConfirmDialog
 * backup restore confirmation using `window.__UCH_TEST_TRANSPORT__`.
 * Does NOT replace Layer 2 real Rust/SQLite/Windows integration tests or the live Windows `.exe` audit.
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
        gamesRegistered: 0,
        gamesStarted: 0,
        gamesCompleted: 0,
        totalPlaytimeSeconds: 0,
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
          phaseStatus: 'phase_2_console_environment',
          notes: 'Prioridade 1 via RPCS3. Execução ao vivo na Fase 3.',
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
        {
          id: 'console-ps4',
          slug: 'ps4',
          name: 'PlayStation 4',
          manufacturer: 'Sony Interactive Entertainment',
          generation: 8,
          releaseYear: 2013,
          phaseStatus: 'planned_phase_5_experimental',
          notes: 'Integração experimental via shadPS4 planejada para Fase 5.',
          capabilities: {
            supportedStates: ['EXPERIMENTAL', 'DIRECT_GAME_ONLY'],
            features: [
              {
                feature: 'original_dashboard',
                states: ['UNSUPPORTED'],
                availability: 'unsupported',
                verification: 'documented_upstream',
                notes: 'Orbis OS completo não é suportado.',
              },
            ],
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
          executablePath: null,
          configPath: null,
          detectedVersion: null,
          integrationPhase: 'phase_2_console_environment',
          capabilities: {
            supportedStates: ['FULL_SYSTEM_BOOT', 'ORIGINAL_DASHBOARD', 'DIRECT_GAME_ONLY'],
            features: [],
          },
        },
      ],
      games: [] as Array<Record<string, unknown>>,
      achievementProviders: [],
      trophies: [],
      firmwareRecords: [],
      recentActivity: [
        {
          id: 'act-1',
          eventType: 'library_scanned',
          entityType: 'console',
          entityId: 'console-ps3',
          summary: 'Scan inicial de PlayStation 3 pronto.',
          occurredAt: '2026-11-25T21:00:00Z',
        },
      ],
      settings: {
        'ui.fullscreen_default': 'false',
        'ui.target_resolution': '1920x1080',
        'emulation.launch_timeout_seconds': '45',
      } as Record<string, string>,
      rpcs3Inspection: {
        configured: false,
        installation: null,
        readiness: null,
        diagnostic: 'RPCS3 ainda não configurado.',
      },
    };

    const scanPaths: Array<Record<string, unknown>> = [];
    const backups: Array<{ entry: Record<string, unknown>; snapshot: typeof state }> = [];

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
        if (command === 'get_console_profile') return null;
        if (command === 'get_console_environment') {
          const slug = String(args?.slug ?? '');
          const c = state.consoles.find((x) => x.slug === slug);
          const isAuth =
            c?.capabilities.supportedStates.includes('FULL_SYSTEM_BOOT') &&
            c?.capabilities.supportedStates.includes('ORIGINAL_DASHBOARD');
          return {
            console: structuredClone(c),
            emulator: structuredClone(state.emulators.find((e) => e.consoleSlug === slug) ?? null),
            environmentMode: isAuth ? 'authentic_system_capable' : 'integrated_alternative_only',
            games: structuredClone(state.games.filter((g) => g.consoleSlug === slug)),
            scanPaths: structuredClone(scanPaths.filter((p) => p.consoleSlug === slug)),
            settings: [],
            firmwareRecords: [],
            runtimeVerifications: [],
            rpcs3Inspection: slug === 'ps3' ? structuredClone(state.rpcs3Inspection) : null,
          };
        }
        if (command === 'add_console_scan_path') {
          const sp = {
            id: `sp-${scanPaths.length + 1}`,
            consoleId: `console-${args?.consoleSlug}`,
            consoleSlug: String(args?.consoleSlug ?? 'ps3'),
            directoryPath: String(args?.directoryPath ?? ''),
            recursive: Boolean(args?.recursive ?? true),
            enabled: true,
            lastScannedAt: null,
            createdAt: '2026-11-25T21:00:00Z',
          };
          scanPaths.push(sp);
          return structuredClone(sp);
        }
        if (command === 'scan_console_library') {
          const onProgress = args?.onProgress as ((evt: unknown) => void) | undefined;
          onProgress?.({ event: 'started', data: { consoleSlug: 'ps3', totalDirectories: 1 } });
          if (state.games.length === 0) {
            state.games.push({
              id: 'game-ps3-1',
              consoleId: 'console-ps3',
              consoleSlug: 'ps3',
              canonicalKey: 'serial:BLUS30443',
              title: "Demon's Souls",
              description: '',
              developer: null,
              publisher: null,
              releaseYear: null,
              genre: null,
              coverPath: null,
              metadataSource: 'ps3_param_sfo',
              installState: 'registered',
              associatedEmulatorId: 'emu-rpcs3',
              totalPlaytimeSeconds: 0,
              lastPlayedAt: null,
              completionPercent: 0,
              isFavorite: false,
              versionCount: 1,
            });
            state.statistics.gamesRegistered = 1;
          }
          onProgress?.({
            event: 'completed',
            data: { scannedFiles: 3, gamesAdded: 1, versionsAdded: 1, skippedExisting: 0 },
          });
          return {
            consoleSlug: 'ps3',
            scannedDirectories: 1,
            scannedFiles: 3,
            gamesAdded: 1,
            versionsAdded: 1,
            skippedExisting: 0,
            errors: [],
          };
        }
        if (command === 'get_game_details') {
          const g = state.games.find((x) => x.id === args?.gameId);
          return {
            game: structuredClone(g),
            versions: [
              {
                id: 'gv-1',
                gameId: String(args?.gameId),
                versionLabel: '01.00',
                region: 'USA',
                serialCode: 'BLUS30443',
                filePath: 'D:\\Jogos PS3\\DemonsSouls\\PS3_GAME\\USRDIR\\EBOOT.BIN',
                installState: 'registered',
                createdAt: '2026-11-25T21:00:00Z',
              },
            ],
            sessions: [],
          };
        }
        if (command === 'update_game_metadata') {
          const req = args?.request as Record<string, unknown>;
          const g = state.games.find((x) => x.id === req.gameId);
          if (g) {
            g.description = req.description;
            g.developer = req.developer;
            g.completionPercent = req.completionPercent;
          }
          return structuredClone(g);
        }
        if (command === 'get_statistics_overview') {
          return {
            totalPlaytimeSeconds: 0,
            totalSessions: 0,
            gamesRegistered: state.games.length,
            gamesStarted: 0,
            gamesCompleted: 0,
            completionRatePercent: 0,
            byConsole: state.consoles.map((c) => ({
              consoleSlug: c.slug,
              consoleName: c.name,
              gamesCount: state.games.filter((g) => g.consoleSlug === c.slug).length,
              sessionsCount: 0,
              totalPlaytimeSeconds: 0,
            })),
            topGames: [],
            monthlyPlaytime: [],
          };
        }
        if (command === 'list_activity_history') return structuredClone(state.recentActivity);
        if (command === 'create_database_backup') {
          const entry = {
            fileName: 'e2e_phase2_snap.db',
            backupPath: 'C:\\Backups\\e2e_phase2_snap.db',
            sizeBytes: 131072,
            modifiedAt: '2026-11-25T22:00:00Z',
          };
          backups.unshift({ entry, snapshot: structuredClone(state) });
          return {
            backupPath: entry.backupPath,
            sizeBytes: entry.sizeBytes,
            integrityOk: true,
            createdAt: entry.modifiedAt,
          };
        }
        if (command === 'list_database_backups') return structuredClone(backups.map((b) => b.entry));
        if (command === 'inspect_database_backup') {
          return {
            backupPath: String(args?.backupPath ?? ''),
            sizeBytes: 131072,
            integrityOk: true,
            schemaVersion: 2,
            profilesCount: 1,
            gamesCount: backups[0]?.snapshot.games.length ?? 0,
            inspectedAt: '2026-11-25T22:05:00Z',
          };
        }
        if (command === 'restore_database_backup') {
          return {
            sourceBackupPath: String(args?.backupPath ?? ''),
            restoredDbPath: 'C:\\App\\universal_console_hub.db',
            preRestoreSnapshotPath: 'C:\\Backups\\pre_restore\\pre_restore_e2e.db',
            integrityOk: true,
            schemaVersion: 2,
            restoredAt: '2026-11-25T22:06:00Z',
          };
        }
        throw new Error(`Unhandled command in phase2 E2E: ${command}`);
      },
    };
  });
});

test('Phase 2 UI flows: independent PS3/PS4 environments, scanner, GameDetails, Statistics, sticky sidebar, and confirmed backup restore', async ({
  page,
}) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Visão Geral do Hub' })).toBeVisible();

  // Verify sticky sidebar CSS on desktop viewport
  const railPosition = await page
    .locator('aside[aria-label="Navegação principal do Hub"]')
    .evaluate((el) => getComputedStyle(el).position);
  expect(railPosition).toBe('sticky');

  // 1. Enter PlayStation 3 independent environment, add scan path, scan, open GameDetailsPage
  await page.getByRole('button', { name: 'Entrar no PlayStation 3' }).click();
  await expect(page.getByRole('heading', { name: 'PlayStation 3', exact: true })).toBeVisible();
  await page.getByRole('tab', { name: /Pastas & Descoberta/i }).click();
  await page.getByLabel('Diretório local de jogos (PS3)').fill('D:\\Jogos PS3');
  await page.getByRole('button', { name: 'Adicionar Pasta' }).click();
  await page.getByRole('button', { name: 'Escanear Biblioteca' }).click();
  await expect(page.getByText(/Varredura concluída: 3 arquivos verificados/i)).toBeVisible();

  await page.getByRole('tab', { name: /Jogos do Console/i }).click();
  await expect(page.getByText("Demon's Souls", { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Detalhes do Jogo' }).click();
  await expect(page.getByRole('heading', { name: "Demon's Souls", exact: true })).toBeVisible();
  await page.getByLabel('Progresso de conclusão (0–100%)').fill('80');
  await page.getByRole('button', { name: 'Salvar Metadados' }).click();
  await expect(page.getByText(/Metadados e progresso do jogo salvos/i)).toBeVisible();

  // 2. Verify PlayStation 4 permanent Integrated Alternative banner
  await page.getByRole('button', { name: 'Consoles', exact: true }).click();
  await page.getByRole('button', { name: 'Abrir detalhes de PlayStation 4' }).click();
  await expect(
    page.getByText('Ambiente Integrado do Hub — Este não é o dashboard original do console')
  ).toBeVisible();

  // 3. Statistics & Activity History
  await page.getByRole('button', { name: 'Estatísticas' }).click();
  await expect(page.getByRole('heading', { name: 'Estatísticas & Histórico' })).toBeVisible();

  // 4. Settings: Create backup -> Inspect -> Cancel dialog -> Re-inspect -> Confirm live restore
  await page.getByRole('button', { name: 'Configurações' }).click();
  await page.getByRole('button', { name: 'Criar Backup SQLite' }).click();
  await expect(page.getByText('e2e_phase2_snap.db', { exact: true })).toBeVisible();

  await page.getByRole('button', { name: 'Inspecionar e Restaurar' }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.getByRole('button', { name: 'Cancelar' }).click();
  await expect(page.getByRole('dialog')).toBeHidden();

  await page.getByRole('button', { name: 'Inspecionar e Restaurar' }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.getByRole('button', { name: 'Confirmar Restauração' }).click();
  await expect(page.getByText(/Banco SQLite restaurado com sucesso/i)).toBeVisible();
});
