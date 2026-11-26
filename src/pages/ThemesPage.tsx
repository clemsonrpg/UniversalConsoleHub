import { useState, type FormEvent } from 'react';
import { Check, Eye, FolderPlus, RotateCcw, Sliders, Trash2 } from 'lucide-react';
import { resolveCoverAssetUrl } from '../api/tauriClient';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { FilePathField } from '../components/FilePathField';
import { DEFAULT_THEME_ID, type ResolvedTheme } from '../themes/builtins';
import { useTheme } from '../themes/ThemeProvider';
import type { ThemeShellLayout } from '../types/domain';
import styles from './Pages.module.css';
import themeStyles from './ThemesPage.module.css';

const SHELL_LABELS: Record<ThemeShellLayout, string> = {
  sidebar: 'Barra Lateral',
  topbar_blades: 'Lâminas Superiores',
  minimal_dock: 'Dock Minimalista',
  xmb_crossbar: 'Barra Cruzada (XMB)',
};

const FOCUS_LABELS: Record<string, string> = {
  ring: 'Anel Preciso',
  glow_bar: 'Barra Tátil',
  bracket: 'Moldura Minimal',
  xmb_wave: 'Halos Luminosos',
};

function ThemeMiniPreview({ theme }: { theme: ResolvedTheme }) {
  const { palette, layout } = theme.manifest;
  const previewUrl = resolveCoverAssetUrl(theme.previewImagePath);

  if (previewUrl) {
    return (
      <img
        src={previewUrl}
        alt={`Prévia visual do tema ${theme.manifest.name}`}
        className={themeStyles.previewImage}
      />
    );
  }

  if (layout.shell === 'sidebar') {
    return (
      <div
        className={`${themeStyles.wireframe} ${themeStyles.wireSidebar}`}
        style={{ background: palette.canvas }}
        aria-hidden="true"
      >
        <div
          className={themeStyles.wireRail}
          style={{ background: palette.rail, border: `1px solid ${palette.borderSubtle}` }}
        >
          <span
            className={themeStyles.wireNavPillActive}
            style={{ background: palette.accentPrimary }}
          />
          <span className={themeStyles.wireNavPill} style={{ background: palette.textMuted }} />
          <span className={themeStyles.wireNavPill} style={{ background: palette.textMuted }} />
          <span className={themeStyles.wireNavPill} style={{ background: palette.textMuted }} />
        </div>
        <div className={themeStyles.wireMain}>
          <div
            className={themeStyles.wireHero}
            style={{
              background: `linear-gradient(135deg, ${palette.surfaceRaised}, ${palette.surface})`,
              border: `1px solid ${palette.borderStrong}`,
            }}
          >
            <span
              style={{
                width: '48%',
                height: '8px',
                borderRadius: '3px',
                background: palette.textPrimary,
                opacity: 0.8,
              }}
            />
            <span
              style={{
                width: '24%',
                height: '14px',
                borderRadius: '3px',
                background: palette.accentPrimary,
              }}
            />
          </div>
          <div className={themeStyles.wireTiles}>
            <div className={themeStyles.wireTile} style={{ background: palette.surfaceRaised }} />
            <div
              className={themeStyles.wireTile}
              style={{
                background: palette.surfaceRaised,
                boxShadow: `0 0 0 2px ${palette.focusRing}`,
              }}
            />
            <div className={themeStyles.wireTile} style={{ background: palette.surface }} />
          </div>
        </div>
      </div>
    );
  }

  if (layout.shell === 'topbar_blades') {
    return (
      <div
        className={themeStyles.wireframe}
        style={{ background: palette.canvas }}
        aria-hidden="true"
      >
        <div
          className={themeStyles.wireTopBar}
          style={{ background: palette.rail, borderBottom: `2px solid ${palette.accentPrimary}` }}
        >
          <span
            className={themeStyles.wireBlade}
            style={{ background: palette.accentPrimary }}
          />
          <span className={themeStyles.wireBlade} style={{ background: palette.surfaceRaised }} />
          <span className={themeStyles.wireBlade} style={{ background: palette.surfaceRaised }} />
        </div>
        <div className={themeStyles.wireTiles}>
          <div
            className={themeStyles.wireTile}
            style={{
              background: palette.surface,
              borderTop: `2px solid ${palette.accentPrimary}`,
            }}
          />
          <div
            className={themeStyles.wireTile}
            style={{
              background: palette.surfaceRaised,
              borderTop: `2px solid ${palette.accentSecondary}`,
            }}
          />
          <div className={themeStyles.wireTile} style={{ background: palette.surface }} />
        </div>
      </div>
    );
  }

  if (layout.shell === 'minimal_dock') {
    return (
      <div
        className={themeStyles.wireframe}
        style={{ background: palette.canvas }}
        aria-hidden="true"
      >
        <div className={themeStyles.wireDockBar}>
          <span
            style={{
              width: '32px',
              height: '6px',
              background: palette.textPrimary,
              borderRadius: '2px',
            }}
          />
          <div className={themeStyles.wireDockDots}>
            <span
              className={themeStyles.wireDockPill}
              style={{ background: palette.accentPrimary }}
            />
            <span
              className={themeStyles.wireDockPill}
              style={{ background: palette.borderStrong }}
            />
            <span
              className={themeStyles.wireDockPill}
              style={{ background: palette.borderStrong }}
            />
          </div>
        </div>
        <div className={themeStyles.wireTiles}>
          <div
            className={themeStyles.wireTile}
            style={{ background: palette.surface, borderColor: palette.accentPrimary }}
          />
          <div className={themeStyles.wireTile} style={{ background: palette.surface }} />
          <div className={themeStyles.wireTile} style={{ background: palette.surface }} />
        </div>
      </div>
    );
  }

  // xmb_crossbar
  return (
    <div
      className={themeStyles.wireframe}
      style={{
        background: `radial-gradient(ellipse at 25% 15%, ${palette.surfaceRaised}, ${palette.canvas} 72%)`,
      }}
      aria-hidden="true"
    >
      <div className={themeStyles.wireXmbBar}>
        <span className={themeStyles.wireXmbIcon} style={{ background: palette.surfaceRaised }} />
        <span
          className={themeStyles.wireXmbIcon}
          style={{
            background: palette.accentPrimary,
            boxShadow: `0 0 10px ${palette.focusRing}`,
          }}
        />
        <span className={themeStyles.wireXmbIcon} style={{ background: palette.surfaceRaised }} />
        <span className={themeStyles.wireXmbIcon} style={{ background: palette.surfaceRaised }} />
      </div>
      <div className={themeStyles.wireXmbCol}>
        <div
          className={themeStyles.wireXmbItem}
          style={{
            background: palette.accentMuted,
            border: `1px solid ${palette.focusRing}`,
          }}
        />
        <div className={themeStyles.wireXmbItem} style={{ background: palette.surface }} />
        <div className={themeStyles.wireXmbItem} style={{ background: palette.surface }} />
      </div>
    </div>
  );
}

