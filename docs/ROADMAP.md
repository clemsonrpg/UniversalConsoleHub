# Roadmap — Universal Console Hub 3.0

> **Requisito Central Inegociável:** Universal Console Hub é uma **plataforma de consoles jogáveis**, não apenas um gerenciador de listas ou lançador genérico de executáveis. Cada console possui ambiente independente, biblioteca e configurações próprias, priorizando dashboards originais autênticos sempre que suportados (ou oferecendo um Ambiente Integrado Alternativo explicitamente identificado quando o boot completo não for suportado).

---

## Visão Geral das Fases (1–7 — Opção A Aprovada)

### Phase 1 — Project Foundation (CONCLUÍDA & AUDITADA — `d7b059a..00488f8`)
- Estrutura limpa Tauri 2 + Rust + React + TypeScript + Vite (`10` commits preservados em `main`).
- 17 skills validadas (`.agents/skills/`) e regras globais (`GEMINI.md`).
- Design System próprio ("Obsidian Hardware") e navegação por foco.
- Banco SQLite `STRICT` + WAL com **19 tabelas** (`Migration 0001`, já incluindo `game_versions`), triggers de imutabilidade de origem de troféus e backup/restauração (`VACUUM INTO`) validados.
- Core modules (`ProcessManager` resistente a pipes não-UTF-8 no Windows, `SecurityManager` SHA-256 streaming sem prefixo `\\?\`, `LoggingSystem`, `AdapterRegistry`).
- Preparação real do **RPCS3** em 4 estágios independentes (`xmb_runtime_confirmed = false`).
- Auditoria prática concluída no executável standalone Windows (`0` erros console/página, persistência e backup confirmados após reinício).

### Phase 2 — Universal Player, Console Environments & Sessions Foundation (CONCLUÍDA & AUDITADA — OPÇÃO A)
Entregou a camada completa de ambientes independentes por console (`ConsoleEnvironmentPage`), descoberta e importação assíncrona de jogos (com parser binário `PARAM.SFO` checado por limites e `Channel<ScanProgressEvent>`), cache seguro de capas (`CoverCache` + `protocol-asset` + `http://asset.localhost`), página individual de jogo com gerenciamento de versões (`game_versions`), fundação real de sessões (`SessionService` com recuperação idempotente por `last_heartbeat_at`), motor de estatísticas/histórico, Migration `0002` (`19 → 22` tabelas `STRICT` + índices aditivos) e fechamento completo das 4 pendências da auditoria pós-Fase 1 (restauração ao vivo de backup SQLite pela UI com snapshot pré-restauração e rollback automático, seletor nativo `tauri-plugin-dialog`, sidebar `sticky` e rótulos amigáveis PT-BR). **Sem execução ao vivo de emuladores ainda (reservada à Fase 3).**

### Phase 3 — First 100% Playable Console: PlayStation 3 via RPCS3 + Real Console Profiles & Auto-Sync (CONCLUÍDA & VERIFICADA — OPÇÃO A)
- **Prioridade total no PS3 antes de qualquer outro console.**
- **Perfis Reais de Console (`console_profiles`) separados de Identidade de Usuários Locais (`console_local_users`):** separação explícita de origem (`native_platform`, `hub_recorded`, `unavailable`), leitura somente leitura dos dados reais do RPCS3 (`RPCS3.log`, `dev_flash/vsh/etc/version.txt`, `dev_hdd0/home/<id>/localusername`, `dev_hdd0/game/`, `PS3_GAME/ICON0.PNG`/`PIC1.PNG`, `dev_hdd0/savedata/`, detecção de diretório `trophy/` com integração real reservada à Fase 4), suporte estrutural a múltiplos usuários locais reais por console (sem jamais inventar usuários) e sincronização automática/incremental/idempotente preservando histórico mesmo se jogos ou o emulador ficarem ausentes (`missing_on_disk`).
- **Fim do cadastro manual de jogos:** remoção completa do formulário manual da `LibraryPage`; a biblioteca é alimentada exclusivamente por diretórios configurados, scanner e sincronização automática.
- **Execução real do PlayStation 3:** boot real do **PS3 XMB (`SystemBoot` via `vsh.self`)** e **execução direta de jogos PS3 (`DirectGameBoot` via `EBOOT.BIN`)** sem shell, monitoramento ao vivo de processo + sessões com heartbeat de 60s, detecção de crash/saída, atualização real de playtime/histórico/estatísticas no Perfil Universal e pós-sincronização automática.
- **Gerenciamento de janela Windows (`WindowsWindowManager`):** minimizar/ocultar Hub e restaurar foco automaticamente ao sair + suspensão de navegação do Hub durante jogo.
- **Verificação independente dos 7 estados de console jogável** (`emulator_launch`, `system_boot`, `original_dashboard`, `game_boot`, `per_game_compatibility`, `session_monitoring`, `hub_return`), sem prometer compatibilidade universal de jogos.

### Phase 4 — Universal Trophy Center (Integrado às Sessões Reais do PS3)
- Leitor somente leitura de troféus locais do RPCS3 (`dev_hdd0/home/<user>/trophy/<NPCOMMID>/` — `TROPCONF.SFM` + `TROPUSR.DAT` + ícones locais em cache), aproveitando a estrutura preparada em `trophies` (#11), `trophy_unlocks` (#12, incluindo vínculo opcional `console_local_user_id`) e `console_local_users` (#24).
- Sincronização automática pós-sessão acoplada ao encerramento de sessões reais do PS3 (`LiveSessionOrchestrator` → `TrophySync`).
- `LocalAchievementProvider` completo, notificações idempotentes, histórico de desbloqueios, filtros, progresso por jogo/geral e raridade honesta.

### Phase 5 — Additional Playable Consoles (Um por Vez, com Validação Real)
Cada console só é marcado funcional após verificação independente das etapas aplicáveis (nunca generalizando compatibilidade universal de títulos):
1. **PlayStation 2 — PCSX2 (Qt)** (PS2 Browser autêntico + Direct Game Boot)
2. **Nintendo Wii / GameCube — Dolphin** (Wii System Menu / GC IPL autênticos + Direct Game Boot)
3. **Xbox Original — xemu** (Xbox Dashboard via HDD + Direct Game Boot)
4. **Xbox 360 — Xenia Canary** (Ambiente Integrado Alternativo explícito + Direct Game Boot; `PARTIAL_DASHBOARD` documentado)
5. **PlayStation 4 — shadPS4** *(Experimental)* (Ambiente Integrado Alternativo explícito + selo `EXPERIMENTAL` + Direct Game Boot; Orbis OS `UNSUPPORTED`)

### Phase 6 — Advanced Hardware Experience & Authorized Integrations
- Ecossistema completo de controles Windows (`gilrs`/SDL3/XInput/DualShock 4/DualSense, glifos dinâmicos, deadzones radiais, calibração, perfis por console e handoff sem vazamento de input).
- Gerenciamento avançado de foco, fullscreen, resolução, monitor alvo e áudio no Windows.
- Integrações oficiais/autorizadas (RetroAchievements e Steam Web API com chave fornecida pelo usuário).

### Phase 7 — Commercial QA & Release
- Auditoria `release-quality-assurance`, otimização de CPU/RAM em idle (0% durante execução de emuladores), empacotamento NSIS/MSI assinado, updater seguro e documentação final.

---

## Detalhamento Oficial da Fase 2 (10 Tasks, Dependências, Contratos & Riscos)

### Grafo Definitivo de Dependências
```text
Task 1 (Migration 0002: 3 Novas Tabelas + Índices em game_versions/game_sessions + Live Restore Windows)
   └──► Task 2 (Domain: ConsoleEnvironment, Scanner, Session & Statistics)
           ├──► Task 3 (PS3 PARAM.SFO Parser & ConsoleGameScanner Engine)
           └──► Task 4 (CoverCache Seguro & Repositórios Estendidos — incl. game_versions)
                   └──► Task 5 (SessionOrchestrator + Statistics + Scanner + Backup + Modular AppServices)
                           └──► Task 6 (13 Comandos Tauri 2 IPC + Channel + protocol-asset + dialogClient)
                                   └──► Task 7 (Audit UI Polish: Sticky Sidebar, Formatters, FilePathField, ConfirmDialog)
                                           └──► Task 8 (ConsoleEnvironmentPage Independente — Modo A & Modo B Explícito)
                                                   └──► Task 9 (GameDetailsPage, StatisticsPage & Restauração de Backup na UI)
                                                           └──► Task 10 (QA 3 Camadas, Playwright E2E & Auditoria no .exe Standalone)
```

> *Nota sobre o DAG:* `Task 3` e `Task 4` dependem de `Task 2` (e `Task 4` também dos artefatos de schema da `Task 1`) e convergem em `Task 5`. A execução ocorre estritamente em ordem sequencial (`Task 1 → Task 2 → ... → Task 10`), um commit por Task.

### Matriz das 10 Tasks da Fase 2

| Task | Dependências | Arquivos Principais | Interfaces & Contratos Esperados | Critérios de Aceitação |
|---|---|---|---|---|
| **Task 1** | Fase 1 (`00488f8`) | `src-tauri/src/infrastructure/db/sql/0002_console_environments.sql`, `migrations.rs`, `manager.rs`, `seed.rs`, `tests/sqlite_integration.rs` | Migration `0002` estritamente aditiva (`user_version = 2`, `19 → 22` tabelas `STRICT`: cria `console_scan_paths`, `console_settings`, `console_runtime_verifications` + índices não-destrutivos na `game_versions` já existente da Fase 1 e em `game_sessions`); `apply_migrations` rejeita `user_version > LATEST_SCHEMA_VERSION` sem modificar o banco; `DatabaseManager::inspect_backup` e `restore_from_backup_live` (snapshot pré-restauração + liberação de handle WAL/SHM + swap com `.rollback.tmp` + auto-migração de backup `v1 → v2` + reabertura WAL/FK). | Upgrade real `v1 → v2` preserva 100% dos registros, índices, constraints e triggers; rejeita `user_version > 2`; live restore no Windows funciona com conexão WAL aberta (incluindo backup `v1`) e faz rollback automático se falhar. |
| **Task 2** | Task 1 | `src-tauri/src/domain/{environment,scanner,session,mod,tests}.rs` | `ConsoleEnvironmentMode`, `RuntimeVerificationFeature` (7 estados independentes), `RuntimeVerificationStatus`, `RuntimeVerificationRecord`, `ConsoleScanPath`, `ConsoleSetting`, `GameDiscoveryPort`, `DiscoveredGameCandidate`, `ScanProgressEvent` (`#[serde(rename_all = "camelCase", tag = "event", content = "data")]`), `GameSessionRecord`, `StatisticsOverview`. | Domínio 100% puro; testes unitários validam derivação de modo de ambiente, contratos dos 7 estados independentes, serialização `Channel` e invariantes de sessão. |
| **Task 3** | Task 2 | `src-tauri/src/adapters/discovery/{mod,param_sfo,ps3_scanner,generic_scanner}.rs`, `adapters/mod.rs`, `tests/scanner_integration.rs` | `parse_param_sfo(&[u8]) -> Result<ParamSfoMetadata, DomainError>` (aritmética checada, zero `panic!`), `Ps3GameDiscovery`, `PlatformExtensionDiscovery`. | Parser `PARAM.SFO` lê títulos/seriais/regiões reais sem `panic!` em buffers truncados/inválidos; re-scan em pasta Unicode é 100% idempotente. |
| **Task 4** | Tasks 1 e 2 | `src-tauri/src/infrastructure/system/cover_cache.rs`, `infrastructure/db/{console_env_repo,session_stats_repo,repositories,mod}.rs`, `tests/cover_and_repo_integration.rs` | `CoverCache::import_local_image` + `validate_authorized_cover_path` (magic bytes PNG/JPEG/WebP, ≤ 10 MB, SHA-256 streaming, restrito a `cache/covers/`); `ConsoleEnvRepository`, `GameDetailsRepository` (leitura/escrita de `games` + `game_versions` existente), `SessionStatsRepository`. | Rejeita arquivos falsificados/traversal/>10 MB; nenhum caminho fora de `cache/covers/` é aceito ou retornado ao frontend; CRUD de ambiente, versões e sessões verificado em SQLite real. |
| **Task 5** | Tasks 3 e 4 | `src-tauri/src/application/{scanner_service,session_service,statistics_service,backup_service,environment_service,services,mod}.rs`, `tests/phase2_services_integration.rs` | Particionamento modular preservando `AppServices::new` (compatibilidade total com testes da Fase 1) + `AppServices::with_dirs`; `SessionService` (`start`, `heartbeat`, `finish`, `reconcile_orphaned_sessions` idempotente via `last_heartbeat_at` → `recovered_after_crash`); `ScannerService`; `StatisticsService`; `BackupService`; `EnvironmentService`. | Recuperação de crash nunca duplica tempo nem conta tempo sem heartbeat; re-executar 2× é no-op; estatísticas refletem apenas sessões reais; testes da Fase 1 permanecem 100% verdes. |
| **Task 6** | Task 5 | `src-tauri/Cargo.toml` (`tauri` feature `protocol-asset`), `src-tauri/tauri.conf.json` (`assetProtocol` restrito a `$APPLOCALDATA/cache/covers/**` + `http://asset.localhost` em `img-src`), `src-tauri/src/ipc/commands.rs`, `lib.rs`, `src/types/domain.ts`, `src/api/{tauriClient,dialogClient,dialogClient.test}.ts` | **13 novos comandos IPC** (total `18 + 13 = 31`, preservando todos os 18 da Fase 1): `get_console_environment`, `add_console_scan_path`, `remove_console_scan_path`, `scan_console_library` (`async fn` + `spawn_blocking` + `Channel<ScanProgressEvent>`), `update_console_setting`, `get_game_details`, `update_game_metadata`, `set_game_cover`, `get_statistics_overview`, `list_activity_history`, `list_database_backups`, `inspect_database_backup`, `restore_database_backup`; `reconcile_orphaned_sessions()` no `setup()` de `lib.rs`; `dialogClient.ts` (`@tauri-apps/plugin-dialog` `2.4+`). | Contratos Rust↔TS 100% alinhados; `cargo clippy --all-targets -- -D warnings`, `cargo test`, `tsc --noEmit` e Vitest verdes. |
| **Task 7** | Task 6 | `src/utils/{formatters,formatters.test}.ts`, `src/components/{AppShell.module.css,AppShell.tsx,FilePathField.tsx,FilePathField.module.css,ConfirmDialog.tsx,ConfirmDialog.module.css,ConfirmDialog.test.tsx}`, páginas atualizadas + alinhamento de strings da Fase 1 para Opção A | Sidebar `sticky` (`height: 100vh`), formatadores PT-BR para todos os slugs/estágios/fases, `FilePathField` com botão nativo "Procurar...", `ConfirmDialog` acessível (`role="dialog"`, `Escape`). | Zero slugs crus visíveis nas tabelas; textos residuais de fases antigas na UI/seed alinhados à Opção A; `ConfirmDialog` e `formatters` cobertos por testes Vitest. |
| **Task 8** | Tasks 6 e 7 | `src/pages/ConsoleEnvironmentPage.{tsx,module.css,test.tsx}`, `ConsolesPage.tsx`, `HomePage.tsx`, `App.tsx` | Ambiente independente por console com 4 abas (Biblioteca do Console, Pastas & Scanner com progresso `Channel`, Emulador/Arquivos & 7 verificações, Configs do Console) e banner explícito no Modo Integrado Alternativo. | PS3/PS2/Wii/Xbox sem botão simulado de boot; Xbox 360/PS4 com banner permanente de Ambiente Integrado Alternativo; isolamento de jogos por console verificado. |
| **Task 9** | Tasks 6, 7 e 8 | `src/pages/{GameDetailsPage,StatisticsPage,LibraryPage,EmulatorsPage,SettingsPage}.tsx`, `App.tsx`, `src/hooks/useHubNavigation.ts` | Página individual de jogo (capa local via `convertFileSrc`, metadados, `game_versions`, `game_sessions`), tela de Estatísticas & Histórico, seletores nativos, e restauração de backup pela UI com `inspect_database_backup` + `ConfirmDialog`. | Cancelar no `ConfirmDialog` nunca altera o banco; confirmar executa restauração ao vivo e recarrega o estado real do Hub. |
| **Task 10** | Task 9 | `tests/e2e/phase2-console-environments.spec.ts`, `src/test/mockTransport.ts`, `docs/{TESTING,KNOWN_LIMITATIONS,DATABASE_SCHEMA,COMPATIBILITY_MATRIX}.md` | Cobertura completa nas 3 camadas + auditoria prática no executável standalone (`cargo build --release --features custom-protocol`). | 0 erros console/página no WebView2, scan real de pasta sintética PS3, capa servida via `http://asset.localhost`, restauração real de backup pela UI + reinício confirmados, sidebar `sticky` validada visualmente. |

### Estratégia TDD (`RED → GREEN → REFACTOR`) e Commits
- Ordem estrita e sequencial: `Task 1 → Task 2 → ... → Task 10`.
- Em cada Task: (1) inspecionar código existente, (2) escrever testes falhando (**RED**), (3) implementar o mínimo correto (**GREEN**), (4) rodar testes afetados + suíte completa + `cargo fmt --check` + `cargo clippy --all-targets -- -D warnings` + `tsc --noEmit`, (5) revisar segurança/regressões, (6) criar **1 commit pequeno e descritivo** em `main` (nunca acumular múltiplas tasks em um commit; nunca fazer `git push`).

### Riscos Técnicos da Fase 2 e Mitigações
| Risco Técnico | Impacto | Mitigação Arquitetural |
|---|---|---|
| **Lock de arquivo SQLite no Windows durante restauração ao vivo (`ERROR_SHARING_VIOLATION`)** | Corrupção ou falha ao restaurar backup pela UI. | Liberar explicitamente a `Connection` ativa dentro de `Mutex<Connection>` antes da troca, usar `VACUUM INTO` em arquivo temporário, renomear banco anterior para `.rollback.tmp` e reverter automaticamente em caso de erro. |
| **Abertura de banco criado por versão futura (`user_version > 2`) ou restauração de backup `v1`** | Corrupção silenciosa ou schema incompatível pós-restauração. | `apply_migrations` e `inspect_backup` rejeitam `user_version > LATEST_SCHEMA_VERSION` sem escrita; restauração de backup `v1` aplica `apply_migrations` + `seed_defaults` automaticamente após reabertura. |
| **Sessões órfãs com PC hibernado/desligado gerando centenas de horas falsas** | Distorção permanente das estatísticas do jogador. | Recuperação utiliza exclusivamente `last_heartbeat_at` (nunca `Utc::now()`), grava `recovered_after_crash` e é 100% idempotente. |
| **Bloqueio de capas locais no WebView2 Windows por CSP ou feature ausente** | Imagens de capa quebradas em produção. | Habilitar feature `"protocol-asset"` em `src-tauri/Cargo.toml`, configurar `assetProtocol.scope = ["$APPLOCALDATA/cache/covers/**"]` e incluir `http://asset.localhost` em `img-src` na CSP. |
| **Falsificação de capa / path traversal via IPC (`set_game_cover`)** | Leitura indevida de arquivos do sistema ou assets quebrados. | Validação de magic bytes + limite 10 MB + SHA-256 no backend + verificação estrita de que `cover_path` pertence a `<app_local_data_dir>/cache/covers/`. |
| **Arquivos `PARAM.SFO` corrompidos ou pastas gigantes no scanner** | Travamento da UI ou `panic!` no backend. | Scanner executa em `tokio::task::spawn_blocking` com progresso por `Channel<ScanProgressEvent>`, parser usa aritmética checada (`checked_add` / slices seguros) sem `unwrap()`, ignorando arquivos inválidos com diagnóstico registrado. |
| **Quebra de compatibilidade com bancos `v1` da Fase 1** | Perda de dados de usuários existentes ou falha em `table game_versions already exists`. | `game_versions` (#8) é preservada sem recriação; Migration `0002` é estritamente aditiva e validada por teste real migrando cópia populada do banco `v1` para `v2`. |

---

## Detalhamento Oficial da Fase 3 (9 Tasks, Dependências, Contratos & Ajustes Obrigatórios)

### Ajustes Arquiteturais Incorporados
- **A. Identidade do Console ≠ Identidade do Jogador:** `console_profiles` (#23) armazena apenas dados reais do console/emulador; usuários locais reais expostos pela plataforma (ex.: `dev_hdd0/home/<id>/localusername` no RPCS3) vivem em `console_local_users` (#24), com vínculo opcional em `game_sessions.console_local_user_id` e `trophy_unlocks.console_local_user_id`. Nenhum usuário é inventado quando a plataforma não expõe identificação.
- **B. Sincronização Persistente:** mudanças no disco ou indisponibilidade do emulador/jogo atualizam `emulator_status` (`available` / `missing_on_disk` / `unconfigured`) e `game_versions.availability_status` (`available` / `missing_on_disk`) preservando 100% do histórico de jogos, sessões e estatísticas.
- **C. Preparação Honesta para Conquistas Completas (Fase 4):** `trophies` (#11) e `trophy_unlocks` (#12 + `console_local_user_id`) ficam prontas para identificador, título, descrição, ícone, estado e data de desbloqueio; a Fase 3 registra apenas detecção real do diretório de troféus (`native_trophy_dir_present`, `trophy_sync_readiness = 'phase4_planned'`) sem dados fictícios.
- **D. Fim do Cadastro Manual de Jogos:** formulário manual removido completamente da `LibraryPage`; descoberta ocorre exclusivamente por diretórios configurados, scanner e sincronização automática.

### Grafo Definitivo de Dependências da Fase 3
```text
Task 1 (Migration 0003: console_profiles #23, console_local_users #24, colunas aditivas em game_versions/game_sessions/trophy_unlocks + testes v1/v2 → v3)
   └──► Task 2 (Domínio: DataOrigin, ConsoleProfileProvider, ConsoleProfileSnapshot, ConsoleLocalUser, ActiveSessionState, LaunchTarget)
           ├──► Task 3 (Ps3ProfileProvider: leitura nativa RPCS3, múltiplos localusername, version.txt, ICON0.PNG/PIC1.PNG, savedata, detecção trophy/)
           └──► Task 5 (WindowsWindowManager + LiveEmulatorRunner: spawn direto sem shell, PID/handle tracking, stdout/stderr assíncrono limitado)
                   ├──► Task 4 (ConsoleProfileSyncService: sync automático/incremental/idempotente, missing_on_disk, auto-cache ICON0.PNG)
                   └──► Task 6 (LiveSessionOrchestrator: guarda sessão única, heartbeat 60s, pós-sync automático, 7 estados runtime honestos)
                           └──► Task 7 (Comandos Tauri 2 IPC + Eventos session:* e profile:synced + contratos TS)
                                   └──► Task 8 (UI Perfil Real PS3 com selos Nativo/Hub/Indisponível, Launch XMB/Jogos, Overlay de Sessão, remoção do formulário manual da LibraryPage)
                                           └──► Task 9 (QA 3 Camadas + Auditoria .exe Standalone + Documentação)
```

> *Execução sequencial estrita:* `Task 1 → Task 2 → Task 3 → Task 4 → Task 5 → Task 6 → Task 7 → Task 8 → Task 9`, com TDD (`RED → GREEN → REFACTOR`), verificação completa e **1 commit por Task** (sem `git push`).

### Matriz Definitiva das 9 Tasks da Fase 3

| Task | Dependências | Arquivos Principais | Interfaces & Contratos Esperados | Critérios de Aceitação |
|---|---|---|---|---|
| **Task 1** | Fase 2 (`a85a6f2`) | `src-tauri/src/infrastructure/db/sql/0003_console_profiles_and_runtime.sql`, `migrations.rs`, `console_profile_repo.rs`, `tests/sqlite_integration.rs` | Migration `0003` estritamente aditiva (`user_version = 3`, `22 → 24` tabelas `STRICT`: cria `console_profiles` #23 e `console_local_users` #24 + `ALTER TABLE` aditivo em `game_versions`, `game_sessions` e `trophy_unlocks` + índices); `ConsoleProfileRepository`. | Upgrade real `v1 → v3` e `v2 → v3` preserva 100% dos dados/triggers; rejeita `user_version > 3` sem escrita; live restore continua íntegro. |
| **Task 2** | Task 1 | `src-tauri/src/domain/{console_profile,session,mod,tests}.rs` | `DataOrigin` (`native_platform`, `hub_recorded`, `unavailable`), `ProfileField<T>`, `ConsoleProfileSnapshot`, `ConsoleLocalUser`, `ConsoleProfileProvider` trait, `SyncTrigger`, `ConsoleSyncReport`, `ActiveSessionState`, `LaunchTarget` (`SystemBoot` / `DirectGameBoot`). | Domínio 100% puro; testes unitários validam origem de dados, separação console vs. usuário local e máquina de estados de sessão única. |
| **Task 3** | Task 2 | `src-tauri/src/adapters/profiles/{mod,ps3_profile}.rs`, `adapters/mod.rs`, `tests/ps3_profile_integration.rs` | `Ps3ProfileProvider` (leitura somente leitura de `RPCS3.log`, `dev_flash/vsh/etc/version.txt`, todos os `dev_hdd0/home/<id>/localusername`, `dev_hdd0/savedata/`, `dev_hdd0/game/`, detecção de `PS3_GAME/ICON0.PNG`/`PIC1.PNG` e presença de `trophy/`). | Testes em diretório RPCS3 sintético com espaços/Unicode; nunca altera arquivos do RPCS3; nunca inventa usuários quando `home/` não existe; tolera arquivos corrompidos sem `panic!`. |
| **Task 4** | Tasks 1, 2 e 3 | `src-tauri/src/application/console_profile_sync_service.rs`, `scanner_service.rs`, `services.rs`, `tests/console_profile_sync_integration.rs` | `ConsoleProfileSyncService::sync_console_profile(console_slug, trigger)` (incremental e idempotente): atualiza `console_profiles` e `console_local_users`, reconcilia `available`/`missing_on_disk` em jogos e emulador sem apagar histórico, importa `ICON0.PNG`/`PIC1.PNG` via `CoverCache`, grava evento em `activity_history`. | Sync 2× consecutivo é 100% idempotente; jogo/emulador removido vira `missing_on_disk` preservando sessões/playtime; usuários locais ausentes preservam histórico; disparado automaticamente em add/scan/configuração. |
| **Task 5** | Task 2 | `src-tauri/src/infrastructure/system/{window_manager,live_runner,mod}.rs`, `tests/live_runner_integration.rs` | `WindowsWindowManager` (minimizar/ocultar janela Hub ao lançar e restaurar foco ao encerrar) e `LiveEmulatorRunner` (spawn direto via `Command::new(exe).arg(...)` sem shell, rastreio de PID/handle, drenagem assíncrona limitada de stdout/stderr, encerramento gracioso antes de término forçado). | Testes reais com binário OS Windows em caminho Unicode com espaços; saída normal `0`, saída não-zero e cancelamento limpo sem deadlock de pipe. |
| **Task 6** | Tasks 4 e 5 | `src-tauri/src/application/live_session_orchestrator.rs`, `services.rs`, `tests/live_session_integration.rs` | `LiveSessionOrchestrator`: guarda de sessão única ativa (`Idle → Launching → Running → Exiting → Idle`), suporte a `SystemBoot` (`vsh.self`) e `DirectGameBoot` (`EBOOT.BIN`), persistência imediata em `game_sessions` (com `console_local_user_id` opcional), heartbeat real de 60s em Tokio, registro independente dos 7 estados em `console_runtime_verifications`, pós-sync automático ao encerrar. | Impede 2 sessões simultâneas; contabiliza duração real; confirmar `game_boot` nunca marca `original_dashboard` como confirmado; pós-sync roda automaticamente ao sair. |
| **Task 7** | Tasks 4 e 6 | `src-tauri/src/ipc/commands.rs`, `lib.rs`, `src/types/domain.ts`, `src/api/tauriClient.ts` | Novos comandos IPC (`get_console_profile`, `sync_console_profile`, `launch_console_system`, `launch_game`, `get_active_session`, `stop_active_session`, `record_runtime_verification`) + eventos Tauri (`session:started`, `session:heartbeat`, `session:exited`, `profile:synced`) preservando todos os 31 comandos das Fases 1–2. | Contratos Rust↔TS 100% tipados; `cargo clippy --all-targets -- -D warnings`, `cargo test` e `tsc --noEmit` verdes. |
| **Task 8** | Task 7 | `src/pages/ConsoleEnvironmentPage.{tsx,module.css,test.tsx}`, `src/pages/LibraryPage.{tsx,test.tsx}`, `src/components/ActiveSessionBanner.tsx`, `src/hooks/useHubNavigation.ts` | Perfil Real do Console com selos explícitos (`Nativo` / `Hub` / `Indisponível`), lista de usuários locais reais detectados (vazia com aviso honesto se indisponível), botão **Sincronizar Agora**, ações reais **Iniciar PS3 XMB** (apenas se `vsh.self` pronto) e **Jogar** (apenas se `available`), overlay de sessão ativa + suspensão de navegação do Hub durante jogo, **remoção completa do formulário manual de cadastro de jogos na `LibraryPage`**. | Zero formulário manual na `LibraryPage`; zero botão fake de dashboard em consoles sem suporte; jogos/emulador ausentes exibem status honesto preservando histórico; testes Vitest verdes. |
| **Task 9** | Task 8 | `tests/e2e/phase3-ps3-profile-and-session.spec.ts`, `src/test/mockTransport.ts`, `docs/{ROADMAP,TESTING,KNOWN_LIMITATIONS,ARCHITECTURE}.md` | Cobertura completa nas 3 camadas separadas + auditoria prática no `universal-console-hub.exe` standalone (`--release --features custom-protocol`). | 0 erros console/página no WebView2, sync real PS3 com `ICON0.PNG` via `http://asset.localhost`, validação real do fluxo de sessão e retorno ao Hub, documentação sincronizada. |
