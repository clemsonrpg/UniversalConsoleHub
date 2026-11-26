import { useMemo, useState } from 'react';
import { Play, Star } from 'lucide-react';
import type { ActiveSessionSnapshot, ConsoleRecord, GameRecord } from '../types/domain';
import {
  formatGameAvailability,
  formatMetadataSource,
  formatPlaytime,
} from '../utils/formatters';
import styles from './Pages.module.css';

export interface LibraryPageProps {
  consoles: ConsoleRecord[];
  games: GameRecord[];
  rpcs3Ready?: boolean;
  activeSession?: ActiveSessionSnapshot | null;
  onToggleFavorite: (gameId: string) => Promise<void>;
  onOpenGame?: (gameId: string) => void;
  onSelectConsole?: (consoleSlug: string) => void;
  onLaunchGame?: (gameId: string) => Promise<void>;
}

export function LibraryPage({
  consoles,
  games,
  rpcs3Ready = false,
  activeSession,
  onToggleFavorite,
  onOpenGame,
  onSelectConsole,
  onLaunchGame,
}: LibraryPageProps) {
  const [search, setSearch] = useState('');
  const [consoleFilter, setConsoleFilter] = useState('all');
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [launchError, setLaunchError] = useState<string | null>(null);
  const [launchingId, setLaunchingId] = useState<string | null>(null);

  const sessionBusy = Boolean(activeSession && activeSession.phase !== 'idle');

  const filteredGames = useMemo(() => {
    const q = search.trim().toLowerCase();
    return games.filter((g) => {
      if (consoleFilter !== 'all' && g.consoleSlug !== consoleFilter) return false;
      if (favoritesOnly && !g.isFavorite) return false;
      if (q && !g.title.toLowerCase().includes(q) && !g.canonicalKey.toLowerCase().includes(q)) {
        return false;
      }
      return true;
    });
  }, [games, search, consoleFilter, favoritesOnly]);

  const handleLaunch = async (gameId: string) => {
    if (!onLaunchGame) return;
    setLaunchError(null);
    setLaunchingId(gameId);
    try {
      await onLaunchGame(gameId);
    } catch (err) {
      setLaunchError(String(err));
    } finally {
      setLaunchingId(null);
    }
  };

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <h1 className={styles.title}>Biblioteca Universal</h1>
          <p className={styles.subtitle}>
            Biblioteca consolidada a partir de descoberta automática, diretórios configurados e
            sincronização real dos perfis de cada console — sem cadastro manual.
          </p>
        </div>
      </header>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Descoberta e Sincronização Automática</h2>
        <p className={styles.muted}>
          Os jogos são identificados automaticamente pelos scanners e arquivos reais de cada
          plataforma (como <code className={styles.mono}>PARAM.SFO</code>,{' '}
          <code className={styles.mono}>dev_hdd0/game/</code> e{' '}
          <code className={styles.mono}>ICON0.PNG</code> no PlayStation 3). Abra o perfil do console
          para configurar pastas monitoradas ou executar uma sincronização.
        </p>
        {onSelectConsole ? (
          <div className={styles.actionsRow}>
            {consoles.map((c) => (
              <button
                key={c.slug}
                type="button"
                className={c.slug === 'ps3' ? styles.btnPrimary : styles.btnSecondary}
                onClick={() => onSelectConsole(c.slug)}
              >
                Abrir Perfil {c.name}
              </button>
            ))}
          </div>
        ) : null}
      </section>

      {launchError ? (
        <div role="alert" className={`${styles.banner} ${styles.bannerError}`}>
          {launchError}
        </div>
      ) : null}

      <section className={styles.section}>
        <div className={styles.header}>
          <h2 className={styles.sectionTitle}>Jogos Descobertos ({filteredGames.length})</h2>
          <div className={styles.actionsRow}>
            <input
              aria-label="Buscar jogos"
              className={styles.input}
              placeholder="Buscar por título ou chave canônica..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
            <select
              aria-label="Filtrar por console"
              className={styles.select}
              value={consoleFilter}
              onChange={(e) => setConsoleFilter(e.target.value)}
            >
              <option value="all">Todos os consoles</option>
              {consoles.map((c) => (
                <option key={c.slug} value={c.slug}>
                  {c.name}
                </option>
              ))}
            </select>
            <button
              type="button"
              className={favoritesOnly ? styles.btnPrimary : styles.btnSecondary}
              onClick={() => setFavoritesOnly((prev) => !prev)}
            >
              {favoritesOnly ? 'Mostrando Favoritos' : 'Apenas Favoritos'}
            </button>
          </div>
        </div>

        {filteredGames.length === 0 ? (
          <p className={styles.muted}>
            Nenhum jogo corresponde aos filtros atuais. Configure as pastas ou o diretório do
            emulador dentro do perfil de cada console para descobrir jogos automaticamente.
          </p>
        ) : (
          <ul className={styles.list}>
            {filteredGames.map((game) => {
              const isMissing = game.installState === 'missing_files';
              const canPlayPs3 =
                game.consoleSlug === 'ps3' && rpcs3Ready && !isMissing && Boolean(onLaunchGame);
              return (
                <li key={game.id} className={styles.listItem}>
                  <div>
                    <strong>{game.title}</strong>
                    <div className={styles.muted}>
                      Console: {game.consoleSlug.toUpperCase()} · Versões: {game.versionCount} ·
                      Tempo: {formatPlaytime(game.totalPlaytimeSeconds)} · Origem:{' '}
                      {formatMetadataSource(game.metadataSource)} · Estado:{' '}
                      <strong>{formatGameAvailability(game.installState)}</strong>
                    </div>
                  </div>
                  <div className={styles.actionsRow}>
                    {canPlayPs3 ? (
                      <button
                        type="button"
                        className={styles.btnPrimary}
                        disabled={sessionBusy || launchingId === game.id}
                        onClick={() => void handleLaunch(game.id)}
                      >
                        <Play
                          size={14}
                          aria-hidden="true"
                          style={{ marginRight: 6, verticalAlign: 'middle' }}
                        />
                        {launchingId === game.id ? 'Iniciando...' : 'Jogar'}
                      </button>
                    ) : null}
                    {onOpenGame ? (
                      <button
                        type="button"
                        className={canPlayPs3 ? styles.btnSecondary : styles.btnPrimary}
                        onClick={() => onOpenGame(game.id)}
                      >
                        Detalhes
                      </button>
                    ) : null}
                    <button
                      type="button"
                      className={styles.btnSecondary}
                      aria-pressed={game.isFavorite}
                      onClick={() => void onToggleFavorite(game.id)}
                    >
                      <Star
                        size={14}
                        aria-hidden="true"
                        fill={game.isFavorite ? 'currentColor' : 'none'}
                        style={{ marginRight: 6, verticalAlign: 'middle' }}
                      />
                      {game.isFavorite ? 'Favorito' : 'Favoritar'}
                    </button>
                  </div>
                </li>
              );
            })}
          </ul>
        )}
      </section>
    </div>
  );
}
