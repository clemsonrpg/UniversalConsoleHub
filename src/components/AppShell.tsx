import { useEffect, useState, type ReactNode } from 'react';
import {
  BarChart3,
  Cpu,
  Gamepad2,
  Home,
  Library,
  Maximize2,
  Minimize2,
  Palette,
  Settings,
  SlidersHorizontal,
  Trophy,
  User,
} from 'lucide-react';
import type { HubSectionId } from '../hooks/useHubNavigation';
import { useInput } from '../input/InputProvider';
import { useTheme } from '../themes/ThemeProvider';
import { ControllerActionBar } from './ControllerActionBar';
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
  { id: 'statistics', label: 'Estatísticas', Icon: BarChart3 },
  { id: 'trophies', label: 'Central de Troféus', Icon: Trophy },
  { id: 'themes', label: 'Temas', Icon: Palette },
  { id: 'emulators', label: 'Emuladores & Firmware', Icon: Cpu },
  { id: 'settings', label: 'Configurações', Icon: Settings },
];

function useLiveClock(enabled: boolean): string {
  const [time, setTime] = useState(() =>
    new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  );
  useEffect(() => {
    if (!enabled) return;
    const id = window.setInterval(() => {
      setTime(new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }));
    }, 30000);
    return () => window.clearInterval(id);
  }, [enabled]);
  return time;
}

