export type CapabilityState =
  | 'FULL_SYSTEM_BOOT'
  | 'ORIGINAL_DASHBOARD'
  | 'PARTIAL_DASHBOARD'
  | 'DIRECT_GAME_ONLY'
  | 'EXPERIMENTAL'
  | 'UNSUPPORTED';

export type FeatureKey =
  | 'system_boot'
  | 'original_dashboard'
  | 'direct_game_boot'
  | 'save_management'
  | 'local_trophies'
  | 'online_services';

export type FeatureAvailability =
  | 'available'
  | 'requires_user_files'
  | 'partial'
  | 'experimental'
  | 'planned'
  | 'unsupported';

export type VerificationLevel =
  | 'unverified'
  | 'documented_upstream'
  | 'files_verified'
  | 'runtime_confirmed';

export interface FeatureCapability {
  feature: FeatureKey;
  states: CapabilityState[];
  availability: FeatureAvailability;
  verification: VerificationLevel;
  notes: string;
}

export interface ConsoleCapabilities {
  supportedStates: CapabilityState[];
  features: FeatureCapability[];
}

export interface ConsoleRecord {
  id: string;
  slug: string;
  name: string;
  manufacturer: string;
  generation: number;
  releaseYear: number;
  phaseStatus: string;
  capabilities: ConsoleCapabilities;
  notes: string;
}

export interface EmulatorRecord {
  id: string;
  slug: string;
  name: string;
  consoleId: string;
  consoleSlug: string;
  executablePath: string | null;
  configPath: string | null;
  detectedVersion: string | null;
  integrationPhase: string;
  capabilities: ConsoleCapabilities;
}

export interface LaunchCommand {
  executable: string;
  args: string[];
  workingDir: string | null;
}

export interface EmulatorInstallation {
  emulatorSlug: string;
  executablePath: string;
  rootDir: string;
  detectedVersion: string | null;
  isValid: boolean;
  issues: string[];
}

export interface SystemBootReadiness {
  firmwarePackageValidated: boolean;
  firmwareSha256: string | null;
  systemFilesPresent: boolean;
  commandBuildable: boolean;
  xmbRuntimeConfirmed: boolean;
  generatedCommand: LaunchCommand | null;
  notes: string[];
}

export interface Rpcs3InspectionDto {
  configured: boolean;
  installation: EmulatorInstallation | null;
  readiness: SystemBootReadiness | null;
  diagnostic: string;
}

export interface FirmwareRecordDto {
  id: string;
  consoleSlug: string;
  emulatorSlug: string;
  fileRole: string;
  filePath: string;
  fileSizeBytes: number;
  sha256: string;
  validationStatus: string;
  notes: string;
  validatedAt: string;
}

export interface ConsoleDetailsDto {
  console: ConsoleRecord;
  emulator: EmulatorRecord | null;
  firmwareRecords: FirmwareRecordDto[];
  rpcs3Inspection: Rpcs3InspectionDto | null;
}

export interface PlayerProfile {
  id: string;
  userId: string;
  username: string;
  handle: string;
  displayName: string;
  avatarPath: string | null;
  bannerPath: string | null;
  bio: string;
  privacyLevel: 'private' | 'friends' | 'public';
  createdAt: string;
  updatedAt: string;
}

export interface ProfileStatistics {
  gamesRegistered: number;
  gamesStarted: number;
  gamesCompleted: number;
  totalPlaytimeSeconds: number;
  totalTrophies: number;
  unlockedTrophies: number;
  rareTrophies: number;
}

export interface GameRecord {
  id: string;
  consoleId: string;
  consoleSlug: string;
  canonicalKey: string;
  title: string;
  description: string;
  developer: string | null;
  publisher: string | null;
  releaseYear: number | null;
  genre: string | null;
  coverPath: string | null;
  metadataSource: string;
  installState: string;
  associatedEmulatorId: string | null;
  totalPlaytimeSeconds: number;
  lastPlayedAt: string | null;
  completionPercent: number;
  isFavorite: boolean;
  versionCount: number;
}

export type TrophyOrigin = 'official' | 'emulator' | 'local';

export type TrophyCategory =
  | 'bronze'
  | 'silver'
  | 'gold'
  | 'platinum'
  | 'xbox_achievement'
  | 'steam_achievement'
  | 'emulator_achievement'
  | 'hub_local';

