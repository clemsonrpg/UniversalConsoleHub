import { useState, type FormEvent } from 'react';
import { ArrowLeft, Eye, EyeOff, Play, Star } from 'lucide-react';
import { resolveCoverAssetUrl } from '../api/tauriClient';
import { FilePathField } from '../components/FilePathField';
import type {
  ActiveSessionSnapshot,
  DetailedTrophyRow,
  GameDetailsDto,
  UpdateGameMetadataRequest,
} from '../types/domain';
import {
  formatGameAvailability,
  formatMetadataSource,
  formatPlaytime,
  formatSessionExitStatus,
} from '../utils/formatters';
import detailStyles from './GameDetailsPage.module.css';
import styles from './Pages.module.css';
import { TrophyItemCard } from './TrophiesPage';
import trophyStyles from './TrophiesPage.module.css';

export interface GameDetailsPageProps {
  details: GameDetailsDto;
  trophies?: DetailedTrophyRow[];
  canLaunch?: boolean;
  activeSession?: ActiveSessionSnapshot | null;
  onBack: () => void;
  onToggleFavorite: (gameId: string) => Promise<void>;
  onUpdateMetadata: (request: UpdateGameMetadataRequest) => Promise<void>;
  onSetCover: (gameId: string, sourceImagePath: string) => Promise<void>;
  onLaunchGame?: (gameId: string, versionId?: string | null) => Promise<void>;
}

