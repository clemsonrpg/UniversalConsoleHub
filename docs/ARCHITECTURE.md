# Architecture — Universal Console Hub 3.0

## Filosofia Arquitetural: Ecossistema de Consoles Jogáveis

O Universal Console Hub separa explicitamente o **Hub Global** (Perfil Universal, Biblioteca Global, Troféus, Estatísticas, Configurações) do **Ambiente Independente de Console (`ConsoleEnvironment`)**. Cada console dispõe de biblioteca própria, pastas de descoberta, configurações independentes, adaptador isolado, estado de sessão, 7 estados independentes de verificação e integração direta ao Perfil Universal.

## Regra de Dependência (Clean Architecture / Ports & Adapters)

As dependências de código-fonte apontam **estritamente para dentro**:

```text
React/TS UI (src/)
 ├── Hub Shell (Visão Global, Perfil Universal, Biblioteca Somente Descoberta, Estatísticas, Troféus, Configurações)
 └── Console Profile & Environment Shell (Perfil Real do Console: Nativo / Hub / Indisponível + Modo A Autêntico / Modo B Integrado)
   │ Tauri 2 IPC (Commands, Events session:* / profile:synced, Streaming Channels + Dialog Plugin)
   ▼
IPC Layer (src-tauri/src/ipc/)
   ▼
Application Layer (src-tauri/src/application/)
 ├── ConsoleProfileSyncService (Sincronização Automática, Incremental & Idempotente por Console)
 ├── LiveSessionOrchestrator (Execução Real RPCS3, Sessão Única, Heartbeat 60s, Pós-Sync, 7 Verificações)
 ├── EnvironmentService (ConsoleEnvironment, Configs & Verificações por Console)
 ├── ScannerService (Descoberta & Importação via tokio::task::spawn_blocking + Tauri Channels)
 ├── SessionService (Persistência & Crash Recovery Idempotente por last_heartbeat_at)
 ├── StatisticsService & ActivityHistory
 ├── Profile & Library Services (compatível com AppServices::new da Fase 1)
 ├── Trophy & Sync Services (Fase 4 — preparação estrutural em trophies/trophy_unlocks + console_local_users)
 └── BackupService & SettingsService
   ▼
Domain Entities & Ports (src-tauri/src/domain/)  ◄── Zero dependência de Tauri, SQLite ou CLI
   ▲                               ▲
   │ implementa ports              │ implementa ports
Infrastructure (db/, system/)   Emulator, Profile & Discovery Adapters (adapters/)
```

## Módulos Principais e Garantias Arquiteturais

