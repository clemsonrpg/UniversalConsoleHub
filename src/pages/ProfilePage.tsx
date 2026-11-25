import { useState, type FormEvent } from 'react';
import { StatTile } from '../components/StatTile';
import type { PlayerProfile, ProfileStatistics, UpdateProfileRequest } from '../types/domain';
import styles from './Pages.module.css';

export interface ProfilePageProps {
  profile: PlayerProfile;
  statistics: ProfileStatistics;
  onUpdateProfile: (request: UpdateProfileRequest) => Promise<void>;
}

export function ProfilePage({ profile, statistics, onUpdateProfile }: ProfilePageProps) {
  const [username, setUsername] = useState(profile.username);
  const [displayName, setDisplayName] = useState(profile.displayName);
  const [bio, setBio] = useState(profile.bio);
  const [privacyLevel, setPrivacyLevel] = useState<'private' | 'friends' | 'public'>(
    profile.privacyLevel
  );
  const [feedback, setFeedback] = useState<{ kind: 'ok' | 'err'; text: string } | null>(null);
  const [saving, setSaving] = useState(false);

  const handleSubmit = async (event: FormEvent) => {
    event.preventDefault();
    setFeedback(null);
    setSaving(true);
    try {
      await onUpdateProfile({ username, displayName, bio, privacyLevel });
      setFeedback({ kind: 'ok', text: 'Perfil salvo com sucesso no banco local SQLite.' });
    } catch (err) {
      setFeedback({ kind: 'err', text: String(err) });
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div>
          <h1 className={styles.title}>Perfil Universal</h1>
          <p className={styles.subtitle}>
            Identidade offline independente dos consoles individuais. Criado em{' '}
            <span className={styles.mono}>{profile.createdAt}</span>.
          </p>
        </div>
      </header>

      <section className={styles.statGrid}>
        <StatTile label="Jogos registrados" value={statistics.gamesRegistered} />
        <StatTile label="Jogos iniciados" value={statistics.gamesStarted} />
        <StatTile label="Jogos concluídos" value={statistics.gamesCompleted} />
        <StatTile label="Troféus raros" value={statistics.rareTrophies} />
      </section>

      <section className={styles.section}>
        <h2 className={styles.sectionTitle}>Personalizar Identidade Local</h2>
        <form onSubmit={handleSubmit} className={styles.page}>
          <div className={styles.formGrid}>
            <div className={styles.field}>
              <label className={styles.label} htmlFor="profile-username">
                Username
              </label>
              <input
                id="profile-username"
                className={styles.input}
                required
                value={username}
                onChange={(e) => setUsername(e.target.value)}
              />
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="profile-display-name">
                Nome de exibição
              </label>
              <input
                id="profile-display-name"
                className={styles.input}
                required
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
              />
            </div>

            <div className={styles.field}>
              <label className={styles.label} htmlFor="profile-privacy">
                Privacidade
              </label>
              <select
                id="profile-privacy"
                className={styles.select}
                value={privacyLevel}
                onChange={(e) =>
                  setPrivacyLevel(e.target.value as 'private' | 'friends' | 'public')
                }
              >
                <option value="private">Privado (Somente Offline)</option>
                <option value="friends">Amigos (Preparado p/ Expansão)</option>
                <option value="public">Público (Preparado p/ Expansão)</option>
              </select>
            </div>
          </div>

          <div className={styles.field}>
            <label className={styles.label} htmlFor="profile-bio">
              Biografia
            </label>
            <textarea
              id="profile-bio"
              className={styles.textarea}
              rows={3}
              value={bio}
              onChange={(e) => setBio(e.target.value)}
            />
          </div>

          <div className={styles.actionsRow}>
            <button type="submit" className={styles.btnPrimary} disabled={saving}>
              {saving ? 'Salvando...' : 'Salvar Perfil'}
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
    </div>
  );
}
