# Universal Player Profile — Universal Console Hub 3.0

## Identidade Offline Unificada
O perfil do Universal Console Hub funciona 100% offline e é independente dos perfis internos de cada console emulado.

### Entidades Principais
- `users` + `profiles`: ID único estável, `username` único, `display_name`, `avatar_path`, `banner_path`, `bio`, `created_at` (UTC), `preferences_json`, `privacy_level`.
- `games` + `game_versions`: Identidade canônica do jogo (`console_id`, `canonical_key`, `title`, metadados e `metadata_source`) separada das versões/regiões/caminhos específicos (`game_versions`).
- `game_sessions`: Sessões reais com `started_at`, `ended_at`, `last_heartbeat_at`, `duration_seconds`, `exit_status`.
- `statistics`, `favorites`, `activity_history`: Derivados exclusivamente de registros reais, sem dados históricos fabricados.

## Escopo da Fase 1 vs. Fase 2
- **Fase 1:** Schema completo, perfil local padrão editável e persistido em SQLite, cadastro/listagem/favoritos na biblioteca base e estatísticas honestas derivadas do banco.
- **Fase 2:** Scanner assíncrono de bibliotecas, deduplicação avançada multi-versão, cache de capas e motor completo de estatísticas mensais.
