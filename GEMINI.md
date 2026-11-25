# Universal Console Hub — Project Engineering Instructions

Universal Console Hub is a local-first Windows desktop application (Tauri 2 + Rust + SQLite + React + TypeScript + Vite) that unifies authentic console emulation, game libraries, player profiles, playtime statistics, Windows gamepad navigation, and trophy/achievement tracking.

## 1. Automatic Skill Activation & Composition

Before designing, implementing, reviewing, or debugging any subsystem, read and follow the relevant skill(s) in `.agents/skills/` and the installed global `superpowers` plugin. Combine multiple skills whenever a task spans domains (for example, `emulator-integration` + `tauri-v2` + `rust-best-practices` + `authentic-console-emulation` when building a new emulator adapter).

| Domain / Task | Required Skill(s) |
|---|---|
| Console boot semantics, firmware/BIOS onboarding, capability tiers | `authentic-console-emulation` |
| Emulator adapters (RPCS3, PCSX2, Dolphin, xemu, Xenia Canary, shadPS4), Windows process & window lifecycle | `emulator-integration` |
| Unified trophies/achievements, local readers, RetroAchievements sync | `universal-trophy-system` |
| Player profiles, game library scanning, playtime sessions, statistics | `universal-player-profile` |
| Windows controllers (XInput, SDL, DualShock, DualSense, USB), deadzones, emulator input handoff | `windows-gamepad-system` |
| Hub UI honesty, console pages, preserving authentic emulated interfaces | `console-ui-fidelity` |
| UI aesthetics, visual direction, typography, motion, copy | `frontend-design` |
| Accessibility, UX review, Web Interface Guidelines audit | `web-design-guidelines` |
| React component architecture and rendering performance | `vercel-react-best-practices` |
| TypeScript type modeling, discriminated unions, strict typing | `typescript-pro` |
| Tauri 2 config, `#[tauri::command]`, IPC, capabilities, Windows bundling | `tauri-v2` |
| Rust ownership, error handling, Tokio async, Clippy, performance | `rust-best-practices` |
| SQLite schemas, WAL mode, migrations, backups, query tuning | `sqlite-engineering` |
| Modular boundaries, Dependency Rule, Ports & Adapters, SOLID | `clean-architecture` |
| Unit & integration testing (Frontend) | `vitest`, `superpowers:test-driven-development` |
| End-to-end testing & UI automation | `playwright-expert` |
| Debugging bugs, crashes, or flaky tests | `superpowers:systematic-debugging` |
| Pre-release QA, security audit, performance & regression sign-off | `release-quality-assurance`, `superpowers:verification-before-completion` |

## 2. Core Engineering Standards

- **Inspect Before Changing**: Always read existing files, traits, schemas, and tests before modifying code. Never assume file contents or symbol signatures.
- **Modular Architecture & Separation of Concerns**: Keep domain entities and use cases independent of Tauri IPC, SQLite drivers, React components, and individual emulator CLI specifics (`clean-architecture`). Each emulator lives in its own isolated adapter module.
- **Avoid Unnecessary Rewrites**: Prefer targeted, incremental changes that respect existing conventions and contracts.
- **Test Instead of Assuming**: Write and execute automated tests (Rust `cargo test`, Vitest, Playwright) and static checks (`tsc --noEmit`, `cargo clippy`) before claiming any task is complete. Never claim success without observed verification output.
- **Document Architectural Decisions**: Record significant architecture, schema, adapter, and security decisions as short Architecture Decision Records (ADRs) in `docs/adr/`.

## 3. Non-Negotiable Authenticity & Legal Boundaries

- **Never Fabricate Capabilities**: Never invent, mock, or exaggerate emulator features, original dashboard support, or achievement/trophy integrations. Verify every capability against real upstream emulator behavior.
- **Preserve Original Console Interfaces**: When an emulator genuinely supports booting a console's original system software/dashboard, launch the real emulator with the user's system files. **Never** create a lookalike web/React UI and present or imply that it is the original console OS.
- **Multi-Capability Feature Classification**: Console and emulator capabilities are modeled as **independent, non-mutually-exclusive per-feature capabilities** (never a single mutually exclusive tier). Every capability and feature must expose its availability and verification state using the six canonical capability states:
  1. `FULL_SYSTEM_BOOT` — Emulator can boot the console's full system software from user-supplied files.
  2. `ORIGINAL_DASHBOARD` — Emulator exposes the authentic original console dashboard/menu (XMB, PS2 Browser, Wii System Menu, Xbox Dashboard).
  3. `PARTIAL_DASHBOARD` — Original dashboard boots only partially or with documented functional caveats.
  4. `DIRECT_GAME_ONLY` — Direct game title launching is supported (may coexist with `FULL_SYSTEM_BOOT` / `ORIGINAL_DASHBOARD` or stand alone).
  5. `EXPERIMENTAL` — Active upstream development with title- and build-dependent stability.
  6. `UNSUPPORTED` — Feature is unavailable and clearly disabled with an explanation.
- **Runtime Confirmation Honesty**: Never declare authentic dashboard boot (e.g., RPCS3 XMB) as runtime-confirmed solely because files exist or a command can be constructed; differentiate firmware package validation, system file presence, command generation, and confirmed runtime execution.
- **Explicit Test Layer Separation**: Always separate and report (1) Unit tests, (2) Real backend integration tests (real SQLite + real OS processes + real filesystem adapters), and (3) UI E2E tests (Playwright — never present injected-transport UI tests as validation of real IPC or real SQLite).
- **Zero Proprietary / Copyrighted Assets**: Never commit, bundle, generate, download, or distribute proprietary console firmware, BIOS, NAND, EEPROM, decryption keys, system fonts/audio, or copyrighted game ROMs/ISOs anywhere in the repository, test fixtures, or release installers.
- **User-Provided Legal System Files Only**: Any console feature requiring BIOS, firmware, NAND, or keys must prompt the user to supply their own legally obtained files locally, validating structure/checksums without uploading or redistributing them.

## 4. Windows Desktop Safety & Quality Floor

- **Safe Process Execution**: Spawn emulators via direct argument arrays (`Command::new(exe).arg(...)`). Never invoke `cmd.exe /C` or PowerShell string interpolation. Handle spaces and non-ASCII Unicode Windows paths safely.
- **Least-Privilege Capabilities**: Keep `src-tauri/capabilities/*.json` strictly scoped to required permissions.
- **Graceful Handoff**: Pause hub gamepad navigation and release exclusive handles before launching an emulator; drain input briefly and restore focus cleanly on exit.
- **Data Integrity**: Enable SQLite `WAL` mode, `foreign_keys = ON`, `busy_timeout`, and `STRICT` tables; use `VACUUM INTO` or SQLite's Online Backup API for backups.
