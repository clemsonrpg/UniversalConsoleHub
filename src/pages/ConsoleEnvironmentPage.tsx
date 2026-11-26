import { useState, type FormEvent } from 'react';
import { Play, RefreshCw, Tv } from 'lucide-react';
import { resolveCoverAssetUrl } from '../api/tauriClient';
import { DataOriginBadge } from '../components/ActiveSessionBanner';
import { CapabilityBadge } from '../components/CapabilityBadge';
import { ConfirmDialog } from '../components/ConfirmDialog';
import { FilePathField } from '../components/FilePathField';
import { ReadinessTracker } from '../components/ReadinessTracker';
import type {
  ActiveSessionSnapshot,
  ConsoleEnvironmentDto,
  ConsoleProfileOverviewDto,
  ConsoleSyncReport,
  RecordRuntimeVerificationRequest,
  RuntimeVerificationFeature,
  ScanProgressEvent,
  ScanSummaryDto,
  TrophyCenterOverviewDto,
  TrophySyncReport,
} from '../types/domain';
import {
  formatAvailabilityStatus,
  formatEmulatorStatus,
  formatFeatureKey,
  formatFirmwareValidationStatus,
  formatGameAvailability,
  formatMetadataSource,
  formatPlaytime,
  formatRuntimeVerificationFeature,
  formatRuntimeVerificationStatus,
  formatSessionExitStatus,
  formatSyncStatus,
  formatTrophyReadiness,
  formatVerificationLevel,
} from '../utils/formatters';
import envStyles from './ConsoleEnvironmentPage.module.css';
import styles from './Pages.module.css';

type EnvironmentTab = 'profile' | 'games' | 'scanner' | 'emulator' | 'settings';

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
  consoleProfile?: ConsoleProfileOverviewDto | null;
  consoleTrophyOverview?: TrophyCenterOverviewDto | null;
  activeSession?: ActiveSessionSnapshot | null;
  onBack: () => void;
  onOpenEmulators: () => void;
  onOpenTrophies?: () => void;
  onOpenGame: (gameId: string) => void;
  onToggleFavorite: (gameId: string) => Promise<void>;
  onAddScanPath: (consoleSlug: string, directoryPath: string, recursive: boolean) => Promise<void>;
  onRemoveScanPath: (pathId: string) => Promise<void>;
  onScanLibrary: (
    consoleSlug: string,
    onProgress?: (event: ScanProgressEvent) => void
  ) => Promise<ScanSummaryDto>;
  onUpdateConsoleSetting: (consoleSlug: string, key: string, value: string) => Promise<void>;
  onSyncProfile?: (consoleSlug: string) => Promise<ConsoleSyncReport>;
  onSyncConsoleTrophies?: (consoleSlug: string) => Promise<TrophySyncReport>;
  onLaunchSystem?: (consoleSlug: string) => Promise<void>;
  onLaunchGame?: (gameId: string, versionId?: string | null) => Promise<void>;
  onRecordVerification?: (request: RecordRuntimeVerificationRequest) => Promise<void>;
}