export interface AchievementProviderRecord {
  id: string;
  slug: string;
  name: string;
  origin: TrophyOrigin;
  availabilityStatus: string;
  supportsAuth: boolean;
  supportsSync: boolean;
  notes: string;
}

export interface TrophyRecord {
  id: string;
  gameId: string;
  consoleId: string;
  providerId: string;
  externalTrophyId: string;
  name: string;
  description: string;
  iconPath: string | null;
  category: TrophyCategory;
  rarityPercent: number | null;
  origin: TrophyOrigin;
  isUnlocked: boolean;
  unlockedAt: string | null;
}

export interface ActivityEntryDto {
  id: string;
  eventType: string;
  entityType: string;
  entityId: string;
  summary: string;
  occurredAt: string;
}

export interface BackupReport {
  backupPath: string;
  sizeBytes: number;
  integrityOk: boolean;
  createdAt: string;
}

export interface HubBootstrapDto {
  appName: string;
  version: string;
  phase: number;
  profile: PlayerProfile;
  statistics: ProfileStatistics;
  consoles: ConsoleRecord[];
  emulators: EmulatorRecord[];
  games: GameRecord[];
  achievementProviders: AchievementProviderRecord[];
  trophies: TrophyRecord[];
  firmwareRecords: FirmwareRecordDto[];
  recentActivity: ActivityEntryDto[];
  settings: Record<string, string>;
  rpcs3Inspection: Rpcs3InspectionDto;
}

export interface UpdateProfileRequest {
  username: string;
  displayName: string;
  bio: string;
  privacyLevel: 'private' | 'friends' | 'public';
}

export interface RegisterGameRequest {
  consoleSlug: string;
  title: string;
  versionLabel: string;
  region: string;
  serialCode: string | null;
  filePath: string;
  metadataSource: string;
}

// ===== Phase 2 Domain Contracts =====

export type ConsoleEnvironmentMode =
  | 'authentic_system_capable'
  | 'integrated_alternative_only';

export type RuntimeVerificationFeature =
  | 'emulator_launch'
  | 'system_boot'
  | 'original_dashboard'
  | 'game_boot'
  | 'per_game_compatibility'
  | 'session_monitoring'
  | 'hub_return';

export type RuntimeVerificationStatus =
  | 'unverified'
  | 'documented_upstream'
  | 'files_verified'
  | 'runtime_confirmed'
  | 'failed';

export interface RuntimeVerificationRecord {
  id: string;
  consoleId: string;
  consoleSlug: string;
  emulatorId: string;
  emulatorSlug: string;
  gameId: string | null;
  scopeKey: string;
  featureKey: RuntimeVerificationFeature;
  emulatorVersion: string;
  status: RuntimeVerificationStatus;
  evidenceNotes: string;
  verifiedAt: string;
}

export interface ConsoleScanPath {
  id: string;
  consoleId: string;
  consoleSlug: string;
  directoryPath: string;
  recursive: boolean;
  enabled: boolean;
  lastScannedAt: string | null;
  createdAt: string;
}

export interface ConsoleSetting {
  id: string;
  consoleId: string;
  consoleSlug: string;
  key: string;
  value: string;
  updatedAt: string;
}

export interface ConsoleEnvironmentDto {
  console: ConsoleRecord;
  emulator: EmulatorRecord | null;
  environmentMode: ConsoleEnvironmentMode;
  games: GameRecord[];
  scanPaths: ConsoleScanPath[];
  settings: ConsoleSetting[];
  firmwareRecords: FirmwareRecordDto[];
  runtimeVerifications: RuntimeVerificationRecord[];
  rpcs3Inspection: Rpcs3InspectionDto | null;
}

export type ScanProgressEvent =
  | { event: 'started'; data: { consoleSlug: string; totalDirectories: number } }
  | { event: 'scanningDirectory'; data: { directoryPath: string; filesChecked: number } }
  | {
      event: 'gameDiscovered';
      data: { title: string; serialCode: string | null; filePath: string };
    }
  | {
      event: 'completed';
      data: {
        scannedFiles: number;
        gamesAdded: number;
        versionsAdded: number;
        skippedExisting: number;
      };
    }
  | { event: 'error'; data: { message: string } };

export interface ScanSummaryDto {
  consoleSlug: string;
  scannedDirectories: number;
  scannedFiles: number;
  gamesAdded: number;
  versionsAdded: number;
  skippedExisting: number;
  errors: string[];
}

