import { ReadinessTracker } from '../components/ReadinessTracker';
import { StatTile } from '../components/StatTile';
import type { HubSectionId } from '../hooks/useHubNavigation';
import type { HubBootstrapDto } from '../types/domain';
import styles from './Pages.module.css';

export interface HomePageProps {
  data: HubBootstrapDto;
  onNavigate: (section: HubSectionId) => void;
  onSelectConsole: (slug: string) => void;
}

function formatPlaytime(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  return `${hours}h ${minutes}m`;
}

export function HomePage({ data, onNavigate, onSelectConsole }: HomePageProps) {
  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <h1 className={styles.title}>Visão Geral do Hub</h1>
          <p className={styles.subtitle}>
            Bem-vindo, {data.profile.displayName}. O Universal Console Hub preserva interfaces
            originais via emuladores reais — sem dashboards falsas e sem distribuir arquivos
            proprietários.
          </p>
        </div>
        <div className={styles.actionsRow}>
          <button
            type="button"
            className={styles.btnPrimary}
            onClick={() => onSelectConsole('ps3')}
          >
            Inspecionar PlayStation 3
          </button>
          <button
            type="button"
            className={styles.btnSecondary}
            onClick={() => onNavigate('emulators')}
          >
            Configurar RPCS3 &amp; Firmware
          </button>
        </div>
      </header>

      <section className={styles.statGrid} aria-label="Resumo real do banco de dados">
        <StatTile label="Consoles no catálogo" value={data.consoles.length} hint="6 prioritários" />
        <StatTile
          label="Jogos registrados"
          value={data.statistics.gamesRegistered}
          hint="Biblioteca local SQLite"
        />
        <StatTile
          label="Tempo real jogado"
          value={formatPlaytime(data.statistics.totalPlaytimeSeconds)}
          hint="Baseado em sessões reais"
        />
        <StatTile
          label="Troféus desbloqueados"
          value={data.statistics.unlockedTrophies}
          hint={`de ${data.statistics.totalTrophies} registrados`}
        />
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Status da Preparação RPCS3 (PlayStation 3)</h2>
        <p className={styles.muted}>{data.rpcs3Inspection.diagnostic}</p>
        {data.rpcs3Inspection.readiness ? (
          <ReadinessTracker readiness={data.rpcs3Inspection.readiness} />
        ) : (
          <div className={styles.actionsRow}>
            <button
              type="button"
              className={styles.btnSecondary}
              onClick={() => onNavigate('emulators')}
            >
              Apontar executável rpcs3.exe
            </button>
          </div>
        )}
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Atividade Recente (SQLite)</h2>
        {data.recentActivity.length === 0 ? (
          <p className={styles.muted}>
            Nenhuma atividade registrada ainda. Atualize seu perfil ou registre um jogo na
            Biblioteca para ver o histórico real persistido em UTC.
          </p>
        ) : (
          <ul className={styles.list}>
            {data.recentActivity.map((entry) => (
              <li key={entry.id} className={styles.listItem}>
                <span>{entry.summary}</span>
                <span className={styles.mono}>{entry.occurredAt}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
