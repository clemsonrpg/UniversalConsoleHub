# Navegação por Controle & Modos Desktop / Fullscreen TV — Universal Console Hub 3.0

O **Universal Console Hub 3.0** foi projetado para uso completo sem mouse nem teclado, mantendo a mesma precisão em monitores Desktop e em TVs a distância (modo **Fullscreen 10-foot**).

---

## 1. Arquitetura de Entrada (Windows)

A infraestrutura centralizada combina duas camadas coordenadas sem dependências desnecessárias:

1. **Monitor XInput Nativo em Rust (`src-tauri/src/infrastructure/system/windows_gamepad.rs`):**
   - Carrega dinamicamente `xinput1_4.dll` / `xinput1_3.dll` / `xinput9_1_0.dll` via Win32 `LoadLibraryW` (acesso **não-exclusivo**, nunca bloqueando emuladores).
   - Monitora os 4 slots XInput com hot-plug automático (conexão, desconexão e reconexão), zona morta radial escalonada (`apply_scaled_radial_deadzone`) e controle anti-repetição (`RepeatGate`).
   - Emite eventos Tauri `gamepad:devices` e `gamepad:action`, além do comando IPC `list_connected_gamepads`.
2. **Camada Unificada no Frontend (`src/input/gamepadEngine.ts` + `src/input/InputProvider.tsx`):**
   - Recebe eventos nativos XInput e também executa polling via **W3C Gamepad API** (fallback para controles DualSense, DualShock 4/5, Switch Pro e DirectInput USB).
   - Detecta a família de dispositivo (`xbox`, `playstation`, `nintendo`, `generic`, `keyboard`) e adapta automaticamente os ícones exibidos na `ControllerActionBar`, no `QuickMenuModal` e nas dicas de tela.
   - Deduplica entradas simultâneas (janela de 55ms) para evitar ativações duplas quando o mesmo dispositivo aparece em ambos os barramentos.

---

## 2. Mapeamento Padrão e Personalização

Todos os 10 botões padrão são remapeáveis em **Configurações → Controles** e persistidos no SQLite (`application_settings['gamepad.config_json']`):

| Controle (Xbox / PlayStation) | Ação Padrão | Equivalente no Teclado |
|---|---|---|
| Analógico esquerdo / D-Pad | Navegação espacial 2D | `↑` `↓` `←` `→` |
| `A` / `✕` | Confirmar / Abrir / Executar | `Enter` / `Espaço` |
| `B` / `○` | Voltar / Fechar modal / Focar menu | `Esc` |
| `X` / `□` | Ação secundária (ex.: Favoritar jogo) | `F` |
| `Y` / `△` | Menu de contexto / Menu Rápido | `M` |
| `LB` / `L1` e `RB` / `R1` | Alternar abas da tela ou seções do Hub | `Alt+←` / `Alt+→` |
| `LT` / `L2` e `RT` / `R2` | Salto rápido em listas grandes (±5 itens) | `PgUp` / `PgDn` |
| `Start` / `Options` | Abrir/fechar Menu Rápido do Hub | `F2` |
| `View` / `Create` | Alternar Modo Desktop ↔ Fullscreen TV | `F11` |

---

## 3. Foco Espacial 2D e Pilha de Modais

O `SpatialFocusManager` (`src/input/focusManager.ts`) gerencia o foco previsível em todas as telas:

- **Projeção 2D por bounding-box:** calcula candidatos alinhados na direção escolhida com penalidade perpendicular para evitar saltos erráticos entre colunas ou barras.
- **Auto-scroll suave:** mantém o card/botão focado visível dentro de listas longas (incluindo bibliotecas com dezenas/centenas de jogos).
- **Memória por seção:** ao trocar de tela, memoriza o último elemento focado e restaura ou foca o ponto de entrada (`[data-focus-default="true"]`) da nova página.
- **Pilha de foco de modais (`pushModalFocus` / `popModalFocus`):** ao abrir `ConfirmDialog` ou `QuickMenuModal`, o foco fica preso dentro do diálogo e retorna **exatamente ao elemento anterior** quando o modal é fechado com `B`/`Esc`.

---

## 4. Handoff Seguro com Emuladores (RPCS3 e Futuros)

Integrado diretamente ao `LiveSessionOrchestrator` (`src-tauri/src/application/live_session_orchestrator.rs`):

1. **Antes de iniciar um emulador:** `WindowsGamepadMonitor::pause()` suspende imediatamente a leitura de entrada do Hub para que o controle opere exclusivamente no console/emulador.
2. **Durante a sessão ativa:** `InputProvider` também recebe `suspended={activeSession.phase !== 'idle'}` e pausa a navegação do Hub.
3. **Ao encerrar a sessão e restaurar foco da janela:** `resume_with_drain(350ms)` descarta botões ainda mantidos pressionados ao sair do emulador, prevenindo cliques acidentais no Hub.

---

## 5. Modos Desktop e Fullscreen (10-Foot TV)

- **Alternância instantânea sem perda de estado:** `F11`, botão no cabeçalho/rodapé, `QuickMenuModal`, Configurações ou botão `View`/`Back` do controle.
- **Escalonamento tipográfico 10-foot (`src/styles/tokens.css`):** `data-display-mode="fullscreen"` amplia fontes, espaçamentos, hit targets, capas e converte a sidebar em HUD superior limpa para leitura confortável em TV.
- **Janela real no Windows:** em runtime Tauri 2, invoca `getCurrentWindow().setFullscreen(...)` e persiste `ui.display_mode` no SQLite.

---

## 6. Transparência sobre Validação

- **Validado automaticamente:** lógica pura XInput (deadzone radial + `RepeatGate` + handoff pause/drain em Rust), polling W3C Gamepad, hot-plug connect/disconnect/reconnect, troca dinâmica de glyphs PlayStation/Xbox, navegação espacial, foco em modais e persistência Desktop↔Fullscreen (Vitest + Playwright).
- **Requer validação manual em hardware físico:** teste ergonômico na máquina do usuário com controles físicos conectados via USB/Bluetooth (Xbox Series/One via XInput, DualSense/DualShock 4 e Nintendo Switch Pro) e verificação prática do handoff em sessão real do RPCS3.
