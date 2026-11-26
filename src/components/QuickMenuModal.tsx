import { useEffect, useRef } from 'react';
import type { HubSectionId } from '../hooks/useHubNavigation';
import { globalFocusManager } from '../input/focusManager';
import { useInput } from '../input/InputProvider';
import { useTheme } from '../themes/ThemeProvider';
import type { ActiveSessionSnapshot } from '../types/domain';
import styles from './QuickMenuModal.module.css';

export interface QuickMenuModalProps {
  open: boolean;
  activeSection: HubSectionId;
  activeSession: ActiveSessionSnapshot;
  onSelectSection: (section: HubSectionId) => void;
  onStopSession: () => Promise<void>;
  onClose: () => void;
}

const QUICK_SECTIONS: Array<{ id: HubSectionId; label: string }> = [
  { id: 'home', label: 'Início' },
  { id: 'consoles', label: 'Consoles' },
  { id: 'library', label: 'Biblioteca' },
  { id: 'trophies', label: 'Central de Troféus' },
  { id: 'profile', label: 'Perfil Universal' },
  { id: 'statistics', label: 'Estatísticas' },
  { id: 'emulators', label: 'Emuladores' },
  { id: 'themes', label: 'Temas' },
  { id: 'settings', label: 'Configurações' },
];

export function QuickMenuModal({
  open,
  activeSection,
  activeSession,
  onSelectSection,
  onStopSession,
  onClose,
}: QuickMenuModalProps) {
  const panelRef = useRef<HTMLDivElement | null>(null);
  const { displayMode, toggleDisplayMode, glyphFor } = useInput();
  const { themes, activeTheme, selectTheme } = useTheme();

  useEffect(() => {
    if (!open) return;
    globalFocusManager.pushModalFocus(panelRef.current);
    return () => {
      globalFocusManager.popModalFocus();
    };
  }, [open]);

  if (!open) return null;

  return (
    <div className={styles.backdrop} onClick={onClose}>
      <div
        ref={panelRef}
        role="dialog"
        aria-modal="true"
        aria-label="Menu Rápido do Hub"
        data-focus-scope="modal"
        className={styles.panel}
        onClick={(e) => e.stopPropagation()}
      >
        <div className={styles.header}>
          <h2 className={styles.title}>Menu Rápido</h2>
          <button
            type="button"
            className={styles.actionBtn}
            data-dialog-cancel="true"
            onClick={onClose}
          >
            Fechar ({glyphFor('back')})
          </button>
        </div>

        <div>
          <p className={styles.sectionLabel}>Modo de Experiência</p>
          <div className={styles.grid}>
            <button
              type="button"
              className={styles.actionBtn}
              data-focus-default="true"
              onClick={() => {
                void toggleDisplayMode();
                onClose();
              }}
            >
              <span>
                {displayMode === 'fullscreen' ? 'Mudar para Modo Desktop' : 'Mudar para Modo Fullscreen (TV)'}
              </span>
              <span>{glyphFor('toggle_mode')}</span>
            </button>
            {activeSession.phase !== 'idle' ? (
              <button
                type="button"
                className={`${styles.actionBtn} ${styles.actionBtnDanger}`}
                onClick={() => {
                  void onStopSession();
                  onClose();
                }}
              >
                <span>Encerrar Sessão Ativa</span>
              </button>
            ) : null}
          </div>
        </div>

        <div>
          <p className={styles.sectionLabel}>Ir para Seção</p>
          <div className={styles.grid}>
            {QUICK_SECTIONS.map((item) => (
              <button
                key={item.id}
                type="button"
                className={`${styles.actionBtn} ${
                  activeSection === item.id ? styles.actionBtnActive : ''
                }`}
                onClick={() => {
                  onSelectSection(item.id);
                  onClose();
                }}
              >
                <span>{item.label}</span>
              </button>
            ))}
          </div>
        </div>

        <div>
          <p className={styles.sectionLabel}>Troca Rápida de Tema</p>
          <div className={styles.grid}>
            {themes.map((t) => (
              <button
                key={t.manifest.id}
                type="button"
                className={`${styles.actionBtn} ${
                  activeTheme.manifest.id === t.manifest.id ? styles.actionBtnActive : ''
                }`}
                onClick={() => {
                  void selectTheme(t.manifest.id);
                }}
              >
                <span>{t.manifest.name}</span>
              </button>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
