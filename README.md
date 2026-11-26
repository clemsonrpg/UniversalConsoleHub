# Universal Console Hub 3.0

**The Authentic Multi-Console Ecosystem for Windows 10/11**

Universal Console Hub é um aplicativo desktop local-first (Tauri 2 + Rust + SQLite + React + TypeScript + Vite) projetado para unificar emulação autêntica de consoles, biblioteca multiplataforma, perfil universal de jogador, estatísticas reais de sessões, ecossistema de controles Windows e central de troféus/conquistas.

## Filosofia de Produto

- **Fidelidade máxima aos consoles reais:** O Hub organiza, valida e inicia emuladores reais preservando inicializações, dashboards, sons e menus originais sempre que suportados pela emulação.
- **Zero dashboards falsas:** O Hub tem sua própria interface desktop de gerenciamento; nunca substitui ou simula o sistema operacional original de um console fingindo ser emulação.
- **Zero ativos proprietários:** Este repositório não contém nem distribui BIOS, firmware (`PS3UPDAT.PUP`), NAND, MCPX, chaves criptográficas ou jogos protegidos. O usuário fornece seus próprios arquivos obtidos legalmente.
- **Transparência técnica:** Capacidades de consoles e emuladores são modeladas por funcionalidade independente e distinguem documentação upstream, presença de arquivos, construção de comando e confirmação em tempo de execução.

## Stack Tecnológica

- **Desktop & Backend:** Tauri 2 + Rust (`x86_64-pc-windows-msvc`)
- **Banco de Dados:** SQLite embarcado (`rusqlite` bundled, WAL, `STRICT`, `foreign_keys = ON`, migrations versionadas, backup/restauração via `VACUUM INTO`)
- **Frontend:** React + TypeScript estrito + Vite + CSS Modular
- **Testes:** Rust Unit (`cargo test --lib`), Real Backend Integration (`cargo test --test '*'`), Frontend Unit (Vitest), UI E2E (Playwright)

## Documentação

- [Arquitetura](docs/ARCHITECTURE.md) · [ADR-001](docs/adr/0001-clean-architecture-tauri2.md)
- [Roadmap](docs/ROADMAP.md)
- [Matriz de Compatibilidade](docs/COMPATIBILITY_MATRIX.md)
- [Integração de Emuladores](docs/EMULATOR_INTEGRATION.md)
- [Sistema de Troféus](docs/TROPHY_SYSTEM.md)
- [Perfil Universal](docs/UNIVERSAL_PROFILE.md)
- [Schema do Banco de Dados](docs/DATABASE_SCHEMA.md)
- [Estratégia de Testes](docs/TESTING.md)
- [Skills & Workflow](docs/SKILLS_AND_WORKFLOW.md)
- [Limitações Conhecidas](docs/KNOWN_LIMITATIONS.md)

## Status Atual

- **Fase 1 (Project Foundation):** Concluída e auditada (`10` commits preservados em `main`: `d7b059a..00488f8`) — fundação Tauri 2/Rust/React/SQLite (`19` tabelas `STRICT`), contrato de adaptadores, preparação RPCS3 em 4 estágios, Design System "Obsidian Hardware" e telas base.
- **Fase 2 (Universal Player, Console Environments & Sessions Foundation — Opção A):** Concluída e auditada (`11` commits em `main`: `c5f6660..a85a6f2`) — `Migration 0002` (`22` tabelas `STRICT` + índices aditivos), ambientes independentes por console (`ConsoleEnvironmentPage`), scanner assíncrono com parser binário `PARAM.SFO`, `CoverCache` seguro via `protocol-asset`, `GameDetailsPage`, `StatisticsPage`, recuperação idempotente de sessões órfãs via `last_heartbeat_at` e restauração SQLite ao vivo pela UI com confirmação e rollback automático.
- **Fase 3 (Perfis Reais de Console + Integração Ao Vivo PS3 via RPCS3 — Opção A):** Implementada e verificada nas 3 camadas automatizadas + auditoria prática no `.exe` standalone (`10` commits em `main`: consolidação documental + 9 Tasks TDD) — `Migration 0003` (`24` tabelas `STRICT`: `console_profiles` #23 e `console_local_users` #24 + colunas aditivas em `game_versions`, `game_sessions`, `trophy_unlocks`), `Ps3ProfileProvider` somente leitura (`RPCS3.log`, `version.txt`, múltiplos `localusername`, `savedata`, `ICON0.PNG`/`PIC1.PNG`, detecção honesta de `trophy/`), `ConsoleProfileSyncService` automático/incremental/idempotente preservando histórico em `missing_on_disk`, `LiveEmulatorRunner` + `WindowsWindowManager` (spawn direto sem shell + `WM_CLOSE` gracioso), `LiveSessionOrchestrator` (sessão única, heartbeat 60s, pós-sync, 7 verificações independentes), 38 comandos IPC + eventos em tempo real e remoção completa do formulário manual da `LibraryPage`. *Nota honesta:* a confirmação runtime final do XMB (`original_dashboard`) e da compatibilidade individual de títulos (`per_game_compatibility`) depende de execução manual com uma instalação real do `rpcs3.exe`, firmware `PS3UPDAT.PUP` e jogos PS3 do usuário.
- **Próxima Fase (Fase 4 — Opção A):** Central Universal de Troféus integrada às sessões reais do PS3 (`TROPCONF.SFM` + `TROPUSR.DAT` somente leitura).
