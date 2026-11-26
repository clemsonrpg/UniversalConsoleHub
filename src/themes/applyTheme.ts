import { resolveCoverAssetUrl } from '../api/tauriClient';
import type { DisplayMode } from '../types/domain';
import type { ResolvedTheme } from './builtins';

function isLightHexColor(color: string): boolean {
  const clean = color.trim();
  if (!clean.startsWith('#')) return false;
  let r = 0;
  let g = 0;
  let b = 0;
  if (clean.length === 4) {
    r = Number.parseInt(`${clean[1]}${clean[1]}`, 16);
    g = Number.parseInt(`${clean[2]}${clean[2]}`, 16);
    b = Number.parseInt(`${clean[3]}${clean[3]}`, 16);
  } else if (clean.length === 7 || clean.length === 9) {
    r = Number.parseInt(clean.slice(1, 3), 16);
    g = Number.parseInt(clean.slice(3, 5), 16);
    b = Number.parseInt(clean.slice(5, 7), 16);
  } else {
    return false;
  }
  if (Number.isNaN(r) || Number.isNaN(g) || Number.isNaN(b)) return false;
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  return luminance > 0.68;
}

const XMB_HORIZON_PALETTES: Record<string, { canvas: string; rail: string; surface: string }> = {
  sapphire: { canvas: '#070f20', rail: '#0b1730', surface: '#102040' },
  obsidian: { canvas: '#090b10', rail: '#10131c', surface: '#161b26' },
  emerald: { canvas: '#061714', rail: '#0a231e', surface: '#0f2f28' },
  crimson: { canvas: '#180910', rail: '#240f1a', surface: '#301424' },
};

const COVER_MIN_WIDTH: Record<string, string> = {
  compact: '165px',
  normal: '205px',
  large: '255px',
};

export function applyThemeToDocument(
  theme: ResolvedTheme,
  overrides: Record<string, string>,
  displayMode: DisplayMode,
  reduceMotionGlobal = false
): void {
  if (typeof document === 'undefined') return;
  const root = document.documentElement;
  const { manifest } = theme;
  const palette = { ...manifest.palette };

  // Apply safe user customizable setting overrides
  const horizonKey = overrides['xmbHorizon'];
  if (manifest.id === 'xmb-inspired' && horizonKey && XMB_HORIZON_PALETTES[horizonKey]) {
    const preset = XMB_HORIZON_PALETTES[horizonKey];
    if (preset) {
      palette.canvas = preset.canvas;
      palette.rail = preset.rail;
      palette.surface = preset.surface;
    }
  }

  const customAccent = overrides['accentPrimary']?.trim();
  if (customAccent && /^#[0-9a-fA-F]{3,8}$/.test(customAccent)) {
    palette.accentPrimary = customAccent;
    palette.accentHover = customAccent;
    palette.focusRing = customAccent;
  }

  const accentContrastText = isLightHexColor(palette.accentPrimary) ? '#0b0c0e' : '#ffffff';

  root.dataset.themeId = manifest.id;
  root.dataset.themeShell = manifest.layout.shell;
  root.dataset.themeFocus = manifest.components.focusStyle;
  root.dataset.themeSurface = manifest.components.surfaceStyle;
  root.dataset.themeAspect = manifest.layout.cardAspect;
  root.dataset.themeDensity = manifest.layout.density;
  root.dataset.themeTransition = manifest.motion.pageTransition;
  root.dataset.displayMode = displayMode;
  root.dataset.xmbWave =
    manifest.layout.shell === 'xmb_crossbar' && overrides['animatedWave'] !== 'false'
      ? 'on'
      : 'off';
  root.dataset.reducedMotion = reduceMotionGlobal ? 'true' : 'false';

  const set = (name: string, value: string) => root.style.setProperty(name, value);

  set('--bg-canvas', palette.canvas);
  set('--bg-rail', palette.rail);
  set('--bg-surface', palette.surface);
  set('--bg-surface-raised', palette.surfaceRaised);
  set('--bg-input', palette.input);
  set('--border-subtle', palette.borderSubtle);
  set('--border-strong', palette.borderStrong);
  set('--text-primary', palette.textPrimary);
  set('--text-secondary', palette.textSecondary);
  set('--text-muted', palette.textMuted);
  set('--accent-primary', palette.accentPrimary);
  set('--accent-primary-hover', palette.accentHover);
  set('--accent-primary-muted', palette.accentMuted);
  set('--accent-secondary', palette.accentSecondary);
  set('--accent-contrast-text', accentContrastText);
  set('--focus-color', palette.focusRing);
  set('--focus-ring', `0 0 0 2px ${palette.canvas}, 0 0 0 4px ${palette.focusRing}`);

  set('--font-display', manifest.fonts.display);
  set('--font-sans', manifest.fonts.body);
  set('--font-mono', manifest.fonts.mono);

  set('--radius-sm', `${manifest.components.radiusSm}px`);
  set('--radius-md', `${manifest.components.radiusMd}px`);
  set('--radius-lg', `${manifest.components.radiusLg}px`);

  const duration = reduceMotionGlobal ? 0 : manifest.motion.durationMs;
  set('--transition-fast', `${duration}ms cubic-bezier(0.2, 0, 0, 1)`);

  const coverScale = overrides['coverScale'] ?? (displayMode === 'fullscreen' ? 'large' : 'normal');
  set('--library-card-min', COVER_MIN_WIDTH[coverScale] ?? '205px');

  const rawBackdropOpacity = Number.parseInt(overrides['heroBackdropOpacity'] ?? '42', 10);
  const backdropOpacity = Number.isNaN(rawBackdropOpacity)
    ? 0.42
    : Math.max(0, Math.min(80, rawBackdropOpacity)) / 100;
  set('--hero-backdrop-opacity', String(backdropOpacity));

  const bgAssetUrl = resolveCoverAssetUrl(theme.backgroundImagePath);
  set('--theme-bg-image', bgAssetUrl ? `url("${bgAssetUrl}")` : 'none');
}
