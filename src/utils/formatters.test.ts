import { describe, expect, it } from 'vitest';
import {
  formatAvailabilityStatus,
  formatBytes,
  formatFeatureKey,
  formatFirmwareValidationStatus,
  formatIntegrationStage,
  formatMetadataSource,
  formatPlaytime,
  formatRuntimeVerificationFeature,
  formatRuntimeVerificationStatus,
  formatSessionExitStatus,
  formatTrophyOrigin,
  formatVerificationLevel,
} from './formatters';

describe('friendly PT-BR formatters for technical identifiers', () => {
  it('formats all emulator/console phase slugs without leaking raw technical tokens', () => {
    expect(formatIntegrationStage('phase_1_preparation')).toBe('Fase 1 · Preparação');
    expect(formatIntegrationStage('phase_1_prepared_phase_4_full')).toBe(
      'Preparado · Jogável na Fase 3'
    );
    expect(formatIntegrationStage('planned_phase_5')).toBe('Planejado · Fase 5');
    expect(formatIntegrationStage('planned_phase_5_experimental')).toBe(
      'Fase 5 · Experimental'
    );
  });

  it('formats availability, verification, trophy origin, firmware, and session statuses cleanly', () => {
    expect(formatFeatureKey('system_boot')).toBe('Boot Completo do Sistema');
    expect(formatAvailabilityStatus('requires_user_files')).toBe('Requer arquivos do usuário');
    expect(formatAvailabilityStatus('ready_offline')).toBe('Pronto (Offline)');
    expect(formatAvailabilityStatus('unavailable_no_official_api')).toBe(
      'Indisponível (sem API oficial)'
    );
    expect(formatAvailabilityStatus('planned_phase_3')).toBe('Planejado · Fase 4');
    expect(formatAvailabilityStatus('planned_phase_6_authorized_key')).toBe(
      'Planejado · Fase 6 (API autorizada)'
    );
    expect(formatVerificationLevel('documented_upstream')).toBe('Documentado upstream');
    expect(formatVerificationLevel('files_verified')).toBe('Arquivos verificados');
    expect(formatVerificationLevel('runtime_confirmed')).toBe('Confirmado em execução');
    expect(formatTrophyOrigin('local')).toBe('Local (Hub)');
    expect(formatTrophyOrigin('emulator')).toBe('Emulador');
    expect(formatTrophyOrigin('official')).toBe('Oficial');
    expect(formatFirmwareValidationStatus('package_header_and_hash_validated')).toBe(
      'Cabeçalho e SHA-256 validados'
    );
    expect(formatMetadataSource('ps3_param_sfo')).toBe('PARAM.SFO (PS3)');
    expect(formatSessionExitStatus('recovered_after_crash')).toBe('Recuperada após queda');
    expect(formatRuntimeVerificationFeature('per_game_compatibility')).toBe(
      'Compatibilidade por Jogo'
    );
    expect(formatRuntimeVerificationStatus('files_verified')).toBe('Arquivos verificados');
  });

  it('formats playtime durations and byte sizes clearly', () => {
    expect(formatPlaytime(0)).toBe('0 min');
    expect(formatPlaytime(90)).toBe('1 min');
    expect(formatPlaytime(3660)).toBe('1h 01min');
    expect(formatBytes(512)).toBe('512 B');
    expect(formatBytes(2048)).toBe('2.0 KB');
    expect(formatBytes(3 * 1024 * 1024)).toBe('3.0 MB');
  });
});
