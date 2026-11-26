import { expect, test } from '@playwright/test';

/**
 * LAYER 3 — UI E2E TESTS (Playwright + Injected Test Transport) — PHASE 3
 *
 * Scope disclaimer:
 * Validates browser UI flows for real Console Profiles (Native vs Hub vs Unavailable badges,
 * separated local console users, zero manual registration form in LibraryPage, PS3 XMB and
 * direct game launch buttons gated by real readiness, ActiveSessionBanner & stop control,
 * and honest PS4 Integrated Alternative boundaries) using `window.__UCH_TEST_TRANSPORT__`.
 * Does NOT replace Layer 2 real Rust/SQLite/Windows OS process tests or the standalone `.exe` audit.
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
            features: [
              {
                feature: 'system_boot',
                states: ['FULL_SYSTEM_BOOT'],
                availability: 'requires_user_files',
                verification: 'files_verified',
                notes: 'PS3UPDAT.PUP instalada em dev_flash/vsh/module/vsh.self.',
              },
              {
                feature: 'original_dashboard',
                states: ['ORIGINAL_DASHBOARD'],
                availability: 'requires_user_files',
                verification: 'files_verified',
                notes: 'PS3 XMB pronto para inicialização real via vsh.self.',
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
          completionPercent: 40,
          isFavorite: false,
          versionCount: 1,
        },
      ],
      achievementProviders: [],
      trophies: [],
      firmwareRecords: [],
      recentActivity: [],
      settings: { 'ui.fullscreen_default': 'false' },
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
          generatedCommand: {
            executable: 'D:\\Emulators\\RPCS3\\rpcs3.exe',
            args: ['--no-gui', 'D:\\Emulators\\RPCS3\\dev_flash\\vsh\\module\\vsh.self'],
            workingDir: 'D:\\Emulators\\RPCS3',
          },
          notes: [],
        },
        diagnostic: 'RPCS3 pronto para inicialização real do PS3 XMB e jogos.',
      },
    };

    const runtimeVerifications: Array<Record<string, unknown>> = [];
    let activeSession: Record<string, unknown> = {
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

    (window as unknown as Record<string, unknown>).__UCH_TEST_TRANSPORT__ = {
      kind: 'injected_test',
      async invoke(command: string, args?: Record<string, unknown>) {
        if (command === 'get_hub_bootstrap') return structuredClone(state);
        if (command === 'get_active_session') return structuredClone(activeSession);
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
            scanPaths: [],
            settings: [],
            firmwareRecords: [],
            runtimeVerifications: structuredClone(
              runtimeVerifications.filter((rv) => rv.consoleSlug === slug)
            ),
            rpcs3Inspection: slug === 'ps3' ? structuredClone(state.rpcs3Inspection) : null,
          };
        }
        if (command === 'get_console_profile') {
          const slug = String(args?.consoleSlug ?? '');
          const c = state.consoles.find((x) => x.slug === slug)!;
          const isPs3 = slug === 'ps3';
          return {
            consoleSlug: slug,
            consoleName: c.name,
            environmentMode: isPs3 ? 'authentic_system_capable' : 'integrated_alternative_only',
            profile: {
              id: `cp-${slug}`,
              consoleId: c.id,
              consoleSlug: slug,
              emulatorId: isPs3 ? 'emu-rpcs3' : null,
              emulatorSlug: isPs3 ? 'rpcs3' : null,
              emulatorStatus: isPs3 ? 'available' : 'unconfigured',
              detectedEmulatorVersion: isPs3 ? 'RPCS3 v0.0.34-17105' : null,
              detectedFirmwareVersion: isPs3 ? '4.91' : null,
              emulatorRootDir: isPs3 ? 'D:\\Emulators\\RPCS3' : null,
              nativeSaveCount: isPs3 ? 4 : 0,
              nativeTrophyDirPresent: isPs3,
              trophySyncReadiness: isPs3 ? 'phase4_planned' : 'unavailable',
              nativeMetadataJson: '{}',
              syncStatus: isPs3 ? 'synced' : 'never_synced',
              syncDiagnostics: '',
              lastSyncedAt: isPs3 ? '2026-11-25T22:00:00Z' : null,
              updatedAt: '2026-11-25T22:00:00Z',
            },
            localUsers: isPs3
              ? [
                  {
                    id: 'clu-1',
                    consoleId: 'console-ps3',
                    consoleSlug: 'ps3',
                    nativeUserKey: '00000001',
                    displayName: 'ClemsonPS3',
                    sourcePath: 'D:\\Emulators\\RPCS3\\dev_hdd0\\home\\00000001',
                    availabilityStatus: 'available',
                    linkedProfileId: 'profile-default',
                    firstDetectedAt: '2026-11-25T21:00:00Z',
                    lastSeenAt: '2026-11-25T22:00:00Z',
                  },
                ]
              : [],
            emulatorStatus: isPs3
              ? { value: 'available', origin: 'native_platform', notes: null }
              : { value: null, origin: 'unavailable', notes: 'Não configurado.' },
            emulatorVersion: isPs3
              ? { value: 'RPCS3 v0.0.34-17105', origin: 'native_platform', notes: null }
              : { value: null, origin: 'unavailable', notes: 'Não detectado.' },
            firmwareVersion: isPs3
              ? { value: '4.91', origin: 'native_platform', notes: null }
              : { value: null, origin: 'unavailable', notes: 'Não detectado.' },
            emulatorRootDir: isPs3
              ? { value: 'D:\\Emulators\\RPCS3', origin: 'native_platform', notes: null }
              : { value: null, origin: 'unavailable', notes: 'Não configurado.' },
            nativeSaveCount: isPs3
              ? { value: 4, origin: 'native_platform', notes: null }
              : { value: null, origin: 'unavailable', notes: 'Indisponível.' },
            trophyReadiness: isPs3
              ? {
                  value: 'phase4_planned',
                  origin: 'native_platform',
                  notes: 'Diretório local de troféus detectado; leitura real planejada para Fase 4.',
                }
              : {
                  value: null,
                  origin: 'unavailable',
                  notes: 'Sem troféus nativos.',
                },
            hubStats: {
              totalPlaytimeSeconds: isPs3 ? 3600 : 0,
              totalSessions: isPs3 ? 1 : 0,
              gamesTotal: isPs3 ? 1 : 0,
              gamesAvailable: isPs3 ? 1 : 0,
              gamesMissingOnDisk: 0,
              mostPlayed: [],
              recentlyPlayed: [],
            },
            recentSessions: [],
          };
        }
        if (command === 'sync_console_profile') {
          return {
            consoleSlug: String(args?.consoleSlug ?? 'ps3'),
            trigger: 'manual',
            syncStatus: 'synced',
            emulatorStatus: 'available',
            localUsersDetected: 1,
            localUsersMarkedMissing: 0,
            gamesDiscovered: 0,
            versionsAvailable: 1,
            versionsMarkedMissing: 0,
            coversImported: 0,
            diagnostics: [],
            syncedAt: '2026-11-25T22:10:00Z',
          };
        }
        if (command === 'launch_console_system') {
          activeSession = {
            phase: 'running',
            sessionId: null,
            target: { kind: 'system_boot', consoleSlug: 'ps3' },
            consoleSlug: 'ps3',
            gameId: null,
            gameTitle: 'Sistema Original do Console (XMB)',
            emulatorSlug: 'rpcs3',
            pid: 4242,
            startedAt: '2026-11-25T22:15:00Z',
            lastHeartbeatAt: '2026-11-25T22:15:00Z',
            elapsedSeconds: 1,
          };
          return structuredClone(activeSession);
        }
        if (command === 'launch_game') {
          activeSession = {
            phase: 'running',
            sessionId: 'sess-live-1',
            target: {
              kind: 'direct_game_boot',
              consoleSlug: 'ps3',
              gameId: String(args?.gameId ?? 'game-ps3-1'),
              gameVersionId: null,
            },
            consoleSlug: 'ps3',
            gameId: String(args?.gameId ?? 'game-ps3-1'),
            gameTitle: "Demon's Souls",
            emulatorSlug: 'rpcs3',
            pid: 4343,
            startedAt: '2026-11-25T22:16:00Z',
            lastHeartbeatAt: '2026-11-25T22:16:00Z',
            elapsedSeconds: 1,
          };
          return structuredClone(activeSession);
        }
        if (command === 'stop_active_session') {
          activeSession = {
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
          return true;
        }
        if (command === 'record_runtime_verification') {
          const req = args?.request as Record<string, unknown>;
          const rec = {
            id: `rv-${runtimeVerifications.length + 1}`,
            consoleId: 'console-ps3',
            consoleSlug: req.consoleSlug,
            emulatorId: 'emu-rpcs3',
            emulatorSlug: req.emulatorSlug,
            gameId: req.gameId ?? null,
            scopeKey: req.gameId ?? 'console',
            featureKey: req.featureKey,
            emulatorVersion: req.emulatorVersion,
            status: req.status,
            evidenceNotes: req.evidenceNotes,
            verifiedAt: '2026-11-25T22:20:00Z',
          };
          runtimeVerifications.push(rec);
          if (req.featureKey === 'original_dashboard' && req.status === 'runtime_confirmed') {
            state.rpcs3Inspection.readiness.xmbRuntimeConfirmed = true;
          }
          return rec;
        }
        throw new Error(`Unhandled Phase 3 E2E command: ${command}`);
      },
    };
  });
});

test('Phase 3 UI flows: real PS3 console profile, separated local users, zero manual form, XMB & game launch, and runtime verification', async ({
  page,
}) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Visão Geral do Hub' })).toBeVisible();

  // 1. LibraryPage has zero manual registration form
  await page.getByRole('button', { name: 'Biblioteca' }).click();
  await expect(page.getByText('Descoberta e Sincronização Automática')).toBeVisible();
  await expect(page.getByText('Registrar Jogo Local no Banco')).toHaveCount(0);

  // 2. Open PlayStation 3 Profile directly from Library shortcut
  await page.getByRole('button', { name: 'Abrir Perfil PlayStation 3' }).click();
  await expect(page.getByRole('heading', { name: 'PlayStation 3', exact: true })).toBeVisible();
  await expect(page.getByText('4.91')).toBeVisible();
  await expect(page.getByText('4 diretório(s)')).toBeVisible();

  // 3. Verify separated real local users in dedicated tab
  await page.getByRole('tab', { name: /Perfil & Usuários Locais/i }).click();
  await expect(page.getByText('ClemsonPS3', { exact: true })).toBeVisible();
  await expect(page.getByText('(00000001)')).toBeVisible();

  // 4. Manual profile sync trigger
  await page.getByRole('button', { name: 'Sincronizar Agora' }).click();
  await expect(page.getByText(/Perfil sincronizado: 1 disponível/i)).toBeVisible();

  // 5. Launch PS3 XMB -> ActiveSessionBanner live -> Stop session
  await page.getByRole('button', { name: 'Iniciar PS3 XMB' }).click();
  await expect(page.getByLabel('Sessão ativa do emulador')).toBeVisible();
  await expect(page.getByText(/PID 4242/)).toBeVisible();
  await page.getByRole('button', { name: 'Encerrar Sessão' }).click();
  await expect(page.getByLabel('Sessão ativa do emulador')).toHaveCount(0);

  // 6. Launch PS3 Game directly -> ActiveSessionBanner -> Stop session
  await page.getByRole('tab', { name: /Jogos do Console/i }).click();
  await page.getByRole('button', { name: 'Jogar', exact: true }).click();
  await expect(page.getByLabel('Sessão ativa do emulador')).toBeVisible();
  await expect(page.getByText(/PID 4343/)).toBeVisible();
  await page.getByRole('button', { name: 'Encerrar Sessão' }).click();
  await expect(page.getByLabel('Sessão ativa do emulador')).toHaveCount(0);

  // 7. Record explicit manual XMB runtime confirmation in Emulador & Verificações tab
  await page.getByRole('tab', { name: /Emulador & Verificações/i }).click();
  await page.getByRole('button', { name: 'Confirmar XMB em Execução' }).click();
  await expect(page.getByText(/Verificação runtime do PS3 XMB registrada/i)).toBeVisible();

  // 8. Verify PlayStation 4 never invents local users and never shows fake XMB button
  await page.getByRole('button', { name: 'Consoles', exact: true }).click();
  await page.getByRole('button', { name: 'Abrir detalhes de PlayStation 4' }).click();
  await expect(page.getByRole('button', { name: /Iniciar PS3 XMB/i })).toHaveCount(0);
  await page.getByRole('tab', { name: /Perfil & Usuários Locais/i }).click();
  await expect(
    page.getByText(/Nenhum usuário local nativo detectado para PlayStation 4/i)
  ).toBeVisible();
});
