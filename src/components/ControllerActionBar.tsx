import { Gamepad2 } from 'lucide-react';
import { useInput } from '../input/InputProvider';
import styles from './ControllerActionBar.module.css';

export function ControllerActionBar() {
  const { connectedGamepads, displayMode, glyphFor, activeGlyphFamily } = useInput();
  const primaryPad = connectedGamepads[0] ?? null;

  return (
    <footer className={styles.bar} aria-label="Atalhos de controle e teclado">
      <div className={styles.hints}>
        <span className={styles.hintItem}>
          <span className={styles.glyph}>{glyphFor('confirm')}</span>
          <span>Confirmar</span>
        </span>
        <span className={styles.hintItem}>
          <span className={styles.glyph}>{glyphFor('back')}</span>
          <span>Voltar</span>
        </span>
        <span className={styles.hintItem}>
          <span className={styles.glyph}>
            {glyphFor('prev_tab')}/{glyphFor('next_tab')}
          </span>
          <span>Abas</span>
        </span>
        <span className={styles.hintItem}>
          <span className={styles.glyph}>{glyphFor('quick_menu')}</span>
          <span>Menu Rápido</span>
        </span>
        <span className={styles.hintItem}>
          <span className={styles.glyph}>{glyphFor('toggle_mode')}</span>
          <span>{displayMode === 'fullscreen' ? 'Modo Desktop' : 'Modo TV / Fullscreen'}</span>
        </span>
      </div>

      <div className={styles.status} aria-live="polite">
        <Gamepad2 size={14} aria-hidden="true" />
        <span
          className={primaryPad ? styles.dotConnected : styles.dotIdle}
          aria-hidden="true"
        />
        <span>
          {primaryPad
            ? `${primaryPad.name} (${activeGlyphFamily.toUpperCase()})`
            : 'Teclado / Controle pronto'}
        </span>
      </div>
    </footer>
  );
}
