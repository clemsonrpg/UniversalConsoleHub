import { ConsoleCard } from '../components/ConsoleCard';
import type { ConsoleRecord, EmulatorRecord } from '../types/domain';
import styles from './Pages.module.css';

export interface ConsolesPageProps {
  consoles: ConsoleRecord[];
  emulators: EmulatorRecord[];
  onSelectConsole: (slug: string) => void;
}

export function ConsolesPage({ consoles, emulators, onSelectConsole }: ConsolesPageProps) {
  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <h1 className={styles.title}>Coleção de Consoles</h1>
          <p className={styles.subtitle}>
            Cada console exibe suas capacidades independentes reais (não mutuamente exclusivas) e o
            estágio real de integração no Roadmap.
          </p>
        </div>
      </header>

      <section className={styles.cardGrid} aria-label="Consoles prioritários">
        {consoles.map((consoleRecord) => {
          const emu = emulators.find((e) => e.consoleSlug === consoleRecord.slug);
          return (
            <ConsoleCard
              key={consoleRecord.id}
              consoleRecord={consoleRecord}
              emulatorName={emu?.name}
              onSelect={onSelectConsole}
            />
          );
        })}
      </section>
    </div>
  );
}
