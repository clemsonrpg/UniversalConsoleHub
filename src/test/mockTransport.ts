import type { HubTransport } from '../api/tauriClient';
import type {
  ActiveSessionSnapshot,
  BackupFileEntry,
  BackupInspectionReport,
  BackupReport,
  ConsoleDetailsDto,
  ConsoleEnvironmentDto,
  ConsoleProfileOverviewDto,
  ConsoleScanPath,
  ConsoleSetting,
  ConsoleSyncReport,
  FirmwareRecordDto,
  GameDetailsDto,
  GameRecord,
  GameVersionRecord,
  HubBootstrapDto,
  LiveRestoreReport,
  PlayerProfile,
  RecordRuntimeVerificationRequest,
  RegisterGameRequest,
  RuntimeVerificationRecord,
  ScanProgressEvent,
  ScanSummaryDto,
  StatisticsOverview,
  UpdateGameMetadataRequest,
  UpdateProfileRequest,
} from '../types/domain';

export function createFixtureBootstrap(): HubBootstrapDto {
  return {
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
      bio: 'Perfil offline local do Universal Console Hub.',
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
              feature: 'direct_game_boot',
              states: ['DIRECT_GAME_ONLY'],
              availability: 'available',
              verification: 'documented_upstream',
              notes: 'Execução direta via EBOOT.BIN.',
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
        integrationPhase: 'phase_1_preparation',
        capabilities: {
          supportedStates: ['FULL_SYSTEM_BOOT', 'ORIGINAL_DASHBOARD', 'DIRECT_GAME_ONLY'],
          features: [],
        },
      },
    ],
    games: [],
    achievementProviders: [
      {
        id: 'prov-local',
        slug: 'local',
        name: 'LocalAchievementProvider',
        origin: 'local',
        availabilityStatus: 'ready_offline',
        supportsAuth: false,
        supportsSync: false,
        notes: 'Provider local offline do Hub.',
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
    firmwareRecords: [],
    recentActivity: [],
    settings: {
      'ui.fullscreen_default': 'false',
      'ui.target_resolution': '1920x1080',
      'ui.reduced_motion': 'false',
      'emulation.launch_timeout_seconds': '45',
    },
    rpcs3Inspection: {
      configured: false,
      installation: null,
      readiness: null,
      diagnostic: 'RPCS3 ainda não configurado.',
    },
  };
}

/**
 * In-memory test transport ONLY for Vitest / Playwright UI flow tests.
 * Never presented as real Tauri IPC or real SQLite validation.
 */
