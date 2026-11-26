import { memo, useMemo, useState } from 'react';
import { LayoutGrid, List, Play, Star } from 'lucide-react';
import { resolveCoverAssetUrl } from '../api/tauriClient';
import type { ActiveSessionSnapshot, ConsoleRecord, GameRecord } from '../types/domain';
import {
  formatGameAvailability,
  formatMetadataSource,
  formatPlaytime,
} from '../utils/formatters';
import libStyles from './LibraryPage.module.css';
import styles from './Pages.module.css';

export interface LibraryPageProps {
  consoles: ConsoleRecord[];
  games: GameRecord[];
  rpcs3Ready?: boolean;
  defaultView?: 'grid' | 'list';
  activeSession?: ActiveSessionSnapshot | null;
  onToggleFavorite: (gameId: string) => Promise<void>;
  onOpenGame?: (gameId: string) => void;
  onSelectConsole?: (consoleSlug: string) => void;
  onLaunchGame?: (gameId: string) => Promise<void>;
}

type SortKey = 'title' | 'recent' | 'playtime' | 'completion';
type AvailabilityCategory = 'all' | 'available' | 'missing';

interface LibraryGameCardProps {
  game: GameRecord;
  viewMode: 'grid' | 'list';
  canPlayPs3: boolean;
  sessionBusy: boolean;
  launching: boolean;
  onLaunch: (gameId: string) => void;
  onOpenGame?: (gameId: string) => void;
  onToggleFavorite: (gameId: string) => void;
}

