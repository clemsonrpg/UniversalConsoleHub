import { useState, type FormEvent } from 'react';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { FilePathField } from '../components/FilePathField';
import type {
  BackupFileEntry,
  BackupInspectionReport,
  BackupReport,
  LiveRestoreReport,
} from '../types/domain';
import { formatBytes } from '../utils/formatters';
import styles from './Pages.module.css';

export interface SettingsPageProps {
  settings: Record<string, string>;
  backups?: BackupFileEntry[];
  onUpdateSetting: (key: string, value: string) => Promise<void>;
  onCreateBackup: () => Promise<BackupReport>;
  onInspectBackup?: (backupPath: string) => Promise<BackupInspectionReport>;
  onRestoreBackup?: (backupPath: string) => Promise<LiveRestoreReport>;
}

export function SettingsPage({
  settings,
  backups = [],
  onUpdateSetting,
  onCreateBackup,
  onInspectBackup,
  onRestoreBackup,
}: SettingsPageProps) {
  const [backupReport, setBackupReport] = useState<BackupReport | null>(null);
  const [customBackupPath, setCustomBackupPath] = useState('');
  const [pendingInspection, setPendingInspection] = useState<BackupInspectionReport | null>(null);
  const [restoreReport, setRestoreReport] = useState<LiveRestoreReport | null>(null);
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

  const handleInspectForRestore = async (targetPath: string) => {
    if (!onInspectBackup) return;
    setFeedback(null);
    setBusy(true);
    try {
      const report = await onInspectBackup(targetPath);
      setPendingInspection(report);
    } catch (err) {
      setFeedback({ kind: 'err', text: `Backup rejeitado na validação: ${String(err)}` });
    } finally {
      setBusy(false);
    }
  };

  const handleInspectCustom = async (event: FormEvent) => {
    event.preventDefault();
    if (!customBackupPath.trim()) return;
    await handleInspectForRestore(customBackupPath.trim());
  };

  const handleConfirmRestore = async () => {
    if (!pendingInspection || !onRestoreBackup) return;
    setBusy(true);
    setFeedback(null);
    try {
      const report = await onRestoreBackup(pendingInspection.backupPath);
      setRestoreReport(report);
      setPendingInspection(null);
      setFeedback({
        kind: 'ok',
        text: `Banco SQLite restaurado com sucesso (schema v${report.schemaVersion}). Snapshot de segurança salvo antes da substituição.`,
      });
    } catch (err) {
      setPendingInspection(null);
      setFeedback({
        kind: 'err',
        text: `Falha ao restaurar backup (banco ativo preservado via rollback): ${String(err)}`,
      });
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
            Preferências persistidas localmente no SQLite e gerenciamento seguro de backups e
            restauração ao vivo em modo WAL.
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
              Tela cheia padrão ao iniciar consoles (Fases 3+)
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
        <h2 className={styles.sectionTitle}>Integridade, Backup e Restauração do SQLite</h2>
        <p className={styles.muted}>
          Gera snapshots consistentes via <code>VACUUM INTO</code> e valida integralmente com{' '}
          <code>PRAGMA quick_check</code> e <code>PRAGMA foreign_key_check</code> antes de
          qualquer restauração. Um snapshot pré-restauração automático protege seus dados atuais.
        </p>
        <div className={styles.actionsRow}>
          <button
            type="button"
            className={styles.btnPrimary}
            disabled={busy}
            onClick={() => void handleBackup()}
          >
            {busy ? 'Processando...' : 'Criar Backup SQLite'}
          </button>
        </div>

        {backupReport ? (
          <div className={styles.banner}>
            <div>
              <strong>Arquivo:</strong>{' '}
              <span className={styles.mono}>{backupReport.backupPath}</span>
            </div>
            <div>
              <strong>Integridade:</strong>{' '}
              {backupReport.integrityOk ? 'OK (quick_check + FK)' : 'Falhou'} ·{' '}
              <strong>UTC:</strong> <span className={styles.mono}>{backupReport.createdAt}</span>
            </div>
          </div>
        ) : null}

        {restoreReport ? (
          <div className={`${styles.banner} ${styles.bannerSuccess}`}>
            <div>
              <strong>Restaurado de:</strong>{' '}
              <span className={styles.mono}>{restoreReport.sourceBackupPath}</span>
            </div>
            <div>
              <strong>Snapshot pré-restauração salvo em:</strong>{' '}
              <span className={styles.mono}>{restoreReport.preRestoreSnapshotPath}</span>
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

        {onInspectBackup && onRestoreBackup ? (
          <>
            <h3 className={styles.sectionTitle}>Backups Disponíveis ({backups.length})</h3>
            {backups.length === 0 ? (
              <p className={styles.muted}>
                Nenhum backup encontrado na pasta padrão ainda. Crie um backup acima ou selecione
                um arquivo .db abaixo.
              </p>
            ) : (
              <ul className={styles.list}>
                {backups.map((item) => (
                  <li key={item.backupPath} className={styles.listItem}>
                    <div>
                      <div className={styles.mono}>{item.fileName}</div>
                      <div className={styles.muted}>
                        {formatBytes(item.sizeBytes)} · Modificado em {item.modifiedAt}
                      </div>
                    </div>
                    <button
                      type="button"
                      className={styles.btnSecondary}
                      disabled={busy}
                      onClick={() => void handleInspectForRestore(item.backupPath)}
                    >
                      Inspecionar e Restaurar
                    </button>
                  </li>
                ))}
              </ul>
            )}

            <form onSubmit={handleInspectCustom} className={styles.formGrid}>
              <div className={styles.field}>
                <label className={styles.label} htmlFor="restore-custom-backup">
                  Selecionar outro arquivo de backup SQLite (.db)
                </label>
                <FilePathField
                  id="restore-custom-backup"
                  mode="file"
                  dialogTitle="Selecionar backup SQLite (.db)"
                  filters={[{ name: 'Banco SQLite', extensions: ['db'] }]}
                  value={customBackupPath}
                  onChange={setCustomBackupPath}
                  placeholder="C:\Backups\uch_backup.db"
                  disabled={busy}
                />
              </div>
              <div className={styles.actionsRow} style={{ alignSelf: 'end' }}>
                <button
                  type="submit"
                  className={styles.btnSecondary}
                  disabled={busy || customBackupPath.trim().length === 0}
                >
                  Inspecionar Arquivo
                </button>
              </div>
            </form>
          </>
        ) : null}
      </section>

      <ConfirmDialog
        open={pendingInspection !== null}
        title="Confirmar restauração do banco SQLite"
        confirmLabel="Confirmar Restauração"
        cancelLabel="Cancelar"
        danger
        busy={busy}
        onConfirm={() => void handleConfirmRestore()}
        onCancel={() => setPendingInspection(null)}
      >
        {pendingInspection ? (
          <>
            <p>
              <strong>Arquivo validado:</strong>{' '}
              <span className={styles.mono}>{pendingInspection.backupPath}</span>
            </p>
            <p>
              <strong>Integridade:</strong>{' '}
              {pendingInspection.integrityOk
                ? 'PRAGMA quick_check = OK · Foreign Keys = OK'
                : 'Falhou'}{' '}
              · <strong>Schema:</strong> v{pendingInspection.schemaVersion} ·{' '}
              <strong>Tamanho:</strong> {formatBytes(pendingInspection.sizeBytes)}
            </p>
            <p>
              <strong>Conteúdo verificado:</strong> {pendingInspection.profilesCount} perfil(is) e{' '}
              {pendingInspection.gamesCount} jogo(s).
            </p>
            <p>
              Ao confirmar, um <strong>snapshot de segurança pré-restauração</strong> dos dados
              atuais será criado automaticamente antes de substituir o banco ativo. Em caso de
              falha, o banco atual é restaurado via rollback automático.
            </p>
          </>
        ) : null}
      </ConfirmDialog>
    </div>
  );
}
