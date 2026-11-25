import { useMemo, useState, type FormEvent } from 'react';
import { Star } from 'lucide-react';
import type { ConsoleRecord, GameRecord, RegisterGameRequest } from '../types/domain';
import styles from './Pages.module.css';

export interface LibraryPageProps {
  consoles: ConsoleRecord[];
  games: GameRecord[];
  onRegisterGame: (request: RegisterGameRequest) => Promise<void>;
  onToggleFavorite: (gameId: string) => Promise<void>;
}

export function LibraryPage({
  consoles,
  games,
  onRegisterGame,
  onToggleFavorite,
}: LibraryPageProps) {
  const [search, setSearch] = useState('');
  const [consoleFilter, setConsoleFilter] = useState('all');
  const [favoritesOnly, setFavoritesOnly] = useState(false);

  const [consoleSlug, setConsoleSlug] = useState(consoles[0]?.slug ?? 'ps3');
  const [title, setTitle] = useState('');
  const [versionLabel, setVersionLabel] = useState('v1.00');
  const [region, setRegion] = useState('USA');
  const [serialCode, setSerialCode] = useState('');
  const [filePath, setFilePath] = useState('');
  const [feedback, setFeedback] = useState<{ kind: 'ok' | 'err'; text: string } | null>(null);
  const [submitting, setSubmitting] = useState(false);

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

  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault();
    setFeedback(null);
    setSubmitting(true);
    try {
      await onRegisterGame({
        consoleSlug,
        title,
        versionLabel,
        region,
        serialCode: serialCode.trim() ? serialCode.trim() : null,
        filePath,
        metadataSource: 'manual_user_entry',
      });
      setTitle('');
      setSerialCode('');
      setFilePath('');
      setFeedback({ kind: 'ok', text: 'Jogo e versão registrados no SQLite com sucesso.' });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <h1 className={styles.title}>Biblioteca Universal</h1>
          <p className={styles.subtitle}>
            Identidade canônica de jogos separada de versões e regiões, com origem identificável de
            metadados.
          </p>
        </div>
      </header>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Registrar Jogo Local no Banco</h2>
        <form onSubmit={handleSubmit} className={styles.page}>
          <div className={styles.formGrid}>
            <div className={styles.field}>
              <label className={styles.label} htmlFor="game-console">
                Console
              </label>
              <select
                id="game-console"
                className={styles.select}
                value={consoleSlug}
                onChange={(e) => setConsoleSlug(e.target.value)}
              >
                {consoles.map((c) => (
                  <option key={c.slug} value={c.slug}>
                    {c.name}
                  </option>
                ))}
              </select>
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="game-title">
                Título do jogo
              </label>
              <input
                id="game-title"
                className={styles.input}
                required
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="Ex.: Demon's Souls"
              />
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="game-serial">
                Código Serial (opcional)
              </label>
              <input
                id="game-serial"
                className={styles.input}
                value={serialCode}
                onChange={(e) => setSerialCode(e.target.value)}
                placeholder="Ex.: BLUS-30443"
              />
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="game-region">
                Região
              </label>
              <input
                id="game-region"
                className={styles.input}
                value={region}
                onChange={(e) => setRegion(e.target.value)}
              />
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="game-version">
                Versão
              </label>
              <input
                id="game-version"
                className={styles.input}
                value={versionLabel}
                onChange={(e) => setVersionLabel(e.target.value)}
              />
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="game-path">
                Caminho local do jogo
              </label>
              <input
                id="game-path"
                className={styles.input}
                required
                value={filePath}
                onChange={(e) => setFilePath(e.target.value)}
                placeholder="D:\PS3\DemonsSouls\PS3_GAME\USRDIR\EBOOT.BIN"
              />
            </div>
          </div>

          <div className={styles.actionsRow}>
            <button type="submit" className={styles.btnPrimary} disabled={submitting}>
              {submitting ? 'Registrando...' : 'Registrar Jogo'}
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
      </section>

      <section className={styles.section}>
        <div className={styles.header}>
          <h2 className={styles.sectionTitle}>Jogos Registrados ({filteredGames.length})</h2>
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
            Nenhum jogo corresponde aos filtros atuais. O scanner automático de diretórios entrará
            na Fase 2.
          </p>
        ) : (
          <ul className={styles.list}>
            {filteredGames.map((game) => (
              <li key={game.id} className={styles.listItem}>
                <div>
                  <strong>{game.title}</strong>
                  <div className={styles.muted}>
                    Console: {game.consoleSlug.toUpperCase()} · Chave: {game.canonicalKey} ·
                    Versões: {game.versionCount} · Origem: {game.metadataSource}
                  </div>
                </div>
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
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}
