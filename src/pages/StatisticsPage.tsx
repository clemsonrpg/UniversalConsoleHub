import { useState } from 'react';
import { StatTile } from '../components/StatTile';
import type { ActivityEntryDto, StatisticsOverview } from '../types/domain';
import { formatActivityEventType, formatPlaytime } from '../utils/formatters';
import styles from './Pages.module.css';

export interface StatisticsPageProps {
  overview: StatisticsOverview;
  activity: ActivityEntryDto[];
  onFilterActivity: (eventType: string) => Promise<ActivityEntryDto[]>;
}

const EVENT_FILTERS = [
  { value: 'all', label: 'Todos os eventos' },
  { value: 'library_scanned', label: 'Biblioteca escaneada' },
  { value: 'game_registered', label: 'Jogo registrado' },
  { value: 'game_metadata_updated', label: 'Metadados atualizados' },
  { value: 'session_completed', label: 'Sessão encerrada' },
  { value: 'session_recovered', label: 'Sessão recuperada' },
  { value: 'profile_updated', label: 'Perfil atualizado' },
];

export function StatisticsPage({
  overview,
  activity,
  onFilterActivity,
}: StatisticsPageProps) {
  const [filter, setFilter] = useState('all');
  const [items, setItems] = useState<ActivityEntryDto[]>(activity);
  const [loading, setLoading] = useState(false);

  const handleFilterChange = async (next: string) => {
    setFilter(next);
    setLoading(true);
    try {
      const result = await onFilterActivity(next);
      setItems(result);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <h1 className={styles.title}>Estatísticas &amp; Histórico</h1>
          <p className={styles.subtitle}>
            Todos os indicadores abaixo são derivados exclusivamente de registros reais gravados no
            banco local SQLite (sem dados sintéticos ou estimativas inventadas).
          </p>
        </div>
      </header>

      <section className={styles.statGrid} aria-label="Resumo geral de estatísticas">
        <StatTile
          label="Tempo total jogado"
          value={formatPlaytime(overview.totalPlaytimeSeconds)}
          hint={`${overview.totalSessions} sessão(ões) reais`}
        />
        <StatTile
          label="Jogos registrados"
          value={overview.gamesRegistered}
          hint={`${overview.gamesStarted} iniciado(s)`}
        />
        <StatTile
          label="Jogos concluídos"
          value={overview.gamesCompleted}
          hint={`Taxa de conclusão: ${overview.completionRatePercent}%`}
        />
        <StatTile
          label="Meses com atividade"
          value={overview.monthlyPlaytime.length}
          hint="Histórico real UTC"
        />
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Distribuição por Console</h2>
        <div className={styles.tableWrap}>
          <table className={styles.table}>
            <thead>
              <tr>
                <th scope="col">Console</th>
                <th scope="col">Jogos</th>
                <th scope="col">Sessões</th>
                <th scope="col">Tempo Total</th>
              </tr>
            </thead>
            <tbody>
              {overview.byConsole.map((row) => (
                <tr key={row.consoleSlug}>
                  <td>
                    <strong>{row.consoleName}</strong>
                  </td>
                  <td>{row.gamesCount}</td>
                  <td>{row.sessionsCount}</td>
                  <td>{formatPlaytime(row.totalPlaytimeSeconds)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Evolução Mensal &amp; Jogos Mais Jogados</h2>
        {overview.topGames.length === 0 && overview.monthlyPlaytime.length === 0 ? (
          <p className={styles.muted}>
            Nenhuma sessão de jogo concluída ainda. Assim que sessões reais forem registradas, o
            ranking de títulos e a linha mensal aparecerão aqui automaticamente.
          </p>
        ) : (
          <div className={styles.formGrid}>
            <div>
              <h3 className={styles.label}>Top Jogos</h3>
              <ul className={styles.list}>
                {overview.topGames.map((g) => (
                  <li key={g.gameId} className={styles.listItem}>
                    <span>
                      <strong>{g.title}</strong> ({g.consoleSlug.toUpperCase()})
                    </span>
                    <span>
                      {formatPlaytime(g.totalPlaytimeSeconds)} · {g.sessionsCount} sessão(ões)
                    </span>
                  </li>
                ))}
              </ul>
            </div>
            <div>
              <h3 className={styles.label}>Playtime por Mês (UTC)</h3>
              <ul className={styles.list}>
                {overview.monthlyPlaytime.map((m) => (
                  <li key={m.month} className={styles.listItem}>
                    <span className={styles.mono}>{m.month}</span>
                    <span>
                      {formatPlaytime(m.totalPlaytimeSeconds)} · {m.sessionsCount} sessão(ões)
                    </span>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        )}
      </section>

      <section className={styles.section}>
        <div className={styles.header}>
          <h2 className={styles.sectionTitle}>Histórico de Atividades ({items.length})</h2>
          <select
            aria-label="Filtrar histórico por tipo"
            className={styles.select}
            value={filter}
            onChange={(e) => void handleFilterChange(e.target.value)}
            disabled={loading}
          >
            {EVENT_FILTERS.map((opt) => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>

        {items.length === 0 ? (
          <p className={styles.muted}>Nenhum evento encontrado para o filtro selecionado.</p>
        ) : (
          <ul className={styles.list}>
            {items.map((entry) => (
              <li key={entry.id} className={styles.listItem}>
                <div>
                  <strong>{formatActivityEventType(entry.eventType)}</strong>
                  <div className={styles.muted}>{entry.summary}</div>
                </div>
                <span className={styles.mono}>{entry.occurredAt}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
