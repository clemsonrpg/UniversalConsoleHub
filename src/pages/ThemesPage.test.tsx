import { fireEvent, render, screen, waitFor, within } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { BUILTIN_THEMES } from '../themes/builtins';
import { ThemeProvider } from '../themes/ThemeProvider';
import type { InstalledThemePackage } from '../types/domain';
import { ThemesPage } from './ThemesPage';

function ThemesHarness({
  initialPackages = [],
  onImportSpy,
  onRemoveSpy,
}: {
  initialPackages?: InstalledThemePackage[];
  onImportSpy?: ReturnType<typeof vi.fn>;
  onRemoveSpy?: ReturnType<typeof vi.fn>;
}) {
  const [settings, setSettings] = useState<Record<string, string>>({
    'theme.active_id': 'universal-modern',
  });
  const [packages, setPackages] = useState<InstalledThemePackage[]>(initialPackages);

  return (
    <ThemeProvider
      settings={settings}
      displayMode="desktop"
      installedPackages={packages}
      onUpdateSetting={async (k, v) => {
        setSettings((prev) => ({ ...prev, [k]: v }));
      }}
      onImportThemePackage={async (sourcePath) => {
        onImportSpy?.(sourcePath);
        const created: InstalledThemePackage = {
          manifest: {
            ...BUILTIN_THEMES[0]!.manifest,
            id: 'custom-obsidian',
            name: 'Obsidian Community',
            author: 'Tester',
            version: '1.2.0',
            description: 'Pacote local validado.',
          },
          installedDir: 'C:\\AppData\\themes\\custom-obsidian',
          previewImagePath: null,
          backgroundImagePath: null,
          assetPaths: {},
          installedAt: '2026-11-26T00:00:00Z',
        };
        setPackages((prev) => [...prev, created]);
        return created;
      }}
      onRemoveInstalledTheme={async (themeId) => {
        onRemoveSpy?.(themeId);
        setPackages((prev) => prev.filter((p) => p.manifest.id !== themeId));
        return true;
      }}
      onRefreshInstalledThemes={async () => {}}
    >
      <ThemesPage />
    </ThemeProvider>
  );
}

describe('ThemesPage (Playnite-inspired Theme Manager)', () => {
  it('lists all 4 distinct built-in themes, previews without persisting, applies, customizes, and resets default', async () => {
    render(<ThemesHarness />);

    expect(screen.getByRole('heading', { name: /Gerenciador de Temas/i })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Universal Modern' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Console Classic' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Minimal' })).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'XMB Inspired' })).toBeInTheDocument();

    // Preview Console Classic -> DOM switches to topbar_blades without changing persisted activeTheme
    const classicCard = document.querySelector('[data-theme-card="console-classic"]') as HTMLElement;
    expect(classicCard).toBeTruthy();
    fireEvent.click(within(classicCard).getByRole('button', { name: /Pré-visualizar/i }));
    await waitFor(() => {
      expect(document.documentElement.dataset.themeId).toBe('console-classic');
      expect(document.documentElement.dataset.themeShell).toBe('topbar_blades');
      expect(screen.getByText(/Modo Prévia Ativo/i)).toBeInTheDocument();
    });

    // Cancel preview reverts immediately back to Universal Modern
    fireEvent.click(screen.getByRole('button', { name: /Cancelar Prévia/i }));
    await waitFor(() => {
      expect(document.documentElement.dataset.themeId).toBe('universal-modern');
      expect(document.documentElement.dataset.themeShell).toBe('sidebar');
    });

    // Apply XMB Inspired and customize horizon preset
    const xmbCard = document.querySelector('[data-theme-card="xmb-inspired"]') as HTMLElement;
    fireEvent.click(within(xmbCard).getByRole('button', { name: /Aplicar Tema/i }));
    await waitFor(() => {
      expect(document.documentElement.dataset.themeId).toBe('xmb-inspired');
      expect(document.documentElement.dataset.themeShell).toBe('xmb_crossbar');
    });

    const horizonSelect = screen.getByLabelText(/Atmosfera do horizonte procedural/i);
    fireEvent.change(horizonSelect, { target: { value: 'emerald' } });
    await waitFor(() => {
      expect(document.documentElement.style.getPropertyValue('--bg-canvas')).toBe('#061714');
    });

    // Reset to default theme
    fireEvent.click(screen.getByRole('button', { name: /Restaurar Tema Padrão/i }));
    await waitFor(() => {
      expect(document.documentElement.dataset.themeId).toBe('universal-modern');
    });
  });

  it('imports a local theme package and removes it with ConfirmDialog safety', async () => {
    const onImport = vi.fn();
    const onRemove = vi.fn();
    render(<ThemesHarness onImportSpy={onImport} onRemoveSpy={onRemove} />);

    fireEvent.change(screen.getByLabelText(/Caminho da pasta do pacote de tema/i), {
      target: { value: 'C:\\Temas\\obsidian' },
    });
    fireEvent.click(screen.getByRole('button', { name: /Importar Pacote de Tema/i }));

    await waitFor(() => {
      expect(onImport).toHaveBeenCalledWith('C:\\Temas\\obsidian');
      expect(screen.getByText('Obsidian Community')).toBeInTheDocument();
    });

    const customCard = document.querySelector('[data-theme-card="custom-obsidian"]') as HTMLElement;
    expect(customCard).toBeTruthy();
    fireEvent.click(within(customCard).getByRole('button', { name: /Remover/i }));

    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole('button', { name: /Remover Tema/i }));

    await waitFor(() => {
      expect(onRemove).toHaveBeenCalledWith('custom-obsidian');
      expect(screen.queryByText('Obsidian Community')).not.toBeInTheDocument();
    });
  });
});