export interface GameVersionRecord {
  id: string;
  gameId: string;
  versionLabel: string;
  region: string;
  serialCode: string | null;
  filePath: string;
  installState: string;
  createdAt: string;
}

export interface GameSessionRecord {
  id: string;
  profileId: string;
  gameId: string;
  gameTitle: string;
  consoleSlug: string;
  gameVersionId: string | null;
  emulatorId: string;
  startedAt: string;
  lastHeartbeatAt: string;
  endedAt: string | null;
  durationSeconds: number;
  exitStatus: string;
}

export interface GameDetailsDto {
  game: GameRecord;
  versions: GameVersionRecord[];
  sessions: GameSessionRecord[];
}

export interface UpdateGameMetadataRequest {
  gameId: string;
  description: string;
  developer: string | null;
  publisher: string | null;
  releaseYear: number | null;
  genre: string | null;
  completionPercent: number;
}

export interface ConsolePlaytimeStat {
  consoleSlug: string;
  consoleName: string;
  gamesCount: number;
  sessionsCount: number;
  totalPlaytimeSeconds: number;
}

export interface TopGameStat {
  gameId: string;
  title: string;
  consoleSlug: string;
  sessionsCount: number;
  totalPlaytimeSeconds: number;
  completionPercent: number;
  lastPlayedAt: string | null;
}

export interface MonthlyPlaytimeStat {
  month: string;
  sessionsCount: number;
  totalPlaytimeSeconds: number;
}

export interface StatisticsOverview {
  totalPlaytimeSeconds: number;
  totalSessions: number;
  gamesRegistered: number;
  gamesStarted: number;
  gamesCompleted: number;
  completionRatePercent: number;
  byConsole: ConsolePlaytimeStat[];
  topGames: TopGameStat[];
  monthlyPlaytime: MonthlyPlaytimeStat[];
}

export interface BackupFileEntry {
  fileName: string;
  backupPath: string;
  sizeBytes: number;
  modifiedAt: string;
}

export interface BackupInspectionReport {
  backupPath: string;
  sizeBytes: number;
  integrityOk: boolean;
  schemaVersion: number;
  profilesCount: number;
  gamesCount: number;
  inspectedAt: string;
}

export interface LiveRestoreReport {
  sourceBackupPath: string;
  restoredDbPath: string;
  preRestoreSnapshotPath: string;
  integrityOk: boolean;
  schemaVersion: number;
  restoredAt: string;
}

// ===== Phase 3 Domain Contracts (Real Console Profiles & Live PS3 Sessions) =====

export type DataOrigin = 'native_platform' | 'hub_recorded' | 'unavailable';

export interface ProfileField<T> {
  value: T | null;
  origin: DataOrigin;
  notes: string | null;
}

export type AvailabilityStatus = 'available' | 'missing_on_disk';

export type EmulatorStatus = 'unconfigured' | 'available' | 'missing_on_disk';

export type TrophySyncReadiness = 'unavailable' | 'phase4_planned' | 'ready';

export type SyncTrigger =
  | 'manual'
  | 'directory_changed'
  | 'scan_completed'
  | 'emulator_configured'
  | 'pre_launch'
  | 'session_ended';

export interface ConsoleProfileRecord {
  id: string;
  consoleId: string;
  consoleSlug: string;
  emulatorId: string | null;
  emulatorSlug: string | null;
  emulatorStatus: EmulatorStatus;
  detectedEmulatorVersion: string | null;
  detectedFirmwareVersion: string | null;
  emulatorRootDir: string | null;
  nativeSaveCount: number;
  nativeTrophyDirPresent: boolean;
  trophySyncReadiness: TrophySyncReadiness;
  nativeMetadataJson: string;
  syncStatus: 'never_synced' | 'synced' | 'partial' | 'error';
  syncDiagnostics: string;
  lastSyncedAt: string | null;
  updatedAt: string;
}

export interface ConsoleLocalUserRecord {
  id: string;
  consoleId: string;
  consoleSlug: string;
  nativeUserKey: string;
  displayName: string;
  sourcePath: string;
  availabilityStatus: AvailabilityStatus;
  linkedProfileId: string | null;
  firstDetectedAt: string;
  lastSeenAt: string;
}

export interface ConsoleHubProfileStatsDto {
  totalPlaytimeSeconds: number;
  totalSessions: number;
  gamesTotal: number;
  gamesAvailable: number;
  gamesMissingOnDisk: number;
  mostPlayed: TopGameStat[];
  recentlyPlayed: GameRecord[];
}

