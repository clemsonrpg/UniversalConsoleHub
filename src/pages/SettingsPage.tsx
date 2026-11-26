import { useState, type FormEvent } from 'react';
import {
  Compass,
  Cpu,
  Database,
  Gamepad2,
  Library,
  Monitor,
  Palette,
  RotateCcw,
  Shield,
} from 'lucide-react';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { FilePathField } from '../components/FilePathField';
import { useOptionalInput } from '../input/InputProvider';
import {
  ACTION_LABELS,
  DEFAULT_GAMEPAD_CONFIG,
  parseGamepadConfig,
  type StandardButtonSlot,
} from '../input/types';
import { useOptionalTheme } from '../themes/ThemeProvider';
import type {
  BackupFileEntry,
  BackupInspectionReport,
  BackupReport,
  GamepadGlyphFamily,
  GamepadNavAction,
  LiveRestoreReport,
} from '../types/domain';
import { formatBytes } from '../utils/formatters';
import styles from './Pages.module.css';
import settingsStyles from './SettingsPage.module.css';

type SettingsTabId =
  | 'all'
  | 'appearance'
  | 'themes'
  | 'controllers'
  | 'navigation'
  | 'library'
  | 'integrations'
  | 'system'
  | 'data';

const SETTINGS_TABS: Array<{ id: SettingsTabId; label: string }> = [
  { id: 'all', label: 'Todas' },
  { id: 'appearance', label: 'Aparência' },
  { id: 'themes', label: 'Temas' },
  { id: 'controllers', label: 'Controles' },
  { id: 'navigation', label: 'Navegação' },
  { id: 'library', label: 'Biblioteca' },
  { id: 'integrations', label: 'Integrações' },
  { id: 'system', label: 'Sistema' },
  { id: 'data', label: 'Dados' },
];

const BUTTON_SLOT_LABELS: Array<{ slot: StandardButtonSlot; label: string; physicalHint: string }> =
  [
    { slot: 'south', label: 'Botão Sul', physicalHint: 'A / ✕' },
    { slot: 'east', label: 'Botão Leste', physicalHint: 'B / ○' },
    { slot: 'west', label: 'Botão Oeste', physicalHint: 'X / □' },
    { slot: 'north', label: 'Botão Norte', physicalHint: 'Y / △' },
    { slot: 'lb', label: 'Ombro Esquerdo', physicalHint: 'LB / L1' },
    { slot: 'rb', label: 'Ombro Direito', physicalHint: 'RB / R1' },
    { slot: 'lt', label: 'Gatilho Esquerdo', physicalHint: 'LT / L2' },
    { slot: 'rt', label: 'Gatilho Direito', physicalHint: 'RT / R2' },
    { slot: 'back', label: 'View / Select', physicalHint: 'Back / Create' },
    { slot: 'start', label: 'Start / Menu', physicalHint: 'Start / Options' },
  ];

const ASSIGNABLE_ACTIONS: GamepadNavAction[] = [
  'confirm',
  'back',
  'secondary',
  'context_menu',
  'prev_tab',
  'next_tab',
  'page_prev',
  'page_next',
  'quick_menu',
  'toggle_mode',
];

export interface SettingsPageProps {
  settings: Record<string, string>;
  backups?: BackupFileEntry[];
  appVersion?: string;
  transportKind?: 'tauri' | 'injected_test';
  gamesCount?: number;
  consolesCount?: number;
  rpcs3Ready?: boolean;
  onUpdateSetting: (key: string, value: string) => Promise<void>;
  onCreateBackup: () => Promise<BackupReport>;
  onInspectBackup?: (backupPath: string) => Promise<BackupInspectionReport>;
  onRestoreBackup?: (backupPath: string) => Promise<LiveRestoreReport>;
  onOpenThemes?: () => void;
  onOpenLibrary?: () => void;
  onOpenEmulators?: () => void;
  onOpenTrophies?: () => void;
}

