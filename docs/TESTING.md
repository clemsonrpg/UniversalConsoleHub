# Testing Strategy — Universal Console Hub 3.0

## Separação Estrita das 3 Camadas de Testes

Para evitar falsas sensações de cobertura, todos os relatórios e comandos distinguem rigorosamente três camadas:

### 1. Testes Unitários (Isolados)
- **Rust (`cargo test --lib`):** Entidades de domínio, modelo de capacidades multi-dimensão, invariantes de origem de troféus, serialização DTO, lógica pura de parsing/validação.
- **Frontend (`npm.cmd run test` — Vitest):** Componentes do Design System (`CapabilityBadge`, `ConsoleCard`), hooks de navegação por foco, validação de cliente IPC.
- *O que NÃO validam:* Persistência real em disco, IPC Tauri nativo ou execução real de emuladores.

### 2. Testes de Integração com Backend Real (`cargo test --test '*'`)
- **SQLite Real:** Arquivo `.db` real em disco com modo WAL, migrations, triggers de imutabilidade de `origin`, constraints `STRICT`/`UNIQUE`/`FOREIGN KEY`, criação de backup via `VACUUM INTO` **e restauração completa verificada**.
- **Sistema Windows Real:** `ProcessManager` executando binários reais do sistema com caminhos contendo espaços e caracteres Unicode (`Área de Trabalho`), guarda de sessão única e drenagem de pipes.
- **Adaptador RPCS3 Real (Preparação):** Inspeção de árvores reais de diretórios temporários (`rpcs3.exe`, `dev_flash/vsh/module/vsh.self`, arquivo `.PUP` sintético) e geração real de comandos.
- **Application Managers Reais:** Casos de uso conectados diretamente ao `DatabaseManager` e `AdapterRegistry` reais (sem mocks).

### 3. Testes E2E de Interface (`npm.cmd run test:e2e` — Playwright)
- Executam o aplicativo React/Vite em navegador Chromium com um **transporte de teste injetado** exclusivo para automação de UI.
- **Declaração explícita:** Testes Playwright com transporte injetado validam **fluxos de interface, navegação por teclado, acessibilidade, badges honestos e contratos de tela** — **NUNCA** substituem nem são reportados como validação do IPC Tauri real ou do SQLite real (que são validados pela Camada 2 + compilação real `cargo build` do binário Tauri).
