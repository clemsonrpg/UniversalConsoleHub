import { useState } from 'react';
import type { BackupReport } from '../types/domain';
import styles from './Pages.module.css';

export interface SettingsPageProps {
  settings: Record<string, string>;
  onUpdateSetting: (key: string, value: string) => Promise<void>;
  onCreateBackup: () => Promise<BackupReport>;
}

export function SettingsPage({
  settings,
  onUpdateSetting,
  onCreateBackup,
}: SettingsPageProps) {
  const [backupReport, setBackupReport] = useState<BackupReport | null>(null);
  const [feedback, setFeedback] = useState<{ kind: 'ok' | 'err'; text: string } | null>(null);
  const [busy, setBusy] = useState(false);

  const handleChange = async (key: string, value: string) => {
    setFeedback(null);
    try {
      await onUpdateSetting(key, value);
      setFeedback({ kind: 'ok', text: `Configuração '${key}' salva no SQLite.` });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    }
  };

  const handleBackup = async () => {
    setFeedback(null);
    setBusy(true);
    try {
      const report = await onCreateBackup();
      setBackupReport(report);
      setFeedback({
        kind: 'ok',
        text: `Backup íntegro gerado via VACUUM INTO (${report.sizeBytes} bytes).`,
      });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <h1 className={styles.title}>Configurações do Sistema</h1>
          <p className={styles.subtitle}>
            Preferências persistidas localmente no SQLite e gerenciamento seguro de backups em modo
            WAL.
          </p>
        </div>
      </header>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Exibição e Execução</h2>
        <div className={styles.formGrid}>
          <div className={styles.field}>
            <label className={styles.label} htmlFor="setting-resolution">
              Resolução alvo
            </label>
            <select
              id="setting-resolution"
              className={styles.select}
              value={settings['ui.target_resolution'] ?? '1920x1080'}
              onChange={(e) => void handleChange('ui.target_resolution', e.target.value)}
            >
              <option value="1920x1080">1920 × 1080 (Full HD)</option>
              <option value="2560x1440">2560 × 1440 (QHD)</option>
              <option value="3840x2160">3840 × 2160 (4K UHD)</option>
            </select>
          </div>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="setting-fullscreen">
              Tela cheia padrão ao iniciar consoles (Fases 4+)
            </label>
            <select
              id="setting-fullscreen"
              className={styles.select}
              value={settings['ui.fullscreen_default'] ?? 'false'}
              onChange={(e) => void handleChange('ui.fullscreen_default', e.target.value)}
            >
              <option value="false">Desativado</option>
              <option value="true">Ativado</option>
            </select>
          </div>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="setting-timeout">
              Timeout de inicialização (segundos)
            </label>
            <select
              id="setting-timeout"
              className={styles.select}
              value={settings['emulation.launch_timeout_seconds'] ?? '45'}
              onChange={(e) =>
                void handleChange('emulation.launch_timeout_seconds', e.target.value)
              }
            >
              <option value="30">30 segundos</option>
              <option value="45">45 segundos</option>
              <option value="90">90 segundos</option>
            </select>
          </div>
        </div>
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Integridade e Backup do Banco SQLite</h2>
        <p className={styles.muted}>
          Gera um snapshot consistente via <code>VACUUM INTO</code> (seguro com banco WAL aberto) e
          valida com <code>PRAGMA quick_check</code> e <code>PRAGMA foreign_key_check</code>.
        </p>
        <div className={styles.actionsRow}>
          <button
            type="button"
            className={styles.btnPrimary}
            disabled={busy}
            onClick={() => void handleBackup()}
          >
            {busy ? 'Gerando Backup...' : 'Criar Backup SQLite'}
          </button>
        </div>

        {backupReport ? (
          <div className={styles.banner}>
            <div>
              <strong>Arquivo:</strong> <span className={styles.mono}>{backupReport.backupPath}</span>
            </div>
            <div>
              <strong>Integridade:</strong> {backupReport.integrityOk ? 'OK (quick_check + FK)' : 'Falhou'} ·{' '}
              <strong>UTC:</strong> <span className={styles.mono}>{backupReport.createdAt}</span>
            </div>
          </div>
        ) : null}

        {feedback ? (
          <div
            role="status"
            aria-live="polite"
            className={`${styles.banner} ${
              feedback.kind === 'ok' ? styles.bannerSuccess : styles.bannerError
            }`}
          >
            {feedback.text}
          </div>
        ) : null}
      </section>
    </div>
  );
}
