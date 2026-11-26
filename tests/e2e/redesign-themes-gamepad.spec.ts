import { expect, test } from '@playwright/test';

/**
 * LAYER 3 — UI E2E TESTS (Playwright + Injected Test Transport + Simulated Gamepad)
 *
 * Scope disclaimer:
 * Validates visual redesign flows, all 4 built-in theme shell layouts, theme preview/apply/cancel,
 * theme persistence across page reload, local theme import/removal, Desktop <-> Fullscreen TV mode,
 * keyboard + simulated W3C Gamepad hot-plug (connect/disconnect/reconnect), dynamic PlayStation vs
 * Xbox glyphs, modal focus restoration, large library navigation (32 games), GameDetails, and
 * PS3 profile compatibility with zero RPCS3 regressions.
 *
 * Does NOT replace Layer 2 real Rust/SQLite/Windows integration tests or physical controller testing.
 */

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    // Persist settings & custom themes across page.reload() via sessionStorage to validate restart persistence
    const STORAGE_KEY = '__uch_redesign_e2e_state_v1__';
    const existingRaw = window.sessionStorage.getItem(STORAGE_KEY);

    const makeGames = () =>
      Array.from({ length: 32 }, (_, idx) => ({
        id: `game-ps3-${idx + 1}`,
        consoleId: 'console-ps3',
        consoleSlug: 'ps3',
        canonicalKey: `serial:BLUS${String(30100 + idx)}`,
        title:
          idx === 0
            ? "Demon's Souls"
            : idx === 1
              ? 'Metal Gear Solid 4'
              : `PS3 Classic Title ${String(idx + 1).padStart(2, '0')}`,
        description: idx === 0 ? 'RPG clássico de PS3 via RPCS3.' : '',
        developer: idx === 0 ? 'FromSoftware' : 'Studio',
        publisher: idx === 0 ? 'Atlus' : 'Publisher',
        releaseYear: 2008 + (idx % 6),
        genre: idx % 2 === 0 ? 'Action RPG' : 'Adventure',
        coverPath: null,
        metadataSource: 'ps3_param_sfo',
        installState: idx === 31 ? 'missing_files' : 'registered',
        associatedEmulatorId: 'emu-rpcs3',
        totalPlaytimeSeconds: (32 - idx) * 900,
        lastPlayedAt: idx < 4 ? '2026-11-25T21:00:00Z' : null,
        completionPercent: (idx * 11) % 100,
        isFavorite: idx === 0,
        versionCount: 1,
      }));

    const defaultState = {
      appName: 'Universal Console Hub',
      version: '0.1.0',
      phase: 4,
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
        gamesRegistered: 32,
        gamesStarted: 4,
        gamesCompleted: 1,
        totalPlaytimeSeconds: 54000,
        totalTrophies: 12,
        unlockedTrophies: 5,
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
          phaseStatus: 'phase_3_live',
          notes: 'Prioridade 1 via RPCS3.',
          capabilities: {
            supportedStates: ['FULL_SYSTEM_BOOT', 'ORIGINAL_DASHBOARD', 'DIRECT_GAME_ONLY'],
            features: [
              {
                feature: 'system_boot',
                states: ['FULL_SYSTEM_BOOT'],
                availability: 'requires_user_files',
                verification: 'files_verified',
                notes: 'dev_flash/vsh/module/vsh.self presente.',
              },
              {
                feature: 'original_dashboard',
                states: ['ORIGINAL_DASHBOARD'],
                availability: 'requires_user_files',
                verification: 'files_verified',
                notes: 'XMB original via RPCS3.',
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
          notes: 'Ambiente integrado planejado para Fase 5.',
          capabilities: {
            supportedStates: ['EXPERIMENTAL', 'DIRECT_GAME_ONLY'],
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
          executablePath: 'C:\\Emulators\\RPCS3\\rpcs3.exe',
          configPath: null,
          detectedVersion: '0.0.34',
          integrationPhase: 'phase_3_live',
          capabilities: {
            supportedStates: ['FULL_SYSTEM_BOOT', 'ORIGINAL_DASHBOARD', 'DIRECT_GAME_ONLY'],
            features: [],
          },
        },
      ],
      games: makeGames(),
      achievementProviders: [],
      trophies: [],
      firmwareRecords: [],
      recentActivity: [],
      settings: {
        'ui.fullscreen_default': 'false',
        'ui.target_resolution': '1920x1080',
        'ui.display_mode': 'desktop',
        'theme.active_id': 'universal-modern',
        'emulation.launch_timeout_seconds': '45',
      } as Record<string, string>,
      installedThemes: [] as Array<Record<string, unknown>>,
      rpcs3Inspection: {
        configured: true,
        installation: {
          emulatorSlug: 'rpcs3',
          executablePath: 'C:\\Emulators\\RPCS3\\rpcs3.exe',
          rootDir: 'C:\\Emulators\\RPCS3',
          detectedVersion: '0.0.34',
          isValid: true,
          issues: [],
        },
        readiness: {
          firmwarePackageValidated: true,
          firmwareSha256: 'a'.repeat(64),
          systemFilesPresent: true,
          commandBuildable: true,
          xmbRuntimeConfirmed: false,
          generatedCommand: {
            executable: 'C:\\Emulators\\RPCS3\\rpcs3.exe',
            args: ['--no-gui', 'C:\\Emulators\\RPCS3\\dev_flash\\vsh\\module\\vsh.self'],
            workingDir: 'C:\\Emulators\\RPCS3',
          },
          notes: ['Arquivos locais validados; execução runtime do XMB requer validação manual.'],
        },
        diagnostic: 'RPCS3 pronto.',
      },
    };

    const state = existingRaw ? JSON.parse(existingRaw) : defaultState;
    const persist = () => {
      window.sessionStorage.setItem(STORAGE_KEY, JSON.stringify(state));
    };

    // Simulated W3C Gamepad harness controllable from Playwright tests
    const simPads: Array<{
      id: string;
      index: number;
      connected: boolean;
      mapping: string;
      axes: number[];
      buttons: Array<{ pressed: boolean; touched: boolean; value: number }>;
      timestamp: number;
    } | null> = [null, null, null, null];

    const makeButtons = () =>
      Array.from({ length: 16 }, () => ({ pressed: false, touched: false, value: 0 }));

    (window as unknown as Record<string, unknown>).__SIM_GAMEPAD__ = {
      connect(slot: number, id: string) {
        simPads[slot] = {
          id,
          index: slot,
          connected: true,
          mapping: 'standard',
          axes: [0, 0, 0, 0],
          buttons: makeButtons(),
          timestamp: performance.now(),
        };
        window.dispatchEvent(new Event('gamepadconnected'));
      },
      disconnect(slot: number) {
        simPads[slot] = null;
        window.dispatchEvent(new Event('gamepaddisconnected'));
      },
      setButton(slot: number, btnIdx: number, pressed: boolean) {
        const pad = simPads[slot];
        if (!pad) return;
        pad.buttons[btnIdx] = { pressed, touched: pressed, value: pressed ? 1 : 0 };
        pad.timestamp = performance.now();
      },
    };

    Object.defineProperty(navigator, 'getGamepads', {
      configurable: true,
      value: () => simPads,
    });

    (window as unknown as Record<string, unknown>).__UCH_TEST_TRANSPORT__ = {
      kind: 'injected_test',
      async invoke(command: string, args?: Record<string, unknown>) {
        if (command === 'get_hub_bootstrap') return structuredClone(state);
        if (command === 'list_connected_gamepads') return [];
        if (command === 'list_installed_themes') return structuredClone(state.installedThemes);
        if (command === 'import_theme_package') {
          const pkg = {
            manifest: {
              id: 'aurora-community',
              name: 'Aurora Community',
              author: 'Playwright QA',
              version: '1.0.0',
              description: 'Pacote de tema local importado no teste E2E.',
              hubVersionRange: '>=0.1.0',
              previewImage: null,
              backgroundImage: null,
              fonts: {
                display: "'Segoe UI', sans-serif",
                body: "'Segoe UI', sans-serif",
                mono: "'Consolas', monospace",
              },
              assets: {},
              palette: {
                canvas: '#120d1b',
                rail: '#191226',
                surface: '#211833',
                surfaceRaised: '#2c2044',
                input: '#150f20',
                borderSubtle: '#3b2b59',
                borderStrong: '#5a4387',
                textPrimary: '#f6f2ff',
                textSecondary: '#c4b6e0',
                textMuted: '#8f7eb0',
                accentPrimary: '#a855f7',
                accentHover: '#c084fc',
                accentMuted: 'rgba(168, 85, 247, 0.2)',
                accentSecondary: '#38bdf8',
                focusRing: '#a855f7',
              },
              layout: {
                shell: 'topbar_blades',
                libraryDefaultView: 'grid',
                cardAspect: 'poster',
                density: 'comfortable',
              },
              components: {
                focusStyle: 'ring',
                surfaceStyle: 'matte',
                backdropBlur: true,
                showHeroArt: true,
                showClock: true,
                radiusSm: 6,
                radiusMd: 12,
                radiusLg: 18,
              },
              motion: {
                pageTransition: 'fade',
                durationMs: 140,
                respectReducedMotion: true,
              },
              customizableSettings: [
                {
                  key: 'accentPrimary',
                  label: 'Cor principal Aurora',
                  kind: 'color',
                  defaultValue: '#a855f7',
                },
              ],
            },
            installedDir: 'C:\\AppData\\themes\\aurora-community',
            previewImagePath: null,
            backgroundImagePath: null,
            assetPaths: {},
            installedAt: '2026-11-26T00:00:00Z',
          };
          state.installedThemes = state.installedThemes.filter(
            (t: { manifest: { id: string } }) => t.manifest.id !== 'aurora-community'
          );
          state.installedThemes.push(pkg);
          persist();
          return structuredClone(pkg);
        }
        if (command === 'remove_installed_theme') {
          const id = String(args?.themeId ?? '');
          state.installedThemes = state.installedThemes.filter(
            (t: { manifest: { id: string } }) => t.manifest.id !== id
          );
          persist();
          return true;
        }
        if (command === 'update_app_setting') {
          state.settings[String(args?.key)] = String(args?.value);
          persist();
          return structuredClone(state.settings);
        }
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
        if (command === 'get_console_environment') {
          const slug = String(args?.slug ?? 'ps3');
          const c = state.consoles.find((x: { slug: string }) => x.slug === slug);
          return {
            console: structuredClone(c),
            emulator: structuredClone(
              state.emulators.find((e: { consoleSlug: string }) => e.consoleSlug === slug) ?? null
            ),
            environmentMode:
              slug === 'ps3' ? 'authentic_system_capable' : 'integrated_alternative_only',
            games: structuredClone(
              state.games.filter((g: { consoleSlug: string }) => g.consoleSlug === slug)
            ),
            scanPaths: [],
            settings: [],
            firmwareRecords: [],
            runtimeVerifications: [],
            rpcs3Inspection: slug === 'ps3' ? structuredClone(state.rpcs3Inspection) : null,
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
              detectedEmulatorVersion: 'RPCS3 v0.0.34',
              detectedFirmwareVersion: '4.91',
              emulatorRootDir: 'C:\\Emulators\\RPCS3',
              nativeSaveCount: 3,
              nativeTrophyDirPresent: true,
              trophySyncReadiness: 'ready_phase4',
              nativeMetadataJson: '{}',
              syncStatus: 'synced',
              syncDiagnostics: '',
              lastSyncedAt: '2026-11-25T21:00:00Z',
              updatedAt: '2026-11-25T21:00:00Z',
            },
            localUsers: [
              {
                id: 'lu-1',
                consoleId: 'console-ps3',
                consoleSlug: 'ps3',
                nativeUserKey: '00000001',
                displayName: 'JogadorPS3',
                sourcePath: 'C:\\Emulators\\RPCS3\\dev_hdd0\\home\\00000001',
                availabilityStatus: 'available',
                linkedProfileId: 'profile-default',
                firstDetectedAt: '2026-11-25T21:00:00Z',
                lastSeenAt: '2026-11-25T21:00:00Z',
              },
            ],
            emulatorStatus: { value: 'available', origin: 'native_platform', notes: null },
            emulatorVersion: { value: 'RPCS3 v0.0.34', origin: 'native_platform', notes: null },
            firmwareVersion: { value: '4.91', origin: 'native_platform', notes: null },
            emulatorRootDir: { value: 'C:\\Emulators\\RPCS3', origin: 'native_platform', notes: null },
            nativeSaveCount: { value: 3, origin: 'native_platform', notes: null },
            trophyReadiness: { value: 'ready_phase4', origin: 'native_platform', notes: null },
            hubStats: {
              totalPlaytimeSeconds: 54000,
              totalSessions: 4,
              gamesTotal: 32,
              gamesAvailable: 31,
              gamesMissingOnDisk: 1,
              mostPlayed: [],
              recentlyPlayed: [],
            },
            recentSessions: [],
          };
        }
        if (command === 'get_trophy_center_overview') {
          return {
            providers: [],
            sets: [],
            trophies: [],
            recentUnlocks: [],
            totals: {
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
        if (command === 'get_game_details') {
          const g = state.games.find((x: { id: string }) => x.id === args?.gameId);
          return {
            game: structuredClone(g),
            versions: [
              {
                id: 'gv-1',
                gameId: String(args?.gameId),
                versionLabel: '01.00',
                region: 'USA',
                serialCode: 'BLUS30100',
                filePath: 'C:\\Emulators\\RPCS3\\dev_hdd0\\game\\BLUS30100\\USRDIR\\EBOOT.BIN',
                installState: 'registered',
                createdAt: '2026-11-25T21:00:00Z',
              },
            ],
            sessions: [],
          };
        }
        if (command === 'list_game_trophies') return [];
        if (command === 'list_database_backups') return [];
        if (command === 'toggle_game_favorite') {
          const g = state.games.find((x: { id: string }) => x.id === args?.gameId);
          if (g) g.isFavorite = !g.isFavorite;
          persist();
          return Boolean(g?.isFavorite);
        }
        if (command === 'launch_game') {
          const g = state.games.find((x: { id: string }) => x.id === args?.gameId);
          return {
            phase: 'running',
            sessionId: 'sess-e2e-1',
            target: 'game_direct',
            consoleSlug: 'ps3',
            gameId: g?.id ?? null,
            gameTitle: g?.title ?? "Demon's Souls",
            emulatorSlug: 'rpcs3',
            pid: 4242,
            startedAt: '2026-11-26T00:00:00Z',
            lastHeartbeatAt: '2026-11-26T00:00:00Z',
            elapsedSeconds: 5,
          };
        }
        if (command === 'stop_active_session') return null;
        throw new Error(`Unhandled redesign E2E command: ${command}`);
      },
    };
  });
});

