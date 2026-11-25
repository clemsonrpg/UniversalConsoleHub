import type { ConsoleRecord } from '../types/domain';
import { formatIntegrationStage } from '../utils/formatters';
import { CapabilityBadge } from './CapabilityBadge';
import styles from './ConsoleCard.module.css';

export interface ConsoleCardProps {
  consoleRecord: ConsoleRecord;
  emulatorName?: string;
  onSelect: (slug: string) => void;
}

export function ConsoleCard({ consoleRecord, emulatorName, onSelect }: ConsoleCardProps) {
  return (
    <button
      type="button"
      className={styles.card}
      onClick={() => onSelect(consoleRecord.slug)}
      aria-label={`Abrir detalhes de ${consoleRecord.name}`}
    >
      <div className={styles.top}>
        <div>
          <h3 className={styles.name}>{consoleRecord.name}</h3>
          <div className={styles.meta}>
            {consoleRecord.manufacturer} · {consoleRecord.releaseYear}
            {emulatorName ? ` · ${emulatorName}` : ''}
          </div>
        </div>
        <span className={styles.phaseTag}>
          {formatIntegrationStage(consoleRecord.phaseStatus)}
        </span>
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
