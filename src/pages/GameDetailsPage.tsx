import { useState, type FormEvent } from 'react';
import { Star } from 'lucide-react';
import { resolveCoverAssetUrl } from '../api/tauriClient';
import { FilePathField } from '../components/FilePathField';
import type { GameDetailsDto, UpdateGameMetadataRequest } from '../types/domain';
import {
  formatMetadataSource,
  formatPlaytime,
  formatSessionExitStatus,
} from '../utils/formatters';
import styles from './Pages.module.css';

export interface GameDetailsPageProps {
  details: GameDetailsDto;
  onBack: () => void;
  onToggleFavorite: (gameId: string) => Promise<void>;
  onUpdateMetadata: (request: UpdateGameMetadataRequest) => Promise<void>;
  onSetCover: (gameId: string, sourceImagePath: string) => Promise<void>;
}

export function GameDetailsPage({
  details,
  onBack,
  onToggleFavorite,
  onUpdateMetadata,
  onSetCover,
}: GameDetailsPageProps) {
  const { game, versions, sessions } = details;
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
  const targetPhase = game.consoleSlug === 'ps3' ? 'Fase 3' : 'Fase 5';

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
      <header className={styles.header}>
        <div>
          <h1 className={styles.title}>{game.title}</h1>
          <p className={styles.subtitle}>
            Console: {game.consoleSlug.toUpperCase()} · Origem:{' '}
            {formatMetadataSource(game.metadataSource)} · Tempo jogado:{' '}
            {formatPlaytime(game.totalPlaytimeSeconds)} · Progresso: {game.completionPercent}%
          </p>
        </div>
        <div className={styles.actionsRow}>
          <button type="button" className={styles.btnSecondary} onClick={onBack}>
            Voltar
          </button>
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
      </header>

      <div className={`${styles.banner} ${styles.bannerInfo}`}>
        Execução ao vivo de títulos pelo emulador real entra na <strong>{targetPhase}</strong>. Na
        Fase 2, este painel gerencia metadados, capa local segura, versões e histórico de sessões
        sem botões simulados de execução.
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

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Capa Local (Cache Seguro SHA-256)</h2>
        {coverUrl ? (
          <div>
            <img
              src={coverUrl}
              alt={`Capa de ${game.title}`}
              style={{
                maxWidth: '220px',
                maxHeight: '280px',
                borderRadius: '8px',
                border: '1px solid var(--border-subtle)',
                objectFit: 'cover',
              }}
            />
          </div>
        ) : (
          <p className={styles.muted}>
            {game.coverPath
              ? `Capa armazenada em cache autorizado: ${game.coverPath}`
              : 'Nenhuma capa local definida ainda.'}
          </p>
        )}

        <form onSubmit={handleSetCover} className={styles.formGrid}>
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
          <div className={styles.actionsRow} style={{ alignSelf: 'end' }}>
            <button type="submit" className={styles.btnSecondary} disabled={busy}>
              Definir Capa Local
            </button>
          </div>
        </form>
      </section>

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
        <h2 className={styles.sectionTitle}>Sessões Reais ({sessions.length})</h2>
        {sessions.length === 0 ? (
          <p className={styles.muted}>
            Nenhuma sessão gravada para este título ainda.
          </p>
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
