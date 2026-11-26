# Universal Trophy System — Universal Console Hub 3.0

## Regra Absoluta de Integridade & Proveniência
Toda conquista/troféu possui uma **origem imutável** (`origin`):
- `official` — Exclusivamente proveniente de API oficial autorizada (nenhuma ativa sem API oficial; PSN e Xbox Live permanecem `unavailable_no_official_api`).
- `emulator` — Lida **estritamente em modo somente leitura** de arquivos locais de emulador (RPCS3 PS3 `dev_hdd0/home/<user>/trophy/<NPCOMMID>/` — **implementado na Fase 4**) ou provedor de emulação autorizado (RetroAchievements com API key do usuário — **Fase 6**).
- `local` — Conquistas locais próprias do Universal Console Hub (`LocalAchievementProvider`).

## Proteções Técnicas & Invariantes (Fases 1–4)
1. **Imutabilidade de Origem no SQLite:** Triggers `BEFORE UPDATE` (`trg_trophies_immutable_origin` e `trg_trophy_unlocks_immutable_origin`) abortam qualquer tentativa de alterar `origin` após a inserção.
2. **Prevenção de Duplicatas & Não-Regressão:**
   - `trophy_sets` (#25): `UNIQUE(provider_id, external_set_id)`.
   - `trophies` (#11): `UNIQUE(game_id, provider_id, external_trophy_id)` com `trophy_set_id` e `is_hidden` (`Migration 0004`).
   - `trophy_unlocks` (#12): `UNIQUE(profile_id, trophy_id)` com `console_local_user_id` opcional. Re-sincronizações preservam sempre o timestamp `unlocked_at` UTC mais antigo verificado e **jamais** regridem um troféu já desbloqueado para bloqueado.
3. **Leitor Local Somente Leitura do RPCS3 (`Rpcs3TrophyReader` — Fase 4):**
   - Lê `TROPCONF.SFM` / `TROP.SFM` (XML com `npcommid`, `title-name`, `title-detail` e nós `<trophy id hidden ttype>` mapeando `B/S/G/P` → Bronze/Prata/Ouro/Platina).
   - Lê `TROPUSR.DAT` (binário big-endian checado por limites sem `panic!`, magic `0x81 0x8F 0x54 0xAD`, tabela tipo `6` com estado de desbloqueio e timestamp em microssegundos desde a época PS3 `0001-01-01`, convertido para RFC3339 UTC).
   - Descobre e importa ícones locais (`ICON0.PNG`, `TROPxxx.PNG`) exclusivamente através do `CoverCache` (validação de magic bytes + SHA-256 em `<app_local_data_dir>/cache/covers/`).
   - **Jamais** escreve, renomeia ou altera qualquer arquivo dentro do diretório do RPCS3.
4. **Sincronização Automática Pós-Sessão & Manual (`TrophySyncService` — Fase 4):**
   - Acoplada automaticamente ao encerramento de sessões reais no `LiveSessionOrchestrator` (`monitor_live_process`), além de acionamento manual via **Sincronizar Troféus Agora** (`sync_console_trophies`).
   - Emite `trophies:synced` em toda sincronização e `trophies:unlocked` apenas para troféus verdadeiramente inéditos (`newly_unlocked == true`), evitando notificações duplicadas.
   - Registra histórico auditável em `synchronization_history` e `activity_history` e recalcula `games.completion_percent`.
5. **Raridade Honesta & Proteção de Spoilers:**
   - Troféus locais offline do RPCS3 mantêm `rarity_percent = NULL` e exibem explicitamente **`Local / Offline`** (nunca inventando percentuais globais).
   - Troféus ocultos ainda bloqueados (`is_hidden = 1` e não desbloqueados) ocultam nome/descrição por padrão na UI, com toggle explícito **Revelar troféus ocultos (spoilers)**.
6. **Providers Arquiteturais:**
   - `LocalAchievementProvider` (`local` — `ready_offline`)
   - `EmulatorAchievementProvider` (`emulator` — `ready_offline`, `supports_sync = 1`, ativo para RPCS3 PS3 na Fase 4)
   - `RetroAchievementsProvider` (`emulator` — `planned_phase_6_authorized_key`)
   - `PlayStationTrophyProvider`, `XboxAchievementProvider`, `SteamAchievementProvider` (`official` — `unavailable_no_official_api` / `planned_phase_6_authorized_key`, nunca falsificados).
