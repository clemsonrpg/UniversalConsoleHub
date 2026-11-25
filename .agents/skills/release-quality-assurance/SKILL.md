---
name: release-quality-assurance
description: >-
  Use when preparing a Universal Console Hub build, feature milestone, or
  Windows release for validation, security review, performance verification,
  regression testing, or release-readiness sign-off.
---

# Release Quality Assurance

## Purpose & Scope

Defines Universal Console Hub's end-to-end quality, security, performance, and release-readiness verification workflow across the React/TypeScript frontend, Rust/Tauri 2 backend, SQLite database, and Windows installer bundle.

**Out of scope:** Red-green TDD cycle (use `superpowers:test-driven-development`), root-cause bug investigation (use `superpowers:systematic-debugging`), Playwright test authoring specifics (use `playwright-expert`), Vitest API specifics (use `vitest`).

## When to Use

- Running pre-merge or pre-release quality gates across frontend and Rust backend.
- Conducting Windows desktop security reviews (Tauri capabilities, process execution, path traversal, SQL injection, dependency advisories).
- Validating performance budgets (startup time, library scan throughput, memory footprint, idle CPU/GPU usage).
- Executing regression suites and verifying clean-machine Windows installation/upgrade paths.

**When NOT to use:** Early exploratory prototyping before code is ready for verification.

## Working Procedure

1. **Static Analysis & Type Safety Gate**:
   - Frontend: run TypeScript typecheck (`tsc --noEmit`), linter, and production Vite build.
   - Backend (Rust): run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`.
   - Zero skipped warnings without documented `#[expect(..., reason = "...")]`.
2. **Automated Test Pyramid Gate**:
   - **Rust unit & integration tests**: emulator adapter command builders (mocked paths, spaces/Unicode, invalid configs), SQLite migrations (fresh DB + upgrade from prior schema), trophy parsers (corrupt/truncated file resilience), playtime heartbeat recovery.
   - **Frontend unit tests (Vitest)**: state stores, capability-tier rendering, formatting utilities, IPC error handling.
   - **End-to-End tests (Playwright)**: critical user journeys (onboarding without proprietary files, library filtering, profile switching, settings persistence, accessibility smoke checks).
3. **Desktop Security Review Gate**:
   - **Tauri Capabilities**: audit `src-tauri/capabilities/*.json` for least privilege (no wildcard shell/fs permissions).
   - **Process Execution**: confirm zero `cmd /C` or `powershell` shell interpolation; all emulator launches use argument arrays.
   - **Path & Input Validation**: verify user-supplied ROM/BIOS/avatar paths are canonicalized and cannot escape expected cache/data boundaries; verify all SQL queries are parameterized.
   - **Supply Chain**: run `cargo audit` (if installed) and `npm audit --omit=dev`; review any newly added crates/packages.
   - **Legal Compliance**: scan repository and build artifacts to confirm zero BIOS, firmware, NAND, keys, or copyrighted game assets are bundled.
4. **Performance & Resource Budget Gate**:
   - Verify idle hub CPU usage stays near 0% (no tight polling loops in gamepad or file watchers).
   - Verify active-emulator overhead: hub releases GPU/animation work and pauses gamepad polling while a game is running.
   - Benchmark library scan and SQLite query latency on a representative dataset (e.g., 500+ games, 5,000+ trophies) — key UI queries should complete in < 50ms.
5. **Windows Packaging & Clean-Install Verification**:
   - Verify Tauri Windows bundle (NSIS/MSI) installs, launches, creates SQLite DB in `%LOCALAPPDATA%` / `%APPDATA%` cleanly, and upgrades an existing DB without data loss.
   - Verify paths containing spaces and non-ASCII characters (e.g., `Área de Trabalho`, accented usernames) work end-to-end.

## Technical Best Practices

- **Evidence Before Claims**: Never report a release or milestone as "passing" without running the actual verification commands and inspecting their exit codes and outputs (`superpowers:verification-before-completion`).
- **Deterministic Fixtures**: Test emulator adapters and trophy parsers with synthetic, cleanly generated fixture data—never copyrighted dumps.
- **Migration Upgrade Tests**: Always test migrating an actual snapshot of the previous release's SQLite schema forward to the latest migration.

## Common Mistakes to Avoid

- **Testing Only in Dev Mode (`tauri dev`)**: Production builds enable stricter CSP, asset bundling, and `windows_subsystem = "windows"` behavior that can hide bugs until release.
- **Over-Broad Tauri Capabilities**: Leaving `shell:allow-open` or broad `fs:allow-read` across the entire filesystem in production builds.
- **Ignoring Idle Resource Drain**: A continuous `requestAnimationFrame` or 1ms gamepad sleep loop drains laptop batteries and steals CPU from CPU-bound emulators like RPCS3.
- **Skipping Unicode/Space Path Tests**: Windows users frequently have spaces or accents in user profile paths; untested string handling breaks launches silently.

## Quality Checklist

- [ ] `tsc --noEmit`, frontend lint, and production Vite build pass with zero errors.
- [ ] `cargo fmt --check`, `cargo clippy --all-targets`, and `cargo test` pass cleanly.
- [ ] Vitest unit suite and Playwright E2E suite pass with zero uninvestigated flakes.
- [ ] Tauri capabilities follow least privilege; zero shell-interpolated process spawns exist.
- [ ] SQLite migrations tested on both fresh DB and upgrade-from-previous-version DB.
- [ ] Zero proprietary firmware, BIOS, NAND, keys, or games exist in repo or installer bundle.
- [ ] Idle CPU usage and active-emulator handoff verified; Unicode Windows paths tested.

## References

- Verification discipline: `superpowers:verification-before-completion`
- Tauri v2 Security & Capabilities: see the `tauri-v2` skill (capabilities reference)
- Playwright E2E testing: see `playwright-expert` skill
- Vitest unit testing: see `vitest` skill
