import { useMemo } from 'react';
import { ConsoleCard } from '../components/ConsoleCard';
import type { ConsoleRecord, EmulatorRecord, GameRecord } from '../types/domain';
import styles from './Pages.module.css';

export interface ConsolesPageProps {
  consoles: ConsoleRecord[];
  emulators: EmulatorRecord[];
  games?: GameRecord[];
  onSelectConsole: (slug: string) => void;
}

export function ConsolesPage({
  consoles,
  emulators,
  games = [],
  onSelectConsole,
}: ConsolesPageProps) {
  const statsByConsole = useMemo(() => {
    const counts = new Map<string, number>();
    const lastPlayed = new Map<string, string>();
    for (const g of games) {
      counts.set(g.consoleSlug, (counts.get(g.consoleSlug) ?? 0) + 1);
      if (g.lastPlayedAt) {
        const prev = lastPlayed.get(g.consoleSlug);
        if (!prev || g.lastPlayedAt > prev) {
          lastPlayed.set(g.consoleSlug, g.lastPlayedAt);
        }
      }
    }
    return { counts, lastPlayed };
  }, [games]);

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <h1 className={styles.title}>Coleção de Consoles</h1>
          <p className={styles.subtitle}>
            Cada console possui perfil próprio, capacidades independentes reais (não mutuamente
            exclusivas) e estado verificado de integração no ecossistema.
          </p>
        </div>
      </header>

      <section className={styles.cardGrid} aria-label="Consoles prioritários">
        {consoles.map((consoleRecord, idx) => {
          const emu = emulators.find((e) => e.consoleSlug === consoleRecord.slug);
          return (
            <div key={consoleRecord.id} data-focus-default={idx === 0 ? 'true' : undefined}>
              <ConsoleCard
                consoleRecord={consoleRecord}
                emulatorName={emu?.name}
                emulatorConfigured={Boolean(emu?.executablePath)}
                gamesCount={statsByConsole.counts.get(consoleRecord.slug) ?? 0}
                lastActivityAt={statsByConsole.lastPlayed.get(consoleRecord.slug) ?? null}
                onSelect={onSelectConsole}
              />
            </div>
          );
        })}
      </section>
    </div>
  );
}
