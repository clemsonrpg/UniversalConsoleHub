import type { AchievementProviderRecord, TrophyRecord } from '../types/domain';
import styles from './Pages.module.css';

export interface TrophiesPageProps {
  providers: AchievementProviderRecord[];
  trophies: TrophyRecord[];
}

export function TrophiesPage({ providers, trophies }: TrophiesPageProps) {
  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <h1 className={styles.title}>Central de Troféus</h1>
          <p className={styles.subtitle}>
            Separação rigorosa e imutável entre origens `official`, `emulator` e `local` protegida
            no domínio Rust e por triggers SQLite. Nenhuma conquista oficial é falsificada.
          </p>
        </div>
      </header>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Providers de Conquistas ({providers.length})</h2>
        <div className={styles.tableWrap}>
          <table className={styles.table}>
            <thead>
              <tr>
                <th scope="col">Provider</th>
                <th scope="col">Origem Imutável</th>
                <th scope="col">Estado Real</th>
                <th scope="col">Detalhes</th>
              </tr>
            </thead>
            <tbody>
              {providers.map((provider) => (
                <tr key={provider.id}>
                  <td>
                    <strong>{provider.name}</strong>
                  </td>
                  <td className={styles.mono}>{provider.origin}</td>
                  <td className={styles.mono}>{provider.availabilityStatus}</td>
                  <td>{provider.notes}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Coleção de Troféus ({trophies.length})</h2>
        {trophies.length === 0 ? (
          <p className={styles.muted}>
            Nenhum troféu registrado ainda. O leitor local somente leitura do RPCS3 (`TROPCONF.SFM`
            + `TROPUSR.DAT`) e o fluxo de notificações entram na Fase 3.
          </p>
        ) : (
          <ul className={styles.list}>
            {trophies.map((trophy) => (
              <li key={trophy.id} className={styles.listItem}>
                <div>
                  <strong>{trophy.name}</strong>
                  <div className={styles.muted}>
                    {trophy.description} · Categoria: {trophy.category} · Origem: {trophy.origin}
                  </div>
                </div>
                <span className={styles.mono}>
                  {trophy.isUnlocked ? trophy.unlockedAt ?? 'Desbloqueado' : 'Bloqueado'}
                </span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
