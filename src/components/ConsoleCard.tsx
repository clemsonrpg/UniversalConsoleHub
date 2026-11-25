import type { ConsoleRecord } from '../types/domain';
import { CapabilityBadge } from './CapabilityBadge';
import styles from './ConsoleCard.module.css';

export interface ConsoleCardProps {
  consoleRecord: ConsoleRecord;
  emulatorName?: string;
  onSelect: (slug: string) => void;
}

function formatPhaseLabel(status: string): string {
  switch (status) {
    case 'phase_1_prepared_phase_4_full':
      return 'Fase 1: Prep · Fase 4: Completo';
    case 'planned_phase_5_experimental':
      return 'Fase 5 · Experimental';
    case 'planned_phase_5':
      return 'Planejado · Fase 5';
    default:
      return status;
  }
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
        <span className={styles.phaseTag}>{formatPhaseLabel(consoleRecord.phaseStatus)}</span>
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
