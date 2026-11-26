import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  DesktopRuntimeRequiredError,
  HubApiClient,
  resolveHubTransport,
  subscribeHubRuntimeEvents,
} from './api/tauriClient';
import { ActiveSessionBanner } from './components/ActiveSessionBanner';
import { AppShell } from './components/AppShell';
import { useHubNavigation, type HubSectionId } from './hooks/useHubNavigation';
import { ConsoleEnvironmentPage } from './pages/ConsoleEnvironmentPage';
import { ConsolesPage } from './pages/ConsolesPage';
import { EmulatorsPage } from './pages/EmulatorsPage';
import { GameDetailsPage } from './pages/GameDetailsPage';
import { HomePage } from './pages/HomePage';
import { LibraryPage } from './pages/LibraryPage';
import pageStyles from './pages/Pages.module.css';
import { ProfilePage } from './pages/ProfilePage';
import { SettingsPage } from './pages/SettingsPage';
import { StatisticsPage } from './pages/StatisticsPage';
import { TrophiesPage } from './pages/TrophiesPage';
import trophyStyles from './pages/TrophiesPage.module.css';
import type {
  ActiveSessionSnapshot,
  ActivityEntryDto,
  BackupFileEntry,
  ConsoleEnvironmentDto,
  ConsoleProfileOverviewDto,
  DetailedTrophyRow,
  GameDetailsDto,
  HubBootstrapDto,
  NewlyUnlockedTrophy,
  RecordRuntimeVerificationRequest,
  ScanProgressEvent,
  StatisticsOverview,
  TrophyCenterOverviewDto,
  UpdateGameMetadataRequest,
  UpdateProfileRequest,
} from './types/domain';

const IDLE_SESSION: ActiveSessionSnapshot = {
  phase: 'idle',
  sessionId: null,
  target: null,
  consoleSlug: null,
  gameId: null,
  gameTitle: null,
  emulatorSlug: null,
  pid: null,
  startedAt: null,
  lastHeartbeatAt: null,
  elapsedSeconds: 0,
};

