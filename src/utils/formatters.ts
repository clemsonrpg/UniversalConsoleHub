import type {
  DataOrigin,
  EmulatorStatus,
  FeatureKey,
  RuntimeVerificationFeature,
  RuntimeVerificationStatus,
  TrophyCategory,
  TrophyOrigin,
  TrophySyncReadiness,
  VerificationLevel,
} from '../types/domain';

const INTEGRATION_STAGE_LABELS: Record<string, string> = {
  phase_1_preparation: 'Fase 1 · Preparação',
  phase_1_prepared_phase_4_full: 'Preparado · Jogável na Fase 3',
  phase_2_console_environment: 'Fase 2 · Ambiente & Scanner',
  phase_3_playable: 'Fase 3 · Jogável',
  planned_phase_5: 'Planejado · Fase 5',
  planned_phase_5_experimental: 'Fase 5 · Experimental',
};

const FEATURE_LABELS: Record<FeatureKey, string> = {
  system_boot: 'Boot Completo do Sistema',
  original_dashboard: 'Dashboard / Menu Original',
  direct_game_boot: 'Execução Direta de Jogos',
  save_management: 'Gerenciamento de Saves',
  local_trophies: 'Troféus / Conquistas Locais',
  online_services: 'Serviços Online Oficiais',
};

const AVAILABILITY_LABELS: Record<string, string> = {
  available: 'Disponível',
  requires_user_files: 'Requer arquivos do usuário',
  partial: 'Parcial',
  experimental: 'Experimental',
  planned: 'Planejado',
  unsupported: 'Não suportado',
  ready_offline: 'Pronto (Offline)',
  planned_phase_3: 'Planejado · Fase 4',
  planned_phase_4: 'Planejado · Fase 4',
  planned_phase_6_authorized_key: 'Planejado · Fase 6 (API autorizada)',
  unavailable_no_official_api: 'Indisponível (sem API oficial)',
};

const VERIFICATION_LABELS: Record<VerificationLevel | string, string> = {
  unverified: 'Não verificado',
  documented_upstream: 'Documentado upstream',
  files_verified: 'Arquivos verificados',
  runtime_confirmed: 'Confirmado em execução',
  failed: 'Falhou na verificação',
};

const RUNTIME_FEATURE_LABELS: Record<RuntimeVerificationFeature, string> = {
  emulator_launch: 'Execução do Emulador',
  system_boot: 'Boot do Sistema Original',
  original_dashboard: 'Dashboard Original',
  game_boot: 'Inicialização de Jogo',
  per_game_compatibility: 'Compatibilidade por Jogo',
  session_monitoring: 'Monitoramento de Sessão',
  hub_return: 'Retorno ao Hub',
};

const TROPHY_ORIGIN_LABELS: Record<TrophyOrigin, string> = {
  official: 'Oficial',
  emulator: 'Emulador',
  local: 'Local (Hub)',
};

const TROPHY_CATEGORY_LABELS: Record<TrophyCategory, string> = {
  bronze: 'Bronze',
  silver: 'Prata',
  gold: 'Ouro',
  platinum: 'Platina',
  xbox_achievement: 'Conquista Xbox',
  steam_achievement: 'Conquista Steam',
  emulator_achievement: 'Emulador',
  hub_local: 'Local (Hub)',
};

const FIRMWARE_STATUS_LABELS: Record<string, string> = {
  package_header_and_hash_validated: 'Cabeçalho e SHA-256 validados',
  file_hash_recorded_phase_1: 'Hash SHA-256 registrado localmente',
  header_unrecognized: 'Cabeçalho não reconhecido',
};

const METADATA_SOURCE_LABELS: Record<string, string> = {
  manual_user_entry: 'Cadastro manual',
  ps3_param_sfo: 'PARAM.SFO (PS3)',
  rpcs3_trophy_set: 'Conjunto de Troféus RPCS3',
  filesystem_filename_scan: 'Scanner de arquivos locais',
};

const SESSION_EXIT_LABELS: Record<string, string> = {
  running: 'Em andamento',
  clean_exit: 'Encerrada normalmente',
  non_zero_exit: 'Encerrada com erro',
  user_stopped: 'Encerrada pelo usuário',
  recovered_after_crash: 'Recuperada após queda',
  launch_failed: 'Falha ao iniciar',
};

const ACTIVITY_EVENT_LABELS: Record<string, string> = {
  profile_updated: 'Perfil atualizado',
  game_registered: 'Jogo registrado',
  game_metadata_updated: 'Metadados atualizados',
  library_scanned: 'Biblioteca escaneada',
  console_profile_synced: 'Perfil do console sincronizado',
  session_completed: 'Sessão encerrada',
  session_recovered: 'Sessão recuperada',
  trophies_unlocked: 'Troféus desbloqueados',
};

