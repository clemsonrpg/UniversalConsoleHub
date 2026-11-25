import { useState, type FormEvent } from 'react';
import { resolveCoverAssetUrl } from '../api/tauriClient';
import { CapabilityBadge } from '../components/CapabilityBadge';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { FilePathField } from '../components/FilePathField';
import { ReadinessTracker } from '../components/ReadinessTracker';
import type {
  ConsoleEnvironmentDto,
  RuntimeVerificationFeature,
  ScanProgressEvent,
  ScanSummaryDto,
} from '../types/domain';
import {
  formatAvailabilityStatus,
  formatFeatureKey,
  formatFirmwareValidationStatus,
  formatMetadataSource,
  formatPlaytime,
  formatRuntimeVerificationFeature,
  formatRuntimeVerificationStatus,
  formatVerificationLevel,
} from '../utils/formatters';
import envStyles from './ConsoleEnvironmentPage.module.css';
import styles from './Pages.module.css';

type EnvironmentTab = 'games' | 'scanner' | 'emulator' | 'settings';

const SEVEN_VERIFICATION_FEATURES: RuntimeVerificationFeature[] = [
  'emulator_launch',
  'system_boot',
  'original_dashboard',
  'game_boot',
  'per_game_compatibility',
  'session_monitoring',
  'hub_return',
];

export interface ConsoleEnvironmentPageProps {
  environment: ConsoleEnvironmentDto;
  onBack: () => void;
  onOpenEmulators: () => void;
  onOpenGame: (gameId: string) => void;
  onToggleFavorite: (gameId: string) => Promise<void>;
  onAddScanPath: (consoleSlug: string, directoryPath: string, recursive: boolean) => Promise<void>;
  onRemoveScanPath: (pathId: string) => Promise<void>;
  onScanLibrary: (
    consoleSlug: string,
    onProgress?: (event: ScanProgressEvent) => void
  ) => Promise<ScanSummaryDto>;
  onUpdateConsoleSetting: (consoleSlug: string, key: string, value: string) => Promise<void>;
}