export function AppShell({
  activeSection,
  onSelectSection,
  profileDisplayName,
  profileHandle,
  transportKind,
  children,
}: AppShellProps) {
  const { effectiveTheme, previewTheme, activeOverrides, applyPreview, cancelPreview } = useTheme();
  const { displayMode, toggleDisplayMode, setQuickMenuOpen, glyphFor } = useInput();

  const shellLayout = effectiveTheme.manifest.layout.shell;
  const showClock =
    activeOverrides['showClock'] !== undefined
      ? activeOverrides['showClock'] === 'true'
      : effectiveTheme.manifest.components.showClock;
  const clockStr = useLiveClock(showClock);

  const transportLabel =
    transportKind === 'tauri' ? 'Tauri IPC + SQLite' : 'Transporte de Teste UI';

  const renderUtilityControls = () => (
    <div className={styles.utilityRow}>
      {showClock ? (
        <span className={styles.clock} aria-label="Relógio local">
          {clockStr}
        </span>
      ) : null}
      <button
        type="button"
        className={styles.utilityBtn}
        onClick={() => void toggleDisplayMode()}
        aria-label={
          displayMode === 'fullscreen' ? 'Alternar para Modo Desktop' : 'Alternar para Modo Fullscreen'
        }
      >
        {displayMode === 'fullscreen' ? (
          <Minimize2 size={14} aria-hidden="true" />
        ) : (
          <Maximize2 size={14} aria-hidden="true" />
        )}
        <span>{displayMode === 'fullscreen' ? 'Desktop' : 'Fullscreen TV'}</span>
      </button>
      <button
        type="button"
        className={styles.utilityBtn}
        onClick={() => setQuickMenuOpen(true)}
        aria-label="Abrir Menu Rápido"
      >
        <SlidersHorizontal size={14} aria-hidden="true" />
        <span>Menu ({glyphFor('quick_menu')})</span>
      </button>
    </div>
  );

  const renderPreviewBanner = () =>
    previewTheme ? (
      <div className={styles.previewBanner} role="status" aria-live="polite">
        <span>
          <strong>Prévia de tema ativa:</strong> {previewTheme.manifest.name} (layout:{' '}
          {previewTheme.manifest.layout.shell})
        </span>
        <div className={styles.previewActions}>
          <button
            type="button"
            className={styles.previewApplyBtn}
            onClick={() => void applyPreview()}
          >
            Aplicar Tema
          </button>
          <button
            type="button"
            className={styles.previewCancelBtn}
            onClick={() => cancelPreview()}
          >
            Cancelar Prévia
          </button>
        </div>
      </div>
    ) : null;

  const renderContentColumn = () => (
    <div className={styles.contentColumn}>
      {renderPreviewBanner()}
      <main className={styles.main} data-focus-scope="main">
        {children}
      </main>
      <ControllerActionBar />
    </div>
  );

  // 1. Console Classic — Topbar Blades layout
  if (shellLayout === 'topbar_blades') {
    return (
      <div className={styles.shell}>
        <header
          className={styles.bladesHeader}
          aria-label="Navegação principal do Hub"
          data-focus-scope="nav"
        >
          <div className={styles.bladesTopStrip}>
            <div className={styles.brand}>
              <span className={styles.brandTitle}>Universal Console Hub</span>
              <span className={styles.brandSub}>
                {profileDisplayName} (@{profileHandle}) · Transporte: {transportLabel}
              </span>
            </div>
            {renderUtilityControls()}
          </div>
          <nav aria-label="Seções" className={styles.bladesNavRow}>
            {NAV_ITEMS.map(({ id, label, Icon }) => {
              const isActive = activeSection === id;
              return (
                <button
                  key={id}
                  type="button"
                  className={`${styles.bladeBtn} ${isActive ? styles.bladeBtnActive : ''}`}
                  aria-current={isActive ? 'page' : undefined}
                  data-focus-id={`nav-${id}`}
                  onClick={() => onSelectSection(id)}
                >
                  <Icon size={16} aria-hidden="true" />
                  <span>{label}</span>
                </button>
              );
            })}
          </nav>
        </header>
        {renderContentColumn()}
      </div>
    );
  }

  // 2. Minimal — Centered Compact Floating Dock layout
  if (shellLayout === 'minimal_dock') {
    return (
      <div className={styles.shell}>
        <header
          className={styles.dockHeader}
          aria-label="Navegação principal do Hub"
          data-focus-scope="nav"
        >
          <div className={styles.brand}>
            <span className={styles.brandTitle}>Universal Console Hub</span>
            <span className={styles.brandSub}>Transporte: {transportLabel}</span>
          </div>
          <nav aria-label="Seções" className={styles.dockNav}>
            {NAV_ITEMS.map(({ id, label, Icon }) => {
              const isActive = activeSection === id;
              return (
                <button
                  key={id}
                  type="button"
                  className={`${styles.dockBtn} ${isActive ? styles.dockBtnActive : ''}`}
                  aria-current={isActive ? 'page' : undefined}
                  data-focus-id={`nav-${id}`}
                  onClick={() => onSelectSection(id)}
                >
                  <Icon size={14} aria-hidden="true" />
                  <span>{label}</span>
                </button>
              );
            })}
          </nav>
          {renderUtilityControls()}
        </header>
        {renderContentColumn()}
      </div>
    );
  }

  // 3. XMB Inspired — Horizontal Category Crossbar + Vertical Flow
  if (shellLayout === 'xmb_crossbar') {
    return (
      <div className={styles.shell}>
        <header
          className={styles.xmbHeader}
          aria-label="Navegação principal do Hub"
          data-focus-scope="nav"
        >
          <div className={styles.xmbStatusRow}>
            <span>
              <strong>Universal Console Hub</strong> · {profileDisplayName} (@{profileHandle}) ·
              Transporte: {transportLabel}
            </span>
            {renderUtilityControls()}
          </div>
          <nav aria-label="Seções" className={styles.xmbCrossbar}>
            {NAV_ITEMS.map(({ id, label, Icon }) => {
              const isActive = activeSection === id;
              return (
                <button
                  key={id}
                  type="button"
                  className={`${styles.xmbCatBtn} ${isActive ? styles.xmbCatBtnActive : ''}`}
                  aria-current={isActive ? 'page' : undefined}
                  data-focus-id={`nav-${id}`}
                  onClick={() => onSelectSection(id)}
                >
                  <Icon size={22} aria-hidden="true" />
                  <span>{label}</span>
                </button>
              );
            })}
          </nav>
        </header>
        {renderContentColumn()}
      </div>
    );
  }

  // 4. Universal Modern (Default) — Precision Sticky Sidebar (collapses to top HUD in Fullscreen)
  return (
    <div className={`${styles.shell} ${styles.shellSidebar}`}>
      <aside
        className={styles.rail}
        aria-label="Navegação principal do Hub"
        data-focus-scope="nav"
      >
        <div className={styles.brand}>
          <span className={styles.brandTitle}>Universal Console Hub</span>
          <span className={styles.brandSub}>{effectiveTheme.manifest.name} · 3.0</span>
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
                    data-focus-id={`nav-${id}`}
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
          <div>
            <span className={styles.profileName}>{profileDisplayName}</span>
            <div>@{profileHandle} · Local</div>
            <div>Transporte: {transportLabel}</div>
          </div>
          {renderUtilityControls()}
        </div>
      </aside>

      {renderContentColumn()}
    </div>
  );
}
