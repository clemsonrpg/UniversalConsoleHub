import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react';
import type { DisplayMode, InstalledThemePackage } from '../types/domain';
import { applyThemeToDocument } from './applyTheme';
import { BUILTIN_THEMES, DEFAULT_THEME_ID, type ResolvedTheme } from './builtins';

export interface ThemeContextValue {
  themes: ResolvedTheme[];
  activeTheme: ResolvedTheme;
  previewTheme: ResolvedTheme | null;
  effectiveTheme: ResolvedTheme;
  disabledThemeIds: string[];
  activeOverrides: Record<string, string>;
  getOverridesForTheme: (themeId: string) => Record<string, string>;
  selectTheme: (themeId: string) => Promise<void>;
  startPreview: (themeId: string) => void;
  cancelPreview: () => void;
  applyPreview: () => Promise<void>;
  resetToDefaultTheme: () => Promise<void>;
  toggleThemeEnabled: (themeId: string) => Promise<void>;
  updateThemeSetting: (themeId: string, key: string, value: string) => Promise<void>;
  importLocalTheme: (sourcePath: string) => Promise<ResolvedTheme>;
  uninstallTheme: (themeId: string) => Promise<void>;
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

function parseJsonRecord(raw: string | undefined): Record<string, string> {
  if (!raw) return {};
  try {
    const parsed = JSON.parse(raw) as unknown;
    if (parsed && typeof parsed === 'object' && !Array.isArray(parsed)) {
      const out: Record<string, string> = {};
      for (const [k, v] of Object.entries(parsed)) {
        if (typeof v === 'string') out[k] = v;
      }
      return out;
    }
  } catch {
    // ignore malformed settings JSON
  }
  return {};
}

function parseJsonStringArray(raw: string | undefined): string[] {
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw) as unknown;
    if (Array.isArray(parsed)) {
      return parsed.filter((item): item is string => typeof item === 'string');
    }
  } catch {
    // ignore malformed JSON
  }
  return [];
}

function toResolvedCustomTheme(pkg: InstalledThemePackage): ResolvedTheme {
  return {
    manifest: pkg.manifest,
    builtIn: false,
    previewImagePath: pkg.previewImagePath,
    backgroundImagePath: pkg.backgroundImagePath,
    installedDir: pkg.installedDir,
    tagline: `Pacote local v${pkg.manifest.version} · por ${pkg.manifest.author}`,
  };
}

export interface ThemeProviderProps {
  settings: Record<string, string>;
  displayMode: DisplayMode;
  installedPackages: InstalledThemePackage[];
  onUpdateSetting: (key: string, value: string) => Promise<void>;
  onImportThemePackage: (sourcePath: string) => Promise<InstalledThemePackage>;
  onRemoveInstalledTheme: (themeId: string) => Promise<boolean>;
  onRefreshInstalledThemes: () => Promise<void>;
  children: ReactNode;
}