export interface ConsoleProfileOverviewDto {
  consoleSlug: string;
  consoleName: string;
  environmentMode: ConsoleEnvironmentMode;
  profile: ConsoleProfileRecord | null;
  localUsers: ConsoleLocalUserRecord[];
  emulatorStatus: ProfileField<EmulatorStatus>;
  emulatorVersion: ProfileField<string>;
  firmwareVersion: ProfileField<string>;
  emulatorRootDir: ProfileField<string>;
  nativeSaveCount: ProfileField<number>;
  trophyReadiness: ProfileField<TrophySyncReadiness>;
  hubStats: ConsoleHubProfileStatsDto;
  recentSessions: GameSessionRecord[];
}

export interface ConsoleSyncReport {
  consoleSlug: string;
  trigger: SyncTrigger;
  syncStatus: string;
  emulatorStatus: EmulatorStatus;
  localUsersDetected: number;
  localUsersMarkedMissing: number;
  gamesDiscovered: number;
  versionsAvailable: number;
  versionsMarkedMissing: number;
  coversImported: number;
  diagnostics: string[];
  syncedAt: string;
}

export type ActiveSessionPhase = 'idle' | 'launching' | 'running' | 'exiting';

export type LaunchTarget =
  | { kind: 'system_boot'; consoleSlug: string }
  | {
      kind: 'direct_game_boot';
      consoleSlug: string;
      gameId: string;
      gameVersionId: string | null;
    };

export interface ActiveSessionSnapshot {
  phase: ActiveSessionPhase;
  sessionId: string | null;
  target: LaunchTarget | null;
  consoleSlug: string | null;
  gameId: string | null;
  gameTitle: string | null;
  emulatorSlug: string | null;
  pid: number | null;
  startedAt: string | null;
  lastHeartbeatAt: string | null;
  elapsedSeconds: number;
}

export interface SessionExitedPayload {
  sessionId: string | null;
  consoleSlug: string;
  gameId: string | null;
  gameTitle: string | null;
  emulatorSlug: string;
  exitCode: number | null;
  exitStatus: string;
  durationSeconds: number;
  diagnostic: string;
  endedAt: string;
}

export interface RecordRuntimeVerificationRequest {
  consoleSlug: string;
  emulatorSlug: string;
  gameId: string | null;
  featureKey: RuntimeVerificationFeature;
  emulatorVersion: string;
  status: RuntimeVerificationStatus;
  evidenceNotes: string;
}

// ===== Phase 4 Domain Contracts (Universal Trophy Center & RPCS3 Read-Only Sync) =====

export interface TrophySetSummaryRow {
  id: string;
  consoleSlug: string;
  gameId: string;
  gameTitle: string;
  providerSlug: string;
  externalSetId: string;
  title: string;
  detail: string;
  iconPath: string | null;
  totalCount: number;
  unlockedCount: number;
  bronzeUnlocked: number;
  silverUnlocked: number;
  goldUnlocked: number;
  platinumUnlocked: number;
  supportsPlatinum: boolean;
  completionPercent: number;
  updatedAt: string;
}

export interface DetailedTrophyRow {
  id: string;
  trophySetId: string | null;
  gameId: string;
  gameTitle: string;
  consoleId: string;
  consoleSlug: string;
  providerId: string;
  providerSlug: string;
  externalTrophyId: string;
  name: string;
  description: string;
  iconPath: string | null;
  category: TrophyCategory;
  rarityPercent: number | null;
  origin: TrophyOrigin;
  isHidden: boolean;
  isUnlocked: boolean;
  unlockedAt: string | null;
  consoleLocalUserName: string | null;
}

export interface TrophyTotalsDto {
  totalSets: number;
  completedSets: number;
  totalTrophies: number;
  unlockedTrophies: number;
  bronzeTotal: number;
  bronzeUnlocked: number;
  silverTotal: number;
  silverUnlocked: number;
  goldTotal: number;
  goldUnlocked: number;
  platinumTotal: number;
  platinumUnlocked: number;
  completionPercent: number;
}

export interface TrophyCenterOverviewDto {
  providers: AchievementProviderRecord[];
  sets: TrophySetSummaryRow[];
  trophies: DetailedTrophyRow[];
  recentUnlocks: DetailedTrophyRow[];
  totals: TrophyTotalsDto;
}

