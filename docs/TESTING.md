# Testing Strategy — Universal Console Hub 3.0

## Separação Estrita das 3 Camadas de Testes (+ Auditoria Prática)

Para evitar falsas sensações de cobertura, todos os relatórios e comandos distinguem rigorosamente três camadas automatizadas e a auditoria prática no executável compilado:

### 1. Testes Unitários (Isolados)
- **Rust (`cargo test --lib`):** Entidades de domínio, modelo de capacidades multi-dimensão, invariantes de origem de troféus, serialização DTO, lógica pura de validação/parsing (incluindo parser binário `PARAM.SFO` com buffers válidos, truncados e malformados na Fase 2, além dos 7 estados independentes de verificação).
- **Frontend (`npm.cmd run test` — Vitest):** Componentes do Design System (`CapabilityBadge`, `ReadinessTracker`, `ConsoleCard`, `ConfirmDialog`, `FilePathField`), formatadores amigáveis (`formatters.ts`), hooks de navegação, `dialogClient` e cliente IPC (`tauriClient`).
- *O que NÃO validam:* Persistência real em disco, IPC Tauri nativo ou execução real de emuladores.

### 2. Testes de Integração com Backend Real (`cargo test --test '*'`)
- **SQLite Real (`sqlite_integration.rs`):**
  - Arquivo `.db` real em disco com modo WAL, constraints `STRICT`/`UNIQUE`/`FOREIGN KEY`, timestamps UTC e triggers de imutabilidade de `origin`.
  - **Fase 2 — Compatibilidade de Migrações:** Upgrade real de uma cópia populada do banco `v1` (Fase 1) para `v2` (`0002_console_environments.sql`), validando preservação integral de dados/índices/triggers + rejeição segura de `user_version > LATEST_SCHEMA_VERSION` sem modificar o arquivo.
  - **Fase 3 — Compatibilidade de Migrações (`v1/v2 → v3`):** Upgrade real de cópias populadas dos bancos `v1` e `v2` para `v3` (`0003_console_profiles_and_runtime.sql`), validando `console_profiles` (#23), `console_local_users` (#24), colunas aditivas em `game_versions`/`game_sessions`/`trophy_unlocks`, preservação integral dos triggers imutáveis de troféus e rejeição de `user_version > 3` sem escrita.
  - **Fase 4 — Compatibilidade de Migrações (`v1/v2/v3 → v4`) & `TrophyRepository` (`sqlite_integration.rs`, `trophy_db_integration.rs`):** Upgrade real até `v4` (`0004_trophy_sets_and_hidden.sql`, `25` tabelas `STRICT`), upsert de `trophy_sets` (#25) e `trophies` (`is_hidden`, `trophy_set_id`), idempotência de `record_trophy_unlock` preservando `unlocked_at` mais antigo, imutabilidade de `origin` via triggers e rejeição de `user_version > 4` sem escrita.
  - **Fase 2 — Backup & Restauração Ao Vivo no Windows:** Criação via `VACUUM INTO`, inspeção pré-restauração, `restore_from_backup_live` com conexão WAL ativa (incluindo snapshot pré-restauração, limpeza de `-wal`/`-shm`, reabertura WAL/FK, rejeição de backup corrompido e rollback seguro preservando o banco original).
- **Sistema Windows Real (`system_integration.rs`, `cover_and_repo_integration.rs`, `live_runner_integration.rs`):**
  - `ProcessManager` executando binários reais do Windows com espaços/Unicode (`Área de Trabalho`), drenagem lossy de pipes não-UTF-8 e guarda de sessão única.
  - **Fase 3 — `LiveEmulatorRunner` & `WindowsWindowManager`:** Spawn direto sem shell, captura assíncrona limitada de stdout/stderr, encerramento gracioso antes de término forçado e tratamento de códigos de saída reais.
  - `SecurityManager` (SHA-256 streaming, remoção do prefixo `\\?\`, anti path-traversal).
  - **Fase 2 — `CoverCache`:** Magic bytes reais PNG/JPEG/WebP, rejeição de arquivos falsificados ou > 10 MB, deduplicação SHA-256 e garantia de que todo caminho retornado pertence exclusivamente ao diretório autorizado `cache/covers/`.
- **Adaptadores, Provedores de Perfil & Scanner Reais (`rpcs3_adapter_integration.rs`, `scanner_integration.rs`, `ps3_profile_integration.rs`):**
  - Inspeção real de árvores temporárias (`rpcs3.exe`, `dev_flash/vsh/module/vsh.self`, `PS3UPDAT.PUP` sintético) com 4 estágios independentes.
  - **Fase 2 — `ConsoleGameScanner`:** Scan recursivo em diretórios Unicode com `PARAM.SFO` + `EBOOT.BIN` sintéticos, extração real de serial/região e idempotência em re-scans.
  - **Fase 3 — `Ps3ProfileProvider`:** Leitura somente leitura de `RPCS3.log`, `dev_flash/vsh/etc/version.txt`, múltiplos `dev_hdd0/home/<id>/localusername`, `dev_hdd0/savedata/`, `PS3_GAME/ICON0.PNG`/`PIC1.PNG` e detecção de `trophy/` sem nunca inventar usuários ausentes.
- **Application Services Reais (`application_integration.rs`, `phase2_services_integration.rs`, `console_profile_sync_integration.rs`, `live_session_integration.rs`):**
  - Casos de uso conectados diretamente ao `DatabaseManager` e `AdapterRegistry` reais (sem mocks).
  - **Fase 2 — `SessionOrchestrator` & `StatisticsService`:** `start` → `heartbeat` → crash simulado (`ended_at IS NULL`) → `reconcile_orphaned_sessions` usando exclusivamente `last_heartbeat_at` (`recovered_after_crash`), idempotência em múltiplas chamadas e ausência de tempo contabilizado sem evidência de heartbeat.
  - **Fase 3 — `ConsoleProfileSyncService` & `LiveSessionOrchestrator`:** Sincronização idempotente, transição `available ↔ missing_on_disk` preservando histórico de jogos/sessões/usuários locais, guarda de sessão única, heartbeat real, registro independente dos 7 estados de verificação e pós-sync automático.
  - **Fase 4 — `Rpcs3TrophyReader` & `TrophySyncService` (`rpcs3_trophy_reader_integration.rs`, `trophy_sync_integration.rs`):** Leitura somente leitura de `TROPCONF.SFM` + `TROPUSR.DAT` binário big-endian em caminhos Unicode (incluindo tolerância sem `panic!` a arquivos truncados/corrompidos), cache de ícones `ICON0.PNG`/`TROPxxx.PNG` via `CoverCache`, vinculação a `console_local_users`, não-regressão de desbloqueios, raridade `None` (`Local / Offline`), registro em `synchronization_history` e acoplamento automático pós-sessão no `LiveSessionOrchestrator`.
  - **Reformulação Visual & Controle (`theme_and_gamepad_integration.rs`):** Validação e importação segura de pacotes de temas locais (rejeitando CSS/JS malicioso, path traversal, IDs reservados e imagens falsificadas), persistência em diretório seguro, remoção limpa e verificação real de deadzone radial, `RepeatGate` anti-repetição e pausa/dreno de 350ms do `WindowsGamepadMonitor`.

### 3. Testes E2E de Interface (`npm.cmd run test:e2e` — Playwright)
- Executam o aplicativo React/Vite em navegador Chromium com um **transporte de teste injetado (`window.__UCH_TEST_TRANSPORT__`)** exclusivo para automação de UI (e `navigator.getGamepads` simulado em `redesign-themes-gamepad.spec.ts`).
- **Declaração explícita:** Testes Playwright com transporte injetado e controle simulado validam **fluxos de interface, 4 layouts de tema, prévia/persistência/importação, navegação por teclado e controle simulado, glyphs dinâmicos, foco em modais, sidebar `sticky`, modo Desktop↔Fullscreen e acessibilidade** — **NUNCA** substituem nem são reportados como validação do IPC Tauri real, do SQLite real ou de controles físicos USB/Bluetooth.

### 4. Auditoria Prática no Executável Compilado (Windows Standalone)
- Ao final de cada fase, o binário release standalone (`cargo build --release --features custom-protocol`) é executado ao vivo no Windows WebView2 (sem Vite e sem transporte de teste), verificando `0` erros de console/página, persistência real entre fechamento e reabertura, redimensionamento (`1024×680` / `1360×820`), rolagem e inspeção direta do SQLite em `%LOCALAPPDATA%`.
- A partir da **Fase 3**, inclui validação prática real de inicialização de emulador, XMB/dashboard, jogo compatível, monitoramento de sessão e retorno ao Hub.
