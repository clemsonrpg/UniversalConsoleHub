# Architecture — Universal Console Hub 3.0

## Filosofia Arquitetural: Ecossistema de Consoles Jogáveis

O Universal Console Hub separa explicitamente o **Hub Global** (Perfil Universal, Biblioteca Global, Troféus, Estatísticas, Configurações) do **Ambiente Independente de Console (`ConsoleEnvironment`)**. Cada console dispõe de biblioteca própria, pastas de descoberta, configurações independentes, adaptador isolado, estado de sessão, 7 estados independentes de verificação e integração direta ao Perfil Universal.

## Regra de Dependência (Clean Architecture / Ports & Adapters)

As dependências de código-fonte apontam **estritamente para dentro**:

```text
React/TS UI (src/)
 ├── Hub Shell (Visão Global, Perfil, Estatísticas, Troféus, Configurações)
 └── ConsoleEnvironment Shell (Ambiente Independente: Modo A Autêntico / Modo B Integrado Explícito)
   │ Tauri 2 IPC (31 Commands [18 Fase 1 + 13 Fase 2], Events, Streaming Channels + Dialog Plugin)
   ▼
IPC Layer (src-tauri/src/ipc/)
   ▼
Application Layer (src-tauri/src/application/)
 ├── EnvironmentService (ConsoleEnvironment, Configs & Verificações por Console)
 ├── ScannerService (Descoberta & Importação via tokio::task::spawn_blocking + Tauri Channels)
 ├── SessionService / SessionOrchestrator (Heartbeat, Crash Recovery Idempotente)
 ├── StatisticsService & ActivityHistory
 ├── Profile & Library Services (compatível com AppServices::new da Fase 1)
 ├── Trophy & Sync Services (Fase 4)
 └── BackupService & SettingsService
   ▼
Domain Entities & Ports (src-tauri/src/domain/)  ◄── Zero dependência de Tauri, SQLite ou CLI
   ▲                               ▲
   │ implementa ports              │ implementa ports
Infrastructure (db/, system/)   Emulator & Discovery Adapters (adapters/)
```

## Módulos Principais e Garantias Arquiteturais

