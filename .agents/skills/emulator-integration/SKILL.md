---
name: emulator-integration
description: >-
  Use when designing or implementing emulator adapters, process lifecycle
  management, CLI argument builders, Windows window/fullscreen management, or
  compatibility detection for RPCS3, PCSX2, Dolphin, xemu, Xenia Canary,
  shadPS4, and future emulators.
---

# Emulator Integration

## Purpose & Scope

Defines the modular adapter architecture and safe Windows process-management patterns for integrating external emulators (RPCS3, PCSX2, Dolphin, xemu, Xenia Canary, shadPS4, and future plugins) into Universal Console Hub's Rust/Tauri backend.

**Out of scope:** Firmware/dashboard truthfulness rules (use `authentic-console-emulation`), controller mapping and handoff (use `windows-gamepad-system`), trophy parsing (use `universal-trophy-system`).

## When to Use

- Creating or updating the `EmulatorAdapter` Rust trait and concrete adapter implementations.
- Implementing safe process spawning, stdout/stderr log capture, crash detection, or graceful termination on Windows.
- Managing emulator executable discovery, version detection, and config/profile overrides.
- Handling Windows fullscreen focus, borderless/exclusive window transitions, and returning focus to Universal Console Hub on exit.

**When NOT to use:** Pure frontend UI styling or SQLite schema migrations.

## Core Architecture: Ports & Adapters

Define a single, strongly typed `EmulatorAdapter` trait in the domain layer; every emulator lives in its own isolated module (`adapters/rpcs3.rs`, `adapters/pcsx2.rs`, etc.):

```rust
pub trait EmulatorAdapter: Send + Sync {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn supported_systems(&self) -> &'static [ConsoleSystem];
    fn capabilities(&self) -> AdapterCapabilities;
    fn detect_installation(&self, configured_path: Option<&Path>) -> Result<EmulatorInstallation, AdapterError>;
    fn build_launch_command(&self, request: &LaunchRequest) -> Result<LaunchCommand, AdapterError>;
    fn parse_exit_status(&self, code: Option<i32>, stderr_tail: &str) -> SessionOutcome;
}
```

## Working Procedure

1. **Isolate Emulator Specifics**: Keep CLI flags, config file formats (INI/TOML/YAML), and save/trophy paths strictly inside each emulator's adapter module. Never leak emulator-specific flags into UI or core session services.
2. **Build Commands Safely (No Shell Injection)**:
   - Always spawn processes using `std::process::Command` or `tokio::process::Command` with discrete `.arg(...)` calls.
   - **Never** construct command strings through `cmd.exe /C` or `powershell -Command`.
   - Validate that the target executable path exists, is a regular file, and matches the user-configured emulator binary.