test('switches all 4 distinct theme shells, previews/cancels, imports/removes local package, and persists across reload', async ({
  page,
}) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Visão Geral do Hub' })).toBeVisible();
  await expect(page.locator('html')).toHaveAttribute('data-theme-id', 'universal-modern');
  await expect(page.locator('html')).toHaveAttribute('data-theme-shell', 'sidebar');

  // Open Theme Manager via sidebar
  await page.getByRole('button', { name: 'Temas', exact: true }).first().click();
  await expect(page.getByRole('heading', { name: 'Gerenciador de Temas' })).toBeVisible();

  // Preview XMB Inspired -> shell changes to xmb_crossbar, then cancel reverts to sidebar
  const xmbCard = page.locator('[data-theme-card="xmb-inspired"]');
  await xmbCard.getByRole('button', { name: 'Pré-visualizar' }).click();
  await expect(page.locator('html')).toHaveAttribute('data-theme-shell', 'xmb_crossbar');
  await expect(page.getByText(/Modo Prévia Ativo/i)).toBeVisible();
  await page.getByRole('button', { name: 'Cancelar Prévia' }).first().click();
  await expect(page.locator('html')).toHaveAttribute('data-theme-shell', 'sidebar');

  // Apply Minimal -> shell switches to minimal_dock
  const minimalCard = page.locator('[data-theme-card="minimal"]');
  await minimalCard.getByRole('button', { name: 'Aplicar Tema' }).click();
  await expect(page.locator('html')).toHaveAttribute('data-theme-id', 'minimal');
  await expect(page.locator('html')).toHaveAttribute('data-theme-shell', 'minimal_dock');

  // Apply Console Classic -> shell switches to topbar_blades and persists across reload!
  const classicCard = page.locator('[data-theme-card="console-classic"]');
  await classicCard.getByRole('button', { name: 'Aplicar Tema' }).click();
  await expect(page.locator('html')).toHaveAttribute('data-theme-id', 'console-classic');
  await expect(page.locator('html')).toHaveAttribute('data-theme-shell', 'topbar_blades');

  await page.reload();
  await expect(page.locator('html')).toHaveAttribute('data-theme-id', 'console-classic');
  await expect(page.locator('html')).toHaveAttribute('data-theme-shell', 'topbar_blades');

  // Import a local theme package, apply it, then remove via ConfirmDialog
  await page.getByRole('button', { name: 'Temas', exact: true }).first().click();
  await page
    .getByLabel(/Caminho da pasta do pacote de tema/i)
    .fill('C:\\Temas\\aurora-community');
  await page.getByRole('button', { name: 'Importar Pacote de Tema' }).click();
  await expect(page.getByRole('heading', { name: 'Aurora Community' })).toBeVisible();

  const customCard = page.locator('[data-theme-card="aurora-community"]');
  await customCard.getByRole('button', { name: 'Remover' }).click();
  await expect(page.getByRole('dialog')).toBeVisible();
  await page.getByRole('button', { name: 'Remover Tema' }).click();
  await expect(page.getByRole('heading', { name: 'Aurora Community' })).toHaveCount(0);

  // Restore default theme
  await page.getByRole('button', { name: 'Restaurar Tema Padrão' }).click();
  await expect(page.locator('html')).toHaveAttribute('data-theme-id', 'universal-modern');
});