### CORE & CONSOLE EXPERIENCE
- **Application Core (`lib.rs`, `application/`)**: Composição via Dependency Injection (`Arc<...>`), particionada por domínio na Fase 2 preservando integralmente as assinaturas da Fase 1 (`AppServices::new` + novo `AppServices::with_dirs`).
- **Console Manager & ConsoleEnvironment**: Catálogo de consoles, capacidades independentes por funcionalidade (`FULL_SYSTEM_BOOT`, `ORIGINAL_DASHBOARD`, `PARTIAL_DASHBOARD`, `DIRECT_GAME_ONLY`, `EXPERIMENTAL`, `UNSUPPORTED`), configurações próprias (`console_settings`) e registro granular de verificação (`console_runtime_verifications`, vinculado a `consoles`, `emulators` e opcionalmente `games` via `scope_key`) separando estritamente **7 estados independentes**: `emulator_launch`, `system_boot`, `original_dashboard`, `game_boot`, `per_game_compatibility`, `session_monitoring` e `hub_return`.
- **ConsoleGameScanner & GameDiscoveryPort**: Scanner assíncrono em `tokio::task::spawn_blocking` com progresso via `tauri::ipc::Channel<ScanProgressEvent>`, parser binário seguro de `PARAM.SFO` (PS3 na Fase 2; parsers específicos dos demais consoles na Fase 5) e deduplicação canônica idempotente sobre `games` (#7) + `game_versions` (#8).
- **SessionOrchestrator (`SessionService`)**: Máquina de estados (`Idle -> Launching -> Running -> Exiting -> Idle`), persistência em `game_sessions` (referenciando `game_versions` via `ON DELETE SET NULL`), heartbeat periódico e recuperação idempotente de sessões interrompidas (`reconcile_orphaned_sessions` disparada no `setup()` de `lib.rs`) utilizando exclusivamente `last_heartbeat_at` (nunca contabilizando tempo indefinido sem evidência de atividade), marcando sessões recuperadas com `exit_status = 'recovered_after_crash'` e impedindo contagem duplicada.
- **Emulator Manager & Adapter Registry**: Registro extensível via trait `EmulatorAdapter` (RPCS3 preparado na Fase 1, ambiente/scanner na Fase 2, 100% jogável na Fase 3; PCSX2/Dolphin/xemu/Xenia/shadPS4 um por vez na Fase 5).
- **Firmware Manager**: Validação local de pacotes e arquivos de sistema fornecidos pelo usuário (estrutura, caminho normalizado sem prefixo verbatim `\\?\`, SHA-256 streaming), nunca distribuindo arquivos proprietários.
- **Process Manager, Window Manager & Gamepad Handoff**: Execução Windows segura via `Command::new(exe).args(...)` (zero shell), drenagem lossy resistente a pipes não-UTF-8; gerenciamento de janela/foco e handoff de controle entram em operação ao vivo na Fase 3 (PS3) e Fase 6 (SDL3/XInput/DualSense completo).

### PLAYER ECOSYSTEM
- **Universal Profile Manager**: Perfis locais offline com estatísticas reais derivadas de sessões e troféus.
- **Game Library Manager & CoverCache**: Identidade canônica (`games`, Tabela #7) separada de versões/edições/regiões (`game_versions`, Tabela #8 já criada na Migration `0001` e indexada adicionalmente na Migration `0002`), biblioteca global e biblioteca isolada por console, página individual de jogo e `CoverCache` seguro (magic bytes PNG/JPEG/WebP reais, limite de 10 MB, SHA-256, bloqueio de path traversal, validação backend obrigatória garantindo que todo `cover_path` servido ao frontend pertença exclusivamente a `<app_local_data_dir>/cache/covers/`). No runtime Tauri 2 Windows, a entrega local de capas utiliza feature Cargo `protocol-asset`, `assetProtocol.scope = ["$APPLOCALDATA/cache/covers/**"]` e diretiva CSP `img-src` permitindo `asset: http://asset.localhost https://asset.localhost`.
- **Trophy Manager & Achievement Provider Manager (Fase 4)**: Separação rígida e imutável entre origens `official`, `emulator` e `local` garantida no domínio e por triggers SQLite, com leitor somente leitura de troféus RPCS3 (`TROPCONF.SFM` + `TROPUSR.DAT`) acoplado ao fim das sessões reais do PS3.
- **Statistics Engine & Activity History**: Baseados exclusivamente em sessões reais gravadas no banco (tempo total, por console, por jogo, evolução mensal e taxa de conclusão).

### SYSTEM
- **Database Manager & Migrations**: SQLite `STRICT` + WAL + `foreign_keys = ON` + `busy_timeout = 5000` + `cache_size = -64000` + migrations versionadas transacionais (`0001` com 19 tabelas na Fase 1; `0002` estritamente aditiva elevando para **22 tabelas** na Fase 2 + índices adicionais em `game_versions` e `game_sessions`). Rejeita explicitamente bancos com `user_version > LATEST_SCHEMA_VERSION` sem alterá-los.
- **Backup Manager (Windows-Safe Live Restore)**:
  1. Validação integral do backup antes de qualquer alteração (`PRAGMA quick_check`, `PRAGMA foreign_key_check`, `1 <= user_version <= LATEST_SCHEMA_VERSION`).
  2. Criação de snapshot pré-restauração consistente do banco ativo (`pre_restore_<utc>.db` via `VACUUM INTO`).
  3. Staging limpo do backup via `VACUUM INTO` em arquivo temporário.
  4. Liberação efetiva de todas as conexões abertas sob `Mutex<Connection>` (swap provisório para conexão in-memory) antes da limpeza dos sidecars `-wal`/`-shm` e troca de arquivos com `.db.rollback.tmp` e rollback automático em caso de falha.
  5. Reabertura obrigatória com WAL e `foreign_keys = ON`, auto-migração transacional caso o backup restaurado seja `v1`, e verificação de integridade pós-restauração.
- **Configuration Manager, Logging System (`tracing`), Security Manager**: Validação backend de caminhos (nunca confiar em caminhos enviados pelo frontend), proteção contra path traversal, normalização sem `\\?\` e logs estruturados.