export function ConsoleEnvironmentPage({
  environment,
  onBack,
  onOpenEmulators,
  onOpenGame,
  onToggleFavorite,
  onAddScanPath,
  onRemoveScanPath,
  onScanLibrary,
  onUpdateConsoleSetting,
}: ConsoleEnvironmentPageProps) {
  const {
    console: consoleRecord,
    emulator,
    environmentMode,
    games,
    scanPaths,
    settings,
    firmwareRecords,
    runtimeVerifications,
    rpcs3Inspection,
  } = environment;

  const [tab, setTab] = useState<EnvironmentTab>('games');
  const [search, setSearch] = useState('');
  const [newScanDir, setNewScanDir] = useState('');
  const [recursive, setRecursive] = useState(true);
  const [pendingRemovePathId, setPendingRemovePathId] = useState<string | null>(null);
  const [scanning, setScanning] = useState(false);
  const [progressText, setProgressText] = useState<string | null>(null);
  const [scanSummary, setScanSummary] = useState<ScanSummaryDto | null>(null);
  const [feedback, setFeedback] = useState<{ kind: 'ok' | 'err'; text: string } | null>(null);
  const [busy, setBusy] = useState(false);

  const settingMap = new Map(settings.map((s) => [s.key, s.value]));
  const [preferredBootMode, setPreferredBootMode] = useState(
    settingMap.get('preferred_boot_mode') ??
      (environmentMode === 'authentic_system_capable' ? 'system_boot' : 'direct_game_boot')
  );
  const [consoleFullscreen, setConsoleFullscreen] = useState(
    settingMap.get('fullscreen') ?? 'true'
  );
  const [consoleNotes, setConsoleNotes] = useState(settingMap.get('custom_notes') ?? '');

  const filteredGames = games.filter((g) => {
    const q = search.trim().toLowerCase();
    return q.length === 0 || g.title.toLowerCase().includes(q);
  });

  const handleAddScanPath = async (event: FormEvent) => {
    event.preventDefault();
    setFeedback(null);
    setBusy(true);
    try {
      await onAddScanPath(consoleRecord.slug, newScanDir, recursive);
      setNewScanDir('');
      setFeedback({ kind: 'ok', text: 'Pasta de descoberta adicionada a este console.' });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setBusy(false);
    }
  };

  const handleConfirmRemovePath = async () => {
    if (!pendingRemovePathId) return;
    setBusy(true);
    setFeedback(null);
    try {
      await onRemoveScanPath(pendingRemovePathId);
      setPendingRemovePathId(null);
      setFeedback({ kind: 'ok', text: 'Pasta removida do scanner deste console.' });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setBusy(false);
    }
  };

  const handleScan = async () => {
    setFeedback(null);
    setScanSummary(null);
    setScanning(true);
    setProgressText('Iniciando varredura local...');
    try {
      const summary = await onScanLibrary(consoleRecord.slug, (evt) => {
        if (evt.event === 'started') {
          setProgressText(`Varrendo ${evt.data.totalDirectories} pasta(s) configurada(s)...`);
        } else if (evt.event === 'scanningDirectory') {
          setProgressText(
            `Verificando ${evt.data.directoryPath} (${evt.data.filesChecked} arquivos analisados)...`
          );
        } else if (evt.event === 'gameDiscovered') {
          setProgressText(`Título identificado: ${evt.data.title}`);
        } else if (evt.event === 'completed') {
          setProgressText(
            `Concluído: ${evt.data.gamesAdded} novos jogos, ${evt.data.versionsAdded} versões adicionadas, ${evt.data.skippedExisting} já existentes.`
          );
        } else if (evt.event === 'error') {
          setProgressText(`Aviso: ${evt.data.message}`);
        }
      });
      setScanSummary(summary);
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setScanning(false);
    }
  };

  const handleSaveConsoleSettings = async (event: FormEvent) => {
    event.preventDefault();
    setFeedback(null);
    setBusy(true);
    try {
      await onUpdateConsoleSetting(consoleRecord.slug, 'preferred_boot_mode', preferredBootMode);
      await onUpdateConsoleSetting(consoleRecord.slug, 'fullscreen', consoleFullscreen);
      await onUpdateConsoleSetting(consoleRecord.slug, 'custom_notes', consoleNotes);
      setFeedback({ kind: 'ok', text: 'Configurações independentes do console salvas no SQLite.' });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setBusy(false);
    }
  };

  const targetPhaseLabel = consoleRecord.slug === 'ps3' ? 'Fase 3' : 'Fase 5';

  return (
    <div className={styles.page}>
      <section className={envStyles.envHeader} aria-label={`Ambiente de ${consoleRecord.name}`}>
        <div className={envStyles.envHeaderTop}>
          <div>
            <h1 className={styles.title}>{consoleRecord.name}</h1>
            <p className={styles.subtitle}>
              {consoleRecord.manufacturer} · Geração {consoleRecord.generation} ·{' '}
              {consoleRecord.releaseYear}
              {emulator ? ` · Emulador: ${emulator.name}` : ''}
            </p>
          </div>
          <div className={styles.actionsRow}>
            <button type="button" className={styles.btnSecondary} onClick={onBack}>
              Voltar ao Hub
            </button>
            <button type="button" className={styles.btnPrimary} onClick={onOpenEmulators}>
              Gerenciar Emulador &amp; Firmware
            </button>
          </div>
        </div>

        <div className={styles.badgeRow}>
          {consoleRecord.capabilities.supportedStates.map((state) => (
            <CapabilityBadge key={state} state={state} />
          ))}
        </div>

        {environmentMode === 'integrated_alternative_only' ? (
          <div className={envStyles.altBanner} role="note">
            <strong>
              Ambiente Integrado do Hub — Este não é o dashboard original do console
            </strong>
            <span>
              Este console não possui boot completo estável do sistema original neste emulador. O
              Hub oferece uma interface integrada própria para organizar e iniciar títulos
              compatíveis diretamente ({targetPhaseLabel}), sem simular a interface da fabricante.
            </span>
          </div>
        ) : (
          <div className={`${styles.banner} ${styles.bannerInfo}`} role="note">
            Ambiente independente pronto para biblioteca, pastas e configurações próprias. A
            inicialização ao vivo do sistema original e execução direta de jogos entram na{' '}
            <strong>{targetPhaseLabel}</strong> — nenhum botão simulado de boot é exibido na Fase 2.
          </div>
        )}
      </section>

      <div className={envStyles.tabBar} role="tablist" aria-label="Abas do ambiente do console">
        <button
          type="button"
          role="tab"
          aria-selected={tab === 'games'}
          className={`${envStyles.tabBtn} ${tab === 'games' ? envStyles.tabBtnActive : ''}`}
          onClick={() => setTab('games')}
        >
          Jogos do Console ({games.length})
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={tab === 'scanner'}
          className={`${envStyles.tabBtn} ${tab === 'scanner' ? envStyles.tabBtnActive : ''}`}
          onClick={() => setTab('scanner')}
        >
          Pastas &amp; Descoberta ({scanPaths.length})
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={tab === 'emulator'}
          className={`${envStyles.tabBtn} ${tab === 'emulator' ? envStyles.tabBtnActive : ''}`}
          onClick={() => setTab('emulator')}
        >
          Emulador &amp; Verificações
        </button>
        <button
          type="button"
          role="tab"
          aria-selected={tab === 'settings'}
          className={`${envStyles.tabBtn} ${tab === 'settings' ? envStyles.tabBtnActive : ''}`}
          onClick={() => setTab('settings')}
        >
          Configurações do Console
        </button>
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

      {tab === 'games' ? (
        <section className={styles.section}>
          <div className={styles.header}>
            <h2 className={styles.sectionTitle}>Biblioteca Isolada de {consoleRecord.name}</h2>
            <input
              className={styles.input}
              aria-label="Filtrar jogos deste console"
              placeholder="Buscar neste console..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>

          {filteredGames.length === 0 ? (
            <p className={styles.muted}>
              Nenhum jogo registrado para {consoleRecord.name} ainda. Configure uma pasta na aba
              Pastas &amp; Descoberta ou registre manualmente na Biblioteca Universal.
            </p>
          ) : (
            <div className={envStyles.gameGrid}>
              {filteredGames.map((game) => {
                const coverUrl = resolveCoverAssetUrl(game.coverPath);
                return (
                  <article key={game.id} className={envStyles.gameCard}>
                    <div className={envStyles.coverSlot}>
                      {coverUrl ? (
                        <img
                          src={coverUrl}
                          alt={`Capa de ${game.title}`}
                          className={envStyles.coverImg}
                        />
                      ) : (
                        <span>{consoleRecord.slug.toUpperCase()} · Sem capa</span>
                      )}
                    </div>
                    <div className={envStyles.gameCardBody}>
                      <h3 className={envStyles.gameTitle}>{game.title}</h3>
                      <div className={envStyles.gameMeta}>
                        {game.versionCount} versão(ões) · {formatPlaytime(game.totalPlaytimeSeconds)}{' '}
                        · {game.completionPercent}%
                      </div>
                      <div className={envStyles.gameMeta}>
                        Origem: {formatMetadataSource(game.metadataSource)}
                      </div>
                      <div className={styles.actionsRow}>
                        <button
                          type="button"
                          className={styles.btnPrimary}
                          onClick={() => onOpenGame(game.id)}
                        >
                          Detalhes do Jogo
                        </button>
                        <button
                          type="button"
                          className={styles.btnSecondary}
                          onClick={() => void onToggleFavorite(game.id)}
                        >
                          {game.isFavorite ? 'Favorito' : 'Favoritar'}
                        </button>
                      </div>
                    </div>
                  </article>
                );
              })}
            </div>
          )}
        </section>
      ) : null}

      {tab === 'scanner' ? (
        <section className={styles.section}>
          <h2 className={styles.sectionTitle}>
            Pastas de Descoberta Local ({consoleRecord.name})
          </h2>
          <p className={styles.muted}>
            {consoleRecord.slug === 'ps3'
              ? 'O scanner PS3 lê metadados reais do cabeçalho binário PARAM.SFO ao lado de USRDIR/EBOOT.BIN de forma segura e idempotente.'
              : 'O scanner identifica arquivos compatíveis com este console e deduplica entradas existentes automaticamente.'}
          </p>

          <form onSubmit={handleAddScanPath} className={styles.formGrid}>
            <div className={styles.field}>
              <label className={styles.label} htmlFor="console-scan-dir">
                Diretório local de jogos ({consoleRecord.slug.toUpperCase()})
              </label>
              <FilePathField
                id="console-scan-dir"
                mode="directory"
                dialogTitle={`Selecionar pasta de jogos ${consoleRecord.name}`}
                value={newScanDir}
                onChange={setNewScanDir}
                placeholder={`D:\\Jogos\\${consoleRecord.slug.toUpperCase()}`}
                required
                disabled={busy}
              />
            </div>

            <div className={styles.actionsRow} style={{ alignSelf: 'end' }}>
              <label className={envStyles.checkboxRow}>
                <input
                  type="checkbox"
                  checked={recursive}
                  onChange={(e) => setRecursive(e.target.checked)}
                />
                <span>Subpastas recursivas</span>
              </label>
              <button type="submit" className={styles.btnPrimary} disabled={busy}>
                Adicionar Pasta
              </button>
              <button
                type="button"
                className={styles.btnSecondary}
                disabled={scanning || scanPaths.length === 0}
                onClick={() => void handleScan()}
              >
                {scanning ? 'Escaneando...' : 'Escanear Biblioteca'}
              </button>
            </div>
          </form>

          {progressText ? (
            <div className={envStyles.progressBox} role="status" aria-live="polite">
              <strong>Progresso do Scanner</strong>
              <span>{progressText}</span>
            </div>
          ) : null}

          {scanSummary ? (
            <div className={`${styles.banner} ${styles.bannerSuccess}`} role="status">
              Varredura concluída: {scanSummary.scannedFiles} arquivos verificados ·{' '}
              {scanSummary.gamesAdded} novos jogos · {scanSummary.versionsAdded} versões adicionadas
              · {scanSummary.skippedExisting} já existentes
              {scanSummary.errors.length > 0
                ? ` · ${scanSummary.errors.length} aviso(s) ignorado(s) com segurança`
                : ''}
              .
            </div>
          ) : null}

          {scanPaths.length === 0 ? (
            <p className={styles.muted}>Nenhuma pasta configurada para este console ainda.</p>
          ) : (
            <ul className={styles.list}>
              {scanPaths.map((sp) => (
                <li key={sp.id} className={styles.listItem}>
                  <div>
                    <div className={styles.mono}>{sp.directoryPath}</div>
                    <div className={styles.muted}>
                      {sp.recursive ? 'Recursivo' : 'Somente raiz'} · Último scan:{' '}
                      {sp.lastScannedAt ?? 'Nunca escaneado'}
                    </div>
                  </div>
                  <button
                    type="button"
                    className={styles.btnSecondary}
                    disabled={busy}
                    onClick={() => setPendingRemovePathId(sp.id)}
                  >
                    Remover Pasta
                  </button>
                </li>
              ))}
            </ul>
          )}
        </section>
      ) : null}

      {tab === 'emulator' ? (
        <>
          <section className={styles.section}>
            <h2 className={styles.sectionTitle}>Matriz de Capacidades por Funcionalidade</h2>
            <div className={styles.tableWrap}>
              <table className={styles.table}>
                <thead>
                  <tr>
                    <th scope="col">Funcionalidade</th>
                    <th scope="col">Estados</th>
                    <th scope="col">Disponibilidade</th>
                    <th scope="col">Nível de Verificação</th>
                    <th scope="col">Observações Reais</th>
                  </tr>
                </thead>
                <tbody>
                  {consoleRecord.capabilities.features.map((feat) => (
                    <tr key={feat.feature}>
                      <td>{formatFeatureKey(feat.feature)}</td>
                      <td>
                        <div className={styles.badgeRow}>
                          {feat.states.map((s) => (
                            <CapabilityBadge key={s} state={s} />
                          ))}
                        </div>
                      </td>
                      <td>{formatAvailabilityStatus(feat.availability)}</td>
                      <td>{formatVerificationLevel(feat.verification)}</td>
                      <td>{feat.notes}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </section>

          <section className={styles.section}>
            <h2 className={styles.sectionTitle}>
              7 Estados Independentes de Verificação Runtime
            </h2>
            <p className={styles.muted}>
              Validar uma etapa jamais marca outra automaticamente. Compatibilidade de jogos varia
              individualmente por título e build do emulador.
            </p>
            <div className={styles.tableWrap}>
              <table className={styles.table}>
                <thead>
                  <tr>
                    <th scope="col">Etapa Independente</th>
                    <th scope="col">Estado Atual</th>
                    <th scope="col">Versão / Evidência</th>
                  </tr>
                </thead>
                <tbody>
                  {SEVEN_VERIFICATION_FEATURES.map((featKey) => {
                    const latest = runtimeVerifications.find((rv) => rv.featureKey === featKey);
                    return (
                      <tr key={featKey}>
                        <td>{formatRuntimeVerificationFeature(featKey)}</td>
                        <td>
                          {latest
                            ? formatRuntimeVerificationStatus(latest.status)
                            : 'Não verificado'}
                        </td>
                        <td className={styles.muted}>
                          {latest
                            ? `${latest.emulatorVersion} · ${latest.evidenceNotes}`
                            : 'Aguardando validação prática na etapa correspondente'}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </section>

          {rpcs3Inspection ? (
            <section className={styles.section}>
              <h2 className={styles.sectionTitle}>
                Preparação RPCS3 (Execução Real na Fase 3)
              </h2>
              <p className={styles.muted}>{rpcs3Inspection.diagnostic}</p>
              {rpcs3Inspection.readiness ? (
                <ReadinessTracker readiness={rpcs3Inspection.readiness} />
              ) : null}
            </section>
          ) : null}

          <section className={styles.section}>
            <h2 className={styles.sectionTitle}>Arquivos de Sistema Validados Localmente</h2>
            {firmwareRecords.length === 0 ? (
              <p className={styles.muted}>
                Nenhum arquivo de firmware/BIOS validado para este console.
              </p>
            ) : (
              <ul className={styles.list}>
                {firmwareRecords.map((fw) => (
                  <li key={fw.id} className={styles.listItem}>
                    <div>
                      <div className={styles.mono}>{fw.filePath}</div>
                      <div className={styles.muted}>
                        Status: {formatFirmwareValidationStatus(fw.validationStatus)} · SHA-256:{' '}
                        {fw.sha256.slice(0, 16)}…
                      </div>
                    </div>
                    <span className={styles.mono}>{fw.validatedAt}</span>
                  </li>
                ))}
              </ul>
            )}
          </section>
        </>
      ) : null}

      {tab === 'settings' ? (
        <section className={styles.section}>
          <h2 className={styles.sectionTitle}>
            Configurações Independentes de {consoleRecord.name}
          </h2>
          <form onSubmit={handleSaveConsoleSettings} className={styles.formGrid}>
            <div className={styles.field}>
              <label className={styles.label} htmlFor="console-boot-mode">
                Modo de inicialização preferido ({targetPhaseLabel})
              </label>
              <select
                id="console-boot-mode"
                className={styles.select}
                value={preferredBootMode}
                onChange={(e) => setPreferredBootMode(e.target.value)}
              >
                {environmentMode === 'authentic_system_capable' ? (
                  <option value="system_boot">Sistema / Dashboard Original (quando pronto)</option>
                ) : null}
                <option value="direct_game_boot">Execução Direta de Jogo</option>
              </select>
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="console-fullscreen">
                Tela cheia ao iniciar emulador
              </label>
              <select
                id="console-fullscreen"
                className={styles.select}
                value={consoleFullscreen}
                onChange={(e) => setConsoleFullscreen(e.target.value)}
              >
                <option value="true">Ativado</option>
                <option value="false">Janela</option>
              </select>
            </div>

            <div className={styles.field} style={{ gridColumn: '1 / -1' }}>
              <label className={styles.label} htmlFor="console-custom-notes">
                Anotações pessoais para este console
              </label>
              <textarea
                id="console-custom-notes"
                className={styles.textarea}
                rows={3}
                value={consoleNotes}
                onChange={(e) => setConsoleNotes(e.target.value)}
                placeholder="Ex.: Perfil DualSense configurado, versão testada do emulador..."
              />
            </div>

            <div className={styles.actionsRow}>
              <button type="submit" className={styles.btnPrimary} disabled={busy}>
                Salvar Configurações do Console
              </button>
            </div>
          </form>
        </section>
      ) : null}

      <ConfirmDialog
        open={pendingRemovePathId !== null}
        title="Remover pasta de descoberta?"
        confirmLabel="Remover Pasta"
        danger
        busy={busy}
        onConfirm={() => void handleConfirmRemovePath()}
        onCancel={() => setPendingRemovePathId(null)}
      >
        <p>
          Esta ação remove apenas o diretório da lista de varredura deste console. Nenhum arquivo de
          jogo no disco será apagado.
        </p>
      </ConfirmDialog>
    </div>
  );
}
