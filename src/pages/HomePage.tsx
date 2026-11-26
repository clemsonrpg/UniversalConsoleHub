import { useMemo } from 'react';
import { Clock, Gamepad2, Palette, Play, Star, Trophy } from 'lucide-react';
import { resolveCoverAssetUrl } from '../api/tauriClient';
import { ReadinessTracker } from '../components/ReadinessTracker';
import { StatTile } from '../components/StatTile';
import type { HubSectionId } from '../hooks/useHubNavigation';
import type { ActiveSessionSnapshot, GameRecord, HubBootstrapDto } from '../types/domain';
import { formatIntegrationStage, formatPlaytime } from '../utils/formatters';
import homeStyles from './HomePage.module.css';
import styles from './Pages.module.css';

export interface HomePageProps {
  data: HubBootstrapDto;
  rpcs3Ready?: boolean;
  activeSession?: ActiveSessionSnapshot | null;
  onNavigate: (section: HubSectionId) => void;
  onSelectConsole: (slug: string) => void;
  onOpenGame?: (gameId: string) => void;
  onLaunchGame?: (gameId: string) => Promise<void>;
}

export function HomePage({
  data,
  rpcs3Ready = false,
  activeSession,
  onNavigate,
  onSelectConsole,
  onOpenGame,
  onLaunchGame,
}: HomePageProps) {
  const sessionBusy = Boolean(activeSession && activeSession.phase !== 'idle');

  const recentGames = useMemo<GameRecord[]>(() => {
    return [...data.games]
      .sort((a, b) => {
        if (a.lastPlayedAt && b.lastPlayedAt) return b.lastPlayedAt.localeCompare(a.lastPlayedAt);
        if (a.lastPlayedAt) return -1;
        if (b.lastPlayedAt) return 1;
        return b.totalPlaytimeSeconds - a.totalPlaytimeSeconds;
      })
      .slice(0, 6);
  }, [data.games]);

  const favoriteGames = useMemo<GameRecord[]>(
    () => data.games.filter((g) => g.isFavorite).slice(0, 6),
    [data.games]
  );

  const heroGame = useMemo<GameRecord | null>(() => {
    const withLastPlayed = recentGames.find((g) => g.lastPlayedAt !== null);
    return withLastPlayed ?? favoriteGames[0] ?? recentGames[0] ?? null;
  }, [recentGames, favoriteGames]);

  const heroCoverUrl = heroGame ? resolveCoverAssetUrl(heroGame.coverPath) : null;
  const heroCanPlay = Boolean(
    heroGame &&
      heroGame.consoleSlug === 'ps3' &&
      rpcs3Ready &&
      heroGame.installState !== 'missing_files' &&
      onLaunchGame
  );

  const gamesPerConsole = useMemo(() => {
    const map = new Map<string, number>();
    for (const g of data.games) {
      map.set(g.consoleSlug, (map.get(g.consoleSlug) ?? 0) + 1);
    }
    return map;
  }, [data.games]);

  const lastSessionActivity = useMemo(
    () => data.recentActivity.find((a) => a.eventType.startsWith('session_')) ?? null,
    [data.recentActivity]
  );

  return (
    <div className={styles.page}>
      <section className={homeStyles.hero} aria-label="Destaque principal do Hub">
        {heroCoverUrl ? (
          <div
            className={homeStyles.heroBackdrop}
            style={{ backgroundImage: `url("${heroCoverUrl}")` }}
            aria-hidden="true"
          />
        ) : null}
        <div className={homeStyles.heroScrim} aria-hidden="true" />

        <div className={homeStyles.heroContent}>
          <div className={homeStyles.heroKicker}>
            <Gamepad2 size={15} aria-hidden="true" />
            <span>
              {heroGame
                ? `Continuar jogando · ${heroGame.consoleSlug.toUpperCase()}`
                : `Central de Entretenimento · ${data.profile.displayName}`}
            </span>
          </div>
          <h1 className={styles.title}>Visão Geral do Hub</h1>
          <p className={styles.subtitle}>
            {heroGame
              ? `${heroGame.title} · Tempo real: ${formatPlaytime(heroGame.totalPlaytimeSeconds)} · Progresso: ${heroGame.completionPercent}%.`
              : `Bem-vindo, ${data.profile.displayName}. O Universal Console Hub preserva interfaces originais via emuladores reais — sem dashboards falsas e sem distribuir arquivos proprietários.`}
          </p>

          <div className={styles.actionsRow}>
            {heroGame && heroCanPlay && onLaunchGame ? (
              <button
                type="button"
                className={styles.btnPrimary}
                data-focus-default="true"
                data-focus-id="home-hero-primary"
                disabled={sessionBusy}
                onClick={() => void onLaunchGame(heroGame.id)}
              >
                <Play
                  size={14}
                  aria-hidden="true"
                  style={{ marginRight: 6, verticalAlign: 'middle' }}
                />
                Continuar {heroGame.title}
              </button>
            ) : (
              <button
                type="button"
                className={styles.btnPrimary}
                data-focus-default="true"
                data-focus-id="home-hero-primary"
                onClick={() => onSelectConsole('ps3')}
              >
                Entrar no PlayStation 3
              </button>
            )}
            {heroGame && onOpenGame ? (
              <button
                type="button"
                className={styles.btnSecondary}
                onClick={() => onOpenGame(heroGame.id)}
              >
                Detalhes do Título
              </button>
            ) : null}
            <button
              type="button"
              className={styles.btnSecondary}
              onClick={() => onNavigate('emulators')}
            >
              Configurar RPCS3 &amp; Firmware
            </button>
            <button
              type="button"
              className={styles.btnSecondary}
              onClick={() => onNavigate('themes')}
            >
              <Palette
                size={14}
                aria-hidden="true"
                style={{ marginRight: 6, verticalAlign: 'middle' }}
              />
              Temas
            </button>
          </div>
        </div>

        {heroCoverUrl && heroGame ? (
          <div className={homeStyles.heroCoverFrame}>
            <img
              src={heroCoverUrl}
              alt={`Capa em destaque de ${heroGame.title}`}
              className={homeStyles.heroCoverImg}
            />
          </div>
        ) : null}
      </section>

      <section className={styles.statGrid} aria-label="Resumo real do banco de dados">
        <StatTile label="Consoles no catálogo" value={data.consoles.length} hint="6 prioritários" />
        <StatTile
          label="Jogos registrados"
          value={data.statistics.gamesRegistered}
          hint="Biblioteca local SQLite"
        />
        <StatTile
          label="Tempo real jogado"
          value={formatPlaytime(data.statistics.totalPlaytimeSeconds)}
          hint="Baseado em sessões reais"
        />
        <StatTile
          label="Troféus desbloqueados"
          value={data.statistics.unlockedTrophies}
          hint={`de ${data.statistics.totalTrophies} registrados`}
        />
      </section>

      {recentGames.length > 0 ? (
        <section className={styles.section}>
          <div className={homeStyles.shelfHeader}>
            <h2 className={styles.sectionTitle}>
              <Clock size={16} aria-hidden="true" style={{ marginRight: 6, verticalAlign: 'middle' }} />
              Continuar Jogando &amp; Recentes
            </h2>
            <button
              type="button"
              className={styles.btnSecondary}
              onClick={() => onNavigate('library')}
            >
              Explorar Catálogo
            </button>
          </div>
          <div className={homeStyles.shelfTrack}>
            {recentGames.map((game) => {
              const coverUrl = resolveCoverAssetUrl(game.coverPath);
              const canPlay =
                game.consoleSlug === 'ps3' &&
                rpcs3Ready &&
                game.installState !== 'missing_files' &&
                Boolean(onLaunchGame);
              return (
                <article key={game.id} className={homeStyles.gameTile} data-focus-card="true">
                  <div className={homeStyles.tileArt}>
                    {coverUrl ? (
                      <img
                        src={coverUrl}
                        alt={`Capa de ${game.title}`}
                        className={homeStyles.tileImg}
                        loading="lazy"
                      />
                    ) : (
                      <span className={homeStyles.tileMonogram}>
                        {game.consoleSlug.toUpperCase()}
                      </span>
                    )}
                    <span className={homeStyles.tileBadge}>{game.consoleSlug.toUpperCase()}</span>
                  </div>
                  <div className={homeStyles.tileBody}>
                    <h3 className={homeStyles.tileTitle} title={game.title}>
                      {game.title}
                    </h3>
                    <div className={homeStyles.tileMeta}>
                      {formatPlaytime(game.totalPlaytimeSeconds)} · {game.completionPercent}%
                    </div>
                    <div className={styles.actionsRow}>
                      {canPlay && onLaunchGame ? (
                        <button
                          type="button"
                          className={styles.btnPrimary}
                          disabled={sessionBusy}
                          onClick={() => void onLaunchGame(game.id)}
                        >
                          Jogar
                        </button>
                      ) : null}
                      {onOpenGame ? (
                        <button
                          type="button"
                          className={styles.btnSecondary}
                          onClick={() => onOpenGame(game.id)}
                        >
                          Detalhes
                        </button>
                      ) : null}
                    </div>
                  </div>
                </article>
              );
            })}
          </div>
        </section>
      ) : null}

      {favoriteGames.length > 0 ? (
        <section className={styles.section}>
          <div className={homeStyles.shelfHeader}>
            <h2 className={styles.sectionTitle}>
              <Star size={16} aria-hidden="true" style={{ marginRight: 6, verticalAlign: 'middle' }} />
              Jogos Favoritos ({favoriteGames.length})
            </h2>
          </div>
          <div className={homeStyles.shelfTrack}>
            {favoriteGames.map((game) => {
              const coverUrl = resolveCoverAssetUrl(game.coverPath);
              return (
                <article key={`fav-${game.id}`} className={homeStyles.gameTile} data-focus-card="true">
                  <div className={homeStyles.tileArt}>
                    {coverUrl ? (
                      <img
                        src={coverUrl}
                        alt={`Capa de ${game.title}`}
                        className={homeStyles.tileImg}
                        loading="lazy"
                      />
                    ) : (
                      <span className={homeStyles.tileMonogram}>
                        {game.consoleSlug.toUpperCase()}
                      </span>
                    )}
                    <span className={homeStyles.tileBadge}>{game.consoleSlug.toUpperCase()}</span>
                  </div>
                  <div className={homeStyles.tileBody}>
                    <h3 className={homeStyles.tileTitle}>{game.title}</h3>
                    <div className={homeStyles.tileMeta}>
                      {formatPlaytime(game.totalPlaytimeSeconds)} · {game.completionPercent}%
                    </div>
                    {onOpenGame ? (
                      <button
                        type="button"
                        className={styles.btnSecondary}
                        onClick={() => onOpenGame(game.id)}
                      >
                        Abrir Jogo
                      </button>
                    ) : null}
                  </div>
                </article>
              );
            })}
          </div>
        </section>
      ) : null}

      <section className={styles.section}>
        <div className={homeStyles.shelfHeader}>
          <h2 className={styles.sectionTitle}>Consoles Disponíveis</h2>
          <button
            type="button"
            className={styles.btnSecondary}
            onClick={() => onNavigate('consoles')}
          >
            Ver Coleção
          </button>
        </div>
        <div className={homeStyles.consoleStrip}>
          {data.consoles.map((c) => {
            const count = gamesPerConsole.get(c.slug) ?? 0;
            const emu = data.emulators.find((e) => e.consoleSlug === c.slug);
            return (
              <button
                key={c.id}
                type="button"
                className={homeStyles.consoleChip}
                onClick={() => onSelectConsole(c.slug)}
              >
                <strong>{c.name}</strong>
                <span className={homeStyles.tileMeta}>
                  {emu?.name ?? '—'} · {count} jogo(s)
                </span>
                <span className={homeStyles.tileMeta}>{formatIntegrationStage(c.phaseStatus)}</span>
              </button>
            );
          })}
        </div>
      </section>

      <div className={homeStyles.splitGrid}>
        <section className={styles.section}>
          <h2 className={styles.sectionTitle}>Status da Preparação RPCS3 (PlayStation 3)</h2>
          <p className={styles.muted}>{data.rpcs3Inspection.diagnostic}</p>
          {data.rpcs3Inspection.readiness ? (
            <ReadinessTracker readiness={data.rpcs3Inspection.readiness} />
          ) : (
            <div className={styles.actionsRow}>
              <button
                type="button"
                className={styles.btnSecondary}
                onClick={() => onNavigate('emulators')}
              >
                Apontar executável rpcs3.exe
              </button>
            </div>
          )}
        </section>

        <section className={styles.section}>
          <div className={homeStyles.shelfHeader}>
            <h2 className={styles.sectionTitle}>Atividade Recente &amp; Última Sessão (SQLite)</h2>
            <button
              type="button"
              className={styles.btnSecondary}
              onClick={() => onNavigate('trophies')}
            >
              <Trophy
                size={14}
                aria-hidden="true"
                style={{ marginRight: 6, verticalAlign: 'middle' }}
              />
              Troféus
            </button>
          </div>
          {lastSessionActivity ? (
            <div className={`${styles.banner} ${styles.bannerInfo}`}>
              <strong>Última sessão:</strong> {lastSessionActivity.summary} (
              <span className={styles.mono}>{lastSessionActivity.occurredAt}</span>)
            </div>
          ) : null}
          {data.recentActivity.length === 0 ? (
            <p className={styles.muted}>
              Nenhuma atividade registrada ainda. Configure o RPCS3 ou sincronize um console para
              registrar histórico real em UTC.
            </p>
          ) : (
            <ul className={styles.list}>
              {data.recentActivity.slice(0, 8).map((entry) => (
                <li key={entry.id} className={styles.listItem}>
                  <span>{entry.summary}</span>
                  <span className={styles.mono}>{entry.occurredAt}</span>
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>
    </div>
  );
}