export function ThemeProvider({
  settings,
  displayMode,
  installedPackages,
  onUpdateSetting,
  onImportThemePackage,
  onRemoveInstalledTheme,
  onRefreshInstalledThemes,
  children,
}: ThemeProviderProps) {
  const [previewThemeId, setPreviewThemeId] = useState<string | null>(null);

  const themes = useMemo<ResolvedTheme[]>(() => {
    const custom = installedPackages.map(toResolvedCustomTheme);
    return [...BUILTIN_THEMES, ...custom];
  }, [installedPackages]);

  const disabledThemeIds = useMemo(
    () => parseJsonStringArray(settings['theme.disabled_ids']),
    [settings]
  );

  const requestedThemeId = settings['theme.active_id'] ?? DEFAULT_THEME_ID;
  const activeTheme = useMemo(() => {
    const found = themes.find((t) => t.manifest.id === requestedThemeId);
    if (found && !disabledThemeIds.includes(found.manifest.id)) {
      return found;
    }
    return BUILTIN_THEMES[0]!;
  }, [themes, requestedThemeId, disabledThemeIds]);

  const previewTheme = useMemo(() => {
    if (!previewThemeId) return null;
    return themes.find((t) => t.manifest.id === previewThemeId) ?? null;
  }, [themes, previewThemeId]);

  const effectiveTheme = previewTheme ?? activeTheme;

  const getOverridesForTheme = useCallback(
    (themeId: string): Record<string, string> => {
      return parseJsonRecord(settings[`theme.overrides.${themeId}`]);
    },
    [settings]
  );

  const activeOverrides = useMemo(
    () => getOverridesForTheme(effectiveTheme.manifest.id),
    [getOverridesForTheme, effectiveTheme.manifest.id]
  );

  const reduceMotionGlobal = settings['ui.reduce_motion'] === 'true';

  useEffect(() => {
    applyThemeToDocument(effectiveTheme, activeOverrides, displayMode, reduceMotionGlobal);
  }, [effectiveTheme, activeOverrides, displayMode, reduceMotionGlobal]);

  const selectTheme = useCallback(
    async (themeId: string) => {
      if (disabledThemeIds.includes(themeId)) {
        const nextDisabled = disabledThemeIds.filter((id) => id !== themeId);
        await onUpdateSetting('theme.disabled_ids', JSON.stringify(nextDisabled));
      }
      await onUpdateSetting('theme.active_id', themeId);
      setPreviewThemeId(null);
    },
    [disabledThemeIds, onUpdateSetting]
  );

  const startPreview = useCallback((themeId: string) => {
    setPreviewThemeId(themeId);
  }, []);

  const cancelPreview = useCallback(() => {
    setPreviewThemeId(null);
  }, []);

  const applyPreview = useCallback(async () => {
    if (!previewThemeId) return;
    await selectTheme(previewThemeId);
  }, [previewThemeId, selectTheme]);

  const resetToDefaultTheme = useCallback(async () => {
    setPreviewThemeId(null);
    await onUpdateSetting('theme.active_id', DEFAULT_THEME_ID);
    await onUpdateSetting(`theme.overrides.${DEFAULT_THEME_ID}`, '{}');
  }, [onUpdateSetting]);

  const toggleThemeEnabled = useCallback(
    async (themeId: string) => {
      if (themeId === DEFAULT_THEME_ID) return;
      const isDisabled = disabledThemeIds.includes(themeId);
      const next = isDisabled
        ? disabledThemeIds.filter((id) => id !== themeId)
        : [...disabledThemeIds, themeId];
      await onUpdateSetting('theme.disabled_ids', JSON.stringify(next));
      if (!isDisabled && activeTheme.manifest.id === themeId) {
        await onUpdateSetting('theme.active_id', DEFAULT_THEME_ID);
      }
    },
    [disabledThemeIds, activeTheme.manifest.id, onUpdateSetting]
  );

  const updateThemeSetting = useCallback(
    async (themeId: string, key: string, value: string) => {
      const current = getOverridesForTheme(themeId);
      const next = { ...current, [key]: value };
      await onUpdateSetting(`theme.overrides.${themeId}`, JSON.stringify(next));
    },
    [getOverridesForTheme, onUpdateSetting]
  );

  const importLocalTheme = useCallback(
    async (sourcePath: string): Promise<ResolvedTheme> => {
      const pkg = await onImportThemePackage(sourcePath);
      await onRefreshInstalledThemes();
      return toResolvedCustomTheme(pkg);
    },
    [onImportThemePackage, onRefreshInstalledThemes]
  );

  const uninstallTheme = useCallback(
    async (themeId: string) => {
      if (activeTheme.manifest.id === themeId) {
        await onUpdateSetting('theme.active_id', DEFAULT_THEME_ID);
      }
      if (previewThemeId === themeId) {
        setPreviewThemeId(null);
      }
      await onRemoveInstalledTheme(themeId);
      await onRefreshInstalledThemes();
    },
    [activeTheme.manifest.id, previewThemeId, onUpdateSetting, onRemoveInstalledTheme, onRefreshInstalledThemes]
  );

  const value = useMemo<ThemeContextValue>(
    () => ({
      themes,
      activeTheme,
      previewTheme,
      effectiveTheme,
      disabledThemeIds,
      activeOverrides,
      getOverridesForTheme,
      selectTheme,
      startPreview,
      cancelPreview,
      applyPreview,
      resetToDefaultTheme,
      toggleThemeEnabled,
      updateThemeSetting,
      importLocalTheme,
      uninstallTheme,
    }),
    [
      themes,
      activeTheme,
      previewTheme,
      effectiveTheme,
      disabledThemeIds,
      activeOverrides,
      getOverridesForTheme,
      selectTheme,
      startPreview,
      cancelPreview,
      applyPreview,
      resetToDefaultTheme,
      toggleThemeEnabled,
      updateThemeSetting,
      importLocalTheme,
      uninstallTheme,
    ]
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

export function useOptionalTheme(): ThemeContextValue | null {
  return useContext(ThemeContext);
}

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext);
  if (!ctx) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return ctx;
}
