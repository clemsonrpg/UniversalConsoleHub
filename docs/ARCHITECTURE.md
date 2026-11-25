# Architecture — Universal Console Hub 3.0

## Regra de Dependência (Clean Architecture / Ports & Adapters)

As dependências de código-fonte apontam **estritamente para dentro**:

```text
React/TS UI (src/)
   │ Tauri 2 IPC (invoke / events / channels)
   ▼
IPC Layer (src-tauri/src/ipc/)
   ▼
Application Managers (src-tauri/src/application/)
   ▼
Domain Entities & Traits (src-tauri/src/domain/)  ◄── Zero dependência de Tauri, SQLite ou CLI
   ▲                               ▲
   │ implementa ports              │ implementa ports
Infrastructure (db/, system/)   Emulator Adapters (adapters/rpcs3.rs, ...)
```

## Módulos Principais

### CORE
- **Application Core (`lib.rs`, `application/`)**: Composição de dependências via `Arc<...>` (Dependency Injection explícita), sem singletons globais mutáveis escondidos.
- **Console Manager**: Catálogo de consoles e capacidades independentes por funcionalidade (`system_boot`, `original_dashboard`, `direct_game_boot`, `save_management`, `local_trophies`, `online_services`).
- **Emulator Manager & Adapter Registry**: Registro e detecção de emuladores via trait `EmulatorAdapter`.
- **Firmware Manager**: Validação local de pacotes e arquivos de sistema fornecidos pelo usuário (estrutura, caminho, SHA-256 streaming), nunca distribuindo arquivos proprietários.
- **Process Manager**: Execução segura no Windows via `Command::new(exe).args(...)` (zero `cmd.exe /C`), guarda de sessão única, drenagem assíncrona de stdout/stderr e encerramento gracioso.
- **Compatibility Manager**: Agregação de estados verificados por funcionalidade (`FULL_SYSTEM_BOOT`, `ORIGINAL_DASHBOARD`, `PARTIAL_DASHBOARD`, `DIRECT_GAME_ONLY`, `EXPERIMENTAL`, `UNSUPPORTED`) e nível de confirmação (`Unverified`, `DocumentedUpstream`, `FilesVerified`, `RuntimeConfirmed`).
- **Window Manager & Controller Manager**: Contratos arquiteturais definidos na Fase 1; controle de foco/fullscreen avançado e SDL3/XInput completos nas Fases 4 e 6.

### PLAYER ECOSYSTEM
- **Universal Profile Manager**: Perfis locais offline com estatísticas reais derivadas de sessões e troféus.
- **Game Library Manager**: Identidade canônica de jogos (`games`) separada de versões/edições/regiões (`game_versions`), busca, filtros e favoritos.
- **Trophy Manager & Achievement Provider Manager**: Separação rígida entre origens `official`, `emulator` e `local` com imutabilidade de origem garantida no domínio e por triggers SQLite.
- **Statistics Engine & Activity History**: Baseados exclusivamente em sessões reais gravadas no banco.

### SYSTEM
- **Database Manager**: SQLite `STRICT` + WAL + `foreign_keys = ON` + `busy_timeout = 5000` + `cache_size = -64000` + migrations versionadas.
- **Backup Manager**: Backups consistentes em modo WAL via `VACUUM INTO`, verificação `PRAGMA quick_check` e restauração validada.
- **Configuration Manager, Logging System (`tracing`), Security Manager**: Validação de caminhos, proteção contra path traversal e logs estruturados em `%LOCALAPPDATA%\UniversalConsoleHub\logs`.
