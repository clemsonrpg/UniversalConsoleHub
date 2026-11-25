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

## 19 Tabelas `STRICT` (Migration `0001_initial_schema.sql`)
1. `users` — Identidade de conta local (`id`, `username UNIQUE`, `created_at` UTC, `updated_at` UTC).
2. `profiles` — Perfil de jogador vinculado a `users` (`handle UNIQUE`, `display_name`, `avatar_path`, `banner_path`, `bio`, `privacy_level`, `preferences_json`).
3. `consoles` — Catálogo de consoles (`slug UNIQUE`, `name`, `manufacturer`, `capabilities_json`, `phase_status`).
4. `emulators` — Emuladores integrados (`slug UNIQUE`, `name`, `console_id`, `executable_path`, `config_path`, `capabilities_json`).
5. `emulator_versions` — Versões detectadas por emulador (`UNIQUE(emulator_id, version)`).
6. `console_firmware` — Registros de arquivos de firmware/BIOS validados localmente pelo usuário (`UNIQUE(console_id, file_path)`, `sha256`, `validation_status` — zero binários armazenados).
7. `games` — Identidade canônica do jogo (`UNIQUE(console_id, canonical_key)`), metadados com `metadata_source` identificável.
8. `game_versions` — Versões/regiões/edições físicas ou digitais do jogo (`UNIQUE(game_id, region, version_label, file_path)`).
9. `game_sessions` — Sessões reais de jogo com timestamps UTC e heartbeat para recuperação de crash.
10. `achievement_providers` — Providers (`local`, `emulator`, `playstation`, `xbox`, `steam`) com `origin` e `availability_status`.
11. `trophies` — Definições de troféus/conquistas (`UNIQUE(game_id, provider_id, external_trophy_id)`, `origin` imutável via trigger `trg_trophies_immutable_origin`).
12. `trophy_unlocks` — Desbloqueios (`UNIQUE(profile_id, trophy_id)`, `origin` imutável via trigger `trg_trophy_unlocks_immutable_origin`).
13. `platform_accounts` — Conexões opcionais de conta (`UNIQUE(profile_id, provider_id, external_account_id)` — zero senhas em texto puro).
14. `statistics` — Métricas consolidadas por perfil/escopo (`UNIQUE(profile_id, scope_type, scope_id)`).
15. `favorites` — Jogos favoritos por perfil (`UNIQUE(profile_id, game_id)`).
16. `activity_history` — Linha do tempo auditável de eventos reais do perfil.
17. `synchronization_history` — Log de execuções de sincronização de providers.
18. `controller_profiles` — Perfis de controle e calibração por dispositivo (`UNIQUE(profile_id, device_signature, profile_name)`).
19. `application_settings` — Configurações persistidas chave/valor (`key PRIMARY KEY`).

## Backup e Restauração Seguros
- Criação: `VACUUM INTO ?1` (consistente em modo WAL com conexões ativas) + `PRAGMA quick_check`.
- Restauração: Valida `PRAGMA quick_check` e `PRAGMA foreign_key_check` no arquivo de backup antes de restaurar sobre o banco alvo com substituição segura.