export function ThemesPage() {
  const {
    themes,
    activeTheme,
    previewTheme,
    effectiveTheme,
    disabledThemeIds,
    activeOverrides,
    selectTheme,
    startPreview,
    cancelPreview,
    applyPreview,
    resetToDefaultTheme,
    toggleThemeEnabled,
    updateThemeSetting,
    importLocalTheme,
    uninstallTheme,
  } = useTheme();

  const [importMode, setImportMode] = useState<'directory' | 'file'>('directory');
  const [importPath, setImportPath] = useState('');
  const [busy, setBusy] = useState(false);
  const [feedback, setFeedback] = useState<{ kind: 'ok' | 'err'; text: string } | null>(null);
  const [pendingUninstall, setPendingUninstall] = useState<ResolvedTheme | null>(null);

  const handleSelect = async (themeId: string, themeName: string) => {
    setBusy(true);
    setFeedback(null);
    try {
      await selectTheme(themeId);
      setFeedback({
        kind: 'ok',
        text: `Tema "${themeName}" aplicado e salvo imediatamente no SQLite.`,
      });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setBusy(false);
    }
  };

  const handleResetDefault = async () => {
    setBusy(true);
    setFeedback(null);
    try {
      await resetToDefaultTheme();
      setFeedback({
        kind: 'ok',
        text: 'Tema padrão "Universal Modern" restaurado com configurações originais.',
      });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setBusy(false);
    }
  };

  const handleToggleEnabled = async (theme: ResolvedTheme) => {
    setBusy(true);
    setFeedback(null);
    try {
      const wasDisabled = disabledThemeIds.includes(theme.manifest.id);
      await toggleThemeEnabled(theme.manifest.id);
      setFeedback({
        kind: 'ok',
        text: wasDisabled
          ? `Tema "${theme.manifest.name}" reativado.`
          : `Tema "${theme.manifest.name}" desativado.`,
      });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setBusy(false);
    }
  };

  const handleImport = async (event: FormEvent) => {
    event.preventDefault();
    if (!importPath.trim()) return;
    setBusy(true);
    setFeedback(null);
    try {
      const installed = await importLocalTheme(importPath.trim());
      setImportPath('');
      setFeedback({
        kind: 'ok',
        text: `Pacote local "${installed.manifest.name}" (v${installed.manifest.version}) validado e instalado com segurança.`,
      });
    } catch (err) {
      setFeedback({
        kind: 'err',
        text: `Falha ao validar/instalar pacote de tema: ${String(err)}`,
      });
    } finally {
      setBusy(false);
    }
  };

  const handleConfirmUninstall = async () => {
    if (!pendingUninstall) return;
    const removedName = pendingUninstall.manifest.name;
    setBusy(true);
    setFeedback(null);
    try {
      await uninstallTheme(pendingUninstall.manifest.id);
      setPendingUninstall(null);
      setFeedback({
        kind: 'ok',
        text: `Tema local "${removedName}" removido do disco e das preferências.`,
      });
    } catch (err) {
      setPendingUninstall(null);
      setFeedback({ kind: 'err', text: `Erro ao remover tema: ${String(err)}` });
    } finally {
      setBusy(false);
    }
  };

  const handleSettingChange = async (key: string, value: string) => {
    try {
      await updateThemeSetting(effectiveTheme.manifest.id, key, value);
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    }
  };

  const handleResetThemeOverrides = async () => {
    setBusy(true);
    try {
      for (const field of effectiveTheme.manifest.customizableSettings) {
        await updateThemeSetting(effectiveTheme.manifest.id, field.key, field.defaultValue);
      }
      setFeedback({
        kind: 'ok',
        text: `Ajustes personalizados de "${effectiveTheme.manifest.name}" restaurados.`,
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
          <h1 className={styles.title}>Gerenciador de Temas</h1>
          <p className={styles.subtitle}>
            Sistema extensível e seguro inspirado no Playnite: altere layout de navegação, paleta,
            tipografia, foco do controle e transições instantaneamente, ou instale pacotes locais
            declarativos (sem execução de código arbitrário).
          </p>
        </div>
        <div className={styles.actionsRow}>
          <button
            type="button"
            className={styles.btnSecondary}
            disabled={busy}
            onClick={() => void handleResetDefault()}
          >
            <RotateCcw
              size={14}
              aria-hidden="true"
              style={{ marginRight: 6, verticalAlign: 'middle' }}
            />
            Restaurar Tema Padrão
          </button>
        </div>
      </header>

      {previewTheme ? (
        <div className={`${styles.banner} ${styles.bannerInfo}`} role="status" aria-live="polite">
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              flexWrap: 'wrap',
              gap: '12px',
            }}
          >
            <span>
              <strong>Modo Prévia Ativo:</strong> visualizando{' '}
              <strong>{previewTheme.manifest.name}</strong> (layout{' '}
              {SHELL_LABELS[previewTheme.manifest.layout.shell]}). Seu tema salvo continua sendo{' '}
              <strong>{activeTheme.manifest.name}</strong>.
            </span>
            <div className={styles.actionsRow}>
              <button
                type="button"
                className={styles.btnPrimary}
                disabled={busy}
                onClick={() => void applyPreview()}
              >
                Aplicar Prévia
              </button>
              <button
                type="button"
                className={styles.btnSecondary}
                disabled={busy}
                onClick={cancelPreview}
              >
                Cancelar Prévia
              </button>
            </div>
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

      <section className={styles.section} aria-label="Galeria de temas disponíveis">
        <div className={styles.header}>
          <h2 className={styles.sectionTitle}>Temas Disponíveis ({themes.length})</h2>
          <span className={styles.muted}>
            Tema ativo: <strong>{activeTheme.manifest.name}</strong>
          </span>
        </div>

        <div className={themeStyles.gallery}>
          {themes.map((theme, index) => {
            const isActive = activeTheme.manifest.id === theme.manifest.id;
            const isPreviewing = previewTheme?.manifest.id === theme.manifest.id;
            const isDisabled = disabledThemeIds.includes(theme.manifest.id);
            const isDefault = theme.manifest.id === DEFAULT_THEME_ID;
            const pal = theme.manifest.palette;

            return (
              <article
                key={theme.manifest.id}
                data-focus-card="true"
                data-theme-card={theme.manifest.id}
                className={`${themeStyles.themeCard} ${
                  isActive ? themeStyles.themeCardActive : ''
                } ${isDisabled ? themeStyles.themeCardDisabled : ''}`}
              >
                <div className={themeStyles.previewStage}>
                  <ThemeMiniPreview theme={theme} />
                  <div className={themeStyles.stageBadges}>
                    <span className={themeStyles.stageBadge}>
                      {theme.builtIn ? 'Integrado' : 'Pacote Local'}
                    </span>
                    {isActive ? (
                      <span className={`${themeStyles.stageBadge} ${themeStyles.stageBadgeActive}`}>
                        Ativo
                      </span>
                    ) : null}
                    {isPreviewing ? (
                      <span className={themeStyles.stageBadge}>Em Prévia</span>
                    ) : null}
                    {isDisabled ? <span className={themeStyles.stageBadge}>Desativado</span> : null}
                  </div>
                  <div className={themeStyles.swatches} aria-hidden="true">
                    <span className={themeStyles.swatch} style={{ background: pal.canvas }} />
                    <span className={themeStyles.swatch} style={{ background: pal.surface }} />
                    <span
                      className={themeStyles.swatch}
                      style={{ background: pal.accentPrimary }}
                    />
                    <span
                      className={themeStyles.swatch}
                      style={{ background: pal.accentSecondary }}
                    />
                    <span className={themeStyles.swatch} style={{ background: pal.focusRing }} />
                  </div>
                </div>

                <div className={themeStyles.cardBody}>
                  <div className={themeStyles.cardTop}>
                    <div>
                      <h3 className={themeStyles.cardTitle}>{theme.manifest.name}</h3>
                      <div className={themeStyles.cardMeta}>
                        v{theme.manifest.version} · {theme.manifest.author} · Hub{' '}
                        {theme.manifest.hubVersionRange}
                      </div>
                    </div>
                  </div>

                  <p className={themeStyles.cardDesc}>{theme.manifest.description}</p>

                  <div className={themeStyles.specPills}>
                    <span className={themeStyles.specPill}>
                      Layout: {SHELL_LABELS[theme.manifest.layout.shell]}
                    </span>
                    <span className={themeStyles.specPill}>
                      Foco:{' '}
                      {FOCUS_LABELS[theme.manifest.components.focusStyle] ??
                        theme.manifest.components.focusStyle}
                    </span>
                    <span className={themeStyles.specPill}>
                      Transição: {theme.manifest.motion.pageTransition} (
                      {theme.manifest.motion.durationMs}ms)
                    </span>
                  </div>

                  <div className={styles.actionsRow}>
                    <button
                      type="button"
                      className={isActive && !isPreviewing ? styles.btnSecondary : styles.btnPrimary}
                      data-focus-default={index === 0 ? 'true' : undefined}
                      disabled={busy || (isActive && !isPreviewing)}
                      onClick={() => void handleSelect(theme.manifest.id, theme.manifest.name)}
                    >
                      <Check
                        size={14}
                        aria-hidden="true"
                        style={{ marginRight: 5, verticalAlign: 'middle' }}
                      />
                      {isActive && !isPreviewing ? 'Tema Ativo' : 'Aplicar Tema'}
                    </button>

                    <button
                      type="button"
                      className={styles.btnSecondary}
                      disabled={busy}
                      onClick={() =>
                        isPreviewing ? cancelPreview() : startPreview(theme.manifest.id)
                      }
                    >
                      <Eye
                        size={14}
                        aria-hidden="true"
                        style={{ marginRight: 5, verticalAlign: 'middle' }}
                      />
                      {isPreviewing ? 'Sair da Prévia' : 'Pré-visualizar'}
                    </button>

                    {!isDefault ? (
                      <button
                        type="button"
                        className={styles.btnSecondary}
                        disabled={busy}
                        onClick={() => void handleToggleEnabled(theme)}
                      >
                        {isDisabled ? 'Reativar' : 'Desativar'}
                      </button>
                    ) : null}

                    {!theme.builtIn ? (
                      <button
                        type="button"
                        className={styles.btnDanger}
                        disabled={busy}
                        onClick={() => setPendingUninstall(theme)}
                      >
                        <Trash2
                          size={14}
                          aria-hidden="true"
                          style={{ marginRight: 5, verticalAlign: 'middle' }}
                        />
                        Remover
                      </button>
                    ) : null}
                  </div>
                </div>
              </article>
            );
          })}
        </div>
      </section>

      <section className={styles.section} aria-label="Personalização do tema em uso">
        <div className={styles.header}>
          <div>
            <h2 className={styles.sectionTitle}>
              <Sliders
                size={16}
                aria-hidden="true"
                style={{ marginRight: 6, verticalAlign: 'middle' }}
              />
              Personalizar: {effectiveTheme.manifest.name}
            </h2>
            <p className={styles.muted}>
              Ajustes declarativos permitidos pelo manifesto do tema. Alterações são refletidas
              imediatamente e salvas por tema no SQLite.
            </p>
          </div>
          {effectiveTheme.manifest.customizableSettings.length > 0 ? (
            <button
              type="button"
              className={styles.btnSecondary}
              disabled={busy}
              onClick={() => void handleResetThemeOverrides()}
            >
              Restaurar Ajustes do Tema
            </button>
          ) : null}
        </div>

        {effectiveTheme.manifest.customizableSettings.length === 0 ? (
          <p className={styles.muted}>
            Este tema não declara opções extras personalizáveis em seu manifesto.
          </p>
        ) : (
          <div className={themeStyles.customGrid}>
            {effectiveTheme.manifest.customizableSettings.map((field) => {
              const currentValue = activeOverrides[field.key] ?? field.defaultValue;
              const fieldId = `theme-setting-${effectiveTheme.manifest.id}-${field.key}`;

              if (field.kind === 'color') {
                return (
                  <div key={field.key} className={styles.field}>
                    <label className={styles.label} htmlFor={fieldId}>
                      {field.label}
                    </label>
                    <div className={themeStyles.colorRow}>
                      <input
                        id={fieldId}
                        type="color"
                        className={themeStyles.colorPicker}
                        value={/^#[0-9a-fA-F]{6}$/.test(currentValue) ? currentValue : '#3b82f6'}
                        onChange={(e) => void handleSettingChange(field.key, e.target.value)}
                      />
                      <span className={styles.mono}>{currentValue}</span>
                    </div>
                  </div>
                );
              }

              if (field.kind === 'select') {
                return (
                  <div key={field.key} className={styles.field}>
                    <label className={styles.label} htmlFor={fieldId}>
                      {field.label}
                    </label>
                    <select
                      id={fieldId}
                      className={styles.select}
                      value={currentValue}
                      onChange={(e) => void handleSettingChange(field.key, e.target.value)}
                    >
                      {(field.options ?? []).map((opt) => (
                        <option key={opt.value} value={opt.value}>
                          {opt.label}
                        </option>
                      ))}
                    </select>
                  </div>
                );
              }

              if (field.kind === 'boolean') {
                return (
                  <div key={field.key} className={styles.field}>
                    <label className={styles.label} htmlFor={fieldId}>
                      {field.label}
                    </label>
                    <select
                      id={fieldId}
                      className={styles.select}
                      value={currentValue === 'false' ? 'false' : 'true'}
                      onChange={(e) => void handleSettingChange(field.key, e.target.value)}
                    >
                      <option value="true">Ativado</option>
                      <option value="false">Desativado</option>
                    </select>
                  </div>
                );
              }

              // range
              const min = field.min ?? 0;
              const max = field.max ?? 100;
              return (
                <div key={field.key} className={styles.field}>
                  <label className={styles.label} htmlFor={fieldId}>
                    {field.label}
                  </label>
                  <div className={themeStyles.rangeRow}>
                    <input
                      id={fieldId}
                      type="range"
                      className={themeStyles.rangeInput}
                      min={min}
                      max={max}
                      value={currentValue}
                      onChange={(e) => void handleSettingChange(field.key, e.target.value)}
                    />
                    <span className={styles.mono}>{currentValue}</span>
                  </div>
                </div>
              );
            })}
          </div>
        )}
      </section>

      <section className={styles.section} aria-label="Instalar pacote local de tema">
        <div className={styles.header}>
          <div>
            <h2 className={styles.sectionTitle}>
              <FolderPlus
                size={16}
                aria-hidden="true"
                style={{ marginRight: 6, verticalAlign: 'middle' }}
              />
              Instalar Tema a partir de Pacote Local
            </h2>
            <p className={styles.muted}>
              Importe uma pasta contendo <code>theme.json</code> (e imagens locais PNG/JPEG/WebP) ou
              um arquivo de manifesto <code>.json</code>. O backend Rust valida esquema, cores,
              fontes seguras e magic bytes das imagens — bloqueando scripts, CSS arbitrário e path
              traversal.
            </p>
          </div>
          <div className={themeStyles.modeToggle} role="group" aria-label="Tipo de origem do tema">
            <button
              type="button"
              className={`${themeStyles.modeBtn} ${
                importMode === 'directory' ? themeStyles.modeBtnActive : ''
              }`}
              onClick={() => setImportMode('directory')}
            >
              Pasta do Pacote
            </button>
            <button
              type="button"
              className={`${themeStyles.modeBtn} ${
                importMode === 'file' ? themeStyles.modeBtnActive : ''
              }`}
              onClick={() => setImportMode('file')}
            >
              Arquivo .json
            </button>
          </div>
        </div>

        <form onSubmit={handleImport} className={styles.formGrid}>
          <div className={styles.field}>
            <label className={styles.label} htmlFor="theme-import-path">
              {importMode === 'directory'
                ? 'Caminho da pasta do pacote de tema (contendo theme.json)'
                : 'Caminho do arquivo de manifesto de tema (.json)'}
            </label>
            <FilePathField
              id="theme-import-path"
              mode={importMode}
              dialogTitle={
                importMode === 'directory'
                  ? 'Selecionar pasta do pacote de tema'
                  : 'Selecionar manifesto de tema (.json)'
              }
              filters={
                importMode === 'file'
                  ? [{ name: 'Manifesto de Tema JSON', extensions: ['json'] }]
                  : undefined
              }
              value={importPath}
              onChange={setImportPath}
              placeholder={
                importMode === 'directory'
                  ? 'C:\\Temas\\meu-tema-hub'
                  : 'C:\\Temas\\meu-tema\\theme.json'
              }
              disabled={busy}
            />
          </div>
          <div className={styles.actionsRow} style={{ alignSelf: 'end' }}>
            <button
              type="submit"
              className={styles.btnPrimary}
              disabled={busy || importPath.trim().length === 0}
            >
              {busy ? 'Validando...' : 'Importar Pacote de Tema'}
            </button>
          </div>
        </form>
      </section>

      <ConfirmDialog
        open={pendingUninstall !== null}
        title="Remover tema local instalado"
        confirmLabel="Remover Tema"
        cancelLabel="Cancelar"
        danger
        busy={busy}
        onConfirm={() => void handleConfirmUninstall()}
        onCancel={() => setPendingUninstall(null)}
      >
        {pendingUninstall ? (
          <p>
            Deseja desinstalar permanentemente o pacote local{' '}
            <strong>{pendingUninstall.manifest.name}</strong> ({pendingUninstall.manifest.id})? Caso
            esteja ativo, o Hub voltará automaticamente para o tema padrão{' '}
            <strong>Universal Modern</strong>.
          </p>
        ) : null}
      </ConfirmDialog>
    </div>
  );
}