const LibraryGameItem = memo(function LibraryGameItem({
  game,
  viewMode,
  canPlayPs3,
  sessionBusy,
  launching,
  onLaunch,
  onOpenGame,
  onToggleFavorite,
}: LibraryGameCardProps) {
  const isMissing = game.installState === 'missing_files';
  const coverUrl = resolveCoverAssetUrl(game.coverPath);

  if (viewMode === 'list') {
    return (
      <li className={styles.listItem} data-focus-card="true">
        <div className={libStyles.listLead}>
          <div className={libStyles.listThumb}>
            {coverUrl ? (
              <img
                src={coverUrl}
                alt={`Capa de ${game.title}`}
                className={libStyles.coverImg}
                loading="lazy"
              />
            ) : (
              <span>{game.consoleSlug.toUpperCase()}</span>
            )}
          </div>
          <div>
            <strong>{game.title}</strong>
            <div className={styles.muted}>
              Console: {game.consoleSlug.toUpperCase()} · Versões: {game.versionCount} · Tempo:{' '}
              {formatPlaytime(game.totalPlaytimeSeconds)} · Progresso: {game.completionPercent}% ·
              Origem: {formatMetadataSource(game.metadataSource)} · Estado:{' '}
              <strong>{formatGameAvailability(game.installState)}</strong>
            </div>
          </div>
        </div>
        <div className={styles.actionsRow}>
          {canPlayPs3 ? (
            <button
              type="button"
              className={styles.btnPrimary}
              disabled={sessionBusy || launching}
              onClick={() => onLaunch(game.id)}
            >
              <Play
                size={14}
                aria-hidden="true"
                style={{ marginRight: 6, verticalAlign: 'middle' }}
              />
              {launching ? 'Iniciando...' : 'Jogar'}
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
            data-gamepad-secondary="true"
            onClick={() => onToggleFavorite(game.id)}
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
  }

  return (
    <article className={libStyles.gameCard} data-focus-card="true">
      <div className={libStyles.coverSlot}>
        {coverUrl ? (
          <img
            src={coverUrl}
            alt={`Capa de ${game.title}`}
            className={libStyles.coverImg}
            loading="lazy"
          />
        ) : (
          <span className={libStyles.coverFallbackTitle} aria-hidden="true">
            {game.consoleSlug.toUpperCase()}
          </span>
        )}
        <span className={libStyles.coverConsoleBadge}>{game.consoleSlug.toUpperCase()}</span>
        {isMissing ? (
          <span className={libStyles.coverMissingBadge}>
            {formatGameAvailability(game.installState)}
          </span>
        ) : null}
      </div>

      <div className={libStyles.cardBody}>
        <h3 className={libStyles.cardTitle} title={game.title}>
          {game.title}
        </h3>
        <div className={libStyles.cardMeta}>
          {formatPlaytime(game.totalPlaytimeSeconds)} · {game.versionCount} versão(ões) ·{' '}
          {game.completionPercent}%
        </div>
        <div className={libStyles.cardMeta}>
          {formatMetadataSource(game.metadataSource)}
          {!isMissing ? ` · ${formatGameAvailability(game.installState)}` : ''}
        </div>
        <div className={libStyles.progressTrack} aria-hidden="true">
          <div
            className={libStyles.progressFill}
            style={{ width: `${game.completionPercent}%` }}
          />
        </div>

        <div className={libStyles.cardActions}>
          {canPlayPs3 ? (
            <button
              type="button"
              className={styles.btnPrimary}
              disabled={sessionBusy || launching}
              onClick={() => onLaunch(game.id)}
            >
              <Play
                size={13}
                aria-hidden="true"
                style={{ marginRight: 5, verticalAlign: 'middle' }}
              />
              {launching ? 'Iniciando...' : 'Jogar'}
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
            data-gamepad-secondary="true"
            onClick={() => onToggleFavorite(game.id)}
          >
            <Star
              size={13}
              aria-hidden="true"
              fill={game.isFavorite ? 'currentColor' : 'none'}
              style={{ marginRight: 5, verticalAlign: 'middle' }}
            />
            {game.isFavorite ? 'Favorito' : 'Favoritar'}
          </button>
        </div>
      </div>
    </article>
  );
});

export function LibraryPage({
  consoles,
  games,
  rpcs3Ready = false,
  defaultView = 'grid',
  activeSession,
  onToggleFavorite,
  onOpenGame,
  onSelectConsole,
  onLaunchGame,
}: LibraryPageProps) {
  const [viewMode, setViewMode] = useState<'grid' | 'list'>(defaultView);
  const [search, setSearch] = useState('');
  const [consoleFilter, setConsoleFilter] = useState('all');
  const [availabilityFilter, setAvailabilityFilter] = useState<AvailabilityCategory>('all');
  const [genreFilter, setGenreFilter] = useState('all');
  const [sortBy, setSortBy] = useState<SortKey>('title');
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [launchError, setLaunchError] = useState<string | null>(null);
  const [launchingId, setLaunchingId] = useState<string | null>(null);

  const sessionBusy = Boolean(activeSession && activeSession.phase !== 'idle');

  const availableGenres = useMemo(() => {
    const set = new Set<string>();
    for (const g of games) {
      if (g.genre && g.genre.trim().length > 0) set.add(g.genre.trim());
    }
    return Array.from(set).sort();
  }, [games]);

  const filteredGames = useMemo(() => {
    const q = search.trim().toLowerCase();
    const list = games.filter((g) => {
      if (consoleFilter !== 'all' && g.consoleSlug !== consoleFilter) return false;
      if (favoritesOnly && !g.isFavorite) return false;
      if (availabilityFilter === 'available' && g.installState === 'missing_files') return false;
      if (availabilityFilter === 'missing' && g.installState !== 'missing_files') return false;
      if (genreFilter !== 'all' && (g.genre ?? '').trim() !== genreFilter) return false;
      if (q && !g.title.toLowerCase().includes(q) && !g.canonicalKey.toLowerCase().includes(q)) {
        return false;
      }
      return true;
    });

    return [...list].sort((a, b) => {
      if (sortBy === 'recent') {
        if (a.lastPlayedAt && b.lastPlayedAt) return b.lastPlayedAt.localeCompare(a.lastPlayedAt);
        if (a.lastPlayedAt) return -1;
        if (b.lastPlayedAt) return 1;
      } else if (sortBy === 'playtime') {
        if (b.totalPlaytimeSeconds !== a.totalPlaytimeSeconds) {
          return b.totalPlaytimeSeconds - a.totalPlaytimeSeconds;
        }
      } else if (sortBy === 'completion') {
        if (b.completionPercent !== a.completionPercent) {
          return b.completionPercent - a.completionPercent;
        }
      }
      return a.title.localeCompare(b.title);
    });
  }, [games, search, consoleFilter, favoritesOnly, availabilityFilter, genreFilter, sortBy]);

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
            Vitrine consolidada a partir de descoberta automática, diretórios configurados e
            sincronização real dos perfis de cada console — sem cadastro manual.
          </p>
        </div>
        <div className={styles.actionsRow}>
          <button
            type="button"
            className={viewMode === 'grid' ? styles.btnPrimary : styles.btnSecondary}
            aria-pressed={viewMode === 'grid'}
            onClick={() => setViewMode('grid')}
          >
            <LayoutGrid
              size={14}
              aria-hidden="true"
              style={{ marginRight: 6, verticalAlign: 'middle' }}
            />
            Grade de Capas
          </button>
          <button
            type="button"
            className={viewMode === 'list' ? styles.btnPrimary : styles.btnSecondary}
            aria-pressed={viewMode === 'list'}
            onClick={() => setViewMode('list')}
          >
            <List
              size={14}
              aria-hidden="true"
              style={{ marginRight: 6, verticalAlign: 'middle' }}
            />
            Lista
          </button>
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
        <div className={libStyles.toolbar}>
          <h2 className={styles.sectionTitle}>Jogos Descobertos ({filteredGames.length})</h2>
          <div className={libStyles.filterGroup}>
            <input
              aria-label="Buscar jogos"
              className={styles.input}
              placeholder="Buscar por título ou serial..."
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
            <select
              aria-label="Ordenar jogos"
              className={styles.select}
              value={sortBy}
              onChange={(e) => setSortBy(e.target.value as SortKey)}
            >
              <option value="title">Ordenar: Título (A–Z)</option>
              <option value="recent">Ordenar: Recentes</option>
              <option value="playtime">Ordenar: Tempo jogado</option>
              <option value="completion">Ordenar: Conclusão</option>
            </select>
            {availableGenres.length > 0 ? (
              <select
                aria-label="Filtrar por categoria"
                className={styles.select}
                value={genreFilter}
                onChange={(e) => setGenreFilter(e.target.value)}
              >
                <option value="all">Todas as categorias</option>
                {availableGenres.map((genre) => (
                  <option key={genre} value={genre}>
                    {genre}
                  </option>
                ))}
              </select>
            ) : null}
            <button
              type="button"
              className={favoritesOnly ? styles.btnPrimary : styles.btnSecondary}
              data-focus-default="true"
              onClick={() => setFavoritesOnly((prev) => !prev)}
            >
              {favoritesOnly ? 'Mostrando Favoritos' : 'Apenas Favoritos'}
            </button>
          </div>
        </div>

        <div className={libStyles.categoryRow} role="group" aria-label="Filtros rápidos de estado">
          {(
            [
              ['all', 'Todos'],
              ['available', 'Disponíveis no disco'],
              ['missing', 'Ausentes no disco'],
            ] as const
          ).map(([key, label]) => (
            <button
              key={key}
              type="button"
              className={`${libStyles.catChip} ${
                availabilityFilter === key ? libStyles.catChipActive : ''
              }`}
              onClick={() => setAvailabilityFilter(key)}
            >
              {label}
            </button>
          ))}
        </div>

        {filteredGames.length === 0 ? (
          <p className={styles.muted}>
            Nenhum jogo corresponde aos filtros atuais. Configure as pastas ou o diretório do
            emulador dentro do perfil de cada console para descobrir jogos automaticamente.
          </p>
        ) : viewMode === 'grid' ? (
          <div className={libStyles.coverGrid}>
            {filteredGames.map((game) => (
              <LibraryGameItem
                key={game.id}
                game={game}
                viewMode="grid"
                canPlayPs3={
                  game.consoleSlug === 'ps3' &&
                  rpcs3Ready &&
                  game.installState !== 'missing_files' &&
                  Boolean(onLaunchGame)
                }
                sessionBusy={sessionBusy}
                launching={launchingId === game.id}
                onLaunch={(id) => void handleLaunch(id)}
                onOpenGame={onOpenGame}
                onToggleFavorite={(id) => void onToggleFavorite(id)}
              />
            ))}
          </div>
        ) : (
          <ul className={styles.list}>
            {filteredGames.map((game) => (
              <LibraryGameItem
                key={game.id}
                game={game}
                viewMode="list"
                canPlayPs3={
                  game.consoleSlug === 'ps3' &&
                  rpcs3Ready &&
                  game.installState !== 'missing_files' &&
                  Boolean(onLaunchGame)
                }
                sessionBusy={sessionBusy}
                launching={launchingId === game.id}
                onLaunch={(id) => void handleLaunch(id)}
                onOpenGame={onOpenGame}
                onToggleFavorite={(id) => void onToggleFavorite(id)}
              />
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
