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

## Preparação RPCS3 (Escopo da Fase 1 vs. Fase 4)

Na **Fase 1**, o adaptador `Rpcs3Adapter` implementa exclusivamente **preparação e diagnóstico**:
1. **Validação de Pacote de Firmware (`firmware_package_validated`)**: valida caminho local, tamanho não-nulo, assinatura/cabeçalho `SCEUF` quando aplicável e calcula SHA-256 via streaming do `PS3UPDAT.PUP` fornecido pelo usuário.
2. **Presença de Arquivos de Sistema (`system_files_present`)**: verifica no diretório do RPCS3 a presença de `dev_flash/vsh/module/vsh.self` e `dev_hdd0/`.
3. **Geração de Comando (`command_buildable`)**: constrói `LaunchCommand` seguro com array de argumentos:
   - System Boot (XMB): `["--no-gui", "<rpcs3_dir>\\dev_flash\\vsh\\module\\vsh.self"]`
   - Direct Game Boot: `["--no-gui", "<path_to_eboot>"]`
4. **Confirmação em Execução (`xmb_runtime_confirmed`)**: explicitamente **`Unconfirmed` durante toda a Fase 1**. A execução ao vivo, monitoramento de processo, captura de sessão, controle de janela e retorno ao Hub pertencem exclusivamente à **Fase 4**.

## Segurança de Processos no Windows
- Todos os comandos usam `std::process::Command::new(exe).args(&cmd.args)`.
- Zero `cmd.exe /C` ou PowerShell interpolado.
- Caminhos Windows com espaços e Unicode são suportados nativamente como `OsString`/`PathBuf`.
