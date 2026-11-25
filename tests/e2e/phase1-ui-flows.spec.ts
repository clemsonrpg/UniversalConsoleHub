import { expect, test } from '@playwright/test';

/**
 * LAYER 3 — UI E2E TESTS (Playwright + Injected Test Transport)
 *
 * Scope disclaimer (Mandatory Adjustment #4):
 * These tests validate browser UI flows, keyboard navigation, accessible controls,
 * honest multi-capability badges, and 4-stage RPCS3 truthfulness in Chromium using
 * an explicit injected test transport (`window.__UCH_TEST_TRANSPORT__`).
 * They do NOT validate real Tauri 2 IPC or real SQLite persistence (those are
 * validated separately by Layer 2 Rust backend integration tests).
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
          phaseStatus: 'phase_1_prepared_phase_4_full',
          notes: 'Prioridade 1 via RPCS3. Boot ao vivo do XMB reservado à Fase 4.',
          capabilities: {
            supportedStates: ['FULL_SYSTEM_BOOT', 'ORIGINAL_DASHBOARD', 'DIRECT_GAME_ONLY'],
            features: [
              {
                feature: 'system_boot',
                states: ['FULL_SYSTEM_BOOT'],
                availability: 'requires_user_files',
                verification: 'documented_upstream',
                notes: 'Requer PS3UPDAT.PUP instalada em dev_flash/vsh/module/vsh.self.',
              },
              {
                feature: 'original_dashboard',
                states: ['ORIGINAL_DASHBOARD'],
                availability: 'requires_user_files',
                verification: 'documented_upstream',
                notes: 'PS3 XMB original; confirmação runtime reservada à Fase 4.',
              },
              {
                feature: 'online_services',
                states: ['UNSUPPORTED'],
                availability: 'unsupported',
                verification: 'documented_upstream',
                notes: 'PSN oficial não é suportada.',
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
                notes: 'Dashboard Orbis OS completo não é suportado.',
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
          executablePath: null as string | null,
          configPath: null,
          detectedVersion: null,
          integrationPhase: 'phase_1_preparation',
          capabilities: {
            supportedStates: ['FULL_SYSTEM_BOOT', 'ORIGINAL_DASHBOARD', 'DIRECT_GAME_ONLY'],
            features: [],
          },
        },
      ],
      games: [] as Array<Record<string, unknown>>,
      achievementProviders: [
        {
          id: 'prov-local',
          slug: 'local',
          name: 'LocalAchievementProvider',
          origin: 'local',
          availabilityStatus: 'ready_offline',
          supportsAuth: false,
          supportsSync: false,
          notes: 'Provider local offline.',
        },
        {
          id: 'prov-playstation',
          slug: 'playstation',
          name: 'PlayStationTrophyProvider',
          origin: 'official',
          availabilityStatus: 'unavailable_no_official_api',
          supportsAuth: true,
          supportsSync: true,
          notes: 'Indisponível sem API oficial autorizada.',
        },
      ],
      trophies: [],
      firmwareRecords: [] as Array<Record<string, unknown>>,
      recentActivity: [] as Array<Record<string, unknown>>,
      settings: {
        'ui.fullscreen_default': 'false',
        'ui.target_resolution': '1920x1080',
        'ui.reduced_motion': 'false',
        'emulation.launch_timeout_seconds': '45',
      } as Record<string, string>,
      rpcs3Inspection: {
        configured: false,
        installation: null as Record<string, unknown> | null,
        readiness: null as Record<string, unknown> | null,
        diagnostic: 'RPCS3 ainda não configurado.',
      },
    };

    (window as unknown as Record<string, unknown>).__UCH_TEST_TRANSPORT__ = {
      kind: 'injected_test',
      async invoke(command: string, args?: Record<string, unknown>) {
        if (command === 'get_hub_bootstrap') return structuredClone(state);
        if (command === 'get_console_details') {
          const slug = String(args?.slug ?? '');
          const consoleRec = state.consoles.find((c) => c.slug === slug);
          return {
            console: structuredClone(consoleRec),
            emulator: structuredClone(state.emulators.find((e) => e.consoleSlug === slug) ?? null),
            firmwareRecords: structuredClone(state.firmwareRecords),
            rpcs3Inspection: slug === 'ps3' ? structuredClone(state.rpcs3Inspection) : null,
          };
        }
        if (command === 'configure_emulator_path') {
          const exePath = String(args?.exePath ?? '');
          state.emulators[0]!.executablePath = exePath;
          state.rpcs3Inspection = {
            configured: true,
            installation: {
              emulatorSlug: 'rpcs3',
              executablePath: exePath,
              rootDir: 'C:\\Emulators\\RPCS3',
              detectedVersion: null,
              isValid: true,
              issues: [],
            },
            readiness: {
              firmwarePackageValidated: state.firmwareRecords.length > 0,
              firmwareSha256: null,
              systemFilesPresent: true,
              commandBuildable: true,
              xmbRuntimeConfirmed: false,
              generatedCommand: {
                executable: exePath,
                args: ['--no-gui', 'C:\\Emulators\\RPCS3\\dev_flash\\vsh\\module\\vsh.self'],
                workingDir: 'C:\\Emulators\\RPCS3',
              },
              notes: ['Estágio 4/4: Execução real do XMB NÃO confirmada na Fase 1.'],
            },
            diagnostic: 'RPCS3 preparado; boot runtime do XMB reservado à Fase 4.',
          };
          return structuredClone(state.emulators[0]);
        }
        if (command === 'register_firmware_file') {
          const rec = {
            id: 'fw-1',
            consoleSlug: 'ps3',
            emulatorSlug: 'rpcs3',
            fileRole: 'system_firmware_package',
            filePath: String(args?.filePath ?? ''),
            fileSizeBytes: 204800,
            sha256: 'b'.repeat(64),
            validationStatus: 'package_header_and_hash_validated',
            notes: 'Hash validado localmente.',
            validatedAt: '2026-11-25T22:10:00Z',
          };
          state.firmwareRecords.unshift(rec);
          if (state.rpcs3Inspection.readiness) {
            state.rpcs3Inspection.readiness.firmwarePackageValidated = true;
          }
          return structuredClone(rec);
        }
        if (command === 'register_game') {
          const req = args?.request as Record<string, string>;
          const g = {
            id: `game-${state.games.length + 1}`,
            consoleId: 'console-ps3',
            consoleSlug: req.consoleSlug,
            canonicalKey: `title:${req.title}`,
            title: req.title,
            description: '',
            developer: null,
            publisher: null,
            releaseYear: null,
            genre: null,
            coverPath: null,
            metadataSource: req.metadataSource,
            installState: 'registered',
            associatedEmulatorId: 'emu-rpcs3',
            totalPlaytimeSeconds: 0,
            lastPlayedAt: null,
            completionPercent: 0,
            isFavorite: false,
            versionCount: 1,
          };
          state.games.push(g);
          state.statistics.gamesRegistered = state.games.length;
          return structuredClone(g);
        }
        if (command === 'toggle_game_favorite') {
          const g = state.games.find((i) => i.id === args?.gameId);
          if (g) g.isFavorite = !g.isFavorite;
          return Boolean(g?.isFavorite);
        }
        if (command === 'update_active_profile') {
          const req = args?.request as Record<string, string>;
          state.profile.username = req.username ?? state.profile.username;
          state.profile.handle = state.profile.username;
          state.profile.displayName = req.displayName ?? state.profile.displayName;
          state.profile.bio = req.bio ?? state.profile.bio;
          return structuredClone(state.profile);
        }
        if (command === 'update_app_setting') {
          state.settings[String(args?.key)] = String(args?.value);
          return structuredClone(state.settings);
        }
        if (command === 'create_database_backup') {
          return {
            backupPath: 'C:\\Users\\player\\AppData\\Local\\UniversalConsoleHub\\backups\\e2e_snap.db',
            sizeBytes: 65536,
            integrityOk: true,
            createdAt: '2026-11-25T22:15:00Z',
          };
        }
        if (command === 'inspect_rpcs3') return structuredClone(state.rpcs3Inspection);
        throw new Error(`Unhandled test command: ${command}`);
      },
    };
  });
});

test('navigates all Phase 1 sections, displays honest multi-capability badges, and keeps XMB runtime unconfirmed', async ({
  page,
}) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Visão Geral do Hub' })).toBeVisible();
  await expect(page.getByText('Transporte: Transporte de Teste UI')).toBeVisible();

  // Consoles & PS3 multi-capability details
  await page.getByRole('button', { name: 'Consoles', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Coleção de Consoles' })).toBeVisible();
  await page.getByRole('button', { name: 'Abrir detalhes de PlayStation 3' }).click();
  await expect(page.getByText('Matriz de Capacidades por Funcionalidade')).toBeVisible();
  await expect(page.getByText('Boot Completo').first()).toBeVisible();
  await expect(page.getByText('Dashboard Original').first()).toBeVisible();
  await expect(page.getByText('Jogo Direto').first()).toBeVisible();

  // Emulators & RPCS3 4-stage truthfulness check
  await page.getByRole('button', { name: 'Emuladores & Firmware' }).click();
  await page.getByLabel('Caminho local do rpcs3.exe').fill('C:\\Emulators\\RPCS3\\rpcs3.exe');
  await page.getByRole('button', { name: 'Validar e Salvar RPCS3' }).click();
  await expect(page.getByText('Não confirmado (Fase 4)')).toBeVisible();

  await page
    .getByLabel('Validar pacote PS3UPDAT.PUP fornecido pelo usuário (SHA-256 local)')
    .fill('C:\\Firmware\\PS3UPDAT.PUP');
  await page.getByRole('button', { name: 'Validar Firmware Local' }).click();
  await expect(page.getByText('PS3UPDAT.PUP validado')).toBeVisible();
  await expect(page.getByText('Não confirmado (Fase 4)')).toBeVisible();

  // Library registration & favorite toggle
  await page.getByRole('button', { name: 'Biblioteca' }).click();
  await page.getByLabel('Título do jogo').fill('Metal Gear Solid 4');
  await page.getByLabel('Caminho local do jogo').fill('D:\\PS3\\MGS4\\EBOOT.BIN');
  await page.getByRole('button', { name: 'Registrar Jogo' }).click();
  await expect(page.getByText('Metal Gear Solid 4', { exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Favoritar', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Favorito', exact: true })).toBeVisible();

  // Profile persistence in UI state
  await page.getByRole('button', { name: 'Perfil Universal' }).click();
  await page.getByLabel('Nome de exibição').fill('Clemson RPG');
  await page.getByRole('button', { name: 'Salvar Perfil' }).click();
  await expect(page.getByText('Perfil salvo com sucesso no banco local SQLite.')).toBeVisible();

  // Trophies honest provider origins
  await page.getByRole('button', { name: 'Central de Troféus' }).click();
  await expect(page.getByText('LocalAchievementProvider')).toBeVisible();
  await expect(page.getByText('unavailable_no_official_api')).toBeVisible();

  // Settings & SQLite backup trigger
  await page.getByRole('button', { name: 'Configurações' }).click();
  await page.getByRole('button', { name: 'Criar Backup SQLite' }).click();
  await expect(page.getByText(/Backup íntegro gerado via VACUUM INTO/i)).toBeVisible();

  // Keyboard Alt+ArrowUp shortcut moves from Settings back to Emulators
  await page.keyboard.press('Alt+ArrowUp');
  await expect(page.getByRole('heading', { name: 'Emuladores & Firmware' })).toBeVisible();
});
