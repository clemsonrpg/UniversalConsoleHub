# Database Schema — Universal Console Hub 3.0

## Configuração Obrigatória por Conexão SQLite
```sql
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA foreign_keys = ON;
PRAGMA busy_timeout = 5000;
PRAGMA cache_size = -64000;
PRAGMA temp_store = MEMORY;
```

## Política de Compatibilidade de Migrações
- Versionamento controlado por `PRAGMA user_version` (`1` na Fase 1; `2` implementada na Fase 2; `3` planejada para a Fase 3).
- **Total de tabelas:** `19` tabelas `STRICT` na Migration `0001` (Fase 1) + `3` novas tabelas `STRICT` na Migration `0002` (Fase 2) + `2` novas tabelas `STRICT` na Migration `0003` (Fase 3) = **`24` tabelas `STRICT`** ao concluir a Migration `0003`.
- **Transacionalidade:** Cada migração executa dentro de `BEGIN IMMEDIATE` com verificação obrigatória de `PRAGMA foreign_key_check` e `PRAGMA quick_check` antes do `COMMIT`.
- **Estritamente aditiva e não-destrutiva:** Migrações jamais descartam ou recriam tabelas, dados, índices, constraints ou triggers existentes (`trg_trophies_immutable_origin`, `trg_trophy_unlocks_immutable_origin`).
- **Nota crítica sobre `game_versions`:** A tabela `game_versions` **já existe como Tabela #8 da Migration `0001`** (já referenciada por `game_sessions.game_version_id` e gravada em produção pela Fase 1). Portanto, as Migrations `0002` e `0003` **não recriam** `game_versions`; elas preservam integralmente os dados existentes e aplicam apenas índices (`0002`) e colunas aditivas com valor padrão/nulo (`0003`).
- **Proteção contra downgrade/versão futura:** Se `PRAGMA user_version > LATEST_SCHEMA_VERSION`, a abertura do banco **aborta com erro explícito sem modificar o arquivo**.
- **Validação obrigatória de upgrade:** Toda nova migração possui testes de integração reais migrando cópias populadas dos schemas anteriores (`v1 → v2`, `v1 → v3` e `v2 → v3`).

---

## 19 Tabelas `STRICT` — Migration `0001_initial_schema.sql` (Fase 1 — Implementada e Preservada)
1. `users` — Identidade de conta local (`id PK`, `username UNIQUE`, `created_at` UTC, `updated_at` UTC).
2. `profiles` — Perfil de jogador vinculado a `users(id) ON DELETE CASCADE` (`handle UNIQUE`, `display_name`, `avatar_path`, `banner_path`, `bio`, `privacy_level`, `preferences_json`, `is_active`).
3. `consoles` — Catálogo de consoles (`id PK`, `slug UNIQUE`, `name`, `manufacturer`, `generation`, `release_year`, `phase_status`, `capabilities_json`, `notes`).
4. `emulators` — Emuladores integrados (`id PK`, `slug UNIQUE`, `name`, `console_id FK → consoles(id) ON DELETE RESTRICT`, `executable_path`, `config_path`, `detected_version`, `integration_phase`, `capabilities_json`).
5. `emulator_versions` — Versões detectadas por emulador (`emulator_id FK → emulators(id) ON DELETE CASCADE`, `UNIQUE(emulator_id, version)`).
6. `console_firmware` — Registros de arquivos de firmware/BIOS validados localmente pelo usuário (`console_id FK`, `emulator_id FK`, `UNIQUE(console_id, file_role, file_path)`, `sha256`, `validation_status` — zero binários armazenados).
7. `games` — Identidade canônica do jogo (`id PK`, `console_id FK → consoles(id) ON DELETE RESTRICT`, `UNIQUE(console_id, canonical_key)`), metadados com `metadata_source` identificável, `associated_emulator_id FK → emulators(id) ON DELETE SET NULL`, `completion_percent` (`0..100`) e `cover_path` restrito ao cache autorizado.
8. `game_versions` *(1:N com `games` — criada na Migration `0001` e indexada adicionalmente na Migration `0002`)* — Versões/regiões/edições/arquivos do jogo:
   - `id TEXT PRIMARY KEY NOT NULL`
   - `game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE`
   - `version_label TEXT NOT NULL`, `region TEXT NOT NULL`, `serial_code TEXT`, `file_path TEXT NOT NULL`
   - `install_state TEXT NOT NULL CHECK (install_state IN ('registered', 'installed', 'missing_files'))`
   - `created_at TEXT NOT NULL CHECK (created_at LIKE '____-__-__T__:__:__%Z')`
   - `UNIQUE(game_id, region, version_label, file_path)`
   - Índice da Fase 1: `idx_game_versions_game(game_id)`.