export function createStatefulTestTransport(): HubTransport {
  const state = createFixtureBootstrap();
  const scanPaths: ConsoleScanPath[] = [];
  const consoleSettings: ConsoleSetting[] = [];
  const gameVersions: GameVersionRecord[] = [];
  const backups: Array<{ entry: BackupFileEntry; snapshot: HubBootstrapDto }> = [];

  return {
    kind: 'injected_test',
    async invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
      switch (command) {
        case 'get_hub_bootstrap':
          return structuredClone(state) as T;
        case 'get_console_environment': {
          const slug = String(args?.slug ?? '');
          const found = state.consoles.find((c) => c.slug === slug);
          if (!found) throw new Error(`Console '${slug}' não encontrado`);
          const isAuthentic =
            found.capabilities.supportedStates.includes('FULL_SYSTEM_BOOT') &&
            found.capabilities.supportedStates.includes('ORIGINAL_DASHBOARD');
          const env: ConsoleEnvironmentDto = {
            console: structuredClone(found),
            emulator: structuredClone(
              state.emulators.find((e) => e.consoleSlug === slug) ?? null
            ),
            environmentMode: isAuthentic
              ? 'authentic_system_capable'
              : 'integrated_alternative_only',
            games: structuredClone(state.games.filter((g) => g.consoleSlug === slug)),
            scanPaths: structuredClone(scanPaths.filter((p) => p.consoleSlug === slug)),
            settings: structuredClone(consoleSettings.filter((s) => s.consoleSlug === slug)),
            firmwareRecords: structuredClone(
              state.firmwareRecords.filter((f) => f.consoleSlug === slug)
            ),
            runtimeVerifications: [],
            rpcs3Inspection: slug === 'ps3' ? structuredClone(state.rpcs3Inspection) : null,
          };
          return env as T;
        }
        case 'add_console_scan_path': {
          const consoleSlug = String(args?.consoleSlug ?? 'ps3');
          const directoryPath = String(args?.directoryPath ?? '');
          const recursive = Boolean(args?.recursive ?? true);
          const item: ConsoleScanPath = {
            id: `sp-${scanPaths.length + 1}`,
            consoleId: `console-${consoleSlug}`,
            consoleSlug,
            directoryPath,
            recursive,
            enabled: true,
            lastScannedAt: null,
            createdAt: '2026-11-25T22:00:00Z',
          };
          scanPaths.push(item);
          return structuredClone(item) as T;
        }
        case 'remove_console_scan_path': {
          const pathId = String(args?.pathId ?? '');
          const idx = scanPaths.findIndex((p) => p.id === pathId);
          if (idx >= 0) {
            scanPaths.splice(idx, 1);
            return true as T;
          }
          return false as T;
        }
        case 'scan_console_library': {
          const consoleSlug = String(args?.consoleSlug ?? 'ps3');
          const onProgress = args?.onProgress as ((evt: ScanProgressEvent) => void) | undefined;
          const matching = scanPaths.filter((p) => p.consoleSlug === consoleSlug);
          onProgress?.({
            event: 'started',
            data: { consoleSlug, totalDirectories: matching.length },
          });
          let gamesAdded = 0;
          let versionsAdded = 0;
          let skippedExisting = 0;
          if (matching.length > 0 && consoleSlug === 'ps3') {
            const exists = state.games.some((g) => g.canonicalKey === 'serial:BLUS30443');
            if (!exists) {
              const g: GameRecord = {
                id: `game-${state.games.length + 1}`,
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
              };
              state.games.push(g);
              gameVersions.push({
                id: `gv-${gameVersions.length + 1}`,
                gameId: g.id,
                versionLabel: '01.00',
                region: 'USA',
                serialCode: 'BLUS30443',
                filePath: `${matching[0]!.directoryPath}\\DemonsSouls\\PS3_GAME\\USRDIR\\EBOOT.BIN`,
                installState: 'registered',
                createdAt: '2026-11-25T22:00:00Z',
              });
              state.statistics.gamesRegistered = state.games.length;
              gamesAdded = 1;
              versionsAdded = 1;
              onProgress?.({
                event: 'gameDiscovered',
                data: {
                  title: g.title,
                  serialCode: 'BLUS30443',
                  filePath: `${matching[0]!.directoryPath}\\DemonsSouls\\PS3_GAME\\USRDIR\\EBOOT.BIN`,
                },
              });
            } else {
              skippedExisting = 1;
            }
          }
          const summary: ScanSummaryDto = {
            consoleSlug,
            scannedDirectories: matching.length,
            scannedFiles: matching.length * 3,
            gamesAdded,
            versionsAdded,
            skippedExisting,
            errors: [],
          };
          onProgress?.({
            event: 'completed',
            data: {
              scannedFiles: summary.scannedFiles,
              gamesAdded,
              versionsAdded,
              skippedExisting,
            },
          });
          return summary as T;
        }
        case 'update_console_setting': {
          const consoleSlug = String(args?.consoleSlug ?? 'ps3');
          const key = String(args?.key ?? '');
          const value = String(args?.value ?? '');
          const existing = consoleSettings.find(
            (s) => s.consoleSlug === consoleSlug && s.key === key
          );
          if (existing) {
            existing.value = value;
            existing.updatedAt = '2026-11-25T22:05:00Z';
            return structuredClone(existing) as T;
          }
          const created: ConsoleSetting = {
            id: `cs-${consoleSettings.length + 1}`,
            consoleId: `console-${consoleSlug}`,
            consoleSlug,
            key,
            value,
            updatedAt: '2026-11-25T22:05:00Z',
          };
          consoleSettings.push(created);
          return structuredClone(created) as T;
        }
        case 'get_game_details': {
          const gameId = String(args?.gameId ?? '');
          const game = state.games.find((g) => g.id === gameId);
          if (!game) throw new Error(`Jogo '${gameId}' não encontrado`);
          const dto: GameDetailsDto = {
            game: structuredClone(game),
            versions: structuredClone(gameVersions.filter((v) => v.gameId === gameId)),
            sessions: [],
          };
          return dto as T;
        }
        case 'update_game_metadata': {
          const req = args?.request as UpdateGameMetadataRequest;
          const game = state.games.find((g) => g.id === req.gameId);
          if (!game) throw new Error('Jogo não encontrado');
          game.description = req.description;
          game.developer = req.developer;
          game.publisher = req.publisher;
          game.releaseYear = req.releaseYear;
          game.genre = req.genre;
          game.completionPercent = req.completionPercent;
          return structuredClone(game) as T;
        }
        case 'set_game_cover': {
          const gameId = String(args?.gameId ?? '');
          const sourceImagePath = String(args?.sourceImagePath ?? '');
          const game = state.games.find((g) => g.id === gameId);
          if (!game) throw new Error('Jogo não encontrado');
          game.coverPath = sourceImagePath;
          return structuredClone(game) as T;
        }
        case 'get_statistics_overview': {
          const overview: StatisticsOverview = {
            totalPlaytimeSeconds: state.statistics.totalPlaytimeSeconds,
            totalSessions: 0,
            gamesRegistered: state.games.length,
            gamesStarted: state.statistics.gamesStarted,
            gamesCompleted: state.games.filter((g) => g.completionPercent === 100).length,
            completionRatePercent:
              state.games.length > 0
                ? Math.floor(
                    (state.games.filter((g) => g.completionPercent === 100).length * 100) /
                      state.games.length
                  )
                : 0,
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
          return overview as T;
        }
        case 'list_activity_history': {
          const filter = args?.eventTypeFilter as string | null | undefined;
          const items =
            filter && filter !== 'all'
              ? state.recentActivity.filter((e) => e.eventType === filter)
              : state.recentActivity;
          return structuredClone(items) as T;
        }
        case 'list_database_backups':
          return structuredClone(backups.map((b) => b.entry)) as T;
        case 'inspect_database_backup': {
          const backupPath = String(args?.backupPath ?? '');
          const match = backups.find((b) => b.entry.backupPath === backupPath);
          const report: BackupInspectionReport = {
            backupPath,
            sizeBytes: match?.entry.sizeBytes ?? 98304,
            integrityOk: true,
            schemaVersion: 2,
            profilesCount: 1,
            gamesCount: match?.snapshot.games.length ?? state.games.length,
            inspectedAt: '2026-11-25T22:10:00Z',
          };
          return report as T;
        }
        case 'restore_database_backup': {
          const backupPath = String(args?.backupPath ?? '');
          const match = backups.find((b) => b.entry.backupPath === backupPath);
          if (match) {
            const restored = structuredClone(match.snapshot);
            state.profile = restored.profile;
            state.games = restored.games;
            state.statistics = restored.statistics;
            state.settings = restored.settings;
          }
          const report: LiveRestoreReport = {
            sourceBackupPath: backupPath,
            restoredDbPath:
              'C:\\Users\\player\\AppData\\Local\\UniversalConsoleHub\\universal_console_hub.db',
            preRestoreSnapshotPath:
              'C:\\Users\\player\\AppData\\Local\\UniversalConsoleHub\\backups\\pre_restore\\pre_restore_test.db',
            integrityOk: true,
            schemaVersion: 2,
            restoredAt: '2026-11-25T22:11:00Z',
          };
          return report as T;
        }
        case 'get_console_details': {
          const slug = String(args?.slug ?? '');
          const found = state.consoles.find((c) => c.slug === slug);
          if (!found) throw new Error(`Console '${slug}' não encontrado`);
          const dto: ConsoleDetailsDto = {
            console: structuredClone(found),
            emulator: structuredClone(
              state.emulators.find((e) => e.consoleSlug === slug) ?? null
            ),
            firmwareRecords: structuredClone(
              state.firmwareRecords.filter((f) => f.consoleSlug === slug)
            ),
            rpcs3Inspection: slug === 'ps3' ? structuredClone(state.rpcs3Inspection) : null,
          };
          return dto as T;
        }
        case 'update_active_profile': {
          const req = args?.request as UpdateProfileRequest;
          const updated: PlayerProfile = {
            ...state.profile,
            username: req.username,
            handle: req.username,
            displayName: req.displayName,
            bio: req.bio,
            privacyLevel: req.privacyLevel,
            updatedAt: '2026-11-25T22:00:00Z',
          };
          state.profile = updated;
          return structuredClone(updated) as T;
        }
        case 'register_game': {
          const req = args?.request as RegisterGameRequest;
          const game: GameRecord = {
            id: `game-${state.games.length + 1}`,
            consoleId: `console-${req.consoleSlug}`,
            consoleSlug: req.consoleSlug,
            canonicalKey: `title:${req.title.toLowerCase().replace(/\s+/g, '-')}`,
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
          state.games.push(game);
          state.statistics.gamesRegistered = state.games.length;
          return structuredClone(game) as T;
        }
        case 'toggle_game_favorite': {
          const gameId = String(args?.gameId ?? '');
          const g = state.games.find((item) => item.id === gameId);
          if (!g) throw new Error('Jogo não encontrado');
          g.isFavorite = !g.isFavorite;
          return g.isFavorite as T;
        }
        case 'configure_emulator_path': {
          const slug = String(args?.slug ?? '');
          const exePath = String(args?.exePath ?? '');
          const emu = state.emulators.find((e) => e.slug === slug);
          if (!emu) throw new Error('Emulador não encontrado');
          emu.executablePath = exePath;
          if (slug === 'rpcs3') {
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
                firmwareSha256: state.firmwareRecords[0]?.sha256 ?? null,
                systemFilesPresent: true,
                commandBuildable: true,
                xmbRuntimeConfirmed: false,
                generatedCommand: {
                  executable: exePath,
                  args: ['--no-gui', 'C:\\Emulators\\RPCS3\\dev_flash\\vsh\\module\\vsh.self'],
                  workingDir: 'C:\\Emulators\\RPCS3',
                },
                notes: [
                  'Estágio 4/4: Execução real do XMB NÃO confirmada na Fase 1 (reservada à Fase 4).',
                ],
              },
              diagnostic: 'RPCS3 configurado com comando preparado; boot runtime reservado à Fase 4.',
            };
          }
          return structuredClone(emu) as T;
        }
        case 'inspect_rpcs3':
          return structuredClone(state.rpcs3Inspection) as T;
        case 'register_firmware_file': {
          const consoleSlug = String(args?.consoleSlug ?? 'ps3');
          const emulatorSlug = String(args?.emulatorSlug ?? 'rpcs3');
          const filePath = String(args?.filePath ?? '');
          const rec: FirmwareRecordDto = {
            id: `fw-${state.firmwareRecords.length + 1}`,
            consoleSlug,
            emulatorSlug,
            fileRole: 'system_firmware_package',
            filePath,
            fileSizeBytes: 204800,
            sha256: 'a'.repeat(64),
            validationStatus: 'package_header_and_hash_validated',
            notes: 'Hash SHA-256 validado localmente (sem armazenar firmware).',
            validatedAt: '2026-11-25T22:00:00Z',
          };
          state.firmwareRecords.unshift(rec);
          if (state.rpcs3Inspection.readiness) {
            state.rpcs3Inspection.readiness.firmwarePackageValidated = true;
            state.rpcs3Inspection.readiness.firmwareSha256 = rec.sha256;
          }
          return structuredClone(rec) as T;
        }
        case 'update_app_setting': {
          const key = String(args?.key ?? '');
          const value = String(args?.value ?? '');
          state.settings[key] = value;
          return structuredClone(state.settings) as T;
        }
        case 'create_database_backup': {
          const idx = backups.length + 1;
          const fileName = `phase2_snapshot_${idx}.db`;
          const backupPath = `C:\\Users\\player\\AppData\\Local\\UniversalConsoleHub\\backups\\${fileName}`;
          const report: BackupReport = {
            backupPath,
            sizeBytes: 98304,
            integrityOk: true,
            createdAt: '2026-11-25T22:00:00Z',
          };
          backups.unshift({
            entry: {
              fileName,
              backupPath,
              sizeBytes: report.sizeBytes,
              modifiedAt: report.createdAt,
            },
            snapshot: structuredClone(state),
          });
          return report as T;
        }
        case 'get_console_profile': {
          const consoleSlug = String(args?.consoleSlug ?? 'ps3');
          const found = state.consoles.find((c) => c.slug === consoleSlug);
          if (!found) throw new Error(`Console '${consoleSlug}' não encontrado`);
          const isPs3 = consoleSlug === 'ps3';
          const rpcs3Ready = Boolean(isPs3 && state.rpcs3Inspection.configured);
          const consoleGames = state.games.filter((g) => g.consoleSlug === consoleSlug);
          const availableGames = consoleGames.filter((g) => g.installState !== 'missing_files');
          const overview: ConsoleProfileOverviewDto = {
            consoleSlug,
            consoleName: found.name,
            environmentMode:
              found.capabilities.supportedStates.includes('FULL_SYSTEM_BOOT') &&
              found.capabilities.supportedStates.includes('ORIGINAL_DASHBOARD')
                ? 'authentic_system_capable'
                : 'integrated_alternative_only',
            profile: {
              id: `cprof-${consoleSlug}`,
              consoleId: found.id,
              consoleSlug,
              emulatorId: isPs3 ? 'emu-rpcs3' : null,
              emulatorSlug: isPs3 ? 'rpcs3' : null,
              emulatorStatus: rpcs3Ready ? 'available' : 'unconfigured',
              detectedEmulatorVersion: rpcs3Ready ? 'RPCS3 v0.0.34-test' : null,
              detectedFirmwareVersion: rpcs3Ready ? '4.91' : null,
              emulatorRootDir: rpcs3Ready ? 'C:\\Emulators\\RPCS3' : null,
              nativeSaveCount: rpcs3Ready ? 2 : 0,
              nativeTrophyDirPresent: rpcs3Ready,
              trophySyncReadiness: isPs3 ? 'phase4_planned' : 'unavailable',
              nativeMetadataJson: '{}',
              syncStatus: rpcs3Ready ? 'synced' : 'never_synced',
              syncDiagnostics: '',
              lastSyncedAt: rpcs3Ready ? '2026-11-25T22:15:00Z' : null,
              updatedAt: '2026-11-25T22:15:00Z',
            },
            localUsers: rpcs3Ready
              ? [
                  {
                    id: 'clu-1',
                    consoleId: found.id,
                    consoleSlug: 'ps3',
                    nativeUserKey: '00000001',
                    displayName: 'ClemsonPS3',
                    sourcePath: 'C:\\Emulators\\RPCS3\\dev_hdd0\\home\\00000001',
                    availabilityStatus: 'available',
                    linkedProfileId: 'profile-default',
                    firstDetectedAt: '2026-11-25T22:00:00Z',
                    lastSeenAt: '2026-11-25T22:15:00Z',
                  },
                ]
              : [],
            emulatorStatus: rpcs3Ready
              ? { value: 'available', origin: 'native_platform', notes: null }
              : {
                  value: null,
                  origin: 'unavailable',
                  notes: 'Emulador ainda não configurado para este console.',
                },
            emulatorVersion: rpcs3Ready
              ? { value: 'RPCS3 v0.0.34-test', origin: 'native_platform', notes: null }
              : {
                  value: null,
                  origin: 'unavailable',
                  notes: 'Versão do emulador não detectada automaticamente.',
                },
            firmwareVersion: rpcs3Ready
              ? { value: '4.91', origin: 'native_platform', notes: null }
              : {
                  value: null,
                  origin: 'unavailable',
                  notes: 'Firmware não detectado no diretório do emulador.',
                },
            emulatorRootDir: rpcs3Ready
              ? { value: 'C:\\Emulators\\RPCS3', origin: 'native_platform', notes: null }
              : { value: null, origin: 'unavailable', notes: 'Diretório não configurado.' },
            nativeSaveCount: rpcs3Ready
              ? { value: 2, origin: 'native_platform', notes: null }
              : { value: null, origin: 'unavailable', notes: 'Sem saves nativos detectados.' },
            trophyReadiness: isPs3
              ? {
                  value: 'phase4_planned',
                  origin: 'native_platform',
                  notes:
                    'Diretório local de troféus detectado; leitura real de TROPCONF.SFM/TROPUSR.DAT planejada para a Fase 4.',
                }
              : {
                  value: null,
                  origin: 'unavailable',
                  notes: 'Esta plataforma não possui sistema nativo de troféus.',
                },
            hubStats: {
              totalPlaytimeSeconds: consoleGames.reduce((acc, g) => acc + g.totalPlaytimeSeconds, 0),
              totalSessions: 0,
              gamesTotal: consoleGames.length,
              gamesAvailable: availableGames.length,
              gamesMissingOnDisk: consoleGames.length - availableGames.length,
              mostPlayed: [],
              recentlyPlayed: [],
            },
            recentSessions: [],
          };
          return overview as T;
        }
        case 'sync_console_profile': {
          const consoleSlug = String(args?.consoleSlug ?? 'ps3');
          const isPs3 = consoleSlug === 'ps3';
          const rpcs3Ready = Boolean(isPs3 && state.rpcs3Inspection.configured);
          const consoleGames = state.games.filter((g) => g.consoleSlug === consoleSlug);
          const report: ConsoleSyncReport = {
            consoleSlug,
            trigger: (args?.trigger as ConsoleSyncReport['trigger']) ?? 'manual',
            syncStatus: 'synced',
            emulatorStatus: rpcs3Ready ? 'available' : 'unconfigured',
            localUsersDetected: rpcs3Ready ? 1 : 0,
            localUsersMarkedMissing: 0,
            gamesDiscovered: 0,
            versionsAvailable: consoleGames.length,
            versionsMarkedMissing: 0,
            coversImported: 0,
            diagnostics: [],
            syncedAt: '2026-11-25T22:20:00Z',
          };
          return report as T;
        }
        case 'get_active_session': {
          const snap: ActiveSessionSnapshot =
            (state as unknown as { __activeSession?: ActiveSessionSnapshot }).__activeSession ?? {
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
          return structuredClone(snap) as T;
        }
        case 'launch_console_system': {
          const consoleSlug = String(args?.consoleSlug ?? 'ps3');
          const snap: ActiveSessionSnapshot = {
            phase: 'running',
            sessionId: null,
            target: { kind: 'system_boot', consoleSlug },
            consoleSlug,
            gameId: null,
            gameTitle: 'Sistema Original do Console (XMB)',
            emulatorSlug: 'rpcs3',
            pid: 4242,
            startedAt: '2026-11-25T22:25:00Z',
            lastHeartbeatAt: '2026-11-25T22:25:00Z',
            elapsedSeconds: 1,
          };
          (state as unknown as { __activeSession?: ActiveSessionSnapshot }).__activeSession = snap;
          return structuredClone(snap) as T;
        }
        case 'launch_game': {
          const gameId = String(args?.gameId ?? '');
          const g = state.games.find((item) => item.id === gameId);
          const snap: ActiveSessionSnapshot = {
            phase: 'running',
            sessionId: `sess-${Date.now()}`,
            target: {
              kind: 'direct_game_boot',
              consoleSlug: g?.consoleSlug ?? 'ps3',
              gameId,
              gameVersionId: (args?.gameVersionId as string | null) ?? null,
            },
            consoleSlug: g?.consoleSlug ?? 'ps3',
            gameId,
            gameTitle: g?.title ?? 'Jogo PS3',
            emulatorSlug: 'rpcs3',
            pid: 4343,
            startedAt: '2026-11-25T22:26:00Z',
            lastHeartbeatAt: '2026-11-25T22:26:00Z',
            elapsedSeconds: 1,
          };
          (state as unknown as { __activeSession?: ActiveSessionSnapshot }).__activeSession = snap;
          return structuredClone(snap) as T;
        }
        case 'stop_active_session': {
          (state as unknown as { __activeSession?: ActiveSessionSnapshot }).__activeSession = {
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
          return true as T;
        }
        case 'record_runtime_verification': {
          const req = args?.request as RecordRuntimeVerificationRequest;
          const rec: RuntimeVerificationRecord = {
            id: `rv-${Date.now()}`,
            consoleId: `console-${req.consoleSlug}`,
            consoleSlug: req.consoleSlug,
            emulatorId: `emu-${req.emulatorSlug}`,
            emulatorSlug: req.emulatorSlug,
            gameId: req.gameId,
            scopeKey: req.gameId ?? 'console',
            featureKey: req.featureKey,
            emulatorVersion: req.emulatorVersion,
            status: req.status,
            evidenceNotes: req.evidenceNotes,
            verifiedAt: '2026-11-25T22:30:00Z',
          };
          if (req.featureKey === 'original_dashboard' && state.rpcs3Inspection.readiness) {
            state.rpcs3Inspection.readiness.xmbRuntimeConfirmed =
              req.status === 'runtime_confirmed';
          }
          return rec as T;
        }
        default:
          throw new Error(`Comando não suportado no transporte de teste: ${command}`);
      }
    },
  };
}
