import { useCallback, useEffect, useMemo, useState } from 'react';
import {
  DesktopRuntimeRequiredError,
  HubApiClient,
  resolveHubTransport,
} from './api/tauriClient';
import { AppShell } from './components/AppShell';
import { useHubNavigation, type HubSectionId } from './hooks/useHubNavigation';
import { ConsoleEnvironmentPage } from './pages/ConsoleEnvironmentPage';
import { ConsolesPage } from './pages/ConsolesPage';
import { EmulatorsPage } from './pages/EmulatorsPage';
import { HomePage } from './pages/HomePage';
import { LibraryPage } from './pages/LibraryPage';
import pageStyles from './pages/Pages.module.css';
import { ProfilePage } from './pages/ProfilePage';
import { SettingsPage } from './pages/SettingsPage';
import { TrophiesPage } from './pages/TrophiesPage';
import type {
  ConsoleEnvironmentDto,
  HubBootstrapDto,
  RegisterGameRequest,
  ScanProgressEvent,
  UpdateProfileRequest,
} from './types/domain';

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
  const [bootstrap, setBootstrap] = useState<HubBootstrapDto | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  const handleSelectSection = useCallback((next: HubSectionId) => {
    setSection(next);
    setSelectedConsoleSlug(null);
    setConsoleEnv(null);
  }, []);

  useHubNavigation(section, handleSelectSection);

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

  useEffect(() => {
    void refreshBootstrap();
  }, [refreshBootstrap]);

  const openConsoleEnvironment = useCallback(
    async (slug: string) => {
      if (!clientResolution.client) return;
      setSection('consoles');
      setSelectedConsoleSlug(slug);
      try {
        const env = await clientResolution.client.getConsoleEnvironment(slug);
        setConsoleEnv(env);
      } catch (err) {
        setLoadError(String(err));
      }
    },
    [clientResolution.client]
  );

  const refreshActiveConsoleEnv = useCallback(async () => {
    if (!clientResolution.client || !selectedConsoleSlug) return;
    const env = await clientResolution.client.getConsoleEnvironment(selectedConsoleSlug);
    setConsoleEnv(env);
  }, [clientResolution.client, selectedConsoleSlug]);

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

  const handleUpdateProfile = async (request: UpdateProfileRequest) => {
    await client.updateActiveProfile(request);
    await refreshBootstrap();
  };

  const handleRegisterGame = async (request: RegisterGameRequest) => {
    await client.registerGame(request);
    await refreshBootstrap();
  };

  const handleToggleFavorite = async (gameId: string) => {
    await client.toggleGameFavorite(gameId);
    await refreshBootstrap();
    await refreshActiveConsoleEnv();
  };

  const handleAddScanPath = async (
    consoleSlug: string,
    directoryPath: string,
    recursive: boolean
  ) => {
    await client.addConsoleScanPath(consoleSlug, directoryPath, recursive);
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

  const handleConfigureEmulator = async (slug: string, exePath: string) => {
    await client.configureEmulatorPath(slug, exePath);
    await refreshBootstrap();
  };

  const handleInspectRpcs3 = async () => {
    await client.inspectRpcs3();
    await refreshBootstrap();
  };

  const handleRegisterFirmware = async (
    consoleSlug: string,
    emulatorSlug: string,
    filePath: string
  ) => {
    await client.registerFirmwareFile(consoleSlug, emulatorSlug, filePath);
    await refreshBootstrap();
  };

  const handleUpdateSetting = async (key: string, value: string) => {
    await client.updateAppSetting(key, value);
    await refreshBootstrap();
  };

  const handleCreateBackup = async () => {
    return client.createDatabaseBackup();
  };

  return (
    <AppShell
      activeSection={section}
      onSelectSection={handleSelectSection}
      profileDisplayName={bootstrap.profile.displayName}
      profileHandle={bootstrap.profile.handle}
      transportKind={client.transportKind}
    >
      {section === 'home' ? (
        <HomePage
          data={bootstrap}
          onNavigate={handleSelectSection}
          onSelectConsole={(slug) => void openConsoleEnvironment(slug)}
        />
      ) : null}

      {section === 'consoles' && !selectedConsoleSlug ? (
        <ConsolesPage
          consoles={bootstrap.consoles}
          emulators={bootstrap.emulators}
          onSelectConsole={(slug) => void openConsoleEnvironment(slug)}
        />
      ) : null}

      {section === 'consoles' && selectedConsoleSlug && consoleEnv ? (
        <ConsoleEnvironmentPage
          environment={consoleEnv}
          onBack={() => {
            setSelectedConsoleSlug(null);
            setConsoleEnv(null);
          }}
          onOpenEmulators={() => handleSelectSection('emulators')}
          onOpenGame={() => handleSelectSection('library')}
          onToggleFavorite={handleToggleFavorite}
          onAddScanPath={handleAddScanPath}
          onRemoveScanPath={handleRemoveScanPath}
          onScanLibrary={handleScanLibrary}
          onUpdateConsoleSetting={handleUpdateConsoleSetting}
        />
      ) : null}

      {section === 'library' ? (
        <LibraryPage
          consoles={bootstrap.consoles}
          games={bootstrap.games}
          onRegisterGame={handleRegisterGame}
          onToggleFavorite={handleToggleFavorite}
        />
      ) : null}

      {section === 'profile' ? (
        <ProfilePage
          profile={bootstrap.profile}
          statistics={bootstrap.statistics}
          onUpdateProfile={handleUpdateProfile}
        />
      ) : null}

      {section === 'trophies' ? (
        <TrophiesPage
          providers={bootstrap.achievementProviders}
          trophies={bootstrap.trophies}
        />
      ) : null}

      {section === 'emulators' ? (
        <EmulatorsPage
          emulators={bootstrap.emulators}
          firmwareRecords={bootstrap.firmwareRecords}
          rpcs3Inspection={bootstrap.rpcs3Inspection}
          onConfigureEmulator={handleConfigureEmulator}
          onInspectRpcs3={handleInspectRpcs3}
          onRegisterFirmware={handleRegisterFirmware}
        />
      ) : null}

      {section === 'settings' ? (
        <SettingsPage
          settings={bootstrap.settings}
          onUpdateSetting={handleUpdateSetting}
          onCreateBackup={handleCreateBackup}
        />
      ) : null}
    </AppShell>
  );
}
