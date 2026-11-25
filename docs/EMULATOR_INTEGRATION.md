# Emulator Integration — Universal Console Hub 3.0

## Contrato `EmulatorAdapter`
Cada emulador implementa o trait `EmulatorAdapter` em seu próprio módulo isolado em `src-tauri/src/adapters/`:

- `detect_installation(&self, configured_exe: Option<&Path>)`
- `verify_version(&self, exe_path: &Path)`
- `validate_firmware(&self, installation: &EmulatorInstallation, firmware_package: Option<&Path>)`
- `validate_system_files(&self, installation: &EmulatorInstallation)`
- `build_system_boot_command(&self, installation: &EmulatorInstallation)`
- `build_game_boot_command(&self, installation: &EmulatorInstallation, game_path: &Path)`
- `capabilities(&self)` → `AdapterCapabilities` (múltiplas capacidades independentes + disponibilidade por funcionalidade)
- `parse_exit_status(&self, exit_code: Option<i32>, stderr_tail: &str)`

## Preparação RPCS3 (Fases 1–2) vs. Execução Completa (Fase 3 — Opção A)

Nas **Fases 1 e 2**, o adaptador `Rpcs3Adapter` implementa exclusivamente **preparação, diagnóstico e descoberta local** (sem lançamento ao vivo de processos do emulador):
1. **Validação de Pacote de Firmware (`firmware_package_validated`)**: valida caminho local, tamanho não-nulo, assinatura/cabeçalho `SCEUF` quando aplicável e calcula SHA-256 via streaming do `PS3UPDAT.PUP` fornecido pelo usuário.
2. **Presença de Arquivos de Sistema (`system_files_present`)**: verifica no diretório do RPCS3 a presença de `dev_flash/vsh/module/vsh.self` e `dev_hdd0/`.
3. **Geração de Comando (`command_buildable`)**: constrói `LaunchCommand` seguro com array de argumentos:
   - System Boot (XMB): `["--no-gui", "<rpcs3_dir>\\dev_flash\\vsh\\module\\vsh.self"]`
   - Direct Game Boot: `["--no-gui", "<path_to_eboot>"]`
4. **Descoberta Local (Fase 2)**: `Ps3GameDiscovery` lê cabeçalhos binários `PARAM.SFO` com checagem estrita de bounds (zero `panic!`) e popula `games` + `game_versions` de forma idempotente.
5. **Confirmação em Execução (`xmb_runtime_confirmed` & 7 Estados Independentes)**: explicitamente **`Unconfirmed` durante as Fases 1 e 2**. A execução ao vivo do XMB e de jogos PS3, monitoramento de processo/sessão, controle de janela/foco e retorno ao Hub pertencem exclusivamente à **Fase 3** (com verificação independente das 7 chaves em `console_runtime_verifications`: `emulator_launch`, `system_boot`, `original_dashboard`, `game_boot`, `per_game_compatibility`, `session_monitoring` e `hub_return`).

## Segurança de Processos no Windows
- Todos os comandos usam `std::process::Command::new(exe).args(&cmd.args)`.
- Zero `cmd.exe /C` ou PowerShell interpolado.
- Caminhos Windows com espaços e Unicode são suportados nativamente como `OsString`/`PathBuf`, sem vazamento de prefixo verbatim `\\?\`.
- Drenagem de stdout/stderr usa leitura byte-a-byte lossy resistente a páginas de código OEM/CP850 do Windows.
