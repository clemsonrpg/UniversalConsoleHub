import { CheckCircle2, Clock, ShieldAlert } from 'lucide-react';
import type { SystemBootReadiness } from '../types/domain';
import styles from './ReadinessTracker.module.css';

export interface ReadinessTrackerProps {
  readiness: SystemBootReadiness;
}

export function ReadinessTracker({ readiness }: ReadinessTrackerProps) {
  return (
    <section className={styles.panel} aria-label="Diagnóstico de prontidão em 4 estágios">
      <h3 className={styles.heading}>Prontidão de Boot Autêntico (4 Estágios Independentes)</h3>

      <ol className={styles.list}>
        <li className={styles.stage}>
          <div className={styles.stageHeader}>
            <span>1. Pacote de Firmware</span>
            {readiness.firmwarePackageValidated ? (
              <CheckCircle2 size={15} className={styles.ok} aria-hidden="true" />
            ) : (
              <Clock size={15} className={styles.pending} aria-hidden="true" />
            )}
          </div>
          <div
            className={`${styles.stageValue} ${
              readiness.firmwarePackageValidated ? styles.ok : styles.pending
            }`}
          >
            {readiness.firmwarePackageValidated ? 'PS3UPDAT.PUP validado' : 'Não validado'}
          </div>
        </li>

        <li className={styles.stage}>
          <div className={styles.stageHeader}>
            <span>2. Arquivos de Sistema</span>
            {readiness.systemFilesPresent ? (
              <CheckCircle2 size={15} className={styles.ok} aria-hidden="true" />
            ) : (
              <Clock size={15} className={styles.pending} aria-hidden="true" />
            )}
          </div>
          <div
            className={`${styles.stageValue} ${
              readiness.systemFilesPresent ? styles.ok : styles.pending
            }`}
          >
            {readiness.systemFilesPresent ? 'vsh.self e dev_hdd0 presentes' : 'Arquivos ausentes'}
          </div>
        </li>

        <li className={styles.stage}>
          <div className={styles.stageHeader}>
            <span>3. Comando Seguro Gerado</span>
            {readiness.commandBuildable ? (
              <CheckCircle2 size={15} className={styles.ok} aria-hidden="true" />
            ) : (
              <Clock size={15} className={styles.pending} aria-hidden="true" />
            )}
          </div>
          <div
            className={`${styles.stageValue} ${
              readiness.commandBuildable ? styles.ok : styles.pending
            }`}
          >
            {readiness.commandBuildable ? 'Pronto (sem shell)' : 'Bloqueado (sem vsh.self)'}
          </div>
        </li>

        <li className={styles.stage}>
          <div className={styles.stageHeader}>
            <span>4. Confirmação Runtime do XMB</span>
            {readiness.xmbRuntimeConfirmed ? (
              <CheckCircle2 size={15} className={styles.ok} aria-hidden="true" />
            ) : (
              <ShieldAlert size={15} className={styles.unconfirmed} aria-hidden="true" />
            )}
          </div>
          <div
            className={`${styles.stageValue} ${
              readiness.xmbRuntimeConfirmed ? styles.ok : styles.unconfirmed
            }`}
          >
            {readiness.xmbRuntimeConfirmed
              ? 'Confirmado em execução'
              : 'Não confirmado (Fase 4)'}
          </div>
        </li>
      </ol>

      {readiness.generatedCommand ? (
        <div className={styles.commandBox} aria-label="Comando seguro preparado">
          <code>
            {readiness.generatedCommand.executable}{' '}
            {readiness.generatedCommand.args.map((a) => `"${a}"`).join(' ')}
          </code>
        </div>
      ) : null}

      {readiness.notes.length > 0 ? (
        <ul className={styles.notes}>
          {readiness.notes.map((note) => (
            <li key={note}>{note}</li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}