const DATA_ORIGIN_LABELS: Record<DataOrigin, string> = {
  native_platform: 'Nativo',
  hub_recorded: 'Hub',
  unavailable: 'Indisponível',
};

const EMULATOR_STATUS_LABELS: Record<EmulatorStatus, string> = {
  unconfigured: 'Não configurado',
  available: 'Disponível',
  missing_on_disk: 'Ausente no disco',
};

const TROPHY_READINESS_LABELS: Record<TrophySyncReadiness, string> = {
  unavailable: 'Sem sistema nativo',
  phase4_planned: 'Planejado · Fase 4',
  ready: 'Pronto',
};

const SYNC_STATUS_LABELS: Record<string, string> = {
  never_synced: 'Nunca sincronizado',
  synced: 'Sincronizado',
  partial: 'Sincronização parcial',
  error: 'Erro na sincronização',
};

const INSTALL_STATE_LABELS: Record<string, string> = {
  installed: 'Disponível no disco',
  registered: 'Disponível no disco',
  missing_files: 'Ausente no disco',
};

function fallbackHumanize(raw: string): string {
  return raw
    .replace(/_/g, ' ')
    .replace(/\b\w/g, (ch) => ch.toUpperCase());
}

export function formatIntegrationStage(raw: string): string {
  return INTEGRATION_STAGE_LABELS[raw] ?? fallbackHumanize(raw);
}

export function formatFeatureKey(key: FeatureKey): string {
  return FEATURE_LABELS[key] ?? fallbackHumanize(key);
}

export function formatAvailabilityStatus(raw: string): string {
  return AVAILABILITY_LABELS[raw] ?? fallbackHumanize(raw);
}

export function formatVerificationLevel(raw: string): string {
  return VERIFICATION_LABELS[raw] ?? fallbackHumanize(raw);
}

export function formatRuntimeVerificationFeature(key: RuntimeVerificationFeature): string {
  return RUNTIME_FEATURE_LABELS[key] ?? fallbackHumanize(key);
}

export function formatRuntimeVerificationStatus(status: RuntimeVerificationStatus): string {
  return VERIFICATION_LABELS[status] ?? fallbackHumanize(status);
}

export function formatTrophyOrigin(origin: TrophyOrigin): string {
  return TROPHY_ORIGIN_LABELS[origin] ?? fallbackHumanize(origin);
}

export function formatTrophyCategory(category: TrophyCategory): string {
  return TROPHY_CATEGORY_LABELS[category] ?? fallbackHumanize(category);
}

export function formatFirmwareValidationStatus(raw: string): string {
  return FIRMWARE_STATUS_LABELS[raw] ?? fallbackHumanize(raw);
}

export function formatMetadataSource(raw: string): string {
  return METADATA_SOURCE_LABELS[raw] ?? fallbackHumanize(raw);
}

export function formatSessionExitStatus(raw: string): string {
  return SESSION_EXIT_LABELS[raw] ?? fallbackHumanize(raw);
}

export function formatActivityEventType(raw: string): string {
  return ACTIVITY_EVENT_LABELS[raw] ?? fallbackHumanize(raw);
}

export function formatDataOrigin(origin: DataOrigin): string {
  return DATA_ORIGIN_LABELS[origin] ?? fallbackHumanize(origin);
}

export function formatEmulatorStatus(status: EmulatorStatus): string {
  return EMULATOR_STATUS_LABELS[status] ?? fallbackHumanize(status);
}

export function formatTrophyReadiness(readiness: TrophySyncReadiness): string {
  return TROPHY_READINESS_LABELS[readiness] ?? fallbackHumanize(readiness);
}

export function formatSyncStatus(status: string): string {
  return SYNC_STATUS_LABELS[status] ?? fallbackHumanize(status);
}

export function formatGameAvailability(installState: string): string {
  return INSTALL_STATE_LABELS[installState] ?? fallbackHumanize(installState);
}

export function formatPlaytime(totalSeconds: number): string {
  const safe = Math.max(0, Math.floor(totalSeconds));
  const hours = Math.floor(safe / 3600);
  const minutes = Math.floor((safe % 3600) / 60);
  if (hours === 0) {
    return `${minutes} min`;
  }
  return `${hours}h ${String(minutes).padStart(2, '0')}min`;
}

export function formatBytes(bytes: number): string {
  const safe = Math.max(0, bytes);
  if (safe < 1024) return `${safe} B`;
  if (safe < 1024 * 1024) return `${(safe / 1024).toFixed(1)} KB`;
  if (safe < 1024 * 1024 * 1024) return `${(safe / (1024 * 1024)).toFixed(1)} MB`;
  return `${(safe / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}
