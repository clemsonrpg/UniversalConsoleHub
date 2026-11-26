import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import type { InstalledThemePackage } from '../types/domain';
import { BUILTIN_THEMES } from './builtins';
import { ThemeProvider, useTheme } from './ThemeProvider';

function ThemeProbe() {
  const {
    themes,
    activeTheme,
    previewTheme,
    effectiveTheme,
    selectTheme,
    startPreview,
    cancelPreview,
    applyPreview,
    resetToDefaultTheme,
    updateThemeSetting,
    importLocalTheme,
    uninstallTheme,
  } = useTheme();
  return (
    <div>
      <div data-testid="count">{themes.length}</div>
      <div data-testid="active-id">{activeTheme.manifest.id}</div>
      <div data-testid="effective-id">{effectiveTheme.manifest.id}</div>
      <div data-testid="preview-id">{previewTheme?.manifest.id ?? 'none'}</div>
      <button type="button" onClick={() => startPreview('console-classic')}>
        Preview Classic
      </button>
      <button type="button" onClick={() => cancelPreview()}>
        Cancel Preview
      </button>
      <button type="button" onClick={() => void applyPreview()}>
        Apply Preview
      </button>
      <button type="button" onClick={() => void selectTheme('xmb-inspired')}>
        Activate XMB
      </button>
      <button
        type="button"
        onClick={() => void updateThemeSetting('xmb-inspired', 'accentPrimary', '#38bdf8')}
      >
        Customize Accent
      </button>
      <button type="button" onClick={() => void resetToDefaultTheme()}>
        Reset Default
      </button>
      <button type="button" onClick={() => void importLocalTheme('C:\\Themes\\nordic.json')}>
        Import Local
      </button>
      <button type="button" onClick={() => void uninstallTheme('custom-test')}>
        Uninstall Custom
      </button>
    </div>
  );
}

describe('Extensible ThemeProvider & 4 distinct built-in themes', () => {
  it('provides 4 distinct built-in shell layouts, supports preview/apply/persist, custom overrides, import/uninstall, and reset', async () => {
    // Verify 4 built-in themes have genuinely distinct shell layouts
    const shells = new Set(BUILTIN_THEMES.map((t) => t.manifest.layout.shell));
    expect(BUILTIN_THEMES).toHaveLength(4);
    expect(shells.size).toBe(4);

    const onRemove = vi.fn().mockResolvedValue(true);

    function Harness() {
      const [settings, setSettings] = useState<Record<string, string>>({});
      const [installed, setInstalled] = useState<InstalledThemePackage[]>([]);

      const handleImport = async (): Promise<InstalledThemePackage> => {
        const pkg: InstalledThemePackage = {
          manifest: {
            ...BUILTIN_THEMES[0]!.manifest,
            id: 'custom-test',
            name: 'Custom Test Theme',
            author: 'Tester',
          },
          installedDir: 'C:\\themes\\custom-test',
          previewImagePath: null,
          backgroundImagePath: null,
          assetPaths: {},
          installedAt: '2026-11-26T00:00:00Z',
        };
        setInstalled([pkg]);
        return pkg;
      };

      const handleRemove = async (id: string) => {
        setInstalled((prev) => prev.filter((item) => item.manifest.id !== id));
        return onRemove(id);
      };

      return (
        <ThemeProvider
          settings={settings}
          displayMode="desktop"
          installedPackages={installed}
          onUpdateSetting={async (k, v) => {
            setSettings((prev) => ({ ...prev, [k]: v }));
          }}
          onImportThemePackage={handleImport}
          onRemoveInstalledTheme={handleRemove}
          onRefreshInstalledThemes={async () => {}}
        >
          <ThemeProbe />
        </ThemeProvider>
      );
    }

    render(<Harness />);
    expect(screen.getByTestId('count').textContent).toBe('4');
    expect(screen.getByTestId('active-id').textContent).toBe('universal-modern');
    expect(document.documentElement.dataset.themeShell).toBe('sidebar');

    // Preview does not mutate active theme until Apply
    fireEvent.click(screen.getByText('Preview Classic'));
    expect(screen.getByTestId('active-id').textContent).toBe('universal-modern');
    expect(screen.getByTestId('effective-id').textContent).toBe('console-classic');
    expect(document.documentElement.dataset.themeShell).toBe('topbar_blades');

    fireEvent.click(screen.getByText('Cancel Preview'));
    expect(screen.getByTestId('effective-id').textContent).toBe('universal-modern');

    // Apply preview persists theme & switches shell layout immediately
    fireEvent.click(screen.getByText('Preview Classic'));
    fireEvent.click(screen.getByText('Apply Preview'));
    await waitFor(() => {
      expect(screen.getByTestId('active-id').textContent).toBe('console-classic');
      expect(document.documentElement.dataset.themeShell).toBe('topbar_blades');
    });

    // Activate XMB Inspired + customize accent
    fireEvent.click(screen.getByText('Activate XMB'));
    await waitFor(() => {
      expect(document.documentElement.dataset.themeShell).toBe('xmb_crossbar');
    });
    fireEvent.click(screen.getByText('Customize Accent'));
    await waitFor(() => {
      expect(document.documentElement.style.getPropertyValue('--accent-primary')).toBe('#38bdf8');
    });

    // Import & remove local theme package
    fireEvent.click(screen.getByText('Import Local'));
    await waitFor(() => expect(screen.getByTestId('count').textContent).toBe('5'));

    fireEvent.click(screen.getByText('Uninstall Custom'));
    await waitFor(() => {
      expect(onRemove).toHaveBeenCalledWith('custom-test');
      expect(screen.getByTestId('count').textContent).toBe('4');
    });

    // Reset to default theme
    fireEvent.click(screen.getByText('Reset Default'));
    await waitFor(() => {
      expect(screen.getByTestId('active-id').textContent).toBe('universal-modern');
    });
  });
});