3. **Manage Windows Process Lifecycle**:
   - Assign spawned emulator processes to a Windows Job Object (with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` only if desired, or tracked PID/process handle) so Universal Console Hub accurately detects exit even if an emulator spawns child processes.
   - Capture bounded stdout/stderr rings (e.g., last 200 lines) for diagnostics without blocking the child's pipe buffers.
   - Support graceful shutdown (`WM_CLOSE` to main window first, timeout, then explicit user-confirmed termination) to avoid corrupting emulator memory cards or NAND/HDD images.
4. **Coordinate Window Focus & Fullscreen**:
   - Minimize or yield focus cleanly before emulator launch so fullscreen Vulkan/Direct3D/OpenGL swapchains acquire display cleanly.
   - Restore Universal Console Hub window focus and re-acquire hub gamepad navigation only after the emulator process exits.
5. **Version & Flag Verification**: Detect emulator version (`--version` or file metadata where supported) and guard CLI flags that vary between stable and canary/nightly builds.

## Verified Emulator CLI Patterns (Verify Against Installed Build)

| Emulator | Typical Systems | Common Launch Characteristics |
|---|---|---|
| **RPCS3** | PS3 | Direct boot via `--no-gui <path/to/EBOOT.BIN>`; VSH/XMB boot via `--no-gui <rpcs3_dir>/dev_flash/vsh/module/vsh.self` once user `PS3UPDAT.PUP` is installed. Differentiate firmware validation, file presence, command generation, and runtime XMB confirmation. |
| **PCSX2** | PS2 | Qt builds support `-batch`, `-fullscreen`, `--cfgpath=<dir>` (legacy `-nogui` dropped in Qt); BIOS boot uses configured BIOS in profile INI. |
| **Dolphin** | GameCube / Wii | `-b` (batch), `-e <game_path>`; Wii System Menu via `-n 0000000100000002 -b` (requires user NAND); GameCube IPL via `-C GameCube.SkipMainMenu=False` (requires user `IPL.bin`). |
| **xemu** | Original Xbox | `-full-screen`, `-dvd_path <xiso>`, `-config_path <xemu.toml>`, `-snapshot`; boots original Xbox dashboard when user MCPX/BIOS/HDD image is configured in `xemu.toml` and `-dvd_path` is omitted. |
| **Xenia Canary** | Xbox 360 | Direct title path launch (`xenia_canary.exe <path.xex|iso>`, `--config`); `dash.xex` dashboard support is partial/niche only (`PARTIAL_DASHBOARD` + `DIRECT_GAME_ONLY`). |
| **shadPS4** | PS4 | `EXPERIMENTAL` + `DIRECT_GAME_ONLY`; CLI (`shadPS4 <CUSA_ID|eboot.bin>`, `--fullscreen true`) evolves rapidly across SDL and QtLauncher builds; full PS4 OS dashboard is `UNSUPPORTED`. |

*Always verify exact CLI flags against the user's installed binary before hardcoding assumptions.*

## Technical Best Practices

- **Non-Destructive Config Overrides**: Prefer CLI arguments or per-game config overrides over mutating the user's global emulator `.ini`/`.toml` files. If writing config files is unavoidable, back up the original first.
- **Single Active Session Guard**: Enforce a session state machine (`Idle -> Launching -> Running -> Exiting -> Idle`) so two emulators do not fight over GPU fullscreen and gamepads simultaneously.
- **Structured Telemetry Events**: Emit Tauri events (`session:started`, `session:exited`, `session:error`) with typed payloads so the playtime tracker and trophy sync react cleanly without tight coupling.

## Common Mistakes to Avoid

- **Spawning via `cmd /C`**: Introduces quoting bugs with spaces/Unicode in Windows paths (e.g., `OneDrive\Área de Trabalho`) and command-injection vulnerabilities.
- **Force-Killing Immediately (`TerminateProcess`)**: Can corrupt PCSX2 memory cards, Dolphin Wii NAND, or xemu qcow2 HDD images. Always request graceful close first.
- **Unread Piped Stdout/Stderr**: Piping stdout/stderr without draining them asynchronously will deadlock the emulator once the OS pipe buffer fills.
- **Hardcoding `C:\Program Files`**: Users frequently install portable emulators on secondary drives (`D:\Emulators\...`). Always support user-configured paths and portable directories.

## Quality Checklist

- [ ] Adapter implements the shared `EmulatorAdapter` trait with zero UI coupling.
- [ ] Process execution uses direct `Command::new(exe).arg(...)` with zero shell interpolation.
- [ ] Paths containing spaces and non-ASCII characters (Unicode) are tested and handled safely.
- [ ] Stdout/stderr pipes are drained asynchronously with bounded memory.
- [ ] Graceful close is attempted before forced termination.
- [ ] Session lifecycle emits typed events for playtime and trophy synchronization.

## References

- Rust `std::process::Command` & Windows `CommandExt`: https://doc.rust-lang.org/std/os/windows/process/trait.CommandExt.html
- Windows Job Objects API (`windows` crate): https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects
- RPCS3 CLI & Wiki: https://wiki.rpcs3.net/
- PCSX2 Command Line Arguments: https://pcsx2.net/docs/usage/cli/
- Dolphin Command Line Usage: https://wiki.dolphin-emu.org/index.php?title=Help:Contents
