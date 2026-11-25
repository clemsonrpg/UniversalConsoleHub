import type { HubTransport } from '../api/tauriClient';
import type {
  BackupReport,
  ConsoleDetailsDto,
  FirmwareRecordDto,
  GameRecord,
  HubBootstrapDto,
  PlayerProfile,
  RegisterGameRequest,
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

  return {
    kind: 'injected_test',
    async invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
      switch (command) {
        case 'get_hub_bootstrap':
          return structuredClone(state) as T;
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
          const report: BackupReport = {
            backupPath: 'C:\\Users\\player\\AppData\\Local\\UniversalConsoleHub\\backups\\phase1_snapshot.db',
            sizeBytes: 98304,
            integrityOk: true,
            createdAt: '2026-11-25T22:00:00Z',
          };
          return report as T;
        }
        default:
          throw new Error(`Comando não suportado no transporte de teste: ${command}`);
      }
    },
  };
}