export function App() {
  const clientResolution = useMemo(() => {
    try {
      const transport = resolveHubTransport();
      return { client: new HubApiClient(transport), error: null as Error | null };
    } catch (err) {
      return {
        client: null as HubApiClient | null,
        error: err instanceof Error ? err : new Error(String(err)),
      };
    }
  }, []);

  const [section, setSection] = useState<HubSectionId>('home');
  const [selectedConsoleSlug, setSelectedConsoleSlug] = useState<string | null>(null);
  const [consoleEnv, setConsoleEnv] = useState<ConsoleEnvironmentDto | null>(null);
  const [consoleProfile, setConsoleProfile] = useState<ConsoleProfileOverviewDto | null>(null);
  const [consoleTrophyOverview, setConsoleTrophyOverview] =
    useState<TrophyCenterOverviewDto | null>(null);
  const [trophyOverview, setTrophyOverview] = useState<TrophyCenterOverviewDto | null>(null);
  const [recentUnlockedToast, setRecentUnlockedToast] = useState<NewlyUnlockedTrophy[]>([]);
  const [selectedGameDetails, setSelectedGameDetails] = useState<GameDetailsDto | null>(null);
  const [selectedGameTrophies, setSelectedGameTrophies] = useState<DetailedTrophyRow[]>([]);
  const [activeSession, setActiveSession] = useState<ActiveSessionSnapshot>(IDLE_SESSION);
  const [statsOverview, setStatsOverview] = useState<StatisticsOverview | null>(null);
  const [activityItems, setActivityItems] = useState<ActivityEntryDto[]>([]);
  const [backups, setBackups] = useState<BackupFileEntry[]>([]);
  const [bootstrap, setBootstrap] = useState<HubBootstrapDto | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  const refreshBootstrap = useCallback(async () => {
    if (!clientResolution.client) return;
    try {
      const data = await clientResolution.client.getBootstrap();
      setBootstrap(data);
      setLoadError(null);
    } catch (err) {
      setLoadError(String(err));
    }
  }, [clientResolution.client]);

  const refreshBackups = useCallback(async () => {
    if (!clientResolution.client) return;
    try {
      const list = await clientResolution.client.listDatabaseBackups();
      setBackups(list);
    } catch {
      setBackups([]);
    }
  }, [clientResolution.client]);

  const refreshStatistics = useCallback(async () => {
    if (!clientResolution.client) return;
    try {
      const [overview, activity] = await Promise.all([
        clientResolution.client.getStatisticsOverview(),
        clientResolution.client.listActivityHistory(null, 100),
      ]);
      setStatsOverview(overview);
      setActivityItems(activity);
    } catch (err) {
      setLoadError(String(err));
    }
  }, [clientResolution.client]);

  const refreshTrophyOverview = useCallback(async () => {
    if (!clientResolution.client) return;
    try {
      const overview = await clientResolution.client.getTrophyCenterOverview(null);
      setTrophyOverview(overview);
    } catch {
      // Fallback safely if legacy test transport does not register Phase 4 command
    }
  }, [clientResolution.client]);

  const refreshActiveConsoleEnv = useCallback(async () => {
    if (!clientResolution.client || !selectedConsoleSlug) return;
    const [env, profile, consoleTrophies] = await Promise.all([
      clientResolution.client.getConsoleEnvironment(selectedConsoleSlug),
      clientResolution.client.getConsoleProfile(selectedConsoleSlug).catch(() => null),
      selectedConsoleSlug === 'ps3'
        ? clientResolution.client.getTrophyCenterOverview('ps3').catch(() => null)
        : Promise.resolve(null),
    ]);
    setConsoleEnv(env);
    setConsoleProfile(profile);
    setConsoleTrophyOverview(consoleTrophies);
  }, [clientResolution.client, selectedConsoleSlug]);

  const handleSelectSection = useCallback(
    (next: HubSectionId) => {
      setSection(next);
      setSelectedConsoleSlug(null);
      setConsoleEnv(null);
      setConsoleProfile(null);
      setConsoleTrophyOverview(null);
      setSelectedGameDetails(null);
      setSelectedGameTrophies([]);
      if (next === 'statistics') {
        void refreshStatistics();
      } else if (next === 'settings') {
        void refreshBackups();
      } else if (next === 'trophies') {
        void refreshTrophyOverview();
      }
    },
    [refreshBackups, refreshStatistics, refreshTrophyOverview]
  );

  // Suspend Hub keyboard shortcuts while a live emulator session is active
  useHubNavigation(section, handleSelectSection, activeSession.phase !== 'idle');

  useEffect(() => {
    void refreshBootstrap();
    if (clientResolution.client) {
      void clientResolution.client
        .getActiveSession()
        .then(setActiveSession)
        .catch(() => setActiveSession(IDLE_SESSION));
    }
  }, [refreshBootstrap, clientResolution.client]);

  useEffect(() => {
    let cleanup: (() => void) | null = null;
    void subscribeHubRuntimeEvents({
      onSessionStarted: (snap) => setActiveSession(snap),
      onSessionHeartbeat: (snap) => setActiveSession(snap),
      onSessionExited: () => {
        setActiveSession(IDLE_SESSION);
        void refreshBootstrap();
        void refreshActiveConsoleEnv();
      },
      onProfileSynced: () => {
        void refreshBootstrap();
        void refreshActiveConsoleEnv();
      },
      onTrophiesSynced: () => {
        void refreshBootstrap();
        void refreshTrophyOverview();
        void refreshActiveConsoleEnv();
      },
      onTrophiesUnlocked: (unlocked) => {
        if (unlocked.length > 0) {
          setRecentUnlockedToast(unlocked);
        }
      },
    }).then((unlisten) => {
      cleanup = unlisten;
    });

    return () => {
      cleanup?.();
    };
  }, [refreshBootstrap, refreshActiveConsoleEnv, refreshTrophyOverview]);

  const openConsoleEnvironment = useCallback(
    async (slug: string) => {
      if (!clientResolution.client) return;
      setSection('consoles');
      setSelectedConsoleSlug(slug);
      setSelectedGameDetails(null);
      setSelectedGameTrophies([]);
      try {
        const [env, profile, consoleTrophies] = await Promise.all([
          clientResolution.client.getConsoleEnvironment(slug),
          clientResolution.client.getConsoleProfile(slug).catch(() => null),
          slug === 'ps3'
            ? clientResolution.client.getTrophyCenterOverview('ps3').catch(() => null)
            : Promise.resolve(null),
        ]);
        setConsoleEnv(env);
        setConsoleProfile(profile);
        setConsoleTrophyOverview(consoleTrophies);
      } catch (err) {
        setLoadError(String(err));
      }
    },
    [clientResolution.client]
  );

  const openGameDetails = useCallback(
    async (gameId: string) => {
      if (!clientResolution.client) return;
      try {
        const [details, gameTrophies] = await Promise.all([
          clientResolution.client.getGameDetails(gameId),
          clientResolution.client.listGameTrophies(gameId).catch(() => []),
        ]);
        setSelectedGameDetails(details);
        setSelectedGameTrophies(gameTrophies);
      } catch (err) {
        setLoadError(String(err));
      }
    },
    [clientResolution.client]
  );

  if (clientResolution.error instanceof DesktopRuntimeRequiredError) {
    return (
      <main style={{ padding: '48px', maxWidth: '720px', margin: '0 auto' }}>
        <h1 className={pageStyles.title}>Universal Console Hub</h1>
        <p className={pageStyles.subtitle} style={{ marginTop: '12px' }}>
          {clientResolution.error.message}
        </p>
        <div
          className={`${pageStyles.banner} ${pageStyles.bannerInfo}`}
          style={{ marginTop: '20px' }}
        >
          Inicie o aplicativo desktop via Tauri 2 para conectar-se ao backend Rust, ao banco SQLite
          real e aos adaptadores de emuladores no Windows.
        </div>
      </main>
    );
  }

  if (clientResolution.error || !clientResolution.client) {
    return (
      <main style={{ padding: '48px' }}>
        <h1 className={pageStyles.title}>Universal Console Hub</h1>
        <p className={pageStyles.muted}>{String(clientResolution.error)}</p>
      </main>
    );
  }

  const client = clientResolution.client;

  if (loadError) {
    return (
      <main style={{ padding: '48px' }}>
        <h1 className={pageStyles.title}>Universal Console Hub</h1>
        <div className={`${pageStyles.banner} ${pageStyles.bannerError}`}>{loadError}</div>
      </main>
    );
  }

  if (!bootstrap) {
    return (
      <main style={{ padding: '48px' }}>
        <h1 className={pageStyles.title}>Universal Console Hub</h1>
        <p className={pageStyles.muted}>Carregando banco local SQLite e catálogo de consoles...</p>
      </main>
    );
  }

  const rpcs3Ready = Boolean(
    bootstrap.rpcs3Inspection.configured && bootstrap.rpcs3Inspection.installation?.isValid
  );

  const handleUpdateProfile = async (request: UpdateProfileRequest) => {
    await client.updateActiveProfile(request);
    await refreshBootstrap();
  };

  const handleToggleFavorite = async (gameId: string) => {
    await client.toggleGameFavorite(gameId);
    await refreshBootstrap();
    await refreshActiveConsoleEnv();
    if (selectedGameDetails?.game.id === gameId) {
      const updated = await client.getGameDetails(gameId);
      setSelectedGameDetails(updated);
    }
  };

  const handleUpdateGameMetadata = async (request: UpdateGameMetadataRequest) => {
    await client.updateGameMetadata(request);
    await refreshBootstrap();
    await refreshActiveConsoleEnv();
    const updated = await client.getGameDetails(request.gameId);
    setSelectedGameDetails(updated);
  };

  const handleSetGameCover = async (gameId: string, sourceImagePath: string) => {
    await client.setGameCover(gameId, sourceImagePath);
    await refreshBootstrap();
    await refreshActiveConsoleEnv();
    const updated = await client.getGameDetails(gameId);
    setSelectedGameDetails(updated);
  };

  const handleAddScanPath = async (
    consoleSlug: string,
    directoryPath: string,
    recursive: boolean
  ) => {
    await client.addConsoleScanPath(consoleSlug, directoryPath, recursive);
    await refreshBootstrap();
    await refreshActiveConsoleEnv();
  };

  const handleRemoveScanPath = async (pathId: string) => {
    await client.removeConsoleScanPath(pathId);
    await refreshActiveConsoleEnv();
  };

  const handleScanLibrary = async (
    consoleSlug: string,
    onProgress?: (evt: ScanProgressEvent) => void
  ) => {
    const summary = await client.scanConsoleLibrary(consoleSlug, onProgress);
    await refreshBootstrap();
    await refreshActiveConsoleEnv();
    return summary;
  };

  const handleUpdateConsoleSetting = async (
    consoleSlug: string,
    key: string,
    value: string
  ) => {
    await client.updateConsoleSetting(consoleSlug, key, value);
    await refreshActiveConsoleEnv();
  };

  const handleSyncConsoleProfile = async (consoleSlug: string) => {
    const report = await client.syncConsoleProfile(consoleSlug, 'manual');
    await refreshBootstrap();
    await refreshActiveConsoleEnv();
    return report;
  };

  const handleSyncConsoleTrophies = async (consoleSlug: string) => {
    const report = await client.syncConsoleTrophies(consoleSlug);
    if (report.newlyUnlocked.length > 0) {
      setRecentUnlockedToast(report.newlyUnlocked);
    }
    await Promise.all([refreshBootstrap(), refreshTrophyOverview(), refreshActiveConsoleEnv()]);
    if (selectedGameDetails) {
      const updatedTrophies = await client
        .listGameTrophies(selectedGameDetails.game.id)
        .catch(() => []);
      setSelectedGameTrophies(updatedTrophies);
    }
    return report;
  };

  const handleLaunchConsoleSystem = async (consoleSlug: string) => {
    const snap = await client.launchConsoleSystem(consoleSlug);
    setActiveSession(snap);
    await refreshActiveConsoleEnv();
  };

  const handleLaunchGame = async (gameId: string, versionId?: string | null) => {
    const snap = await client.launchGame(gameId, versionId ?? null);
    setActiveSession(snap);
    await refreshActiveConsoleEnv();
  };

  const handleStopActiveSession = async () => {
    await client.stopActiveSession();
    const snap = await client.getActiveSession();
    setActiveSession(snap);
    await refreshBootstrap();
    await refreshActiveConsoleEnv();
  };

  const handleRecordRuntimeVerification = async (request: RecordRuntimeVerificationRequest) => {
    await client.recordRuntimeVerification(request);
    await refreshBootstrap();
    await refreshActiveConsoleEnv();
  };

  const handleConfigureEmulator = async (slug: string, exePath: string) => {
    await client.configureEmulatorPath(slug, exePath);
    await refreshBootstrap();
    await refreshActiveConsoleEnv();
  };

  const handleInspectRpcs3 = async () => {
    await client.inspectRpcs3();
    await refreshBootstrap();
    await refreshActiveConsoleEnv();
  };

  const handleRegisterFirmware = async (
    consoleSlug: string,
    emulatorSlug: string,
    filePath: string
  ) => {
    await client.registerFirmwareFile(consoleSlug, emulatorSlug, filePath);
    await refreshBootstrap();
    await refreshActiveConsoleEnv();
  };

  const handleUpdateSetting = async (key: string, value: string) => {
    await client.updateAppSetting(key, value);
    await refreshBootstrap();
  };

  const handleCreateBackup = async () => {
    const report = await client.createDatabaseBackup();
    await refreshBackups();
    return report;
  };

  const handleInspectBackup = async (backupPath: string) => {
    return client.inspectDatabaseBackup(backupPath);
  };

  const handleRestoreBackup = async (backupPath: string) => {
    const report = await client.restoreDatabaseBackup(backupPath);
    await refreshBootstrap();
    await refreshBackups();
    return report;
  };

  const effectiveTrophyOverview: TrophyCenterOverviewDto = trophyOverview ?? {
    providers: bootstrap.achievementProviders,
    sets: [],
    trophies: [],
    recentUnlocks: [],
    totals: {
      totalSets: 0,
      completedSets: 0,
      totalTrophies: bootstrap.trophies.length,
      unlockedTrophies: bootstrap.trophies.filter((t) => t.isUnlocked).length,
      bronzeTotal: 0,
      bronzeUnlocked: 0,
      silverTotal: 0,
      silverUnlocked: 0,
      goldTotal: 0,
      goldUnlocked: 0,
      platinumTotal: 0,
      platinumUnlocked: 0,
      completionPercent: 0,
    },
  };

  return (
    <AppShell
      activeSection={section}
      onSelectSection={handleSelectSection}
      profileDisplayName={bootstrap.profile.displayName}
      profileHandle={bootstrap.profile.handle}
      transportKind={client.transportKind}
    >
      <ActiveSessionBanner session={activeSession} onStopSession={handleStopActiveSession} />

      {recentUnlockedToast.length > 0 ? (
        <div
          className={trophyStyles.toastBanner}
          role="status"
          aria-live="polite"
          aria-label="Novos troféus desbloqueados"
        >
          <div>
            <strong>
              {recentUnlockedToast.length === 1 && recentUnlockedToast[0]
                ? `Novo troféu desbloqueado: ${recentUnlockedToast[0].name}`
                : `${recentUnlockedToast.length} novos troféus desbloqueados`}
            </strong>
            <div className={pageStyles.muted}>
              {recentUnlockedToast.map((t) => `${t.gameTitle} — ${t.name}`).join(' · ')}
            </div>
          </div>
          <div className={pageStyles.actionsRow}>
            <button
              type="button"
              className={pageStyles.btnSecondary}
              onClick={() => {
                setRecentUnlockedToast([]);
                handleSelectSection('trophies');
              }}
            >
              Ver Troféus
            </button>
            <button
              type="button"
              className={pageStyles.btnSecondary}
              onClick={() => setRecentUnlockedToast([])}
            >
              Fechar
            </button>
          </div>
        </div>
      ) : null}

      {selectedGameDetails ? (
        <GameDetailsPage
          details={selectedGameDetails}
          trophies={selectedGameTrophies}
          canLaunch={selectedGameDetails.game.consoleSlug === 'ps3' && rpcs3Ready}
          activeSession={activeSession}
          onBack={() => {
            setSelectedGameDetails(null);
            setSelectedGameTrophies([]);
          }}
          onToggleFavorite={handleToggleFavorite}
          onUpdateMetadata={handleUpdateGameMetadata}
          onSetCover={handleSetGameCover}
          onLaunchGame={handleLaunchGame}
        />
      ) : null}

      {!selectedGameDetails && section === 'home' ? (
        <HomePage
          data={bootstrap}
          onNavigate={handleSelectSection}
          onSelectConsole={(slug) => void openConsoleEnvironment(slug)}
        />
      ) : null}

      {!selectedGameDetails && section === 'consoles' && !selectedConsoleSlug ? (
        <ConsolesPage
          consoles={bootstrap.consoles}
          emulators={bootstrap.emulators}
          onSelectConsole={(slug) => void openConsoleEnvironment(slug)}
        />
      ) : null}

      {!selectedGameDetails && section === 'consoles' && selectedConsoleSlug && consoleEnv ? (
        <ConsoleEnvironmentPage
          environment={consoleEnv}
          consoleProfile={consoleProfile}
          consoleTrophyOverview={consoleTrophyOverview}
          activeSession={activeSession}
          onBack={() => {
            setSelectedConsoleSlug(null);
            setConsoleEnv(null);
            setConsoleProfile(null);
            setConsoleTrophyOverview(null);
          }}
          onOpenEmulators={() => handleSelectSection('emulators')}
          onOpenTrophies={() => handleSelectSection('trophies')}
          onOpenGame={(gameId) => void openGameDetails(gameId)}
          onToggleFavorite={handleToggleFavorite}
          onAddScanPath={handleAddScanPath}
          onRemoveScanPath={handleRemoveScanPath}
          onScanLibrary={handleScanLibrary}
          onUpdateConsoleSetting={handleUpdateConsoleSetting}
          onSyncProfile={handleSyncConsoleProfile}
          onSyncConsoleTrophies={handleSyncConsoleTrophies}
          onLaunchSystem={handleLaunchConsoleSystem}
          onLaunchGame={handleLaunchGame}
          onRecordVerification={handleRecordRuntimeVerification}
        />
      ) : null}

      {!selectedGameDetails && section === 'library' ? (
        <LibraryPage
          consoles={bootstrap.consoles}
          games={bootstrap.games}
          rpcs3Ready={rpcs3Ready}
          activeSession={activeSession}
          onToggleFavorite={handleToggleFavorite}
          onOpenGame={(gameId) => void openGameDetails(gameId)}
          onSelectConsole={(slug) => void openConsoleEnvironment(slug)}
          onLaunchGame={(gameId) => handleLaunchGame(gameId, null)}
        />
      ) : null}

      {!selectedGameDetails && section === 'profile' ? (
        <ProfilePage
          profile={bootstrap.profile}
          statistics={bootstrap.statistics}
          onUpdateProfile={handleUpdateProfile}
        />
      ) : null}

      {!selectedGameDetails && section === 'statistics' && statsOverview ? (
        <StatisticsPage
          overview={statsOverview}
          activity={activityItems}
          onFilterActivity={(eventType) => client.listActivityHistory(eventType, 100)}
        />
      ) : null}

      {!selectedGameDetails && section === 'trophies' ? (
        <TrophiesPage
          overview={effectiveTrophyOverview}
          onSyncConsoleTrophies={handleSyncConsoleTrophies}
          onOpenGame={(gameId) => void openGameDetails(gameId)}
        />
      ) : null}

      {!selectedGameDetails && section === 'emulators' ? (
        <EmulatorsPage
          emulators={bootstrap.emulators}
          firmwareRecords={bootstrap.firmwareRecords}
          rpcs3Inspection={bootstrap.rpcs3Inspection}
          onConfigureEmulator={handleConfigureEmulator}
          onInspectRpcs3={handleInspectRpcs3}
          onRegisterFirmware={handleRegisterFirmware}
        />
      ) : null}

      {!selectedGameDetails && section === 'settings' ? (
        <SettingsPage
          settings={bootstrap.settings}
          backups={backups}
          onUpdateSetting={handleUpdateSetting}
          onCreateBackup={handleCreateBackup}
          onInspectBackup={handleInspectBackup}
          onRestoreBackup={handleRestoreBackup}
        />
      ) : null}
    </AppShell>
  );
}