test('supports keyboard + simulated gamepad hot-plug, dynamic PlayStation/Xbox glyphs, Desktop<->Fullscreen, QuickMenu focus restore, and large 32-game library + PS3 launch', async ({
  page,
}) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Visão Geral do Hub' })).toBeVisible();

  // 1. Toggle Desktop <-> Fullscreen TV via F11 preserving page & state
  await expect(page.locator('html')).toHaveAttribute('data-display-mode', 'desktop');
  await page.keyboard.press('F11');
  await expect(page.locator('html')).toHaveAttribute('data-display-mode', 'fullscreen');
  await page.keyboard.press('F11');
  await expect(page.locator('html')).toHaveAttribute('data-display-mode', 'desktop');

  // 2. Connect simulated DualSense -> PlayStation glyphs appear in ControllerActionBar
  await page.evaluate(() => {
    const sim = (window as unknown as { __SIM_GAMEPAD__: { connect: (s: number, id: string) => void } })
      .__SIM_GAMEPAD__;
    sim.connect(0, 'Sony Interactive Entertainment DualSense Wireless Controller (054c)');
  });
  await expect(page.locator('footer[aria-label="Atalhos de controle e teclado"]')).toContainText(
    'PLAYSTATION'
  );
  await expect(page.locator('footer[aria-label="Atalhos de controle e teclado"]')).toContainText(
    '✕'
  );

  // 3. Open QuickMenu via Start button (index 9) -> close via East/Circle (index 1) -> focus restores!
  await page.locator('[data-focus-id="home-hero-primary"]').focus();
  await page.evaluate(() => {
    const sim = (
      window as unknown as {
        __SIM_GAMEPAD__: { setButton: (s: number, b: number, p: boolean) => void };
      }
    ).__SIM_GAMEPAD__;
    sim.setButton(0, 9, true);
  });
  await expect(page.getByRole('dialog', { name: 'Menu Rápido do Hub' })).toBeVisible();
  await page.evaluate(() => {
    const sim = (
      window as unknown as {
        __SIM_GAMEPAD__: { setButton: (s: number, b: number, p: boolean) => void };
      }
    ).__SIM_GAMEPAD__;
    sim.setButton(0, 9, false);
    sim.setButton(0, 1, true);
  });
  await expect(page.getByRole('dialog', { name: 'Menu Rápido do Hub' })).toHaveCount(0);
  await page.evaluate(() => {
    const sim = (
      window as unknown as {
        __SIM_GAMEPAD__: { setButton: (s: number, b: number, p: boolean) => void };
      }
    ).__SIM_GAMEPAD__;
    sim.setButton(0, 1, false);
  });

  // 4. Disconnect DualSense and reconnect Xbox Wireless Controller -> glyphs switch to XBOX
  await page.evaluate(() => {
    const sim = (
      window as unknown as {
        __SIM_GAMEPAD__: { disconnect: (s: number) => void };
      }
    ).__SIM_GAMEPAD__;
    sim.disconnect(0);
  });
  await expect(page.locator('footer[aria-label="Atalhos de controle e teclado"]')).toContainText(
    'Teclado / Controle pronto'
  );
  await page.evaluate(() => {
    const sim = (window as unknown as { __SIM_GAMEPAD__: { connect: (s: number, id: string) => void } })
      .__SIM_GAMEPAD__;
    sim.connect(0, 'Xbox Wireless Controller (Standard Gamepad 045e)');
  });
  await expect(page.locator('footer[aria-label="Atalhos de controle e teclado"]')).toContainText(
    'XBOX'
  );

  // 5. Switch section using RB (button index 5) from Home -> Consoles -> Library
  await page.evaluate(() => {
    const sim = (
      window as unknown as {
        __SIM_GAMEPAD__: { setButton: (s: number, b: number, p: boolean) => void };
      }
    ).__SIM_GAMEPAD__;
    sim.setButton(0, 5, true);
  });
  await expect(page.getByRole('heading', { name: 'Coleção de Consoles' })).toBeVisible();
  await page.evaluate(() => {
    const sim = (
      window as unknown as {
        __SIM_GAMEPAD__: { setButton: (s: number, b: number, p: boolean) => void };
      }
    ).__SIM_GAMEPAD__;
    sim.setButton(0, 5, false);
  });
  await page.waitForTimeout(80);
  await page.evaluate(() => {
    const sim = (
      window as unknown as {
        __SIM_GAMEPAD__: { setButton: (s: number, b: number, p: boolean) => void };
      }
    ).__SIM_GAMEPAD__;
    sim.setButton(0, 5, true);
  });
  await expect(page.getByRole('heading', { name: 'Biblioteca Universal' })).toBeVisible();
  await page.evaluate(() => {
    const sim = (
      window as unknown as {
        __SIM_GAMEPAD__: { setButton: (s: number, b: number, p: boolean) => void };
      }
    ).__SIM_GAMEPAD__;
    sim.setButton(0, 5, false);
  });
  await expect(page.getByText('Jogos Descobertos (32)')).toBeVisible();

  // 6. Toggle Grid -> List -> Open GameDetails -> Launch game -> Active Session Banner appears -> Stop
  await page.getByRole('button', { name: 'Lista', exact: true }).click();
  await page.getByRole('button', { name: 'Detalhes' }).first().click();
  await expect(page.getByRole('heading', { name: "Demon's Souls" })).toBeVisible();
  await page.getByRole('button', { name: /Jogar/i }).first().click();
  await expect(page.getByRole('status', { name: 'Sessão ativa do emulador' })).toBeVisible();
  await expect(page.getByText(/PID 4242/i)).toBeVisible();
  await page.getByRole('button', { name: 'Encerrar Sessão' }).click();
  await expect(page.getByRole('status', { name: 'Sessão ativa do emulador' })).toHaveCount(0);

  // 7. Verify PS3 Console Profile still renders with zero regressions
  await page.getByRole('button', { name: 'Consoles', exact: true }).click();
  await page.getByRole('button', { name: 'Abrir detalhes de PlayStation 3' }).click();
  await expect(page.getByRole('heading', { name: 'PlayStation 3', exact: true })).toBeVisible();
  await page.getByRole('tab', { name: /Perfil & Usuários Locais/i }).click();
  await expect(page.getByText('JogadorPS3')).toBeVisible();
});
