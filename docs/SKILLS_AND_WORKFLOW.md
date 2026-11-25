# Skills & Workflow — Universal Console Hub 3.0

## Skills Instaladas no Projeto (`.agents/skills/`)

### Skills Exclusivas do Projeto (8)
1. `authentic-console-emulation` — Capacidades multi-dimensão, boot autêntico vs. direto, onboarding legal de firmware.
2. `emulator-integration` — Contrato `EmulatorAdapter`, ciclo de vida seguro de processos Windows, CLI auditadas.
3. `universal-trophy-system` — Modelo canônico, origem imutável (`official`/`emulator`/`local`), providers honestos.
4. `universal-player-profile` — Identidade offline, biblioteca unificada, sessões reais e estatísticas.
5. `windows-gamepad-system` — XInput/SDL/DualShock/DualSense, deadzones radiais, handoff limpo ao emulador.
6. `console-ui-fidelity` — Preservação das interfaces originais e proibição de recriações falsas.
7. `release-quality-assurance` — Gates de qualidade, segurança Windows, performance e empacotamento.
8. `sqlite-engineering` — Pragmas WAL, tabelas `STRICT`, migrations atômicas, triggers e `VACUUM INTO`.

### Skills Comunitárias Verificadas (9)
`frontend-design`, `vercel-react-best-practices`, `web-design-guidelines`, `typescript-pro`, `tauri-v2`, `rust-best-practices`, `clean-architecture`, `vitest`, `playwright-expert`.

### Skills Globais (`superpowers`)
`test-driven-development`, `systematic-debugging`, `verification-before-completion`, `writing-plans`, `executing-plans`, `subagent-driven-development`, `requesting-code-review`.

## Fluxo de Engenharia (TDD Obrigatório)
1. Inspecionar código/contrato existente.
2. Escrever teste falhando (**RED**) e confirmar a falha esperada.
3. Implementar o mínimo limpo (**GREEN**) e rodar toda a suíte afetada.
4. Refatorar (**REFACTOR**) mantendo todos os testes verdes e `clippy`/`tsc` limpos.
5. Commits pequenos e descritivos por Task.
