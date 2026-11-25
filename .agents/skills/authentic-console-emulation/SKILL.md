---
name: authentic-console-emulation
description: >-
  Use when designing, documenting, or implementing console boot flows, original
  firmware/BIOS/NAND onboarding, authentic dashboard launch support, or
  capability matrices for emulated consoles in Universal Console Hub.
---

# Authentic Console Emulation

## Purpose & Scope

Ensures Universal Console Hub treats console emulation with technical honesty and legal compliance. Covers firmware/BIOS/NAND user-provisioning workflows, authentic system boot vs. direct game launch classification, and honest capability reporting per emulator.

**Out of scope:** Process spawning mechanics and adapter traits (use `emulator-integration`), hub UI visual rules (use `console-ui-fidelity`).

## When to Use

- Defining or updating the console capability matrix (which consoles support booting their original OS/dashboard vs. direct game launch only).
- Designing user onboarding flows for legally obtained BIOS, firmware, NAND, EEPROM, or decryption keys.
- Implementing system-file validation (checksum/structure verification without bundling proprietary files).
- Handling emulator-specific boot limitations (e.g., PS4/shadPS4 or Xbox 360/Xenia Canary dashboard limitations).

**When NOT to use:** General Tauri/Rust IPC work unrelated to console boot semantics or system files.

## Multi-Capability Classification Model

Capabilities are **independent and non-mutually-exclusive** per console and per feature—a single emulator (such as RPCS3) can simultaneously declare `FULL_SYSTEM_BOOT`, `ORIGINAL_DASHBOARD`, and `DIRECT_GAME_ONLY` while marking online services as `UNSUPPORTED`. Every capability also tracks its verification level (`unverified`, `documented_upstream`, `files_verified`, `runtime_confirmed`):

| Capability State | Meaning | Typical Examples (Verify Against Installed Build) |
|---|---|---|
| `FULL_SYSTEM_BOOT` | Emulator can boot the full console system software from user-supplied files. | RPCS3 (`dev_flash/vsh/module/vsh.self`), PCSX2 (PS2 BIOS), Dolphin (Wii NAND / GC IPL), xemu (MCPX + BIOS + HDD). |
| `ORIGINAL_DASHBOARD` | Authentic original console dashboard/menu is bootable and navigable via real emulation. | PS3 XMB (RPCS3), PS2 Browser (PCSX2), Wii System Menu (Dolphin), Original Xbox Dashboard (xemu). |
| `PARTIAL_DASHBOARD` | Original dashboard boots with documented caveats or advanced manual prerequisites. | Xenia Canary (`dash.xex` partial/niche support) or region-restricted firmware menus. |
| `DIRECT_GAME_ONLY` | Direct game title boot is supported (either alongside `FULL_SYSTEM_BOOT` or as the sole stable boot mode). | All 6 priority emulators support direct game launch; primary mode for Xenia Canary and shadPS4. |
| `EXPERIMENTAL` | Active upstream development; boot and game stability vary heavily by title and build. | shadPS4 (PS4); must display an explicit Experimental badge in UI. |
| `UNSUPPORTED` | Feature cannot be performed by the emulator; must be disabled and explained plainly. | Official PSN / Xbox Live online services, PS4 full Orbis OS dashboard in shadPS4. |

### Four-Stage Readiness Differentiation (Mandatory for System Boot)

Never collapse system-boot readiness into a single boolean. Track four independent stages:
1. **Firmware Package Validated**: User-supplied installer/package (e.g., `PS3UPDAT.PUP`) validated locally via header/SHA-256.
2. **System Files Present**: Extracted/installed system files verified on disk (e.g., RPCS3 `dev_flash/vsh/module/vsh.self` and `dev_hdd0`).
3. **Command Buildable**: Safe argument-array `LaunchCommand` generated without shell interpolation.
4. **Runtime Boot Confirmed**: Authentic dashboard execution actually observed and confirmed at runtime on the user's machine (never assumed from file presence alone).

## Working Procedure

1. **Verify Upstream Reality First**: Before declaring any console feature supported, verify against the target emulator's official documentation and CLI/config behavior. Never extrapolate from another emulator.
2. **Enforce Zero Proprietary Assets**: Confirm the repository, installers, tests, and fixtures contain **zero** BIOS dumps, firmware PUPs, NAND dumps, encryption keys, or copyrighted ROMs/ISOs.
3. **Design User-Provisioned System File Flows**:
   - Prompt the user to locate their legally dumped/obtained system files on disk.
   - Validate file presence, expected structure/header, and optional known-good SHA-256 checksums locally (never upload user dumps).
   - Store only file paths or copy into the emulator's designated user data directory with explicit user consent.
4. **Expose Honest Capability Metadata**: Model each console adapter's capabilities as typed Rust/TypeScript enums (`BootMode::OriginalDashboard`, `BootMode::DirectGameLaunch`, `SupportTier::Experimental`, etc.) so the UI reflects reality automatically.
5. **Graceful Degradation**: When required firmware/BIOS is missing, disable "Boot System" with a clear, actionable setup checklist rather than crashing or faking a dashboard.

## Technical Best Practices

- **Checksum Validation Without Distribution**: Store only cryptographic hashes (SHA-256/MD5) or magic-byte signatures to verify user-provided BIOS/firmware files—never store binary blobs.
- **Region & Revision Awareness**: Track console region (NTSC-U, NTSC-J, PAL) and firmware revision where emulators require matching BIOS/IPL files.
- **Immutable Legal Boundary**: Document in user-facing setup screens that Universal Console Hub is a launcher/integration hub and does not provide games or system software.

## Common Mistakes to Avoid

- **Fabricating Dashboard Support**: Claiming Xenia Canary or shadPS4 boots a full authentic console dashboard when upstream only supports direct game execution.
- **Bundling "Test" BIOS or Keys**: Including even tiny proprietary boot ROMs or decryption keys in `tests/fixtures/` violates copyright and project rules.
- **Hiding Upstream Limitations**: Silently falling back from "Boot Original Dashboard" to a custom hub menu without telling the user.

## Quality Checklist

- [ ] Every console/emulator entry declares an explicit `SupportTier` and supported `BootMode` list.
- [ ] Zero proprietary firmware, BIOS, NAND, keys, or game images exist anywhere in the repo.
- [ ] Missing system files produce clear, actionable onboarding guidance.
- [ ] Experimental emulators (e.g., shadPS4, Xenia Canary edge features) are clearly labeled in metadata and UI.
- [ ] All capability claims are backed by verified upstream emulator behavior.

## References

- RPCS3 Quickstart & Firmware Docs: https://rpcs3.net/quickstart
- PCSX2 Documentation (BIOS & Booting): https://pcsx2.net/docs/
- Dolphin Wiki (Wii NAND & GameCube IPL): https://wiki.dolphin-emu.org/
- xemu Documentation (Required System Files): https://xemu.app/docs/required-files/
- Xenia Canary Documentation: https://github.com/xenia-canary/xenia-canary/wiki
- shadPS4 Repository & Compatibility: https://github.com/shadps4-emu/shadPS4
