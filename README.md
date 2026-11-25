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

- **Fase 1 (Project Foundation):** Concluída e auditada no executável standalone Windows (`10` commits preservados em `main`: `d7b059a..00488f8`) — fundação Tauri 2/Rust/React/SQLite (`19` tabelas `STRICT`), contrato de adaptadores, preparação RPCS3 em 4 estágios, Design System "Obsidian Hardware" e telas base.
- **Fase 2 (Universal Player, Console Environments & Sessions Foundation — Opção A):** Planejamento arquitetural e schema (`Migration 0002`, `19 → 22` tabelas `STRICT` + índices aditivos) consolidados e prontos para execução sequencial em 10 Tasks sob TDD.
