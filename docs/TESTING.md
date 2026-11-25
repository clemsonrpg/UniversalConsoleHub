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
  - **Fase 2 — Backup & Restauração Ao Vivo no Windows:** Criação via `VACUUM INTO`, inspeção pré-restauração, `restore_from_backup_live` com conexão WAL ativa (incluindo snapshot pré-restauração, limpeza de `-wal`/`-shm`, reabertura WAL/FK, rejeição de backup corrompido e rollback seguro preservando o banco original).
- **Sistema Windows Real (`system_integration.rs`, `cover_and_repo_integration.rs`):**
  - `ProcessManager` executando binários reais do Windows com espaços/Unicode (`Área de Trabalho`), drenagem lossy de pipes não-UTF-8 e guarda de sessão única.
  - `SecurityManager` (SHA-256 streaming, remoção do prefixo `\\?\`, anti path-traversal).
  - **Fase 2 — `CoverCache`:** Magic bytes reais PNG/JPEG/WebP, rejeição de arquivos falsificados ou > 10 MB, deduplicação SHA-256 e garantia de que todo caminho retornado pertence exclusivamente ao diretório autorizado `cache/covers/`.
- **Adaptadores & Scanner Reais (`rpcs3_adapter_integration.rs`, `scanner_integration.rs`):**
  - Inspeção real de árvores temporárias (`rpcs3.exe`, `dev_flash/vsh/module/vsh.self`, `PS3UPDAT.PUP` sintético) com 4 estágios independentes.
  - **Fase 2 — `ConsoleGameScanner`:** Scan recursivo em diretórios Unicode com `PARAM.SFO` + `EBOOT.BIN` sintéticos, extração real de serial/região e idempotência em re-scans.
- **Application Services Reais (`application_integration.rs`, `phase2_services_integration.rs`):**
  - Casos de uso conectados diretamente ao `DatabaseManager` e `AdapterRegistry` reais (sem mocks).
  - **Fase 2 — `SessionOrchestrator` & `StatisticsService`:** `start` → `heartbeat` → crash simulado (`ended_at IS NULL`) → `reconcile_orphaned_sessions` usando exclusivamente `last_heartbeat_at` (`recovered_after_crash`), idempotência em múltiplas chamadas e ausência de tempo contabilizado sem evidência de heartbeat.

### 3. Testes E2E de Interface (`npm.cmd run test:e2e` — Playwright)
- Executam o aplicativo React/Vite em navegador Chromium com um **transporte de teste injetado (`window.__UCH_TEST_TRANSPORT__`)** exclusivo para automação de UI.
- **Declaração explícita:** Testes Playwright com transporte injetado validam **fluxos de interface, navegação por teclado, sidebar `sticky`, acessibilidade, badges honestos, banner de Ambiente Integrado Alternativo e modais de confirmação** — **NUNCA** substituem nem são reportados como validação do IPC Tauri real ou do SQLite real.

### 4. Auditoria Prática no Executável Compilado (Windows Standalone)
- Ao final de cada fase, o binário release standalone (`cargo build --release --features custom-protocol`) é executado ao vivo no Windows WebView2 (sem Vite e sem transporte de teste), verificando `0` erros de console/página, persistência real entre fechamento e reabertura, redimensionamento (`1024×680` / `1360×820`), rolagem e inspeção direta do SQLite em `%LOCALAPPDATA%`.
- A partir da **Fase 3**, inclui validação prática real de inicialização de emulador, XMB/dashboard, jogo compatível, monitoramento de sessão e retorno ao Hub.