9. `game_sessions` — Sessões reais de jogo (`profile_id FK`, `game_id FK`, `game_version_id FK → game_versions(id) ON DELETE SET NULL`, `emulator_id FK → emulators(id) ON DELETE RESTRICT`, `started_at`, `last_heartbeat_at`, `ended_at` UTC, `duration_seconds >= 0`, `exit_status`, incluindo `recovered_after_crash`).
10. `achievement_providers` — Providers (`local`, `emulator`, `playstation`, `xbox`, `steam`) com `origin` e `availability_status`.
11. `trophies` — Definições de troféus/conquistas (`UNIQUE(game_id, provider_id, external_trophy_id)`, `origin` imutável via trigger `trg_trophies_immutable_origin`).
12. `trophy_unlocks` — Desbloqueios (`UNIQUE(profile_id, trophy_id)`, `origin` imutável via trigger `trg_trophy_unlocks_immutable_origin`).
13. `platform_accounts` — Conexões opcionais de conta (`UNIQUE(profile_id, provider_id, external_account_id)` — zero senhas em texto puro).
14. `statistics` — Métricas consolidadas por perfil/escopo (`UNIQUE(profile_id, scope_type, scope_id)`).
15. `favorites` — Jogos favoritos por perfil (`UNIQUE(profile_id, game_id)`).
16. `activity_history` — Linha do tempo auditável de eventos reais do perfil.
17. `synchronization_history` — Log de execuções de sincronização de providers.
18. `controller_profiles` — Perfis de controle e calibração por dispositivo (`UNIQUE(profile_id, device_signature, profile_name)`).
19. `application_settings` — Configurações globais persistidas chave/valor (`key PRIMARY KEY`).

---

## Schema Definitivo — Migration `0002_console_environments.sql` (Fase 2 — Implementada, `user_version = 2`)

