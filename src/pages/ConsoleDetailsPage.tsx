import { CapabilityBadge } from '../components/CapabilityBadge';
import { ReadinessTracker } from '../components/ReadinessTracker';
import type { ConsoleDetailsDto, FeatureKey } from '../types/domain';
import styles from './Pages.module.css';

export interface ConsoleDetailsPageProps {
  details: ConsoleDetailsDto;
  onBack: () => void;
  onOpenEmulators: () => void;
}

const FEATURE_LABELS: Record<FeatureKey, string> = {
  system_boot: 'Boot Completo do Sistema',
  original_dashboard: 'Dashboard / Menu Original',
  direct_game_boot: 'Execução Direta de Jogos',
  save_management: 'Gerenciamento de Saves',
  local_trophies: 'Troféus / Conquistas Locais',
  online_services: 'Serviços Online Oficiais',
};

export function ConsoleDetailsPage({
  details,
  onBack,
  onOpenEmulators,
}: ConsoleDetailsPageProps) {
  const { console: consoleRecord, emulator, firmwareRecords, rpcs3Inspection } = details;

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <h1 className={styles.title}>{consoleRecord.name}</h1>
          <p className={styles.subtitle}>
            {consoleRecord.manufacturer} · Geração {consoleRecord.generation} ·{' '}
            {consoleRecord.releaseYear}
            {emulator ? ` · Emulador associado: ${emulator.name}` : ''}
          </p>
        </div>
        <div className={styles.actionsRow}>
          <button type="button" className={styles.btnSecondary} onClick={onBack}>
            Voltar aos Consoles
          </button>
          <button type="button" className={styles.btnPrimary} onClick={onOpenEmulators}>
            Gerenciar Emulador &amp; Firmware
          </button>
        </div>
      </header>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Capacidades Declaradas Simultaneamente</h2>
        <div className={styles.badgeRow}>
          {consoleRecord.capabilities.supportedStates.map((state) => (
            <CapabilityBadge key={state} state={state} />
          ))}
        </div>
        <p className={styles.muted}>{consoleRecord.notes}</p>
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Matriz de Capacidades por Funcionalidade</h2>
        <div className={styles.tableWrap}>
          <table className={styles.table}>
            <thead>
              <tr>
                <th scope="col">Funcionalidade</th>
                <th scope="col">Estados</th>
                <th scope="col">Disponibilidade</th>
                <th scope="col">Nível de Verificação</th>
                <th scope="col">Observações Reais</th>
              </tr>
            </thead>
            <tbody>
              {consoleRecord.capabilities.features.map((feat) => (
                <tr key={feat.feature}>
                  <td>{FEATURE_LABELS[feat.feature]}</td>
                  <td>
                    <div className={styles.badgeRow}>
                      {feat.states.map((s) => (
                        <CapabilityBadge key={s} state={s} />
                      ))}
                    </div>
                  </td>
                  <td className={styles.mono}>{feat.availability}</td>
                  <td className={styles.mono}>{feat.verification}</td>
                  <td>{feat.notes}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      {rpcs3Inspection ? (
        <section className={styles.section}>
          <h2 className={styles.sectionTitle}>Preparação RPCS3 (Escopo da Fase 1)</h2>
          <p className={styles.muted}>{rpcs3Inspection.diagnostic}</p>
          {rpcs3Inspection.readiness ? (
            <ReadinessTracker readiness={rpcs3Inspection.readiness} />
          ) : null}
          <div className={`${styles.banner} ${styles.bannerInfo}`}>
            Execução ao vivo do RPCS3, monitoramento de sessão e retorno ao Hub pertencem à Fase 4.
            Na Fase 1, nenhum botão simulado de lançamento do XMB é exibido.
          </div>
        </section>
      ) : null}

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Arquivos de Sistema Validados Localmente</h2>
        {firmwareRecords.length === 0 ? (
          <p className={styles.muted}>
            Nenhum arquivo de firmware/BIOS validado para este console. Forneça apenas arquivos
            legais obtidos pelo próprio usuário.
          </p>
        ) : (
          <ul className={styles.list}>
            {firmwareRecords.map((fw) => (
              <li key={fw.id} className={styles.listItem}>
                <div>
                  <div className={styles.mono}>{fw.filePath}</div>
                  <div className={styles.muted}>
                    Status: {fw.validationStatus} · SHA-256: {fw.sha256.slice(0, 16)}…
                  </div>
                </div>
                <span className={styles.mono}>{fw.validatedAt}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
