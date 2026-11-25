# Compatibility Matrix — Universal Console Hub 3.0

As capacidades abaixo são **independentes por funcionalidade** (não mutuamente exclusivas) e separam claramente o que é **documentado upstream** do que foi **verificado em arquivo** ou **confirmado em tempo de execução** nos **7 estados independentes** (`emulator_launch`, `system_boot`, `original_dashboard`, `game_boot`, `per_game_compatibility`, `session_monitoring`, `hub_return`). Compatibilidade de jogos varia individualmente por título, build do emulador e hardware — nunca é prometida de forma universal.

## Estados Canônicos de Capacidade
- `FULL_SYSTEM_BOOT` — Inicialização completa do software de sistema do console (requer arquivos legais do usuário).
- `ORIGINAL_DASHBOARD` — Interface/menu original do console acessível via emulação real.
- `PARTIAL_DASHBOARD` — Interface original disponível apenas parcialmente ou com ressalvas documentadas.
- `DIRECT_GAME_ONLY` — Execução direta de jogos suportada (pode coexistir com `FULL_SYSTEM_BOOT` ou ser o único modo estável).
- `EXPERIMENTAL` — Emulador ou recurso em desenvolvimento ativo com estabilidade dependente de build/título.
- `UNSUPPORTED` — Recurso indisponível; desabilitado e explicado claramente.

## Consoles Prioritários (Fases 1–5 — Opção A Aprovada)

| Console | Emulador | System Boot | Original Dashboard | Direct Game Boot | Local Trophies | Online Services | Fase de Integração |
|---|---|---|---|---|---|---|---|
| **PlayStation 3** | **RPCS3** | `FULL_SYSTEM_BOOT` (requer `PS3UPDAT.PUP` instalada → `vsh.self`; confirmação runtime na **Fase 3**) | `ORIGINAL_DASHBOARD` (PS3 XMB; **não confirmado em runtime nas Fases 1–2**) | `DIRECT_GAME_ONLY` (`EBOOT.BIN`; execução ao vivo na **Fase 3**) | Suportado via leitura local somente leitura (`TROPCONF.SFM` / `TROPUSR.DAT` — **Fase 4**) | `UNSUPPORTED` (sem PSN oficial) | Prep Fase 1 · Ambiente/Scanner Fase 2 · Jogável **Fase 3** · Troféus **Fase 4** |
| **PlayStation 2** | **PCSX2 (Qt)** | `FULL_SYSTEM_BOOT` (requer BIOS PS2 legal do usuário; planejado **Fase 5**) | `ORIGINAL_DASHBOARD` (PS2 Browser; planejado **Fase 5**) | `DIRECT_GAME_ONLY` (`-batch -fullscreen`) | Via RetroAchievements (**Fase 6**) | `UNSUPPORTED` | Ambiente Fase 2 · Jogável **Fase 5** |
| **Nintendo Wii** | **Dolphin** | `FULL_SYSTEM_BOOT` (`-n 0000000100000002`, requer NAND legal do usuário) | `ORIGINAL_DASHBOARD` (Wii System Menu) | `DIRECT_GAME_ONLY` (`-b -e`) | Via RetroAchievements (GC — **Fase 6**) | `UNSUPPORTED` | Ambiente Fase 2 · Jogável **Fase 5** |
| **Xbox Original** | **xemu** | `FULL_SYSTEM_BOOT` (requer MCPX + BIOS + HDD qcow2 em `xemu.toml`) | `ORIGINAL_DASHBOARD` (Dashboard Xbox no HDD) | `DIRECT_GAME_ONLY` (`-dvd_path`) | `UNSUPPORTED` (sem API/formato local estável) | `UNSUPPORTED` | Ambiente Fase 2 · Jogável **Fase 5** |
| **Xbox 360** | **Xenia Canary** | `PARTIAL_DASHBOARD` (`dash.xex` limitado/avançado) | `PARTIAL_DASHBOARD` (Modo Integrado Alternativo explícito) | `DIRECT_GAME_ONLY` (modo primário estável) | `EXPERIMENTAL` (verificar formato local na **Fase 5**) | `UNSUPPORTED` | Ambiente Fase 2 · Jogável **Fase 5** |
| **PlayStation 4** | **shadPS4** | `UNSUPPORTED` | `UNSUPPORTED` (Orbis OS completo não suportado; Modo Integrado Alternativo explícito) | `EXPERIMENTAL` + `DIRECT_GAME_ONLY` | `EXPERIMENTAL` (em evolução upstream) | `UNSUPPORTED` | Ambiente Fase 2 · Jogável **Fase 5** |

## Expansão Arquitetural Futura
A estrutura de `consoles`, `console_runtime_verifications` e `AdapterRegistry` já está preparada para PS1, GameCube, NES, SNES, N64, Wii U, PSP, PS Vita, Xbox One (quando existir emulação compatível) e Switch (apenas se houver emulação legítima e tecnicamente disponível).
