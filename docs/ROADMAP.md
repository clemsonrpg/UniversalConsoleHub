# Roadmap — Universal Console Hub 3.0

## Phase 1 — Project Foundation (EM EXECUÇÃO)
- Estrutura limpa Tauri 2 + Rust + React + TypeScript + Vite.
- Skills validadas (`.agents/skills/`) e regras globais (`GEMINI.md`).
- Design System próprio ("Obsidian Hardware") e navegação por foco.
- Banco SQLite `STRICT` + WAL com 19 tabelas, triggers de integridade e backup/restauração validados.
- Core modules (`ProcessManager`, `SecurityManager`, `LoggingSystem`, `AdapterRegistry`).
- Contrato padronizado de adaptadores + preparação real do **RPCS3** (detecção, firmware, comandos, diagnóstico 4 estágios — sem lançamento ao vivo ainda).
- Telas iniciais funcionais conectadas ao backend real.

## Phase 2 — Universal Player (PENDENTE)
- Scanner assíncrono de diretórios com Tauri Channels.
- Detecção avançada de duplicatas e associação entre versões do mesmo jogo.
- Cache local de capas, página individual de jogo, Statistics Engine e Activity History completos.

## Phase 3 — Trophy Center (PENDENTE)
- Leitor somente leitura de troféus locais RPCS3 (`TROPCONF.SFM` + `TROPUSR.DAT`).
- Fluxo completo de notificações idempotentes, raridade honesta e histórico de desbloqueios.

## Phase 4 — First Authentic Console: PlayStation 3 via RPCS3 (PENDENTE)
- Inicialização real de XMB (`SystemBoot`) e jogos (`DirectGameBoot`) via `ProcessManager`.
- Monitoramento de sessão com heartbeat de tempo de jogo, encerramento seguro e retorno ao Hub.
- Confirmação real em tempo de execução (`RuntimeConfirmed`) do boot do XMB.

## Phase 5 — Additional Consoles (PENDENTE)
- Adaptadores reais para **PCSX2 (PS2)**, **Dolphin (Wii/GameCube)**, **xemu (Xbox Original)**, **Xenia Canary (Xbox 360)** e **shadPS4 (PS4 experimental)**.

## Phase 6 — Advanced Experience (PENDENTE)
- Ecossistema completo de controles Windows (SDL3/gilrs/XInput/DualShock/DualSense, calibração e handoff).
- Gerenciamento avançado de foco, fullscreen, resolução e áudio no Windows.
- Integrações oficiais/autorizadas (ex.: RetroAchievements com chave fornecida pelo usuário).

## Phase 7 — Release (PENDENTE)
- Auditoria QA (`release-quality-assurance`), otimização de memória/CPU idle, instalador NSIS/MSI, atualizações assinadas e documentação final.
