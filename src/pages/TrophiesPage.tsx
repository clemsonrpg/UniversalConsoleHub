import { useMemo, useState } from 'react';
import { Award, Eye, EyeOff, Lock, RefreshCw, Trophy } from 'lucide-react';
import { resolveCoverAssetUrl } from '../api/tauriClient';
import type {
  DetailedTrophyRow,
  TrophyCategory,
  TrophyCenterOverviewDto,
  TrophyOrigin,
  TrophySyncReport,
} from '../types/domain';
import {
  formatAvailabilityStatus,
  formatTrophyCategory,
  formatTrophyOrigin,
} from '../utils/formatters';
import styles from './Pages.module.css';
import trophyStyles from './TrophiesPage.module.css';

export interface TrophiesPageProps {
  overview: TrophyCenterOverviewDto;
  onSyncConsoleTrophies?: (consoleSlug: string) => Promise<TrophySyncReport>;
  onOpenGame?: (gameId: string) => void;
}

function categoryPillClass(category: TrophyCategory): string {
  switch (category) {
    case 'platinum':
      return trophyStyles.pillPlatinum ?? '';
    case 'gold':
      return trophyStyles.pillGold ?? '';
    case 'silver':
      return trophyStyles.pillSilver ?? '';
    case 'bronze':
      return trophyStyles.pillBronze ?? '';
    default:
      return '';
  }
}

export function TrophyItemCard({
  trophy,
  showHiddenSpoilers,
  showGameTitle = true,
}: {
  trophy: DetailedTrophyRow;
  showHiddenSpoilers: boolean;
  showGameTitle?: boolean;
}) {
  const masked = trophy.isHidden && !trophy.isUnlocked && !showHiddenSpoilers;
  const iconUrl = masked ? null : resolveCoverAssetUrl(trophy.iconPath);

  return (
    <li
      className={`${trophyStyles.trophyRow} ${
        !trophy.isUnlocked ? trophyStyles.trophyRowLocked : ''
      }`}
    >
      <div className={trophyStyles.trophyMain}>
        <div className={trophyStyles.trophyIconBox} aria-hidden="true">
          {iconUrl ? (
            <img src={iconUrl} alt="" className={trophyStyles.trophyIconImg} />
          ) : masked ? (
            <EyeOff size={18} />
          ) : trophy.isUnlocked ? (
            <Trophy size={18} />
          ) : (
            <Lock size={18} />
          )}
        </div>

        <div className={trophyStyles.trophyBody}>
          <div className={trophyStyles.trophyNameRow}>
            <span className={trophyStyles.trophyName}>
              {masked ? 'Troféu Oculto' : trophy.name}
            </span>
            <span className={`${trophyStyles.pill} ${categoryPillClass(trophy.category)}`}>
              {formatTrophyCategory(trophy.category)}
            </span>
            <span className={trophyStyles.pill}>{formatTrophyOrigin(trophy.origin)}</span>
            {trophy.isHidden ? (
              <span className={`${trophyStyles.pill} ${trophyStyles.pillHidden}`}>Oculto</span>
            ) : null}
          </div>

          <p className={trophyStyles.trophyDesc}>
            {masked
              ? 'Descrição oculta até o desbloqueio para evitar spoilers. Ative "Revelar troféus ocultos" para visualizar.'
              : trophy.description || 'Sem descrição registrada nos arquivos locais.'}
          </p>

          <div className={trophyStyles.trophyMeta}>
            {showGameTitle ? <span>Jogo: {trophy.gameTitle}</span> : null}
            <span>Console: {trophy.consoleSlug.toUpperCase()}</span>
            <span>
              Raridade:{' '}
              {trophy.rarityPercent !== null ? `${trophy.rarityPercent}%` : 'Local / Offline'}
            </span>
            {trophy.consoleLocalUserName ? (
              <span>Usuário local: {trophy.consoleLocalUserName}</span>
            ) : null}
          </div>
        </div>
      </div>

      <div className={trophyStyles.trophyRight}>
        <span
          className={`${trophyStyles.pill} ${
            trophy.isUnlocked ? trophyStyles.pillUnlocked : ''
          }`}
        >
          {trophy.isUnlocked ? 'Desbloqueado' : 'Bloqueado'}
        </span>
        <span className={styles.mono}>
          {trophy.isUnlocked ? trophy.unlockedAt ?? 'Desbloqueado' : '—'}
        </span>
      </div>
    </li>
  );
}

