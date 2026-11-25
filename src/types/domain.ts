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
