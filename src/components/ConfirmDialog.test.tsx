import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ConfirmDialog } from './ConfirmDialog';

describe('ConfirmDialog accessible confirmation modal', () => {
  it('supports confirm, cancel, and Escape key handling', () => {
    const onConfirm = vi.fn();
    const onCancel = vi.fn();

    render(
      <ConfirmDialog
        open
        title="Confirmar restauração do banco SQLite"
        confirmLabel="Restaurar Backup"
        cancelLabel="Cancelar"
        danger
        onConfirm={onConfirm}
        onCancel={onCancel}
      >
        <p>Um snapshot pré-restauração será criado automaticamente.</p>
      </ConfirmDialog>
    );

    const dialog = screen.getByRole('dialog');
    expect(dialog).toHaveAttribute('aria-modal', 'true');
    expect(screen.getByText(/snapshot pré-restauração/i)).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: 'Cancelar' }));
    expect(onCancel).toHaveBeenCalledTimes(1);

    fireEvent.keyDown(window, { key: 'Escape' });
    expect(onCancel).toHaveBeenCalledTimes(2);

    fireEvent.click(screen.getByRole('button', { name: 'Restaurar Backup' }));
    expect(onConfirm).toHaveBeenCalledTimes(1);
  });

  it('renders nothing when closed', () => {
    const { container } = render(
      <ConfirmDialog open={false} title="Oculto" onConfirm={() => {}} onCancel={() => {}}>
        <p>Não deve aparecer</p>
      </ConfirmDialog>
    );
    expect(container.firstChild).toBeNull();
  });
});
