# ADR-001: Clean Architecture com Tauri 2, Rust, SQLite e Adaptadores Isolados

- **Status:** Aceito (Fase 1)
- **Data:** 2026-11-25

## Contexto
Universal Console Hub precisa integrar múltiplos emuladores desktop no Windows (RPCS3, PCSX2, Dolphin, xemu, Xenia Canary, shadPS4), persistir bibliotecas, perfis, sessões e troféus offline com integridade forte e entregar uma UI responsiva para teclado, mouse e controles (1080p/4K).

## Decisão
1. **Desktop Runtime:** Adotar Tauri 2 (WebView2 nativo no Windows) com backend Rust e capabilities de menor privilégio em `src-tauri/capabilities/default.json`.
2. **Arquitetura Limpa:** Isolar entidades e traits (`EmulatorAdapter`, `AchievementProvider`, repositórios) em `src-tauri/src/domain/` sem dependência de Tauri, SQLite ou CLIs de emuladores.
3. **Banco Local:** Utilizar SQLite via `rusqlite` (`bundled`) com tabelas `STRICT`, WAL, `foreign_keys = ON`, migrations versionadas, triggers de imutabilidade de origem de troféus e backups/restauração via `VACUUM INTO`.
4. **Capacidades Multi-Dimensão:** Substituir classificação exclusiva única por capacidades independentes por funcionalidade acompanhadas de nível de confirmação (`DocumentedUpstream`, `FilesVerified`, `RuntimeConfirmed`).
5. **Testabilidade Honesta:** Separar estritamente testes unitários, testes de integração com backend real (SQLite/OS/filesystem reais) e testes E2E de UI (Playwright).

## Consequências
- Adição de novos emuladores na Fase 5 não exige reescrita do domínio, do banco nem da UI.
- O frontend jamais contém lógica de CLI de emuladores ou acesso direto a arquivos do sistema.
- Cada afirmação de suporte na UI é dirigida por dados estruturados do backend.
