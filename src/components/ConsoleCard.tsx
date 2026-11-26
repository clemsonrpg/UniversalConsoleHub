import type { ConsoleRecord } from '../types/domain';
import { formatIntegrationStage } from '../utils/formatters';
import { CapabilityBadge } from './CapabilityBadge';
import styles from './ConsoleCard.module.css';

export interface ConsoleCardProps {
  consoleRecord: ConsoleRecord;
  emulatorName?: string;
  emulatorConfigured?: boolean;
  gamesCount?: number;
  lastActivityAt?: string | null;
  onSelect: (slug: string) => void;
}

function platformAccentClass(slug: string): string {
  if (slug.startsWith('ps')) return styles.accentSony ?? '';
  if (slug.startsWith('xbox')) return styles.accentXbox ?? '';
  if (slug === 'wii' || slug === 'gamecube') return styles.accentNintendo ?? '';
  return '';
}

function platformShortCode(slug: string): string {
  const map: Record<string, string> = {
    ps3: 'PS3',
    ps2: 'PS2',
    ps4: 'PS4',
    wii: 'WII',
    xbox: 'XBX',
    xbox360: '360',
  };
  return map[slug] ?? slug.slice(0, 3).toUpperCase();
}

export function ConsoleCard({
  consoleRecord,
  emulatorName,
  emulatorConfigured = false,
  gamesCount = 0,
  lastActivityAt = null,
  onSelect,
}: ConsoleCardProps) {
  const supportsAuthentic =
    consoleRecord.capabilities.supportedStates.includes('FULL_SYSTEM_BOOT') &&
    consoleRecord.capabilities.supportedStates.includes('ORIGINAL_DASHBOARD');

  return (
    <button
      type="button"
      className={styles.card}
      data-focus-id={`console-card-${consoleRecord.slug}`}
      onClick={() => onSelect(consoleRecord.slug)}
      aria-label={`Abrir detalhes de ${consoleRecord.name}`}
    >
      <span
        className={`${styles.accentStrip} ${platformAccentClass(consoleRecord.slug)}`}
        aria-hidden="true"
      />
      <div className={styles.top}>
        <div className={styles.identity}>
          <span className={styles.monogram} aria-hidden="true">
            {platformShortCode(consoleRecord.slug)}
          </span>
          <div>
            <h3 className={styles.name}>{consoleRecord.name}</h3>
            <div className={styles.meta}>
              {consoleRecord.manufacturer} · Geração {consoleRecord.generation} ·{' '}
              {consoleRecord.releaseYear}
              {emulatorName ? ` · ${emulatorName}` : ''}
            </div>
          </div>
        </div>
        <span className={styles.phaseTag}>
          {formatIntegrationStage(consoleRecord.phaseStatus)}
        </span>
      </div>

      <div className={styles.statsRow}>
        <div className={styles.statItem}>
          <span>Jogos detectados</span>
          <span className={styles.statValue}>{gamesCount}</span>
        </div>
        <div className={styles.statItem}>
          <span>Ambiente</span>
          <span className={styles.statValue}>
            {supportsAuthentic ? 'Sistema Original + Jogo Direto' : 'Ambiente Integrado do Hub'}
          </span>
        </div>
        <div className={styles.statItem}>
          <span>Emulador</span>
          <span className={styles.statValue}>
            {emulatorConfigured ? 'Configurado' : 'Aguardando executável'}
          </span>
        </div>
        <div className={styles.statItem}>
          <span>Última atividade</span>
          <span className={styles.statValue}>{lastActivityAt ?? 'Sem sessões ainda'}</span>
        </div>
      </div>

      <div className={styles.badges}>
        {consoleRecord.capabilities.supportedStates.map((state) => (
          <CapabilityBadge key={state} state={state} />
        ))}
      </div>

      <p className={styles.notes}>{consoleRecord.notes}</p>
    </button>
  );
}
