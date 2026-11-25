# Known Limitations — Universal Console Hub 3.0

Este documento registra com total transparência todas as limitações técnicas conhecidas no estágio atual (**Fase 1**):

1. **Execução Ao Vivo de Emuladores (Reservada às Fases 4 e 5):** Na Fase 1, o adaptador RPCS3 realiza apenas detecção de instalação, validação local de arquivos/firmware (`vsh.self` e `PS3UPDAT.PUP`), geração segura de comandos e diagnóstico em 4 estágios. O lançamento real de processos RPCS3, confirmação runtime do XMB, monitoramento de sessão e retorno ao Hub serão implementados na **Fase 4**.
2. **Consoles Adicionais (PCSX2, Dolphin, xemu, Xenia Canary, shadPS4):** Expostos no catálogo com capacidades documentadas upstream e claramente marcados como **Planejado — Fase 5** (sem botões falsos de execução).
3. **Troféus & Providers Externos:** Apenas `LocalAchievementProvider` está operacional na camada base da Fase 1. Leitura de `TROPUSR.DAT` do RPCS3 entra na **Fase 3**; RetroAchievements e integrações autorizadas entram na **Fase 6**. Não há sincronização oficial PSN/Xbox Live (sem API autorizada).
4. **Controles Windows Avançados (SDL3/XInput/DualSense HID):** A Fase 1 inclui navegação por teclado, mouse e suporte básico à Gamepad API no Hub; a camada nativa Rust com SDL3/XInput, calibração e handoff completo entra nas **Fases 4 e 6**.
5. **Playwright E2E vs. WebView2 Tauri:** Os testes Playwright da Fase 1 validam a interface e seus fluxos usando um transporte de teste injetado no Vite/Chromium, separados dos testes reais de integração Rust/SQLite/Windows.
6. **PowerShell no Windows (`npm.ps1`):** Em ambientes Windows com política de execução restrita para scripts `.ps1`, deve-se invocar `npm.cmd` em vez de `npm` no PowerShell.