### CORE & CONSOLE EXPERIENCE
- **Application Core (`lib.rs`, `application/`)**: Composição via Dependency Injection (`Arc<...>`), particionada por domínio preservando integralmente as assinaturas da Fase 1 (`AppServices::new` + `AppServices::with_dirs`).
- **Console Profile vs. Local Player Identity (Separação Obrigatória — Fase 3)**:
  - **Identidade do Console (`console_profiles`, Tabela #23):** armazena dados reais do console e do emulador (`emulator_status`, versão detectada do emulador, versão real do firmware, diretório raiz, contagem de saves nativos, prontidão honesta de troféus `phase4_planned`/`unavailable`, metadados nativos JSON e diagnóstico da última sincronização).
  - **Identidade de Usuários Locais do Console (`console_local_users`, Tabela #24):** representa exclusivamente usuários locais reais expostos pela plataforma (ex.: `dev_hdd0/home/<id>/localusername` no RPCS3), suportando múltiplos usuários por console e vínculo opcional a `game_sessions` e `trophy_unlocks`. **Nunca inventa usuários** quando a plataforma não expõe identificação.
  - **Origem Explícita (`DataOrigin`):** todo campo de perfil distingue `native_platform` (extraído de arquivos reais), `hub_recorded` (medido pelo Hub) e `unavailable` (não suportado ou não verificado).
- **ConsoleProfileProvider & ConsoleProfileSyncService (Fase 3)**:
  - Cada plataforma implementa seu próprio provedor somente leitura em `adapters/profiles/` (começando por `Ps3ProfileProvider`).
  - Sincronização automática e idempotente disparada em alterações de pastas, conclusão de scans, configuração de emulador/firmware, pré-lançamento, encerramento de sessão e botão manual.
  - **Sincronização Persistente:** quando um jogo ou o próprio emulador fica indisponível no disco, o estado transita para `missing_on_disk` preservando 100% do histórico de jogos, sessões e estatísticas.
- **Console Manager & ConsoleEnvironment**: Catálogo de consoles, capacidades independentes por funcionalidade (`FULL_SYSTEM_BOOT`, `ORIGINAL_DASHBOARD`, `PARTIAL_DASHBOARD`, `DIRECT_GAME_ONLY`, `EXPERIMENTAL`, `UNSUPPORTED`), configurações próprias (`console_settings`) e registro granular de verificação (`console_runtime_verifications`, vinculado a `consoles`, `emulators` e opcionalmente `games` via `scope_key`) separando estritamente **7 estados independentes**: `emulator_launch`, `system_boot`, `original_dashboard`, `game_boot`, `per_game_compatibility`, `session_monitoring` e `hub_return`.
- **ConsoleGameScanner & GameDiscoveryPort**: Scanner assíncrono em `tokio::task::spawn_blocking` com progresso via `tauri::ipc::Channel<ScanProgressEvent>`, parser binário seguro de `PARAM.SFO` (PS3 na Fase 2; artes nativas `ICON0.PNG`/`PIC1.PNG` na Fase 3; parsers específicos dos demais consoles na Fase 5) e deduplicação canônica idempotente sobre `games` (#7) + `game_versions` (#8). Na Fase 3, o cadastro manual de jogos é removido da `LibraryPage`, consolidando a descoberta automática como única fonte de verdade.
- **LiveSessionOrchestrator & SessionService**: Máquina de estados de sessão única (`Idle -> Launching -> Running -> Exiting -> Idle`), persistência imediata em `game_sessions` (referenciando `game_versions` e opcionalmente `console_local_users` via `ON DELETE SET NULL`), heartbeat real de 60s em Tokio, recuperação idempotente de sessões interrompidas (`reconcile_orphaned_sessions` no `setup()` de `lib.rs`) usando exclusivamente `last_heartbeat_at` (`exit_status = 'recovered_after_crash'`) e pós-sincronização automática do perfil ao encerrar.
- **Emulator Manager & Adapter Registry**: Registro extensível via trait `EmulatorAdapter` (RPCS3 preparado na Fase 1, ambiente/scanner na Fase 2, 100% jogável na Fase 3; PCSX2/Dolphin/xemu/Xenia/shadPS4 um por vez na Fase 5).
- **Firmware Manager**: Validação local de pacotes e arquivos de sistema fornecidos pelo usuário (estrutura, caminho normalizado sem prefixo verbatim `\\?\`, SHA-256 streaming), nunca distribuindo arquivos proprietários.
- **Process Manager, LiveEmulatorRunner, WindowsWindowManager & Gamepad Handoff**: Execução Windows segura via `Command::new(exe).args(...)` (zero shell), drenagem assíncrona limitada resistente a pipes não-UTF-8, minimização/restauração segura de foco da janela do Hub e suspensão de navegação durante jogo na Fase 3 (PS3) e ecossistema completo de controles na Fase 6.

### PLAYER ECOSYSTEM
- **Universal Profile Manager**: Perfis locais offline com estatísticas reais derivadas de sessões e troféus.
- **Game Library Manager & CoverCache**: Identidade canônica (`games`, Tabela #7) separada de versões/edições/regiões (`game_versions`, Tabela #8 já criada na Migration `0001` e indexada adicionalmente na Migration `0002`), biblioteca global e biblioteca isolada por console, página individual de jogo e `CoverCache` seguro (magic bytes PNG/JPEG/WebP reais, limite de 10 MB, SHA-256, bloqueio de path traversal, validação backend obrigatória garantindo que todo `cover_path` servido ao frontend pertença exclusivamente a `<app_local_data_dir>/cache/covers/`). No runtime Tauri 2 Windows, a entrega local de capas utiliza feature Cargo `protocol-asset`, `assetProtocol.scope = ["$APPLOCALDATA/cache/covers/**"]` e diretiva CSP `img-src` permitindo `asset: http://asset.localhost https://asset.localhost`.
- **Trophy Manager & Achievement Provider Manager (Fase 4)**: Separação rígida e imutável entre origens `official`, `emulator` e `local` garantida no domínio e por triggers SQLite, com leitor somente leitura de troféus RPCS3 (`TROPCONF.SFM` + `TROPUSR.DAT`) acoplado ao fim das sessões reais do PS3.
- **Statistics Engine & Activity History**: Baseados exclusivamente em sessões reais gravadas no banco (tempo total, por console, por jogo, evolução mensal e taxa de conclusão).

### SYSTEM
- **Database Manager & Migrations**: SQLite `STRICT` + WAL + `foreign_keys = ON` + `busy_timeout = 5000` + `cache_size = -64000` + migrations versionadas transacionais (`0001` com 19 tabelas na Fase 1; `0002` com 22 tabelas na Fase 2; `0003` planejada na Fase 3 elevando para **24 tabelas `STRICT`** com `console_profiles` #23 e `console_local_users` #24 + colunas aditivas em `game_versions`, `game_sessions` e `trophy_unlocks`). Rejeita explicitamente bancos com `user_version > LATEST_SCHEMA_VERSION` sem alterá-los.
- **Backup Manager (Windows-Safe Live Restore)**:
  1. Validação integral do backup antes de qualquer alteração (`PRAGMA quick_check`, `PRAGMA foreign_key_check`, `1 <= user_version <= LATEST_SCHEMA_VERSION`).
  2. Criação de snapshot pré-restauração consistente do banco ativo (`pre_restore_<utc>.db` via `VACUUM INTO`).
  3. Staging limpo do backup via `VACUUM INTO` em arquivo temporário.
  4. Liberação efetiva de todas as conexões abertas sob `Mutex<Connection>` (swap provisório para conexão in-memory) antes da limpeza dos sidecars `-wal`/`-shm` e troca de arquivos com `.db.rollback.tmp` e rollback automático em caso de falha.
  5. Reabertura obrigatória com WAL e `foreign_keys = ON`, auto-migração transacional caso o backup restaurado seja `v1`, e verificação de integridade pós-restauração.
- **Configuration Manager, Logging System (`tracing`), Security Manager**: Validação backend de caminhos (nunca confiar em caminhos enviados pelo frontend), proteção contra path traversal, normalização sem `\\?\` e logs estruturados.
