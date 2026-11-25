import { useState, type FormEvent } from 'react';
import { CapabilityBadge } from '../components/CapabilityBadge';
import { ReadinessTracker } from '../components/ReadinessTracker';
import type {
  EmulatorRecord,
  FirmwareRecordDto,
  Rpcs3InspectionDto,
} from '../types/domain';
import styles from './Pages.module.css';

export interface EmulatorsPageProps {
  emulators: EmulatorRecord[];
  firmwareRecords: FirmwareRecordDto[];
  rpcs3Inspection: Rpcs3InspectionDto;
  onConfigureEmulator: (slug: string, exePath: string) => Promise<void>;
  onInspectRpcs3: () => Promise<void>;
  onRegisterFirmware: (
    consoleSlug: string,
    emulatorSlug: string,
    filePath: string
  ) => Promise<void>;
}

export function EmulatorsPage({
  emulators,
  firmwareRecords,
  rpcs3Inspection,
  onConfigureEmulator,
  onInspectRpcs3,
  onRegisterFirmware,
}: EmulatorsPageProps) {
  const rpcs3 = emulators.find((e) => e.slug === 'rpcs3');
  const [rpcs3Path, setRpcs3Path] = useState(rpcs3?.executablePath ?? '');
  const [firmwarePath, setFirmwarePath] = useState('');
  const [feedback, setFeedback] = useState<{ kind: 'ok' | 'err'; text: string } | null>(null);
  const [busy, setBusy] = useState(false);

  const handleSaveRpcs3 = async (event: FormEvent) => {
    event.preventDefault();
    setFeedback(null);
    setBusy(true);
    try {
      await onConfigureEmulator('rpcs3', rpcs3Path);
      setFeedback({
        kind: 'ok',
        text: 'Caminho do RPCS3 validado e salvo. Diagnóstico de 4 estágios atualizado.',
      });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setBusy(false);
    }
  };

  const handleValidateFirmware = async (event: FormEvent) => {
    event.preventDefault();
    setFeedback(null);
    setBusy(true);
    try {
      await onRegisterFirmware('ps3', 'rpcs3', firmwarePath);
      setFirmwarePath('');
      setFeedback({
        kind: 'ok',
        text: 'Arquivo de firmware validado localmente via SHA-256 (nenhum binário proprietário armazenado ou distribuído).',
      });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setBusy(false);
    }
  };

  const handleReinspect = async () => {
    setFeedback(null);
    setBusy(true);
    try {
      await onInspectRpcs3();
      setFeedback({ kind: 'ok', text: 'Inspeção local do RPCS3 atualizada.' });
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
          <h1 className={styles.title}>Emuladores &amp; Firmware</h1>
          <p className={styles.subtitle}>
            O Universal Console Hub nunca distribui BIOS, NAND, chaves ou firmware proprietário.
            Configure seus próprios executáveis e valide arquivos obtidos legalmente localmente.
          </p>
        </div>
      </header>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Preparação RPCS3 (PlayStation 3 — Fase 1)</h2>
        <p className={styles.muted}>{rpcs3Inspection.diagnostic}</p>

        <form onSubmit={handleSaveRpcs3} className={styles.formGrid}>
          <div className={styles.field}>
            <label className={styles.label} htmlFor="rpcs3-exe-path">
              Caminho local do rpcs3.exe
            </label>
            <input
              id="rpcs3-exe-path"
              className={styles.input}
              required
              value={rpcs3Path}
              onChange={(e) => setRpcs3Path(e.target.value)}
              placeholder="C:\Emulators\RPCS3\rpcs3.exe"
            />
          </div>
          <div className={styles.actionsRow} style={{ alignSelf: 'end' }}>
            <button type="submit" className={styles.btnPrimary} disabled={busy}>
              Validar e Salvar RPCS3
            </button>
            <button
              type="button"
              className={styles.btnSecondary}
              disabled={busy}
              onClick={() => void handleReinspect()}
            >
              Reinspecionar Disco
            </button>
          </div>
        </form>

        <form onSubmit={handleValidateFirmware} className={styles.formGrid}>
          <div className={styles.field}>
            <label className={styles.label} htmlFor="ps3-pup-path">
              Validar pacote PS3UPDAT.PUP fornecido pelo usuário (SHA-256 local)
            </label>
            <input
              id="ps3-pup-path"
              className={styles.input}
              required
              value={firmwarePath}
              onChange={(e) => setFirmwarePath(e.target.value)}
              placeholder="C:\Firmware\PS3UPDAT.PUP"
            />
          </div>
          <div className={styles.actionsRow} style={{ alignSelf: 'end' }}>
            <button type="submit" className={styles.btnSecondary} disabled={busy}>
              Validar Firmware Local
            </button>
          </div>
        </form>

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

        {rpcs3Inspection.readiness ? (
          <ReadinessTracker readiness={rpcs3Inspection.readiness} />
        ) : null}
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Catálogo de Emuladores ({emulators.length})</h2>
        <div className={styles.tableWrap}>
          <table className={styles.table}>
            <thead>
              <tr>
                <th scope="col">Emulador</th>
                <th scope="col">Console</th>
                <th scope="col">Estágio</th>
                <th scope="col">Capacidades</th>
                <th scope="col">Executável Configurado</th>
              </tr>
            </thead>
            <tbody>
              {emulators.map((emu) => (
                <tr key={emu.id}>
                  <td>
                    <strong>{emu.name}</strong>
                  </td>
                  <td className={styles.mono}>{emu.consoleSlug.toUpperCase()}</td>
                  <td className={styles.mono}>{emu.integrationPhase}</td>
                  <td>
                    <div className={styles.badgeRow}>
                      {emu.capabilities.supportedStates.map((s) => (
                        <CapabilityBadge key={s} state={s} />
                      ))}
                    </div>
                  </td>
                  <td className={styles.mono}>{emu.executablePath ?? 'Não configurado'}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Registros Locais de Firmware ({firmwareRecords.length})</h2>
        {firmwareRecords.length === 0 ? (
          <p className={styles.muted}>Nenhum pacote validado localmente ainda.</p>
        ) : (
          <ul className={styles.list}>
            {firmwareRecords.map((fw) => (
              <li key={fw.id} className={styles.listItem}>
                <div>
                  <div className={styles.mono}>{fw.filePath}</div>
                  <div className={styles.muted}>
                    {fw.consoleSlug.toUpperCase()} · {fw.validationStatus} · SHA-256:{' '}
                    {fw.sha256}
                  </div>
                </div>
                <span className={styles.mono}>{fw.validatedAt}</span>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
