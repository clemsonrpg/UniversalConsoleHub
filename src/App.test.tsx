import { fireEvent, render, screen, waitFor } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';
import { App } from './App';
import { createStatefulTestTransport } from './test/mockTransport';

describe('App Phase 1 functional UI flows (unit layer with injected test transport)', () => {
  afterEach(() => {
    delete window.__UCH_TEST_TRANSPORT__;
  });

  it('shows honest DesktopRuntimeRequired notice when opened outside Tauri without test transport', () => {
    render(<App />);
    expect(screen.getByRole('heading', { name: /Universal Console Hub/i })).toBeInTheDocument();
    expect(screen.getByText(/requer o runtime desktop Tauri 2/i)).toBeInTheDocument();
  });

  it('boots full Phase 1 hub shell, opens PS3 details, registers game, edits profile, and creates backup', async () => {
    window.__UCH_TEST_TRANSPORT__ = createStatefulTestTransport();
    render(<App />);

    // Home loaded with real bootstrap data
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /Visão Geral do Hub/i })).toBeInTheDocument();
    });

    // Navigate to Consoles & inspect PlayStation 3 multi-capability details
    fireEvent.click(screen.getByRole('button', { name: /Consoles/i }));
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /Coleção de Consoles/i })).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole('button', { name: /Abrir detalhes de PlayStation 3/i }));
    await waitFor(() => {
      expect(screen.getByRole('tab', { name: /Emulador & Verificações/i })).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole('tab', { name: /Emulador & Verificações/i }));
    await waitFor(() => {
      expect(screen.getByText(/Matriz de Capacidades por Funcionalidade/i)).toBeInTheDocument();
    });

    // Navigate to Library & confirm zero manual registration form (auto-discovery only)
    fireEvent.click(screen.getByRole('button', { name: /Biblioteca/i }));
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /Biblioteca Universal/i })).toBeInTheDocument();
    });
    expect(screen.queryByText(/Registrar Jogo Local no Banco/i)).not.toBeInTheDocument();
    expect(screen.getByText(/Descoberta e Sincronização Automática/i)).toBeInTheDocument();

    // Navigate to Profile & update display name
    fireEvent.click(screen.getByRole('button', { name: /Perfil Universal/i }));
    await waitFor(() => {
      expect(screen.getByLabelText(/Nome de exibição/i)).toBeInTheDocument();
    });
    fireEvent.change(screen.getByLabelText(/Nome de exibição/i), {
      target: { value: 'Clemson RPG' },
    });
    fireEvent.click(screen.getByRole('button', { name: /Salvar Perfil/i }));
    await waitFor(() => {
      expect(screen.getByText(/Perfil salvo com sucesso/i)).toBeInTheDocument();
    });

    // Navigate to Settings & create SQLite backup
    fireEvent.click(screen.getByRole('button', { name: /Configurações/i }));
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /Criar Backup SQLite/i })).toBeInTheDocument();
    });
    fireEvent.click(screen.getByRole('button', { name: /Criar Backup SQLite/i }));
    await waitFor(() => {
      expect(screen.getByText(/Backup íntegro gerado/i)).toBeInTheDocument();
    });
  });
});
