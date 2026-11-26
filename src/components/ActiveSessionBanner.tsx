import { useEffect, useState } from 'react';
import type { ActiveSessionSnapshot, DataOrigin } from '../types/domain';
import { formatDataOrigin } from '../utils/formatters';
import styles from './ActiveSessionBanner.module.css';

export function DataOriginBadge({ origin }: { origin: DataOrigin }) {
  const cls =
    origin === 'native_platform'
      ? styles.originNative
      : origin === 'hub_recorded'
        ? styles.originHub
        : styles.originUnavailable;
  return <span className={`${styles.originBadge} ${cls}`}>{formatDataOrigin(origin)}</span>;
}

function formatClock(totalSeconds: number): string {
  const safe = Math.max(0, Math.floor(totalSeconds));
  const h = Math.floor(safe / 3600);
  const m = Math.floor((safe % 3600) / 60);
  const s = safe % 60;
  return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

export interface ActiveSessionBannerProps {
  session: ActiveSessionSnapshot;
  onStopSession: () => Promise<void>;
}

export function ActiveSessionBanner({ session, onStopSession }: ActiveSessionBannerProps) {
  const [stopping, setStopping] = useState(false);
  const [tickSeconds, setTickSeconds] = useState(session.elapsedSeconds);

  useEffect(() => {
    setTickSeconds(session.elapsedSeconds);
    if (session.phase !== 'running') return;
    const id = window.setInterval(() => {
      setTickSeconds((prev) => prev + 1);
    }, 1000);
    return () => window.clearInterval(id);
  }, [session.phase, session.elapsedSeconds, session.startedAt]);

  if (session.phase === 'idle') return null;

  const phaseText =
    session.phase === 'launching'
      ? 'Iniciando processo do emulador...'
      : session.phase === 'exiting'
        ? 'Encerrando sessão e sincronizando perfil...'
        : 'Em execução · Navegação por atalhos do Hub suspensa';

  const handleStop = async () => {
    setStopping(true);
    try {
      await onStopSession();
    } finally {
      setStopping(false);
    }
  };

  return (
    <section
      className={styles.sessionBar}
      role="status"
      aria-live="polite"
      aria-label="Sessão ativa do emulador"
    >
      <div className={styles.sessionInfo}>
        <div className={styles.sessionTitleRow}>
          <span className={styles.liveDot} aria-hidden="true" />
          <span>{session.gameTitle ?? 'Sessão Ativa'}</span>
          {session.consoleSlug ? <span>({session.consoleSlug.toUpperCase()})</span> : null}
        </div>
        <div className={styles.sessionMeta}>
          {phaseText} · Emulador: {session.emulatorSlug?.toUpperCase() ?? '—'}
          {session.pid ? ` · PID ${session.pid}` : ''} · Tempo: {formatClock(tickSeconds)}
        </div>
      </div>

      <button
        type="button"
        className={styles.stopBtn}
        disabled={stopping || session.phase !== 'running'}
        onClick={() => void handleStop()}
      >
        {stopping ? 'Encerrando...' : 'Encerrar Sessão'}
      </button>
    </section>
  );
}
