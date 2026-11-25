import type { ReactNode } from 'react';
import {
  Cpu,
  Gamepad2,
  Home,
  Library,
  Settings,
  Trophy,
  User,
} from 'lucide-react';
import type { HubSectionId } from '../hooks/useHubNavigation';
import styles from './AppShell.module.css';

export interface AppShellProps {
  activeSection: HubSectionId;
  onSelectSection: (section: HubSectionId) => void;
  profileDisplayName: string;
  profileHandle: string;
  transportKind: 'tauri' | 'injected_test';
  children: ReactNode;
}

const NAV_ITEMS: Array<{ id: HubSectionId; label: string; Icon: typeof Home }> = [
  { id: 'home', label: 'Início', Icon: Home },
  { id: 'consoles', label: 'Consoles', Icon: Gamepad2 },
  { id: 'library', label: 'Biblioteca', Icon: Library },
  { id: 'profile', label: 'Perfil Universal', Icon: User },
  { id: 'trophies', label: 'Central de Troféus', Icon: Trophy },
  { id: 'emulators', label: 'Emuladores & Firmware', Icon: Cpu },
  { id: 'settings', label: 'Configurações', Icon: Settings },
];

export function AppShell({
  activeSection,
  onSelectSection,
  profileDisplayName,
  profileHandle,
  transportKind,
  children,
}: AppShellProps) {
  return (
    <div className={styles.shell}>
      <aside className={styles.rail} aria-label="Navegação principal do Hub">
        <div className={styles.brand}>
          <span className={styles.brandTitle}>Universal Console Hub</span>
          <span className={styles.brandSub}>Fase 1 · Fundação Desktop</span>
        </div>

        <nav aria-label="Seções">
          <ul className={styles.navList}>
            {NAV_ITEMS.map(({ id, label, Icon }) => {
              const isActive = activeSection === id;
              return (
                <li key={id}>
                  <button
                    type="button"
                    className={`${styles.navButton} ${isActive ? styles.navButtonActive : ''}`}
                    aria-current={isActive ? 'page' : undefined}
                    onClick={() => onSelectSection(id)}
                  >
                    <Icon size={17} aria-hidden="true" />
                    <span>{label}</span>
                  </button>
                </li>
              );
            })}
          </ul>
        </nav>

        <div className={styles.railFooter}>
          <span className={styles.profileName}>{profileDisplayName}</span>
          <span>@{profileHandle} · Offline</span>
          <span>
            Transporte: {transportKind === 'tauri' ? 'Tauri IPC + SQLite' : 'Transporte de Teste UI'}
          </span>
        </div>
      </aside>

      <main className={styles.main}>{children}</main>
    </div>
  );
}
