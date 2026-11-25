import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ReadinessTracker } from './ReadinessTracker';

describe('ReadinessTracker (4-stage RPCS3 truthfulness)', () => {
  it('explicitly marks XMB runtime as unconfirmed even when stages 1 to 3 succeed', () => {
    render(
      <ReadinessTracker
        readiness={{
          firmwarePackageValidated: true,
          firmwareSha256: 'abc123',
          systemFilesPresent: true,
          commandBuildable: true,
          xmbRuntimeConfirmed: false,
          generatedCommand: {
            executable: 'C:\\RPCS3\\rpcs3.exe',
            args: ['--no-gui', 'C:\\RPCS3\\dev_flash\\vsh\\module\\vsh.self'],
            workingDir: 'C:\\RPCS3',
          },
          notes: ['Estágio 4/4: Execução real do XMB NÃO confirmada na Fase 1.'],
        }}
      />
    );

    expect(screen.getByText(/Pacote de Firmware/i)).toBeInTheDocument();
    expect(screen.getByText(/Arquivos de Sistema/i)).toBeInTheDocument();
    expect(screen.getByText(/Comando Seguro Gerado/i)).toBeInTheDocument();
    expect(screen.getByText(/Confirmação Runtime do XMB/i)).toBeInTheDocument();
    expect(screen.getByText(/Não confirmado \(Fase 4\)/i)).toBeInTheDocument();
  });
});