export function SettingsPage({
  settings,
  backups = [],
  appVersion = '0.1.0',
  transportKind = 'tauri',
  gamesCount = 0,
  consolesCount = 6,
  rpcs3Ready = false,
  onUpdateSetting,
  onCreateBackup,
  onInspectBackup,
  onRestoreBackup,
  onOpenThemes,
  onOpenLibrary,
  onOpenEmulators,
  onOpenTrophies,
}: SettingsPageProps) {
  const themeCtx = useOptionalTheme();
  const inputCtx = useOptionalInput();

  const [activeTab, setActiveTab] = useState<SettingsTabId>('all');
  const [backupReport, setBackupReport] = useState<BackupReport | null>(null);
  const [customBackupPath, setCustomBackupPath] = useState('');
  const [pendingInspection, setPendingInspection] = useState<BackupInspectionReport | null>(null);
  const [restoreReport, setRestoreReport] = useState<LiveRestoreReport | null>(null);
  const [feedback, setFeedback] = useState<{ kind: 'ok' | 'err'; text: string } | null>(null);
  const [busy, setBusy] = useState(false);

  const showSection = (id: Exclude<SettingsTabId, 'all'>) =>
    activeTab === 'all' || activeTab === id;

  const gamepadConfig =
    inputCtx?.gamepadConfig ?? parseGamepadConfig(settings['gamepad.config_json']);
  const connectedGamepads = inputCtx?.connectedGamepads ?? [];
  const lastAction = inputCtx?.lastAction ?? null;
  const displayMode =
    inputCtx?.displayMode ??
    (settings['ui.display_mode'] === 'fullscreen' ? 'fullscreen' : 'desktop');

  const handleChange = async (key: string, value: string) => {
    setFeedback(null);
    try {
      await onUpdateSetting(key, value);
      setFeedback({ kind: 'ok', text: `Configuração '${key}' salva no SQLite.` });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    }
  };

  const handleDisplayModeChange = async (mode: 'desktop' | 'fullscreen') => {
    setFeedback(null);
    try {
      if (inputCtx) {
        await inputCtx.setDisplayMode(mode);
      } else {
        await onUpdateSetting('ui.display_mode', mode);
      }
      setFeedback({
        kind: 'ok',
        text:
          mode === 'fullscreen'
            ? 'Modo Fullscreen (TV 10-foot) ativado e salvo.'
            : 'Modo Desktop ativado e salvo.',
      });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    }
  };

  const handleGamepadPatch = async (patch: Parameters<NonNullable<typeof inputCtx>['updateGamepadConfig']>[0]) => {
    setFeedback(null);
    try {
      if (inputCtx) {
        await inputCtx.updateGamepadConfig(patch);
      } else {
        const merged = {
          ...gamepadConfig,
          ...patch,
          buttonMap: { ...gamepadConfig.buttonMap, ...(patch.buttonMap ?? {}) },
        };
        await onUpdateSetting('gamepad.config_json', JSON.stringify(merged));
      }
      setFeedback({ kind: 'ok', text: 'Configuração de controle salva no SQLite.' });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    }
  };

  const handleResetGamepad = async () => {
    setFeedback(null);
    try {
      if (inputCtx) {
        await inputCtx.resetGamepadConfig();
      } else {
        await onUpdateSetting('gamepad.config_json', JSON.stringify(DEFAULT_GAMEPAD_CONFIG));
      }
      setFeedback({ kind: 'ok', text: 'Mapeamento e calibragem de controle restaurados.' });
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
            Central organizada por categorias — aparência, temas, controles configuráveis, navegação
            Desktop/TV, biblioteca, integrações e backups SQLite em modo WAL.
          </p>
        </div>
      </header>

      <div
        className={settingsStyles.tabsBar}
        role="tablist"
        aria-label="Categorias de configurações"
      >
        {SETTINGS_TABS.map((tab, idx) => (
          <button
            key={tab.id}
            type="button"
            role="tab"
            aria-selected={activeTab === tab.id}
            data-focus-default={idx === 0 ? 'true' : undefined}
            className={`${settingsStyles.tabBtn} ${
              activeTab === tab.id ? settingsStyles.tabBtnActive : ''
            }`}
            onClick={() => setActiveTab(tab.id)}
          >
            {tab.label}
          </button>
        ))}
      </div>

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

      {showSection('appearance') ? (
        <section className={styles.section} aria-label="Configurações de aparência">
          <h2 className={styles.sectionTitle}>
            <Monitor
              size={16}
              aria-hidden="true"
              style={{ marginRight: 6, verticalAlign: 'middle' }}
            />
            Aparência &amp; Modo de Exibição
          </h2>
          <div className={styles.formGrid}>
            <div className={styles.field}>
              <label className={styles.label} htmlFor="setting-display-mode">
                Modo de exibição do Hub (F11 / View)
              </label>
              <select
                id="setting-display-mode"
                className={styles.select}
                value={displayMode}
                onChange={(e) =>
                  void handleDisplayModeChange(
                    e.target.value === 'fullscreen' ? 'fullscreen' : 'desktop'
                  )
                }
              >
                <option value="desktop">Desktop (PC — mouse, teclado e controle)</option>
                <option value="fullscreen">Fullscreen TV (10-foot — otimizado para controle)</option>
              </select>
            </div>

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
                Tela cheia padrão ao iniciar consoles
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
              <label className={styles.label} htmlFor="setting-reduce-motion">
                Reduzir animações e ondas de fundo
              </label>
              <select
                id="setting-reduce-motion"
                className={styles.select}
                value={settings['ui.reduce_motion'] ?? 'false'}
                onChange={(e) => void handleChange('ui.reduce_motion', e.target.value)}
              >
                <option value="false">Desativado (animações suaves)</option>
                <option value="true">Ativado (acessibilidade / desempenho)</option>
              </select>
            </div>
          </div>
        </section>
      ) : null}

      {showSection('themes') ? (
        <section className={styles.section} aria-label="Configurações de temas">
          <h2 className={styles.sectionTitle}>
            <Palette
              size={16}
              aria-hidden="true"
              style={{ marginRight: 6, verticalAlign: 'middle' }}
            />
            Temas
          </h2>
          <div className={styles.formGrid}>
            <div className={styles.field}>
              <label className={styles.label} htmlFor="setting-active-theme">
                Tema visual ativo
              </label>
              <select
                id="setting-active-theme"
                className={styles.select}
                value={themeCtx?.activeTheme.manifest.id ?? settings['theme.active_id'] ?? 'universal-modern'}
                onChange={(e) => {
                  const id = e.target.value;
                  if (themeCtx) {
                    void themeCtx.selectTheme(id);
                  } else {
                    void handleChange('theme.active_id', id);
                  }
                }}
              >
                {(themeCtx?.themes ?? []).length > 0 ? (
                  themeCtx!.themes
                    .filter((t) => !themeCtx!.disabledThemeIds.includes(t.manifest.id))
                    .map((t) => (
                      <option key={t.manifest.id} value={t.manifest.id}>
                        {t.manifest.name} ({t.manifest.layout.shell})
                      </option>
                    ))
                ) : (
                  <>
                    <option value="universal-modern">Universal Modern (sidebar)</option>
                    <option value="console-classic">Console Classic (topbar_blades)</option>
                    <option value="minimal">Minimal (minimal_dock)</option>
                    <option value="xmb-inspired">XMB Inspired (xmb_crossbar)</option>
                  </>
                )}
              </select>
            </div>
            <div className={styles.actionsRow} style={{ alignSelf: 'end' }}>
              {onOpenThemes ? (
                <button type="button" className={styles.btnPrimary} onClick={onOpenThemes}>
                  Abrir Gerenciador de Temas
                </button>
              ) : null}
              {themeCtx ? (
                <button
                  type="button"
                  className={styles.btnSecondary}
                  onClick={() => void themeCtx.resetToDefaultTheme()}
                >
                  Restaurar Padrão
                </button>
              ) : null}
            </div>
          </div>
        </section>
      ) : null}

      {showSection('controllers') ? (
        <section className={styles.section} aria-label="Configurações de controles">
          <div className={styles.header}>
            <div>
              <h2 className={styles.sectionTitle}>
                <Gamepad2
                  size={16}
                  aria-hidden="true"
                  style={{ marginRight: 6, verticalAlign: 'middle' }}
                />
                Controles &amp; Mapeamento Configurável
              </h2>
              <p className={styles.muted}>
                Detecção não-exclusiva via XInput nativo no Windows + fallback W3C Gamepad API
                (DualSense, DualShock, Switch Pro e USB padrão) com hot-plug e pausa automática ao
                iniciar emuladores.
              </p>
            </div>
            <button
              type="button"
              className={styles.btnSecondary}
              onClick={() => void handleResetGamepad()}
            >
              <RotateCcw
                size={14}
                aria-hidden="true"
                style={{ marginRight: 6, verticalAlign: 'middle' }}
              />
              Restaurar Mapeamento Padrão
            </button>
          </div>

          {connectedGamepads.length === 0 ? (
            <div className={`${styles.banner} ${styles.bannerInfo}`}>
              Nenhum controle físico conectado no momento. Conecte um controle Xbox (XInput),
              PlayStation ou USB padrão — a detecção e reconexão ocorrem automaticamente em tempo
              real.
            </div>
          ) : (
            <div className={settingsStyles.deviceGrid}>
              {connectedGamepads.map((pad) => (
                <div key={`${pad.backend}-${pad.slot}`} className={settingsStyles.deviceCard}>
                  <div className={settingsStyles.deviceHeader}>
                    <span className={settingsStyles.deviceName}>{pad.name}</span>
                    <span className={settingsStyles.statusOnline}>
                      <span className={settingsStyles.statusDot} aria-hidden="true" />
                      Conectado
                    </span>
                  </div>
                  <div className={styles.muted}>
                    Slot #{pad.slot + 1} · Backend: <strong>{pad.backend}</strong> · Família de
                    botões: <strong>{pad.glyphFamily.toUpperCase()}</strong>
                  </div>
                </div>
              ))}
            </div>
          )}

          <div className={settingsStyles.telemetryBox} aria-live="polite">
            <span>
              <strong>Monitor de entrada em tempo real:</strong>{' '}
              {lastAction
                ? `${ACTION_LABELS[lastAction.action]} (${lastAction.action})`
                : 'Aguardando entrada de controle ou teclado...'}
            </span>
            <span className={styles.mono}>
              Glyphs ativos: {(inputCtx?.activeGlyphFamily ?? 'keyboard').toUpperCase()}
            </span>
          </div>

          <div className={styles.formGrid}>
            <div className={styles.field}>
              <label className={styles.label} htmlFor="setting-glyph-override">
                Família de botões exibida (Glyphs)
              </label>
              <select
                id="setting-glyph-override"
                className={styles.select}
                value={gamepadConfig.glyphOverride}
                onChange={(e) =>
                  void handleGamepadPatch({
                    glyphOverride: e.target.value as 'auto' | GamepadGlyphFamily,
                  })
                }
              >
                <option value="auto">Automático (conforme controle conectado)</option>
                <option value="xbox">Xbox (A / B / X / Y · LB / RB)</option>
                <option value="playstation">PlayStation (✕ / ○ / □ / △ · L1 / R1)</option>
                <option value="nintendo">Nintendo (B / A / Y / X · L / R)</option>
                <option value="generic">Genérico USB (1 / 2 / 3 / 4)</option>
              </select>
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="setting-deadzone">
                Zona morta do analógico esquerdo ({Math.round(gamepadConfig.deadzone * 100)}%)
              </label>
              <input
                id="setting-deadzone"
                type="range"
                min={0.1}
                max={0.5}
                step={0.02}
                value={gamepadConfig.deadzone}
                onChange={(e) =>
                  void handleGamepadPatch({ deadzone: Number.parseFloat(e.target.value) })
                }
              />
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="setting-repeat-delay">
                Atraso inicial de repetição (anti-duplo clique)
              </label>
              <select
                id="setting-repeat-delay"
                className={styles.select}
                value={String(gamepadConfig.repeatDelayMs)}
                onChange={(e) =>
                  void handleGamepadPatch({ repeatDelayMs: Number.parseInt(e.target.value, 10) })
                }
              >
                <option value="240">240 ms (Rápido)</option>
                <option value="320">320 ms (Responsivo)</option>
                <option value="380">380 ms (Padrão equilibrado)</option>
                <option value="480">480 ms (Conservador)</option>
              </select>
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="setting-repeat-interval">
                Intervalo de repetição ao segurar direção
              </label>
              <select
                id="setting-repeat-interval"
                className={styles.select}
                value={String(gamepadConfig.repeatIntervalMs)}
                onChange={(e) =>
                  void handleGamepadPatch({
                    repeatIntervalMs: Number.parseInt(e.target.value, 10),
                  })
                }
              >
                <option value="80">80 ms (Muito rápido)</option>
                <option value="115">115 ms (Padrão console)</option>
                <option value="150">150 ms (Moderado)</option>
                <option value="200">200 ms (Preciso)</option>
              </select>
            </div>
          </div>

          <h3 className={styles.sectionTitle}>Mapeamento de Botões</h3>
          <div className={settingsStyles.mappingGrid}>
            {BUTTON_SLOT_LABELS.map((item) => {
              const selectId = `gamepad-map-${item.slot}`;
              return (
                <div key={item.slot} className={settingsStyles.mappingRow}>
                  <label className={settingsStyles.mappingLabel} htmlFor={selectId}>
                    <span>{item.label}</span>
                    <span className={settingsStyles.mappingBadge}>{item.physicalHint}</span>
                  </label>
                  <select
                    id={selectId}
                    className={styles.select}
                    value={gamepadConfig.buttonMap[item.slot]}
                    onChange={(e) =>
                      void handleGamepadPatch({
                        buttonMap: {
                          ...gamepadConfig.buttonMap,
                          [item.slot]: e.target.value as GamepadNavAction,
                        },
                      })
                    }
                  >
                    {ASSIGNABLE_ACTIONS.map((act) => (
                      <option key={act} value={act}>
                        {ACTION_LABELS[act]}
                      </option>
                    ))}
                  </select>
                </div>
              );
            })}
          </div>
        </section>
      ) : null}

      {showSection('navigation') ? (
        <section className={styles.section} aria-label="Configurações de navegação">
          <h2 className={styles.sectionTitle}>
            <Compass
              size={16}
              aria-hidden="true"
              style={{ marginRight: 6, verticalAlign: 'middle' }}
            />
            Navegação &amp; Atalhos
          </h2>
          <div className={styles.formGrid}>
            <div className={styles.field}>
              <label className={styles.label} htmlFor="setting-browser-gamepad">
                Fallback Web Gamepad API (DualSense / Switch / DirectInput)
              </label>
              <select
                id="setting-browser-gamepad"
                className={styles.select}
                value={gamepadConfig.enableBrowserFallback ? 'true' : 'false'}
                onChange={(e) =>
                  void handleGamepadPatch({ enableBrowserFallback: e.target.value === 'true' })
                }
              >
                <option value="true">Ativado (Recomendado)</option>
                <option value="false">Desativado (somente XInput nativo)</option>
              </select>
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="setting-library-view">
                Visualização padrão da Biblioteca
              </label>
              <select
                id="setting-library-view"
                className={styles.select}
                value={settings['ui.library_default_view'] ?? 'auto'}
                onChange={(e) => void handleChange('ui.library_default_view', e.target.value)}
              >
                <option value="auto">Seguir definição do tema ativo</option>
                <option value="grid">Grade de Capas</option>
                <option value="list">Lista Compacta</option>
              </select>
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="setting-action-bar">
                Exibir barra inferior de indicações do controle
              </label>
              <select
                id="setting-action-bar"
                className={styles.select}
                value={settings['ui.show_action_bar'] ?? 'true'}
                onChange={(e) => void handleChange('ui.show_action_bar', e.target.value)}
              >
                <option value="true">Visível</option>
                <option value="false">Oculta</option>
              </select>
            </div>
          </div>

          <div className={settingsStyles.shortcutGrid}>
            <div className={settingsStyles.shortcutItem}>
              <span>Navegação espacial 2D</span>
              <span className={settingsStyles.kbd}>Setas / D-Pad / Stick</span>
            </div>
            <div className={settingsStyles.shortcutItem}>
              <span>Confirmar / Voltar</span>
              <span className={settingsStyles.kbd}>Enter / Esc</span>
            </div>
            <div className={settingsStyles.shortcutItem}>
              <span>Alternar seções / abas</span>
              <span className={settingsStyles.kbd}>Alt+Setas / LB·RB</span>
            </div>
            <div className={settingsStyles.shortcutItem}>
              <span>Menu Rápido</span>
              <span className={settingsStyles.kbd}>F2 / Start</span>
            </div>
            <div className={settingsStyles.shortcutItem}>
              <span>Desktop ↔ Fullscreen TV</span>
              <span className={settingsStyles.kbd}>F11 / View</span>
            </div>
            <div className={settingsStyles.shortcutItem}>
              <span>Salto rápido em listas</span>
              <span className={settingsStyles.kbd}>PgUp·PgDn / LT·RT</span>
            </div>
          </div>
        </section>
      ) : null}

      {showSection('library') ? (
        <section className={styles.section} aria-label="Configurações de biblioteca">
          <h2 className={styles.sectionTitle}>
            <Library
              size={16}
              aria-hidden="true"
              style={{ marginRight: 6, verticalAlign: 'middle' }}
            />
            Biblioteca &amp; Descoberta
          </h2>
          <p className={styles.muted}>
            O Universal Console Hub cataloga jogos automaticamente a partir das pastas reais dos
            emuladores e consoles ({gamesCount} jogo(s) em {consolesCount} console(s) no catálogo).
          </p>
          <div className={styles.formGrid}>
            <div className={styles.field}>
              <label className={styles.label} htmlFor="setting-auto-sync">
                Sincronizar perfis automaticamente ao iniciar
              </label>
              <select
                id="setting-auto-sync"
                className={styles.select}
                value={settings['library.auto_sync_startup'] ?? 'false'}
                onChange={(e) => void handleChange('library.auto_sync_startup', e.target.value)}
              >
                <option value="false">Manual (sob demanda)</option>
                <option value="true">Automático na abertura</option>
              </select>
            </div>
            <div className={styles.actionsRow} style={{ alignSelf: 'end' }}>
              {onOpenLibrary ? (
                <button type="button" className={styles.btnSecondary} onClick={onOpenLibrary}>
                  Abrir Biblioteca Universal
                </button>
              ) : null}
            </div>
          </div>
        </section>
      ) : null}

      {showSection('integrations') ? (
        <section className={styles.section} aria-label="Configurações de integrações">
          <h2 className={styles.sectionTitle}>
            <Cpu
              size={16}
              aria-hidden="true"
              style={{ marginRight: 6, verticalAlign: 'middle' }}
            />
            Integrações &amp; Execução de Emuladores
          </h2>
          <div className={styles.formGrid}>
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

            <div className={styles.field}>
              <span className={styles.label}>Status real da integração prioritária</span>
              <div className={styles.banner}>
                RPCS3 (PlayStation 3):{' '}
                <strong>{rpcs3Ready ? 'Instalado e validado' : 'Pendente de configuração'}</strong> ·
                Handoff de controle com dreno de 350ms ativo automaticamente.
              </div>
            </div>
          </div>
          <div className={styles.actionsRow}>
            {onOpenEmulators ? (
              <button type="button" className={styles.btnSecondary} onClick={onOpenEmulators}>
                Abrir Emuladores &amp; Firmware
              </button>
            ) : null}
            {onOpenTrophies ? (
              <button type="button" className={styles.btnSecondary} onClick={onOpenTrophies}>
                Abrir Central de Troféus
              </button>
            ) : null}
          </div>
        </section>
      ) : null}

      {showSection('system') ? (
        <section className={styles.section} aria-label="Informações do sistema">
          <h2 className={styles.sectionTitle}>
            <Shield
              size={16}
              aria-hidden="true"
              style={{ marginRight: 6, verticalAlign: 'middle' }}
            />
            Sistema &amp; Segurança Local-First
          </h2>
          <div className={styles.banner}>
            <div>
              <strong>Versão do Hub:</strong> <span className={styles.mono}>v{appVersion}</span> ·{' '}
              <strong>Transporte ativo:</strong>{' '}
              {transportKind === 'tauri'
                ? 'Tauri 2 Desktop IPC + SQLite (WAL / STRICT)'
                : 'Transporte de Teste UI'}
            </div>
            <div className={styles.muted} style={{ marginTop: 4 }}>
              Execução 100% local no Windows via spawn direto por array de argumentos, zero
              telemetria externa, zero execução de scripts em pacotes de temas e zero distribuição
              de firmware/BIOS proprietários.
            </div>
          </div>
        </section>
      ) : null}

      {showSection('data') ? (
        <section className={styles.section} aria-label="Dados e backups SQLite">
          <h2 className={styles.sectionTitle}>
            <Database
              size={16}
              aria-hidden="true"
              style={{ marginRight: 6, verticalAlign: 'middle' }}
            />
            Integridade, Backup e Restauração do SQLite
          </h2>
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
                <strong>UTC:</strong>{' '}
                <span className={styles.mono}>{backupReport.createdAt}</span>
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
      ) : null}

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