export interface NewlyUnlockedTrophy {
  trophyId: string;
  externalTrophyId: string;
  gameId: string;
  gameTitle: string;
  name: string;
  category: TrophyCategory;
  unlockedAt: string;
  iconPath: string | null;
}

export interface TrophySyncReport {
  consoleSlug: string;
  setsSynced: number;
  trophiesSynced: number;
  unlocksRecorded: number;
  newlyUnlocked: NewlyUnlockedTrophy[];
  iconsCached: number;
  diagnostics: string[];
  syncedAt: string;
}

// ===== Visual Redesign: Extensible Theme Engine, Display Modes & Gamepad Contracts =====

export type DisplayMode = 'desktop' | 'fullscreen';

export type ThemeShellLayout = 'sidebar' | 'topbar_blades' | 'minimal_dock' | 'xmb_crossbar';
export type ThemeLibraryView = 'grid' | 'list';
export type ThemeCardAspect = 'poster' | 'square' | 'wide';
export type ThemeDensity = 'comfortable' | 'compact' | 'tv_large';
export type ThemeFocusStyle = 'ring' | 'glow_bar' | 'bracket' | 'xmb_wave';
export type ThemeSurfaceStyle = 'matte' | 'beveled' | 'flat' | 'layered';
export type ThemePageTransition = 'fade' | 'slide' | 'scale' | 'none';
export type ThemeCustomSettingKind = 'color' | 'select' | 'boolean' | 'range';

export interface ThemeFonts {
  display: string;
  body: string;
  mono: string;
}

export interface ThemePalette {
  canvas: string;
  rail: string;
  surface: string;
  surfaceRaised: string;
  input: string;
  borderSubtle: string;
  borderStrong: string;
  textPrimary: string;
  textSecondary: string;
  textMuted: string;
  accentPrimary: string;
  accentHover: string;
  accentMuted: string;
  accentSecondary: string;
  focusRing: string;
}

export interface ThemeLayoutConfig {
  shell: ThemeShellLayout;
  libraryDefaultView: ThemeLibraryView;
  cardAspect: ThemeCardAspect;
  density: ThemeDensity;
}

export interface ThemeComponentsConfig {
  focusStyle: ThemeFocusStyle;
  surfaceStyle: ThemeSurfaceStyle;
  backdropBlur: boolean;
  showHeroArt: boolean;
  showClock: boolean;
  radiusSm: number;
  radiusMd: number;
  radiusLg: number;
}

export interface ThemeMotionConfig {
  pageTransition: ThemePageTransition;
  durationMs: number;
  respectReducedMotion: boolean;
}

export interface ThemeSettingOption {
  value: string;
  label: string;
}

export interface ThemeCustomSettingDef {
  key: string;
  label: string;
  kind: ThemeCustomSettingKind;
  defaultValue: string;
  options?: ThemeSettingOption[];
  min?: number | null;
  max?: number | null;
}

export interface ThemeManifest {
  id: string;
  name: string;
  author: string;
  version: string;
  description: string;
  hubVersionRange: string;
  previewImage: string | null;
  backgroundImage: string | null;
  fonts: ThemeFonts;
  assets: Record<string, string>;
  palette: ThemePalette;
  layout: ThemeLayoutConfig;
  components: ThemeComponentsConfig;
  motion: ThemeMotionConfig;
  customizableSettings: ThemeCustomSettingDef[];
}

export interface InstalledThemePackage {
  manifest: ThemeManifest;
  installedDir: string;
  previewImagePath: string | null;
  backgroundImagePath: string | null;
  assetPaths: Record<string, string>;
  installedAt: string;
}

export type GamepadGlyphFamily = 'xbox' | 'playstation' | 'nintendo' | 'generic';

export type GamepadNavAction =
  | 'up'
  | 'down'
  | 'left'
  | 'right'
  | 'confirm'
  | 'back'
  | 'secondary'
  | 'context_menu'
  | 'prev_tab'
  | 'next_tab'
  | 'page_prev'
  | 'page_next'
  | 'quick_menu'
  | 'toggle_mode';

export interface ConnectedGamepadInfo {
  slot: number;
  name: string;
  backend: string;
  glyphFamily: GamepadGlyphFamily;
  connected: boolean;
}

export interface GamepadActionEvent {
  slot: number;
  action: GamepadNavAction;
  glyphFamily: GamepadGlyphFamily;
  repeated: boolean;
}