export function TrophiesPage({
  overview,
  onSyncConsoleTrophies,
  onOpenGame,
}: TrophiesPageProps) {
  const { providers, sets, trophies, recentUnlocks, totals } = overview;

  const [consoleFilter, setConsoleFilter] = useState<string>('all');
  const [originFilter, setOriginFilter] = useState<TrophyOrigin | 'all'>('all');
  const [statusFilter, setStatusFilter] = useState<'all' | 'unlocked' | 'locked'>('all');
  const [selectedSetId, setSelectedSetId] = useState<string>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [showHiddenSpoilers, setShowHiddenSpoilers] = useState(false);
  const [syncing, setSyncing] = useState(false);
  const [syncFeedback, setSyncFeedback] = useState<{ kind: 'ok' | 'err'; text: string } | null>(
    null
  );

  const availableConsoles = useMemo(() => {
    const unique = new Set<string>();
    for (const s of sets) unique.add(s.consoleSlug);
    for (const t of trophies) unique.add(t.consoleSlug);
    return Array.from(unique).sort();
  }, [sets, trophies]);

  const filteredTrophies = useMemo(() => {
    const q = searchQuery.trim().toLowerCase();
    return trophies.filter((t) => {
      if (consoleFilter !== 'all' && t.consoleSlug !== consoleFilter) return false;
      if (originFilter !== 'all' && t.origin !== originFilter) return false;
      if (statusFilter === 'unlocked' && !t.isUnlocked) return false;
      if (statusFilter === 'locked' && t.isUnlocked) return false;
      if (selectedSetId !== 'all' && t.trophySetId !== selectedSetId) return false;
      if (q.length > 0) {
        const masked = t.isHidden && !t.isUnlocked && !showHiddenSpoilers;
        const hay = masked
          ? `${t.gameTitle} troféu oculto`.toLowerCase()
          : `${t.name} ${t.description} ${t.gameTitle}`.toLowerCase();
        if (!hay.includes(q)) return false;
      }
      return true;
    });
  }, [
    trophies,
    consoleFilter,
    originFilter,
    statusFilter,
    selectedSetId,
    searchQuery,
    showHiddenSpoilers,
  ]);

  const handleSyncPs3 = async () => {
    if (!onSyncConsoleTrophies) return;
    setSyncFeedback(null);
    setSyncing(true);
    try {
      const report = await onSyncConsoleTrophies('ps3');
      const diagNote =
        report.diagnostics.length > 0 ? ` Avisos: ${report.diagnostics.join(' | ')}` : '';
      setSyncFeedback({
        kind: 'ok',
        text: `Sincronização RPCS3 concluída: ${report.setsSynced} conjunto(s), ${report.trophiesSynced} troféu(s), ${report.unlocksRecorded} desbloqueio(s) verificado(s), ${report.newlyUnlocked.length} novo(s), ${report.iconsCached} ícone(s) em cache.${diagNote}`,
      });
    } catch (err) {
      setSyncFeedback({ kind: 'err', text: String(err) });
    } finally {
      setSyncing(false);
    }
  };

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <h1 className={styles.title}>Central de Troféus</h1>
          <p className={styles.subtitle}>
            Leitura local somente leitura de conjuntos de troféus do RPCS3 (TROPCONF.SFM e
            TROPUSR.DAT), com separação imutável entre origens oficiais, de emulador e locais.
            Troféus offline nunca exibem raridade global fictícia.
          </p>
        </div>
        {onSyncConsoleTrophies ? (
          <div className={styles.actionsRow}>
            <button
              type="button"
              className={styles.btnPrimary}
              disabled={syncing}
              onClick={() => void handleSyncPs3()}
            >
              <RefreshCw
                size={14}
                aria-hidden="true"
                style={{ marginRight: 6, verticalAlign: 'middle' }}
              />
              {syncing ? 'Sincronizando...' : 'Sincronizar Troféus Agora'}
            </button>
          </div>
        ) : null}
      </header>

      {syncFeedback ? (
        <div
          role="status"
          aria-live="polite"
          className={`${styles.banner} ${
            syncFeedback.kind === 'ok' ? styles.bannerSuccess : styles.bannerError
          }`}
        >
          {syncFeedback.text}
        </div>
      ) : null}

      <section className={styles.section} aria-label="Resumo geral de troféus">
        <div className={trophyStyles.summaryGrid}>
          <div className={`${trophyStyles.metalCard} ${trophyStyles.accentOverall}`}>
            <span className={trophyStyles.metalLabel}>
              Conclusão Geral <Award size={14} aria-hidden="true" />
            </span>
            <span className={trophyStyles.metalValue}>{totals.completionPercent}%</span>
            <div className={trophyStyles.progressTrack} aria-hidden="true">
              <div
                className={trophyStyles.progressFill}
                style={{ width: `${totals.completionPercent}%` }}
              />
            </div>
            <span className={trophyStyles.metalSub}>
              {totals.unlockedTrophies} / {totals.totalTrophies} troféus · {totals.completedSets}/
              {totals.totalSets} sets 100%
            </span>
          </div>

          <div className={`${trophyStyles.metalCard} ${trophyStyles.accentPlatinum}`}>
            <span className={trophyStyles.metalLabel}>Platina</span>
            <span className={trophyStyles.metalValue}>{totals.platinumUnlocked}</span>
            <span className={trophyStyles.metalSub}>de {totals.platinumTotal} disponíveis</span>
          </div>

          <div className={`${trophyStyles.metalCard} ${trophyStyles.accentGold}`}>
            <span className={trophyStyles.metalLabel}>Ouro</span>
            <span className={trophyStyles.metalValue}>{totals.goldUnlocked}</span>
            <span className={trophyStyles.metalSub}>de {totals.goldTotal} disponíveis</span>
          </div>

          <div className={`${trophyStyles.metalCard} ${trophyStyles.accentSilver}`}>
            <span className={trophyStyles.metalLabel}>Prata</span>
            <span className={trophyStyles.metalValue}>{totals.silverUnlocked}</span>
            <span className={trophyStyles.metalSub}>de {totals.silverTotal} disponíveis</span>
          </div>

          <div className={`${trophyStyles.metalCard} ${trophyStyles.accentBronze}`}>
            <span className={trophyStyles.metalLabel}>Bronze</span>
            <span className={trophyStyles.metalValue}>{totals.bronzeUnlocked}</span>
            <span className={trophyStyles.metalSub}>de {totals.bronzeTotal} disponíveis</span>
          </div>
        </div>
      </section>

      <section className={styles.section}>
        <div className={styles.header}>
          <h2 className={styles.sectionTitle}>Conjuntos de Troféus por Jogo ({sets.length})</h2>
          {selectedSetId !== 'all' ? (
            <button
              type="button"
              className={styles.btnSecondary}
              onClick={() => setSelectedSetId('all')}
            >
              Mostrar Todos os Jogos
            </button>
          ) : null}
        </div>

        {sets.length === 0 ? (
          <p className={styles.muted}>
            Nenhum conjunto de troféus sincronizado ainda. Configure o executável do RPCS3 em
            Emuladores e clique em &ldquo;Sincronizar Troféus Agora&rdquo; — ou encerre uma sessão
            do PlayStation 3 para sincronizar automaticamente.
          </p>
        ) : (
          <div className={trophyStyles.setsGrid}>
            {sets.map((set) => {
              const setIconUrl = resolveCoverAssetUrl(set.iconPath);
              const isSelected = selectedSetId === set.id;
              return (
                <article
                  key={set.id}
                  className={`${trophyStyles.setCard} ${
                    isSelected ? trophyStyles.setCardSelected : ''
                  }`}
                >
                  <div className={trophyStyles.setTop}>
                    <div className={trophyStyles.setIconBox}>
                      {setIconUrl ? (
                        <img
                          src={setIconUrl}
                          alt={`Ícone do conjunto ${set.title}`}
                          className={trophyStyles.setIconImg}
                        />
                      ) : (
                        <span>{set.consoleSlug.toUpperCase()}</span>
                      )}
                    </div>
                    <div className={trophyStyles.setHeaderMeta}>
                      <h3 className={trophyStyles.setTitle}>{set.title}</h3>
                      <div className={styles.badgeRow}>
                        <span className={trophyStyles.pill}>{set.consoleSlug.toUpperCase()}</span>
                        <span className={`${trophyStyles.pill} ${styles.mono}`}>
                          {set.externalSetId}
                        </span>
                      </div>
                    </div>
                  </div>

                  <div>
                    <div className={trophyStyles.metalLabel}>
                      <span>
                        {set.unlockedCount} / {set.totalCount} troféus
                      </span>
                      <strong>{set.completionPercent}%</strong>
                    </div>
                    <div className={trophyStyles.progressTrack} aria-hidden="true">
                      <div
                        className={trophyStyles.progressFill}
                        style={{ width: `${set.completionPercent}%` }}
                      />
                    </div>
                  </div>

                  <div className={trophyStyles.metalCountsRow}>
                    <span>Platina: {set.platinumUnlocked}</span>
                    <span>Ouro: {set.goldUnlocked}</span>
                    <span>Prata: {set.silverUnlocked}</span>
                    <span>Bronze: {set.bronzeUnlocked}</span>
                  </div>

                  <div className={styles.actionsRow}>
                    <button
                      type="button"
                      className={styles.btnSecondary}
                      onClick={() => setSelectedSetId(isSelected ? 'all' : set.id)}
                    >
                      {isSelected ? 'Limpar Filtro' : 'Ver Troféus'}
                    </button>
                    {onOpenGame ? (
                      <button
                        type="button"
                        className={styles.btnSecondary}
                        onClick={() => onOpenGame(set.gameId)}
                      >
                        Abrir Jogo
                      </button>
                    ) : null}
                  </div>
                </article>
              );
            })}
          </div>
        )}
      </section>

      <section className={styles.section}>
        <div className={styles.header}>
          <h2 className={styles.sectionTitle}>
            Coleção de Troféus ({filteredTrophies.length} de {trophies.length})
          </h2>
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
            Revelar troféus ocultos (spoilers)
          </label>
        </div>

        <div className={trophyStyles.filtersBar}>
          <div className={styles.field}>
            <label className={styles.label} htmlFor="trophy-filter-console">
              Filtrar por console
            </label>
            <select
              id="trophy-filter-console"
              className={styles.select}
              value={consoleFilter}
              onChange={(e) => setConsoleFilter(e.target.value)}
            >
              <option value="all">Todos os consoles</option>
              {availableConsoles.map((slug) => (
                <option key={slug} value={slug}>
                  {slug.toUpperCase()}
                </option>
              ))}
            </select>
          </div>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="trophy-filter-origin">
              Filtrar por origem
            </label>
            <select
              id="trophy-filter-origin"
              className={styles.select}
              value={originFilter}
              onChange={(e) => setOriginFilter(e.target.value as TrophyOrigin | 'all')}
            >
              <option value="all">Todas as origens</option>
              <option value="emulator">Emulador (RPCS3 local)</option>
              <option value="official">Oficial</option>
              <option value="local">Local (Hub)</option>
            </select>
          </div>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="trophy-filter-status">
              Filtrar por status
            </label>
            <select
              id="trophy-filter-status"
              className={styles.select}
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value as 'all' | 'unlocked' | 'locked')}
            >
              <option value="all">Todos</option>
              <option value="unlocked">Desbloqueados</option>
              <option value="locked">Bloqueados</option>
            </select>
          </div>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="trophy-search">
              Buscar troféu ou jogo
            </label>
            <input
              id="trophy-search"
              className={styles.input}
              placeholder="Nome ou descrição..."
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
            />
          </div>
        </div>

        {filteredTrophies.length === 0 ? (
          <p className={styles.muted}>
            {trophies.length === 0
              ? 'Nenhum troféu importado ainda.'
              : 'Nenhum troféu corresponde aos filtros atuais.'}
          </p>
        ) : (
          <ul className={trophyStyles.trophyList}>
            {filteredTrophies.map((trophy) => (
              <TrophyItemCard
                key={trophy.id}
                trophy={trophy}
                showHiddenSpoilers={showHiddenSpoilers}
              />
            ))}
          </ul>
        )}
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>
          Linha do Tempo de Desbloqueios ({recentUnlocks.length})
        </h2>
        {recentUnlocks.length === 0 ? (
          <p className={styles.muted}>
            Nenhum desbloqueio cronológico registrado ainda. Desbloqueios lidos do TROPUSR.DAT
            aparecem aqui com timestamp UTC real.
          </p>
        ) : (
          <ul className={styles.list}>
            {recentUnlocks.map((u) => (
              <li key={`timeline-${u.id}`} className={styles.listItem}>
                <div>
                  <strong>{u.name}</strong> ·{' '}
                  <span className={`${trophyStyles.pill} ${categoryPillClass(u.category)}`}>
                    {formatTrophyCategory(u.category)}
                  </span>
                  <div className={styles.muted}>
                    {u.gameTitle} ({u.consoleSlug.toUpperCase()})
                    {u.consoleLocalUserName ? ` · Usuário local: ${u.consoleLocalUserName}` : ''}
                  </div>
                </div>
                <span className={styles.mono}>{u.unlockedAt ?? '—'}</span>
              </li>
            ))}
          </ul>
        )}
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Providers de Conquistas ({providers.length})</h2>
        <div className={styles.tableWrap}>
          <table className={styles.table}>
            <thead>
              <tr>
                <th scope="col">Provider</th>
                <th scope="col">Origem Imutável</th>
                <th scope="col">Estado Real</th>
                <th scope="col">Detalhes</th>
              </tr>
            </thead>
            <tbody>
              {providers.map((provider) => (
                <tr key={provider.id}>
                  <td>
                    <strong>{provider.name}</strong>
                  </td>
                  <td>{formatTrophyOrigin(provider.origin)}</td>
                  <td>{formatAvailabilityStatus(provider.availabilityStatus)}</td>
                  <td>{provider.notes}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}