A Migration `0002` é **100% aditiva e não-destrutiva**: cria **3 novas tabelas `STRICT` (#20–#22)** e adiciona **índices de performance não-destrutivos** nas novas tabelas e nas tabelas existentes `game_versions` (#8) e `game_sessions` (#9).

```sql
-- Universal Console Hub 3.0 — Console Environments & Verification Schema (Migration 0002)
-- Strictly additive & non-destructive over Migration 0001 (19 -> 22 STRICT tables).
-- Note: `game_versions` already exists as Table #8 in Migration 0001 and is preserved intact.

-- Table #20: Independent discovery directories per console
CREATE TABLE console_scan_paths (
    id TEXT PRIMARY KEY NOT NULL,
    console_id TEXT NOT NULL REFERENCES consoles(id) ON DELETE CASCADE,
    directory_path TEXT NOT NULL,
    recursive INTEGER NOT NULL DEFAULT 1 CHECK (recursive IN (0, 1)),
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    last_scanned_at TEXT CHECK (last_scanned_at IS NULL OR last_scanned_at LIKE '____-__-__T__:__:__%Z'),
    created_at TEXT NOT NULL CHECK (created_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(console_id, directory_path)
) STRICT;

-- Table #21: Independent per-console configuration settings
CREATE TABLE console_settings (
    id TEXT PRIMARY KEY NOT NULL,
    console_id TEXT NOT NULL REFERENCES consoles(id) ON DELETE CASCADE,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL CHECK (updated_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(console_id, key)
) STRICT;

-- Table #22: Granular independent runtime verification records per console, emulator, feature, version, and optional game
-- Supports both console-level checks (game_id IS NULL, scope_key = 'console')
-- and per-title checks such as `per_game_compatibility` (game_id = <games.id>, scope_key = game_id).
CREATE TABLE console_runtime_verifications (
    id TEXT PRIMARY KEY NOT NULL,
    console_id TEXT NOT NULL REFERENCES consoles(id) ON DELETE CASCADE,
    emulator_id TEXT NOT NULL REFERENCES emulators(id) ON DELETE CASCADE,
    game_id TEXT REFERENCES games(id) ON DELETE CASCADE,
    scope_key TEXT NOT NULL DEFAULT 'console',
    feature_key TEXT NOT NULL CHECK (feature_key IN (
        'emulator_launch',
        'system_boot',
        'original_dashboard',
        'game_boot',
        'per_game_compatibility',
        'session_monitoring',
        'hub_return'
    )),
    emulator_version TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN (
        'unverified',
        'documented_upstream',
        'files_verified',
        'runtime_confirmed',
        'failed'
    )),
    evidence_notes TEXT NOT NULL DEFAULT '',
    verified_at TEXT NOT NULL CHECK (verified_at LIKE '____-__-__T__:__:__%Z'),
    CHECK ((game_id IS NULL AND scope_key = 'console') OR (game_id IS NOT NULL AND scope_key = game_id)),
    UNIQUE(console_id, emulator_id, feature_key, emulator_version, scope_key)
) STRICT;

-- Additive indexes for Migration 0002 tables
CREATE INDEX IF NOT EXISTS idx_console_scan_paths_console
    ON console_scan_paths(console_id);
CREATE INDEX IF NOT EXISTS idx_console_settings_console
    ON console_settings(console_id);
CREATE INDEX IF NOT EXISTS idx_console_runtime_verifications_console_emu
    ON console_runtime_verifications(console_id, emulator_id);
CREATE INDEX IF NOT EXISTS idx_console_runtime_verifications_game
    ON console_runtime_verifications(game_id) WHERE game_id IS NOT NULL;

-- Additive non-destructive indexes on existing Migration 0001 tables (`game_versions` #8 and `game_sessions` #9)
CREATE INDEX IF NOT EXISTS idx_game_versions_file_path
    ON game_versions(file_path);
CREATE INDEX IF NOT EXISTS idx_game_versions_serial
    ON game_versions(serial_code) WHERE serial_code IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_game_sessions_open
    ON game_sessions(ended_at) WHERE ended_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_game_sessions_game_profile
    ON game_sessions(game_id, profile_id, started_at DESC);
```

### Relacionamentos Verificados entre Entidades (Migration 0002)
- **`games` (1) → `game_versions` (N):** `game_versions.game_id REFERENCES games(id) ON DELETE CASCADE`. Cada jogo canônico (`UNIQUE(console_id, canonical_key)`) pode possuir múltiplas edições/regiões/seriais/caminhos em `game_versions` (`UNIQUE(game_id, region, version_label, file_path)`).
- **`game_sessions` (N) → `game_versions` (0..1):** `game_sessions.game_version_id REFERENCES game_versions(id) ON DELETE SET NULL`, preservando histórico de playtime caso uma versão local seja removida.
- **`consoles` (1) → `emulators` (N) + `console_runtime_verifications` (N):** `console_runtime_verifications` referencia tanto `consoles(id)` quanto `emulators(id)` (`ON DELETE CASCADE`), permitindo que emuladores multi-console (ex.: Dolphin para Wii e GameCube) mantenham verificações independentes por console. O par `(game_id, scope_key)` com `CHECK` garante unicidade real no SQLite (evitando o comportamento `NULL != NULL` em `UNIQUE`) tanto para verificações de console quanto para `per_game_compatibility` por título.

---

## Schema Definitivo — Migration `0003_console_profiles_and_runtime.sql` (Fase 3 — Planejada, `user_version = 3`)

A Migration `0003` é **100% aditiva e não-destrutiva**: cria **2 novas tabelas `STRICT` (#23–#24)** — separando estritamente a **identidade do console** (`console_profiles`) da **identidade de usuários locais reais do console** (`console_local_users`) — e adiciona colunas e índices não-destrutivos em `game_versions` (#8), `game_sessions` (#9) e `trophy_unlocks` (#12).

```sql
-- Universal Console Hub 3.0 — Real Console Profiles & Multi-User Readiness (Migration 0003)
-- Strictly additive & non-destructive over Migration 0002 (22 -> 24 STRICT tables).

-- Table #23: Persistent Console Profile Snapshot (Console identity, NEVER player identity)
CREATE TABLE console_profiles (
    id TEXT PRIMARY KEY NOT NULL,
    console_id TEXT NOT NULL UNIQUE REFERENCES consoles(id) ON DELETE CASCADE,
    emulator_id TEXT REFERENCES emulators(id) ON DELETE SET NULL,
    emulator_status TEXT NOT NULL DEFAULT 'unconfigured' CHECK (emulator_status IN (
        'unconfigured',
        'available',
        'missing_on_disk'
    )),
    detected_emulator_version TEXT,
    detected_firmware_version TEXT,
    emulator_root_dir TEXT,
    native_save_count INTEGER NOT NULL DEFAULT 0 CHECK (native_save_count >= 0),
    native_trophy_dir_present INTEGER NOT NULL DEFAULT 0 CHECK (native_trophy_dir_present IN (0, 1)),
    trophy_sync_readiness TEXT NOT NULL DEFAULT 'unavailable' CHECK (trophy_sync_readiness IN (
        'unavailable',
        'phase4_planned',
        'ready'
    )),
    native_metadata_json TEXT NOT NULL DEFAULT '{}',
    sync_status TEXT NOT NULL DEFAULT 'never_synced' CHECK (sync_status IN (
        'never_synced',
        'synced',
        'partial',
        'error'
    )),
    sync_diagnostics TEXT NOT NULL DEFAULT '',
    last_synced_at TEXT CHECK (last_synced_at IS NULL OR last_synced_at LIKE '____-__-__T__:__:__%Z'),
    updated_at TEXT NOT NULL CHECK (updated_at LIKE '____-__-__T__:__:__%Z')
) STRICT;

-- Table #24: Real Local Console Users (only populated when the platform actually exposes local users, e.g. RPCS3 dev_hdd0/home/<id>/localusername)
CREATE TABLE console_local_users (
    id TEXT PRIMARY KEY NOT NULL,
    console_id TEXT NOT NULL REFERENCES consoles(id) ON DELETE CASCADE,
    native_user_key TEXT NOT NULL,
    display_name TEXT NOT NULL,
    source_path TEXT NOT NULL,
    availability_status TEXT NOT NULL DEFAULT 'available' CHECK (availability_status IN (
        'available',
        'missing_on_disk'
    )),
    linked_profile_id TEXT REFERENCES profiles(id) ON DELETE SET NULL,
    first_detected_at TEXT NOT NULL CHECK (first_detected_at LIKE '____-__-__T__:__:__%Z'),
    last_seen_at TEXT NOT NULL CHECK (last_seen_at LIKE '____-__-__T__:__:__%Z'),
    UNIQUE(console_id, native_user_key)
) STRICT;

-- Additive columns on existing tables (preserving 100% of existing rows, constraints and triggers)
ALTER TABLE game_versions
    ADD COLUMN availability_status TEXT NOT NULL DEFAULT 'available'
    CHECK (availability_status IN ('available', 'missing_on_disk'));

ALTER TABLE game_versions
    ADD COLUMN last_verified_at TEXT
    CHECK (last_verified_at IS NULL OR last_verified_at LIKE '____-__-__T__:__:__%Z');

ALTER TABLE game_sessions
    ADD COLUMN console_local_user_id TEXT REFERENCES console_local_users(id) ON DELETE SET NULL;

ALTER TABLE trophy_unlocks
    ADD COLUMN console_local_user_id TEXT REFERENCES console_local_users(id) ON DELETE SET NULL;

-- Additive performance indexes
CREATE INDEX IF NOT EXISTS idx_console_local_users_console
    ON console_local_users(console_id);
CREATE INDEX IF NOT EXISTS idx_game_versions_availability
    ON game_versions(game_id, availability_status);
CREATE INDEX IF NOT EXISTS idx_game_sessions_local_user
    ON game_sessions(console_local_user_id) WHERE console_local_user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_trophy_unlocks_local_user
    ON trophy_unlocks(console_local_user_id) WHERE console_local_user_id IS NOT NULL;
```

### Garantias da Migration `0003`
- **Separação Console vs. Jogador:** `console_profiles` (#23) nunca mistura nome de jogador com identidade de hardware/emulador; `console_local_users` (#24) suporta `0..N` usuários locais reais por console (nunca inventando registros quando a plataforma não fornece usuários).
- **Preservação de Histórico:** `availability_status = 'missing_on_disk'` em `game_versions` e `console_local_users` + `emulator_status = 'missing_on_disk'` em `console_profiles` garantem que nenhuma sessão, estatística ou jogo histórico seja apagado quando arquivos forem movidos ou removidos.
- **Prontidão para Conquistas Completas (Fase 4):** `trophies` (#11) já armazena `external_trophy_id`, `name`, `description`, `icon_path`, `category`, `rarity_percent` e `origin` (imutável); `trophy_unlocks` (#12) já armazena `unlocked_at`, `synced_at`, `origin` (imutável) e passa a aceitar `console_local_user_id` opcional para atribuição futura por usuário local do console na Fase 4.

---

## Protocolo Seguro de Backup e Restauração no Windows
- **Criação:** `VACUUM INTO ?1` (consistente em modo WAL com conexões abertas) seguido de validação read-only `PRAGMA quick_check` e `PRAGMA foreign_key_check`.
- **Inspeção Pré-Restauração (`inspect_backup`):** Abre o backup em modo somente leitura, executa `PRAGMA quick_check`, `PRAGMA foreign_key_check`, confirma `1 <= user_version <= LATEST_SCHEMA_VERSION` e coleta contagem de perfis/jogos para confirmação explícita na UI.
- **Restauração Ao Vivo Segura (`restore_from_backup_live`):**
  1. Revalida integralmente o arquivo de backup (`quick_check`, `foreign_key_check`, `1 <= user_version <= LATEST_SCHEMA_VERSION`); aborta sem tocar no banco ativo se inválido ou de versão futura.
  2. Gera snapshot pré-restauração consistente (`pre_restore_<utc>.db`) via `VACUUM INTO` e valida sua integridade.
  3. Gera cópia limpa do backup via `VACUUM INTO <target>.restore.tmp`.
  4. Sob lock exclusivo de `Mutex<Connection>`, substitui temporariamente a conexão ativa por uma conexão in-memory para liberar 100% os handles OS do `.db`, `-wal` e `-shm` no Windows.
  5. Move o `.db` atual para `.db.rollback.tmp`, limpa `-wal` e `-shm` obsoletos e promove `.restore.tmp` para `.db`.
  6. Reabre a conexão com `WAL` + `foreign_keys = ON` + demais pragmas; se o backup restaurado era `v1` (`user_version < LATEST_SCHEMA_VERSION`), aplica `apply_migrations` + `seed_defaults` de forma transacional; executa `PRAGMA quick_check` e `PRAGMA foreign_key_check`; somente então remove `.db.rollback.tmp`.
  7. **Recuperação automática de falha:** Se qualquer etapa entre (5) e (6) falhar, restaura `.db.rollback.tmp` (ou o snapshot `pre_restore_<utc>.db`), reabre o banco original intacto e retorna erro claro — o banco ativo nunca fica parcialmente restaurado ou corrompido.