export function GameDetailsPage({
  details,
  trophies = [],
  canLaunch = false,
  activeSession,
  onBack,
  onToggleFavorite,
  onUpdateMetadata,
  onSetCover,
  onLaunchGame,
}: GameDetailsPageProps) {
  const { game, versions, sessions } = details;
  const [showHiddenSpoilers, setShowHiddenSpoilers] = useState(false);
  const [description, setDescription] = useState(game.description);
  const [developer, setDeveloper] = useState(game.developer ?? '');
  const [publisher, setPublisher] = useState(game.publisher ?? '');
  const [releaseYear, setReleaseYear] = useState(game.releaseYear ? String(game.releaseYear) : '');
  const [genre, setGenre] = useState(game.genre ?? '');
  const [completionPercent, setCompletionPercent] = useState(String(game.completionPercent));
  const [coverSourcePath, setCoverSourcePath] = useState('');
  const [feedback, setFeedback] = useState<{ kind: 'ok' | 'err'; text: string } | null>(null);
  const [busy, setBusy] = useState(false);

  const coverUrl = resolveCoverAssetUrl(game.coverPath);
  const isMissing = game.installState === 'missing_files';
  const sessionBusy = Boolean(activeSession && activeSession.phase !== 'idle');
  const playableNow = game.consoleSlug === 'ps3' && canLaunch && !isMissing && Boolean(onLaunchGame);
  const unlockedTrophiesCount = trophies.filter((t) => t.isUnlocked).length;

  const handleLaunchNow = async (versionId?: string | null) => {
    if (!onLaunchGame) return;
    setFeedback(null);
    setBusy(true);
    try {
      await onLaunchGame(game.id, versionId ?? null);
      setFeedback({
        kind: 'ok',
        text: `Inicializando '${game.title}' via RPCS3 com monitoramento de sessão ativo.`,
      });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setBusy(false);
    }
  };

  const handleSaveMetadata = async (event: FormEvent) => {
    event.preventDefault();
    setFeedback(null);
    setBusy(true);
    try {
      const parsedYear = releaseYear.trim() ? Number.parseInt(releaseYear.trim(), 10) : null;
      const parsedCompletion = Number.parseInt(completionPercent, 10);
      await onUpdateMetadata({
        gameId: game.id,
        description,
        developer: developer.trim() || null,
        publisher: publisher.trim() || null,
        releaseYear: Number.isNaN(parsedYear ?? NaN) ? null : parsedYear,
        genre: genre.trim() || null,
        completionPercent: Number.isNaN(parsedCompletion) ? 0 : parsedCompletion,
      });
      setFeedback({ kind: 'ok', text: 'Metadados e progresso do jogo salvos no SQLite.' });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setBusy(false);
    }
  };

  const handleSetCover = async (event: FormEvent) => {
    event.preventDefault();
    setFeedback(null);
    setBusy(true);
    try {
      await onSetCover(game.id, coverSourcePath);
      setCoverSourcePath('');
      setFeedback({
        kind: 'ok',
        text: 'Capa validada por assinatura binária (PNG/JPEG/WebP) e armazenada no cache seguro local.',
      });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className={styles.page}>
      <section className={detailStyles.hero} aria-label={`Detalhes de ${game.title}`}>
        {coverUrl ? (
          <div
            className={detailStyles.backdrop}
            style={{ backgroundImage: `url("${coverUrl}")` }}
            aria-hidden="true"
          />
        ) : null}
        <div className={detailStyles.scrim} aria-hidden="true" />

        <div className={detailStyles.coverFrame}>
          {coverUrl ? (
            <img src={coverUrl} alt={`Capa de ${game.title}`} className={detailStyles.coverImg} />
          ) : (
            <span className={detailStyles.coverPlaceholder}>{game.consoleSlug.toUpperCase()}</span>
          )}
        </div>

        <div className={detailStyles.heroBody}>
          <div className={detailStyles.metaPills}>
            <span className={detailStyles.pill}>{game.consoleSlug.toUpperCase()}</span>
            <span className={detailStyles.pill}>{formatGameAvailability(game.installState)}</span>
            <span className={detailStyles.pill}>{formatMetadataSource(game.metadataSource)}</span>
            {game.developer ? <span className={detailStyles.pill}>{game.developer}</span> : null}
            {game.releaseYear ? <span className={detailStyles.pill}>{game.releaseYear}</span> : null}
            {game.genre ? <span className={detailStyles.pill}>{game.genre}</span> : null}
          </div>

          <h1 className={styles.title}>{game.title}</h1>
          {game.description ? (
            <p className={detailStyles.description}>{game.description}</p>
          ) : null}

          <div className={detailStyles.statsStrip}>
            <div className={detailStyles.statBox}>
              <span className={detailStyles.statLabel}>Tempo jogado</span>
              <span className={detailStyles.statValue}>
                {formatPlaytime(game.totalPlaytimeSeconds)}
              </span>
            </div>
            <div className={detailStyles.statBox}>
              <span className={detailStyles.statLabel}>Sessões reais</span>
              <span className={detailStyles.statValue}>{sessions.length}</span>
            </div>
            <div className={detailStyles.statBox}>
              <span className={detailStyles.statLabel}>Conclusão</span>
              <span className={detailStyles.statValue}>{game.completionPercent}%</span>
            </div>
            <div className={detailStyles.statBox}>
              <span className={detailStyles.statLabel}>Troféus</span>
              <span className={detailStyles.statValue}>
                {unlockedTrophiesCount} / {trophies.length}
              </span>
            </div>
          </div>

          <div className={styles.actionsRow}>
            {playableNow ? (
              <button
                type="button"
                className={styles.btnPrimary}
                data-focus-default="true"
                disabled={busy || sessionBusy}
                onClick={() => void handleLaunchNow(null)}
              >
                <Play
                  size={15}
                  aria-hidden="true"
                  style={{ marginRight: 6, verticalAlign: 'middle' }}
                />
                Jogar
              </button>
            ) : null}
            <button
              type="button"
              className={styles.btnSecondary}
              aria-pressed={game.isFavorite}
              data-gamepad-secondary="true"
              data-focus-default={!playableNow ? 'true' : undefined}
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
            <button type="button" className={styles.btnSecondary} onClick={onBack}>
              <ArrowLeft
                size={14}
                aria-hidden="true"
                style={{ marginRight: 6, verticalAlign: 'middle' }}
              />
              Voltar
            </button>
          </div>
        </div>
      </section>

      {isMissing ? (
        <div className={`${styles.banner} ${styles.bannerError}`} role="note">
          Arquivo do jogo ausente no disco ({formatGameAvailability(game.installState)}). Todo o
          histórico de sessões e tempo jogado permanece preservado.
        </div>
      ) : game.consoleSlug === 'ps3' ? (
        <div className={`${styles.banner} ${styles.bannerInfo}`} role="note">
          {playableNow
            ? 'PlayStation 3 pronto para execução direta via RPCS3 (--no-gui) com heartbeat e retorno automático ao Hub.'
            : 'Configure o executável rpcs3.exe em Emuladores para habilitar a execução direta deste título.'}
        </div>
      ) : (
        <div className={`${styles.banner} ${styles.bannerInfo}`} role="note">
          Execução ao vivo deste console entra na <strong>Fase 5</strong>. Nenhum botão simulado de
          execução é exibido antes da integração real do emulador.
        </div>
      )}

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

      <div className={detailStyles.twoCol}>
        <section className={styles.section}>
          <h2 className={styles.sectionTitle}>Metadados &amp; Progresso</h2>
          <form onSubmit={handleSaveMetadata} className={styles.page}>
            <div className={styles.formGrid}>
              <div className={styles.field}>
                <label className={styles.label} htmlFor="meta-dev">
                  Desenvolvedora
                </label>
                <input
                  id="meta-dev"
                  className={styles.input}
                  value={developer}
                  onChange={(e) => setDeveloper(e.target.value)}
                />
              </div>

              <div className={styles.field}>
                <label className={styles.label} htmlFor="meta-pub">
                  Publicadora
                </label>
                <input
                  id="meta-pub"
                  className={styles.input}
                  value={publisher}
                  onChange={(e) => setPublisher(e.target.value)}
                />
              </div>

              <div className={styles.field}>
                <label className={styles.label} htmlFor="meta-year">
                  Ano de lançamento
                </label>
                <input
                  id="meta-year"
                  type="number"
                  min={1970}
                  max={2100}
                  className={styles.input}
                  value={releaseYear}
                  onChange={(e) => setReleaseYear(e.target.value)}
                />
              </div>

              <div className={styles.field}>
                <label className={styles.label} htmlFor="meta-genre">
                  Gênero
                </label>
                <input
                  id="meta-genre"
                  className={styles.input}
                  value={genre}
                  onChange={(e) => setGenre(e.target.value)}
                />
              </div>

              <div className={styles.field}>
                <label className={styles.label} htmlFor="meta-completion">
                  Progresso de conclusão (0–100%)
                </label>
                <input
                  id="meta-completion"
                  type="number"
                  min={0}
                  max={100}
                  className={styles.input}
                  value={completionPercent}
                  onChange={(e) => setCompletionPercent(e.target.value)}
                />
              </div>
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="meta-desc">
                Descrição / notas
              </label>
              <textarea
                id="meta-desc"
                rows={3}
                className={styles.textarea}
                value={description}
                onChange={(e) => setDescription(e.target.value)}
              />
            </div>

            <div className={styles.actionsRow}>
              <button type="submit" className={styles.btnPrimary} disabled={busy}>
                Salvar Metadados
              </button>
            </div>
          </form>
        </section>

        <section className={styles.section}>
          <h2 className={styles.sectionTitle}>Capa Local (Cache Seguro SHA-256)</h2>
          <p className={styles.muted}>
            {game.coverPath
              ? `Capa armazenada em cache autorizado: ${game.coverPath}`
              : 'Nenhuma capa local definida ainda. Selecione uma imagem PNG, JPEG ou WebP (até 10 MB).'}
          </p>
          <form onSubmit={handleSetCover} className={styles.page}>
            <div className={styles.field}>
              <label className={styles.label} htmlFor="game-cover-path">
                Selecionar imagem local (PNG, JPEG ou WebP — máx. 10 MB)
              </label>
              <FilePathField
                id="game-cover-path"
                mode="file"
                dialogTitle={`Definir capa local para ${game.title}`}
                filters={[{ name: 'Imagens', extensions: ['png', 'jpg', 'jpeg', 'webp'] }]}
                value={coverSourcePath}
                onChange={setCoverSourcePath}
                placeholder="C:\Capas\capa.png"
                required
                disabled={busy}
              />
            </div>
            <div className={styles.actionsRow}>
              <button type="submit" className={styles.btnSecondary} disabled={busy}>
                Definir Capa Local
              </button>
            </div>
          </form>
        </section>
      </div>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Versões Registradas ({versions.length})</h2>
        {versions.length === 0 ? (
          <p className={styles.muted}>Nenhuma versão registrada.</p>
        ) : (
          <div className={styles.tableWrap}>
            <table className={styles.table}>
              <thead>
                <tr>
                  <th scope="col">Versão</th>
                  <th scope="col">Região</th>
                  <th scope="col">Serial</th>
                  <th scope="col">Caminho Local</th>
                </tr>
              </thead>
              <tbody>
                {versions.map((ver) => (
                  <tr key={ver.id}>
                    <td>{ver.versionLabel}</td>
                    <td>{ver.region}</td>
                    <td className={styles.mono}>{ver.serialCode ?? '—'}</td>
                    <td className={styles.mono}>{ver.filePath}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      <section className={styles.section}>
        <div className={styles.header}>
          <h2 className={styles.sectionTitle}>
            Troféus do Jogo ({unlockedTrophiesCount} / {trophies.length})
          </h2>
          {trophies.some((t) => t.isHidden && !t.isUnlocked) ? (
            <label className={trophyStyles.spoilerToggle}>
              <input
                type="checkbox"
                checked={showHiddenSpoilers}
                onChange={(e) => setShowHiddenSpoilers(e.target.checked)}
              />
              {showHiddenSpoilers ? (
                <Eye size={14} aria-hidden="true" />
              ) : (
                <EyeOff size={14} aria-hidden="true" />
              )}
              Revelar ocultos
            </label>
          ) : null}
        </div>
        {trophies.length === 0 ? (
          <p className={styles.muted}>
            {game.consoleSlug === 'ps3'
              ? 'Nenhum conjunto de troféus sincronizado para este título ainda. Sincronize na Central de Troféus após executar o jogo no RPCS3.'
              : 'Esta plataforma não possui leitor de troféus nativos nesta fase.'}
          </p>
        ) : (
          <ul className={trophyStyles.trophyList}>
            {trophies.map((trophy) => (
              <TrophyItemCard
                key={trophy.id}
                trophy={trophy}
                showHiddenSpoilers={showHiddenSpoilers}
                showGameTitle={false}
              />
            ))}
          </ul>
        )}
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Sessões Reais ({sessions.length})</h2>
        {sessions.length === 0 ? (
          <p className={styles.muted}>Nenhuma sessão gravada para este título ainda.</p>
        ) : (
          <div className={styles.tableWrap}>
            <table className={styles.table}>
              <thead>
                <tr>
                  <th scope="col">Início (UTC)</th>
                  <th scope="col">Duração</th>
                  <th scope="col">Status</th>
                </tr>
              </thead>
              <tbody>
                {sessions.map((s) => (
                  <tr key={s.id}>
                    <td className={styles.mono}>{s.startedAt}</td>
                    <td>{formatPlaytime(s.durationSeconds)}</td>
                    <td>{formatSessionExitStatus(s.exitStatus)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>
    </div>
  );
}