export function ConsoleEnvironmentPage({
  environment,
  consoleProfile,
  consoleTrophyOverview,
  activeSession,
  onBack,
  onOpenEmulators,
  onOpenTrophies,
  onOpenGame,
  onToggleFavorite,
  onAddScanPath,
  onRemoveScanPath,
  onScanLibrary,
  onUpdateConsoleSetting,
  onSyncProfile,
  onSyncConsoleTrophies,
  onLaunchSystem,
  onLaunchGame,
  onRecordVerification,
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
  const [syncing, setSyncing] = useState(false);
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

  const sessionBusy = Boolean(activeSession && activeSession.phase !== 'idle');
  const isPs3Playable = consoleRecord.slug === 'ps3';
  const rpcs3Configured = Boolean(rpcs3Inspection?.configured && rpcs3Inspection.installation?.isValid);
  const ps3XmbReady = Boolean(
    isPs3Playable &&
      rpcs3Inspection?.readiness?.systemFilesPresent &&
      rpcs3Inspection?.readiness?.commandBuildable
  );

  const filteredGames = games.filter((g) => {
    const q = search.trim().toLowerCase();
    return q.length === 0 || g.title.toLowerCase().includes(q);
  });

  const handleSyncNow = async () => {
    if (!onSyncProfile) return;
    setFeedback(null);
    setSyncing(true);
    try {
      const report = await onSyncProfile(consoleRecord.slug);
      setFeedback({
        kind: 'ok',
        text: `Perfil sincronizado: ${report.versionsAvailable} disponível(is), ${report.versionsMarkedMissing} ausente(s), ${report.localUsersDetected} usuário(s) local(is) real(is), ${report.coversImported} capa(s) nativa(s) importada(s).`,
      });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setSyncing(false);
    }
  };

  const handleLaunchSystem = async () => {
    if (!onLaunchSystem) return;
    setFeedback(null);
    setBusy(true);
    try {
      await onLaunchSystem(consoleRecord.slug);
      setFeedback({
        kind: 'ok',
        text: 'RPCS3 iniciado em modo PS3 XMB (vsh.self). O Hub restaurará o foco automaticamente ao encerrar.',
      });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setBusy(false);
    }
  };

  const handleLaunchGame = async (gameId: string, title: string) => {
    if (!onLaunchGame) return;
    setFeedback(null);
    setBusy(true);
    try {
      await onLaunchGame(gameId, null);
      setFeedback({
        kind: 'ok',
        text: `Executando '${title}' via RPCS3 com monitoramento de sessão ativo.`,
      });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setBusy(false);
    }
  };

  const handleRecordXmbVerification = async (status: 'runtime_confirmed' | 'failed') => {
    if (!onRecordVerification || !emulator) return;
    setFeedback(null);
    setBusy(true);
    try {
      const ver =
        rpcs3Inspection?.installation?.detectedVersion ?? emulator.detectedVersion ?? 'unknown';
      await onRecordVerification({
        consoleSlug: consoleRecord.slug,
        emulatorSlug: emulator.slug,
        gameId: null,
        featureKey: 'original_dashboard',
        emulatorVersion: ver,
        status,
        evidenceNotes:
          status === 'runtime_confirmed'
            ? 'PS3 XMB original confirmado manualmente em execução pelo usuário.'
            : 'Falha reportada pelo usuário durante tentativa de boot do PS3 XMB.',
      });
      setFeedback({
        kind: 'ok',
        text:
          status === 'runtime_confirmed'
            ? 'Verificação runtime do PS3 XMB registrada com sucesso.'
            : 'Falha de verificação runtime do PS3 XMB registrada.',
      });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setBusy(false);
    }
  };

  const handleAddScanPath = async (event: FormEvent) => {
    event.preventDefault();
    setFeedback(null);
    setBusy(true);
    try {
      await onAddScanPath(consoleRecord.slug, newScanDir, recursive);
      setNewScanDir('');
      setFeedback({
        kind: 'ok',
        text: 'Pasta adicionada e sincronização automática do perfil executada.',
      });
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

  const targetPhaseLabel = isPs3Playable ? 'Fase 3' : 'Fase 5';

  return (
    <div className={styles.page}>
      <section className={envStyles.envHeader} aria-label={`Perfil de ${consoleRecord.name}`}>
        <div className={envStyles.envHeaderTop}>
          <div>
            <h1 className={styles.title}>{consoleRecord.name}</h1>
            <p className={styles.subtitle}>
              {consoleRecord.manufacturer} · Geração {consoleRecord.generation} ·{' '}
              {consoleRecord.releaseYear}
              {emulator ? ` · Emulador: ${emulator.name}` : ''}
              {consoleProfile?.profile?.lastSyncedAt
                ? ` · Última sincronização: ${consoleProfile.profile.lastSyncedAt}`
                : ''}
            </p>
          </div>
          <div className={styles.actionsRow}>
            {isPs3Playable && environmentMode === 'authentic_system_capable' && onLaunchSystem ? (
              <button
                type="button"
                className={styles.btnPrimary}
                data-focus-default={ps3XmbReady ? 'true' : undefined}
                disabled={!ps3XmbReady || busy || sessionBusy}
                title={
                  ps3XmbReady
                    ? 'Iniciar o sistema original PS3 XMB via RPCS3 (vsh.self)'
                    : 'Configure rpcs3.exe e instale PS3UPDAT.PUP (dev_flash/vsh/module/vsh.self) para habilitar o boot do XMB'
                }
                onClick={() => void handleLaunchSystem()}
              >
                <Tv size={14} aria-hidden="true" style={{ marginRight: 6, verticalAlign: 'middle' }} />
                Iniciar PS3 XMB
              </button>
            ) : null}
            {onSyncProfile ? (
              <button
                type="button"
                className={styles.btnSecondary}
                data-focus-default={!ps3XmbReady ? 'true' : undefined}
                disabled={syncing || busy}
                onClick={() => void handleSyncNow()}
              >
                <RefreshCw
                  size={14}
                  aria-hidden="true"
                  style={{ marginRight: 6, verticalAlign: 'middle' }}
                />
                {syncing ? 'Sincronizando...' : 'Sincronizar Agora'}
              </button>
            ) : null}
            <button type="button" className={styles.btnSecondary} onClick={onOpenEmulators}>
              Emulador &amp; Firmware
            </button>
            <button type="button" className={styles.btnSecondary} onClick={onBack}>
              Voltar ao Hub
            </button>
          </div>
        </div>

        <div className={styles.badgeRow}>
          {consoleRecord.capabilities.supportedStates.map((state) => (
            <CapabilityBadge key={state} state={state} />
          ))}
        </div>

        {consoleProfile ? (
          <div
            className={envStyles.profileMetricsGrid}
            aria-label="Resumo verificável do perfil do console"
          >
            <div className={envStyles.metricTile}>
              <div className={envStyles.metricHeader}>
                <span>Emulador</span>
                <DataOriginBadge origin={consoleProfile.emulatorStatus.origin} />
              </div>
              <div className={envStyles.metricValue}>
                {consoleProfile.emulatorStatus.value
                  ? formatEmulatorStatus(consoleProfile.emulatorStatus.value)
                  : 'Não configurado'}
              </div>
              <div className={envStyles.metricNote}>
                {consoleProfile.emulatorVersion.value ??
                  consoleProfile.emulatorVersion.notes ??
                  'Versão não detectada'}
              </div>
            </div>

            <div className={envStyles.metricTile}>
              <div className={envStyles.metricHeader}>
                <span>Firmware / Sistema</span>
                <DataOriginBadge origin={consoleProfile.firmwareVersion.origin} />
              </div>
              <div className={envStyles.metricValue}>
                {consoleProfile.firmwareVersion.value ?? 'Não detectado'}
              </div>
              <div className={envStyles.metricNote}>
                {consoleProfile.firmwareVersion.notes ?? 'Lido de arquivos reais do sistema'}
              </div>
            </div>

            <div className={envStyles.metricTile}>
              <div className={envStyles.metricHeader}>
                <span>Saves Nativos</span>
                <DataOriginBadge origin={consoleProfile.nativeSaveCount.origin} />
              </div>
              <div className={envStyles.metricValue}>
                {consoleProfile.nativeSaveCount.value !== null
                  ? `${consoleProfile.nativeSaveCount.value} diretório(s)`
                  : 'Indisponível'}
              </div>
              <div className={envStyles.metricNote}>
                {consoleProfile.trophyReadiness.value
                  ? `Troféus: ${formatTrophyReadiness(consoleProfile.trophyReadiness.value)}`
                  : 'Troféus: Indisponível'}
              </div>
            </div>

            <div className={envStyles.metricTile}>
              <div className={envStyles.metricHeader}>
                <span>Tempo &amp; Sessões</span>
                <DataOriginBadge origin="hub_recorded" />
              </div>
              <div className={envStyles.metricValue}>
                {formatPlaytime(consoleProfile.hubStats.totalPlaytimeSeconds)} ·{' '}
                {consoleProfile.hubStats.totalSessions} sessão(ões)
              </div>
              <div className={envStyles.metricNote}>
                {consoleProfile.hubStats.gamesAvailable} disponível(is) ·{' '}
                {consoleProfile.hubStats.gamesMissingOnDisk} ausente(s) no disco
              </div>
            </div>
          </div>
        ) : null}

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
        ) : isPs3Playable ? (
          <div className={`${styles.banner} ${styles.bannerInfo}`} role="note">
            {ps3XmbReady
              ? 'PlayStation 3 pronto para execução real do PS3 XMB (vsh.self) e boot direto de jogos (EBOOT.BIN) via RPCS3 com monitoramento de sessão e retorno automático ao Hub.'
              : 'Ambiente PlayStation 3 ativo (Fase 3). Configure rpcs3.exe e instale seu PS3UPDAT.PUP legal para habilitar o boot do PS3 XMB; jogos válidos podem ser iniciados diretamente assim que o rpcs3.exe estiver configurado.'}
          </div>
        ) : (
          <div className={`${styles.banner} ${styles.bannerInfo}`} role="note">
            Ambiente independente pronto para biblioteca, pastas e configurações próprias. A
            inicialização ao vivo deste console entra na <strong>Fase 5</strong> — nenhum botão
            simulado de boot é exibido antes da integração real.
          </div>
        )}
      </section>

      <div className={envStyles.tabBar} role="tablist" aria-label="Abas do perfil do console">
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
          aria-selected={tab === 'profile'}
          className={`${envStyles.tabBtn} ${tab === 'profile' ? envStyles.tabBtnActive : ''}`}
          onClick={() => setTab('profile')}
        >
          Perfil &amp; Usuários Locais
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

      {tab === 'profile' ? (
        <>
          <section className={styles.section}>
            <div className={styles.header}>
              <h2 className={styles.sectionTitle}>
                Usuários Locais Reais do Console ({consoleProfile?.localUsers.length ?? 0})
              </h2>
              <DataOriginBadge
                origin={
                  consoleProfile && consoleProfile.localUsers.length > 0
                    ? 'native_platform'
                    : 'unavailable'
                }
              />
            </div>
            <p className={styles.muted}>
              A identidade do console é mantida separada da identidade de usuários locais. Apenas
              usuários reais detectados nos arquivos da plataforma são exibidos — o Hub jamais
              inventa usuários quando a plataforma não fornece identificação.
            </p>
            {!consoleProfile || consoleProfile.localUsers.length === 0 ? (
              <p className={styles.muted}>
                Nenhum usuário local nativo detectado para {consoleRecord.name}.
              </p>
            ) : (
              <ul className={styles.list}>
                {consoleProfile.localUsers.map((user) => (
                  <li key={user.id} className={styles.listItem}>
                    <div>
                      <strong>{user.displayName}</strong>{' '}
                      <span className={styles.mono}>({user.nativeUserKey})</span>
                      <div className={styles.muted}>
                        Caminho real: <span className={styles.mono}>{user.sourcePath}</span> ·
                        Visto em: {user.lastSeenAt}
                      </div>
                    </div>
                    {user.availabilityStatus === 'missing_on_disk' ? (
                      <span className={envStyles.missingBadge}>Ausente no disco</span>
                    ) : (
                      <DataOriginBadge origin="native_platform" />
                    )}
                  </li>
                ))}
              </ul>
            )}
          </section>

          <section className={styles.section}>
            <div className={styles.header}>
              <h2 className={styles.sectionTitle}>Conquistas &amp; Progresso Nativo</h2>
              <DataOriginBadge
                origin={consoleProfile?.trophyReadiness.origin ?? 'unavailable'}
              />
            </div>
            <p className={styles.muted}>
              {consoleProfile?.trophyReadiness.notes ??
                'Sincronize o perfil para verificar a disponibilidade nativa de troféus desta plataforma.'}
            </p>
            {isPs3Playable && consoleTrophyOverview ? (
              <div className={envStyles.progressBox}>
                <strong>
                  Troféus PS3 (RPCS3 local): {consoleTrophyOverview.totals.unlockedTrophies} /{' '}
                  {consoleTrophyOverview.totals.totalTrophies} (
                  {consoleTrophyOverview.totals.completionPercent}%)
                </strong>
                <span>
                  Platina: {consoleTrophyOverview.totals.platinumUnlocked} · Ouro:{' '}
                  {consoleTrophyOverview.totals.goldUnlocked} · Prata:{' '}
                  {consoleTrophyOverview.totals.silverUnlocked} · Bronze:{' '}
                  {consoleTrophyOverview.totals.bronzeUnlocked} · Conjuntos:{' '}
                  {consoleTrophyOverview.totals.totalSets}
                </span>
              </div>
            ) : null}
            {isPs3Playable && (onSyncConsoleTrophies || onOpenTrophies) ? (
              <div className={styles.actionsRow}>
                {onSyncConsoleTrophies ? (
                  <button
                    type="button"
                    className={styles.btnSecondary}
                    disabled={syncing || busy}
                    onClick={() => {
                      setBusy(true);
                      setFeedback(null);
                      void onSyncConsoleTrophies(consoleRecord.slug)
                        .then((rep) => {
                          setFeedback({
                            kind: 'ok',
                            text: `Troféus PS3 sincronizados: ${rep.setsSynced} conjunto(s), ${rep.trophiesSynced} troféu(s), ${rep.newlyUnlocked.length} novo(s) desbloqueio(s).`,
                          });
                        })
                        .catch((err) => setFeedback({ kind: 'err', text: String(err) }))
                        .finally(() => setBusy(false));
                    }}
                  >
                    Sincronizar Troféus RPCS3
                  </button>
                ) : null}
                {onOpenTrophies ? (
                  <button type="button" className={styles.btnSecondary} onClick={onOpenTrophies}>
                    Abrir Central de Troféus
                  </button>
                ) : null}
              </div>
            ) : null}
          </section>

          <section className={styles.section}>
            <div className={styles.header}>
              <h2 className={styles.sectionTitle}>Mais Jogados em {consoleRecord.name}</h2>
              <DataOriginBadge origin="hub_recorded" />
            </div>
            {!consoleProfile || consoleProfile.hubStats.mostPlayed.length === 0 ? (
              <p className={styles.muted}>
                Nenhuma sessão concluída registrada neste console ainda.
              </p>
            ) : (
              <ul className={styles.list}>
                {consoleProfile.hubStats.mostPlayed.map((item) => (
                  <li key={item.gameId} className={styles.listItem}>
                    <div>
                      <strong>{item.title}</strong>
                      <div className={styles.muted}>
                        {item.sessionsCount} sessão(ões) ·{' '}
                        {formatPlaytime(item.totalPlaytimeSeconds)} · {item.completionPercent}%
                      </div>
                    </div>
                    <button
                      type="button"
                      className={styles.btnSecondary}
                      onClick={() => onOpenGame(item.gameId)}
                    >
                      Detalhes
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </section>

          <section className={styles.section}>
            <div className={styles.header}>
              <h2 className={styles.sectionTitle}>Histórico Recente de Sessões</h2>
              <DataOriginBadge origin="hub_recorded" />
            </div>
            {!consoleProfile || consoleProfile.recentSessions.length === 0 ? (
              <p className={styles.muted}>Nenhuma sessão registrada para este console.</p>
            ) : (
              <div className={styles.tableWrap}>
                <table className={styles.table}>
                  <thead>
                    <tr>
                      <th scope="col">Jogo</th>
                      <th scope="col">Início (UTC)</th>
                      <th scope="col">Duração</th>
                      <th scope="col">Status</th>
                    </tr>
                  </thead>
                  <tbody>
                    {consoleProfile.recentSessions.map((sess) => (
                      <tr key={sess.id}>
                        <td>{sess.gameTitle}</td>
                        <td className={styles.mono}>{sess.startedAt}</td>
                        <td>{formatPlaytime(sess.durationSeconds)}</td>
                        <td>{formatSessionExitStatus(sess.exitStatus)}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </section>

          {consoleProfile?.profile ? (
            <section className={styles.section}>
              <h2 className={styles.sectionTitle}>Estado da Sincronização</h2>
              <p className={styles.muted}>
                Status: <strong>{formatSyncStatus(consoleProfile.profile.syncStatus)}</strong> ·
                Última execução: {consoleProfile.profile.lastSyncedAt ?? 'Nunca'}
              </p>
              {consoleProfile.profile.syncDiagnostics ? (
                <p className={styles.muted}>{consoleProfile.profile.syncDiagnostics}</p>
              ) : null}
            </section>
          ) : null}
        </>
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
              Nenhum jogo descoberto para {consoleRecord.name} ainda. Configure uma pasta na aba
              Pastas &amp; Descoberta ou sincronize após configurar o emulador.
            </p>
          ) : (
            <div className={envStyles.gameGrid}>
              {filteredGames.map((game) => {
                const coverUrl = resolveCoverAssetUrl(game.coverPath);
                const isMissing = game.installState === 'missing_files';
                const canLaunchTitle =
                  isPs3Playable && rpcs3Configured && !isMissing && Boolean(onLaunchGame);
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
                        Origem: {formatMetadataSource(game.metadataSource)} ·{' '}
                        {isMissing ? (
                          <span className={envStyles.missingBadge}>Ausente no disco</span>
                        ) : (
                          formatGameAvailability(game.installState)
                        )}
                      </div>
                      <div className={styles.actionsRow}>
                        {canLaunchTitle ? (
                          <button
                            type="button"
                            className={styles.btnPrimary}
                            disabled={busy || sessionBusy}
                            onClick={() => void handleLaunchGame(game.id, game.title)}
                          >
                            <Play
                              size={14}
                              aria-hidden="true"
                              style={{ marginRight: 6, verticalAlign: 'middle' }}
                            />
                            Jogar
                          </button>
                        ) : null}
                        <button
                          type="button"
                          className={canLaunchTitle ? styles.btnSecondary : styles.btnPrimary}
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
            {isPs3Playable
              ? 'O scanner PS3 lê metadados reais do cabeçalho binário PARAM.SFO ao lado de USRDIR/EBOOT.BIN e importa automaticamente ícones nativos ICON0.PNG via cache seguro.'
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

            {isPs3Playable && rpcs3Configured && onRecordVerification ? (
              <div className={styles.actionsRow} style={{ marginTop: 12 }}>
                <button
                  type="button"
                  className={styles.btnSecondary}
                  disabled={busy}
                  onClick={() => void handleRecordXmbVerification('runtime_confirmed')}
                >
                  Confirmar XMB em Execução
                </button>
                <button
                  type="button"
                  className={styles.btnSecondary}
                  disabled={busy}
                  onClick={() => void handleRecordXmbVerification('failed')}
                >
                  Registrar Falha no XMB
                </button>
              </div>
            ) : null}
          </section>

          {rpcs3Inspection ? (
            <section className={styles.section}>
              <h2 className={styles.sectionTitle}>Prontidão RPCS3 (PlayStation 3 · Fase 3)</h2>
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
