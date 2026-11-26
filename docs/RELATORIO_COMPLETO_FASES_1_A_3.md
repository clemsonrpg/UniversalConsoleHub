# Relatório Completo — Universal Console Hub 3.0 (Fases 1, 2 e 3)

**Data de consolidação:** 25/11/2026  
**Branch local:** `main` (nenhum `git push` executado)  
**Estratégia aprovada:** Opção A (Fases 1 → 7, com prioridade total no PlayStation 3 antes dos demais consoles)  
**Stack:** Tauri 2 + Rust (`x86_64-pc-windows-msvc`) + SQLite (`STRICT`, WAL, `foreign_keys = ON`) + React + TypeScript estrito + Vite + Vitest + Playwright

---

## 1. Resumo Executivo do Que Foi Construído

O **Universal Console Hub 3.0** saiu do zero até uma plataforma desktop Windows local-first com:
- **Banco SQLite `STRICT` + WAL com 24 tabelas** através de 3 migrations transacionais e estritamente aditivas (`0001` → `0002` → `0003`), com triggers imutáveis de origem de troféus e **restauração ao vivo segura no Windows** via `VACUUM INTO` + snapshot pré-restauração + rollback automático.
- **Perfis Reais de Console (`console_profiles` #23)** estritamente separados de **Usuários Locais Reais do Console (`console_local_users` #24)**, distinguindo explicitamente dados **Nativos** (`native_platform`), dados do **Hub** (`hub_recorded`) e dados **Indisponíveis** (`unavailable`).
- **Sincronização automática, incremental e idempotente (`ConsoleProfileSyncService`)** que lê arquivos reais do RPCS3 (`RPCS3.log`, `dev_flash/vsh/etc/version.txt`, múltiplos `dev_hdd0/home/<id>/localusername`, `dev_hdd0/game/`, `PS3_GAME/PARAM.SFO`, `ICON0.PNG`/`PIC1.PNG`, `dev_hdd0/savedata/`, detecção de `trophy/`), importa capas nativas pelo `CoverCache` seguro e preserva 100% do histórico de jogos, sessões e usuários quando arquivos ou o emulador ficam ausentes (`missing_on_disk`).
- **Integração ao vivo do PlayStation 3 via RPCS3 (Fase 3 — implementada e verificada por testes automatizados + `.exe` standalone):** lançamento real do **PS3 XMB (`vsh.self`)** e **boot direto de jogos (`EBOOT.BIN`)** sem shell (`Command::new(exe).args(...)`), monitoramento ao vivo de sessões com heartbeat de 60s, encerramento gracioso Win32 `WM_CLOSE`, minimização/restauração automática de foco da janela do Hub (`WindowsWindowManager`), suspensão de atalhos do Hub durante o jogo e registro independente dos **7 estados de verificação em runtime**. *(A confirmação runtime final com uma instalação real do `rpcs3.exe`, firmware `PS3UPDAT.PUP` e títulos PS3 reais depende de validação manual na máquina do usuário.)*
- **Remoção completa do formulário manual de cadastro de jogos da `LibraryPage`:** a biblioteca é alimentada exclusivamente por diretórios configurados, scanner binário e sincronização automática.
- **38 comandos Tauri 2 IPC + 4 eventos em tempo real** (`session:started`, `session:heartbeat`, `session:exited`, `profile:synced`) + streaming `Channel<ScanProgressEvent>`.

---

## 2. Histórico Completo por Fase e Commits

### Fase 1 — Project Foundation (`10` commits preservados: `d7b059a..00488f8`)

| Commit | Entrega |
|---|---|
| `d7b059a` | Governança (`GEMINI.md`), 17 skills em `.agents/skills/`, modelo multi-capacidade e documentação inicial. |
| `53c6e95` | Scaffold Tauri 2 + Rust + React + TypeScript estrito + Vite + Vitest + Playwright. |
| `7d013f8` | Camada de domínio pura (capacidades multi-dimensão, comandos seguros sem shell, timestamps UTC, origem imutável de troféus). |
| `dd18e2c` | Migration `0001` (`19` tabelas `STRICT`), triggers `trg_trophies_immutable_origin` e `trg_trophy_unlocks_immutable_origin`, repositórios e backup/restauração WAL. |
| `ac2d6f9` | `ProcessManager` seguro no Windows (pipes resistentes a não-UTF-8), `SecurityManager` (SHA-256 streaming sem prefixo `\\?\`) e `tracing`. |
| `55ac070` | `AdapterRegistry` e `Rpcs3Adapter` com prontidão em 4 estágios independentes (`xmb_runtime_confirmed = false`). |
| `ba714c3` | `AppServices`, 18 comandos Tauri 2 IPC e cliente tipado `HubApiClient` com proteção contra execução fora do Tauri. |
| `aee1945` | Design System próprio "Obsidian Hardware" e componentes reutilizáveis (`AppShell`, `CapabilityBadge`, `ReadinessTracker`, `ConsoleCard`). |
| `763cedc` | Telas funcionais da Fase 1 (`HomePage`, `ConsolesPage`, `LibraryPage`, `ProfilePage`, `TrophiesPage`, `EmulatorsPage`, `SettingsPage`) + E2E Playwright. |
| `00488f8` | Correções da auditoria prática no `.exe` standalone (`custom-protocol`, CSP `connect-src`, remoção de `\\?\`, polimento PT-BR). |

---

### Fase 2 — Universal Player, Console Environments & Sessions Foundation (`11` commits: `c5f6660..a85a6f2`)

| Commit | Entrega |
|---|---|
| `c5f6660` | Consolidação documental da Fase 2, Roadmap Opção A e schema definitivo da Migration `0002`. |
| `e6c5bcc` | **Task 1:** Migration `0002` (`19 → 22` tabelas `STRICT`: `console_scan_paths` #20, `console_settings` #21, `console_runtime_verifications` #22 + 8 índices aditivos) e restauração SQLite ao vivo segura no Windows (`inspect_backup`, `restore_from_backup_live`, snapshot pré-restauração e rollback automático). |
| `835d95f` | **Task 2:** Domínio puro de `ConsoleEnvironmentMode`, 7 estados independentes (`RuntimeVerificationFeature`), `GameDiscoveryPort`, `ScanProgressEvent` e `compute_elapsed_seconds_utc`. |
| `7c8a28f` | **Task 3:** Parser binário `PARAM.SFO` checado por limites (zero `panic!`), `Ps3GameDiscovery` e `PlatformExtensionDiscovery`. |
| `2809f64` | **Task 4:** `CoverCache` seguro (magic bytes PNG/JPEG/WebP, ≤ 10 MB, SHA-256, diretório autorizado `cache/covers/`) + repositórios de ambiente/detalhes/sessões com recuperação idempotente `reconcile_orphaned_sessions` usando exclusivamente `last_heartbeat_at`. |
| `552e03e` | **Task 5:** Particionamento modular de `AppServices` (`environment_service`, `scanner_service`, `session_service`, `statistics_service`, `backup_service`) preservando `AppServices::new`. |
| `d72cf7a` | **Task 6:** 13 novos comandos Tauri 2 IPC (total `31`), `Channel<ScanProgressEvent>` em `spawn_blocking`, feature `protocol-asset` (`http://asset.localhost`), `tauri-plugin-dialog` e `reconcile_orphaned_sessions()` no `setup()`. |
| `9bf2df8` | **Task 7:** Sidebar `sticky`, formatadores PT-BR (`src/utils/formatters.ts`), `FilePathField` nativo e `ConfirmDialog` acessível. |
| `9176244` | **Task 8:** `ConsoleEnvironmentPage` independente com 4 abas e banner permanente de Ambiente Integrado Alternativo para PS4/Xbox 360. |
| `8875577` | **Task 9:** `GameDetailsPage`, `StatisticsPage` e fluxo completo de restauração de backup SQLite pela UI com inspeção + `ConfirmDialog`. |
| `a85a6f2` | **Task 10:** Suíte Playwright da Fase 2, alinhamento de copy à Opção A e auditoria prática no `.exe` standalone via WebView2. |

---

### Fase 3 — Perfis Reais de Console + Sincronização Automática + PlayStation 3 100% Jogável via RPCS3 (`10` commits: `1b221cf..HEAD`)

| Commit | Entrega |
|---|---|
| `1b221cf` | Consolidação documental da Fase 3 com os 4 ajustes obrigatórios (A: identidade console ≠ jogador; B: sincronização persistente `missing_on_disk`; C: prontidão honesta da Fase 4 para troféus; D: remoção do cadastro manual). |
| `bae793c` | **Task 1:** Migration `0003` (`22 → 24` tabelas `STRICT`: `console_profiles` #23 e `console_local_users` #24 + colunas aditivas em `game_versions`, `game_sessions` e `trophy_unlocks` + índices) + `ConsoleProfileRepository` + testes reais `v1 → v3` e `v2 → v3`. |
| `619666b` | **Task 2:** Domínio puro de perfil (`DataOrigin`, `ProfileField<T>`, `AvailabilityStatus`, `EmulatorStatus`, `TrophySyncReadiness`, `SyncTrigger`, `DetectedLocalUser`, `ConsoleProfileProvider`, `ConsoleSyncReport`) e máquina de estados de sessão ao vivo (`ActiveSessionPhase`, `LaunchTarget`, `ActiveSessionSnapshot`). |
| `3230387` | **Task 3:** `Ps3ProfileProvider` somente leitura (`RPCS3.log`, `dev_flash/vsh/etc/version.txt`, múltiplos `dev_hdd0/home/<id>/localusername`, `savedata`, `dev_hdd0/game/`, `ICON0.PNG`/`PIC1.PNG`, detecção honesta de `trophy/`) + `GenericPlatformProfileProvider` (sem nunca inventar usuários). |
| `cf18253` | **Task 4:** `ConsoleProfileSyncService` (sincronização automática/incremental/idempotente, reconciliação `available ↔ missing_on_disk` preservando histórico, auto-cache de `ICON0.PNG`/`PIC1.PNG` via `CoverCache`, gatilhos automáticos ao configurar emulador/firmware/pastas/scan). |
| `2abf108` | **Task 5:** `WindowsWindowManager` (`TauriWindowManager` minimiza Hub ao lançar e restaura foco ao sair) e `LiveEmulatorRunner` (spawn direto sem shell, drenagem assíncrona limitada de stdout/stderr e encerramento gracioso Win32 `WM_CLOSE` antes de término forçado). |
| `bd46224` | **Task 6:** `LiveSessionOrchestrator` (guarda de sessão única, boot real `SystemBoot`/`vsh.self` e `DirectGameBoot`/`EBOOT.BIN`, heartbeat real de 60s em Tokio, registro independente dos 7 estados em `console_runtime_verifications`, pós-sync automático ao sair e reflexão real de `xmb_runtime_confirmed`). |
| `f66e8c6` | **Task 7:** 7 novos comandos IPC (total `38`: `get_console_profile`, `sync_console_profile`, `launch_console_system`, `launch_game`, `get_active_session`, `stop_active_session`, `record_runtime_verification`), eventos Tauri (`session:started`, `session:heartbeat`, `session:exited`, `profile:synced`), hooks anexados no `setup()` de `lib.rs` e contratos TypeScript. |
| `ea5332f` | **Task 8:** Evolução de `ConsoleEnvironmentPage` para Perfil Real do Console (selos `Nativo`/`Hub`/`Indisponível`, aba `Perfil & Usuários Locais`, botão **Sincronizar Agora**, ações reais **Iniciar PS3 XMB** e **Jogar**, confirmação manual independente do XMB), `ActiveSessionBanner`, suspensão de navegação do Hub durante jogo e **remoção completa do formulário manual da `LibraryPage`**. |
| *(Task 9)* | **Task 9:** Suíte Playwright da Fase 3 (`phase3-ps3-profile-and-session.spec.ts`), validação completa nas 3 camadas, atualização de `README.md`/`docs/*.md` e este relatório completo. |

---

## 3. Arquitetura Atual do Banco de Dados (24 Tabelas `STRICT` — `user_version = 3`)

- **Migration `0001` (19 tabelas):** `users`, `profiles`, `consoles`, `emulators`, `emulator_versions`, `console_firmware`, `games`, `game_versions`, `game_sessions`, `achievement_providers`, `trophies`, `trophy_unlocks`, `platform_accounts`, `statistics`, `favorites`, `activity_history`, `synchronization_history`, `controller_profiles`, `application_settings`.
- **Migration `0002` (+3 tabelas = 22):** `console_scan_paths`, `console_settings`, `console_runtime_verifications` + 8 índices aditivos.
- **Migration `0003` (+2 tabelas = 24):** `console_profiles` (#23) e `console_local_users` (#24) + colunas aditivas (`game_versions.availability_status`, `game_versions.last_verified_at`, `game_sessions.console_local_user_id`, `trophy_unlocks.console_local_user_id`) + 4 índices aditivos.

---

## 4. Resultados Verificados nas 3 Camadas de Testes

### Camada 1 — Testes Unitários Isolados
- **Rust (`cargo test --lib`):** **15/15 PASS** (domínio puro, capacidades multi-dimensão, origem de dados `DataOrigin`, máquina de estados de sessão única, parser `PARAM.SFO`, 7 estados de verificação, imutabilidade de origem de troféus).
- **Frontend (`npm.cmd run test` — Vitest):** **22/22 PASS** em **10 arquivos** (`tauriClient`, `dialogClient`, `formatters`, `CapabilityBadge`, `ReadinessTracker`, `ConfirmDialog`, `LibraryPage` sem formulário manual, `ConsoleEnvironmentPage`, `Phase2Pages`, `App`).

### Camada 2 — Testes de Integração Real (SQLite Real + Processos OS Windows + Filesystem)
- **`cargo test --test '*'`:** **34/34 PASS** em **10 suítes** (`sqlite_integration` 9, `rpcs3_adapter_integration` 5, `system_integration` 4, `application_integration` 3, `cover_and_repo_integration` 3, `scanner_integration` 3, `live_runner_integration` 2, `ps3_profile_integration` 2, `console_profile_sync_integration` 1, `live_session_integration` 1, `phase2_services_integration` 1).

### Camada 3 — Testes E2E de Interface (Playwright + Transporte de Teste Injetado)
- **`npm.cmd run test:e2e`:** **3/3 PASS** (`phase1-ui-flows.spec.ts`, `phase2-console-environments.spec.ts`, `phase3-ps3-profile-and-session.spec.ts`).
- *Nota de transparência:* Os testes Playwright validam fluxos de interface com `window.__UCH_TEST_TRANSPORT__` e são reportados estritamente separados dos testes de integração real Rust/SQLite/Windows da Camada 2.

### Qualidade Estática & Build
- `cargo fmt --check`: **PASS**
- `cargo clippy --all-targets -- -D warnings`: **PASS (0 warnings)**
- `tsc --noEmit`: **PASS (0 erros)**
- `vite build`: **PASS** (`dist/` compilado)

---

## 5. Como Compilar e Abrir o Aplicativo no Windows

Se o aplicativo anterior estiver aberto, feche a janela do Universal Console Hub primeiro para liberar o arquivo `.exe`, depois execute:

```powershell
npm.cmd run build
cargo build --manifest-path src-tauri/Cargo.toml --release --features custom-protocol
```

Em seguida, abra diretamente pelo Explorador de Arquivos do Windows:
`src-tauri\target\release\universal-console-hub.exe`

---

## 6. Próximas Fases Planejadas (Opção A)

- **Fase 4:** Central Universal de Troféus integrada às sessões reais do PlayStation 3 (leitor somente leitura de `TROPCONF.SFM` + `TROPUSR.DAT` no diretório `dev_hdd0/home/<user>/trophy/` já detectado na Fase 3).
- **Fase 5:** Demais consoles jogáveis um por vez (PS2 via PCSX2 Qt → Wii/GameCube via Dolphin → Xbox Original via xemu → Xbox 360 via Xenia Canary → PS4 Experimental via shadPS4).
- **Fase 6:** Ecossistema completo de controles Windows (SDL3/XInput/DualShock 4/DualSense) e integrações externas autorizadas (RetroAchievements / Steam).
- **Fase 7:** QA comercial, otimização idle e empacotamento instalador Windows (NSIS/MSI).
