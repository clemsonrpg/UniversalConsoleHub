---
name: console-ui-fidelity
description: >-
  Use when designing, reviewing, or modifying Universal Console Hub's user
  interface, console pages, launch flows, or system branding to ensure authentic
  emulated console interfaces are preserved and never misrepresented as original
  console operating systems.
---

# Console UI Fidelity

## Purpose & Scope

Protects the core product promise of Universal Console Hub: **authentic console interfaces come from real emulation, never from lookalike web recreations pretending to be the original system.** Guides how the hub's own premium UI coexists honestly with genuine emulated console dashboards and direct-launch games.

**Out of scope:** General visual design craft (use `frontend-design`), WCAG/Web Interface compliance (use `web-design-guidelines`), firmware verification mechanics (use `authentic-console-emulation`).

## When to Use

- Designing console detail pages, system selector cards, or "Boot System / Launch Game" UI flows.
- Reviewing UI copy, badges, or screen transitions for emulators with different dashboard capabilities.
- Deciding whether a UI feature belongs in Universal Console Hub's native shell vs. inside the emulated console's own system software.

**When NOT to use:** Backend Rust/SQLite code or Playwright selector debugging.

## Core Principle: Zero Imitation, Full Honesty

Universal Console Hub has **one** cohesive, high-craft hub interface (for library management, player profile, trophies, controller setup, and launching). When a user wants a console's original dashboard (PS3 XMB, PS2 Browser, Wii System Menu, Original Xbox Dashboard), the hub **launches the real emulator booting the user's real system software**.

**NEVER** build a fake HTML/CSS recreation of XMB, Xbox Blades, PS4 Home, or Wii Channels and present or imply that it is the original console interface.

## Working Procedure

1. **Separate Hub Shell From Console Runtime Clearly**:
   - Design Universal Console Hub's own UI with a refined, original desktop-application identity (consistent typography, keyboard + gamepad navigation, subtle console-accent theming if desired, without impersonating proprietary OS shells).
   - Present a crisp transition state ("Launching RPCS3 — Booting PS3 System Software...") when handing off to the real emulator.
2. **Display Explicit Support Badges on Every Console View**:
   - Every console page MUST render a clear, unambiguous status indicator driven by the adapter's real capability metadata:
     - **Original Dashboard Supported** (requires user firmware/BIOS — show Ready / Setup Required)
     - **Partial Dashboard Support** (include concise tooltip/note explaining exact caveats)
     - **Direct Game Launch Only** (no fake "Boot Dashboard" button; explain clearly that this emulator launches titles directly)
     - **Experimental Emulator** (distinct warning badge + link to compatibility info)
     - **Unsupported** (disabled action with plain explanation)
3. **Keep Console Actions Truthful**:
   - Only enable "Boot Original System" when the emulator genuinely supports it AND required user system files are verified present.
   - If the user has not yet configured BIOS/firmware, replace the primary action with "Configure System Files" (not a dead button or fake screen).
4. **Respect Trademark & Asset Boundaries**:
   - Do not ship copyrighted console OS sound effects, proprietary system fonts, or ripped firmware artwork in the repository.
   - Use clean geometric/original iconography or user-provided/openly licensed artwork caches.
5. **Optimize for Both Desktop (Mouse/Keyboard) and 10-Foot (Gamepad) Modes**:
   - Ensure every interactive element has visible focus rings, logical spatial navigation order, and consistent button-glyph hints (coordinated with `windows-gamepad-system`).

## Technical Best Practices

- **Data-Driven Capability UI**: Never hardcode "Boot Dashboard" buttons per console in React components—render actions dynamically from the Rust backend's `AdapterCapabilities` struct so UI claims cannot drift from backend reality.
- **Honest Empty & Error States**: When an emulator fails to boot a dashboard (e.g., wrong BIOS region or uninitialized NAND), display the real diagnostic reason and actionable next step.
- **Reduced Motion & High Contrast**: Respect `prefers-reduced-motion` and keep status badges legible via both icon + text label (never color alone).

## Common Mistakes to Avoid

- **Building a Fake XMB / Fake Xbox 360 Blades in React**: Directly violates Universal Console Hub's authenticity rule, misleads users, and creates massive maintenance debt.
- **Showing a "Boot Dashboard" Button for Direct-Launch-Only Emulators**: Misrepresents Xenia Canary or shadPS4 capabilities.
- **Hiding Experimental Status**: Presenting an experimental emulator (like shadPS4) with the same stability promise as Dolphin or PCSX2 sets false expectations.
- **Relying on Hover-Only Controls**: Breaks gamepad and keyboard-only navigation in desktop console hub workflows.

## Quality Checklist

- [ ] No hub screen imitates or claims to be an original console OS/dashboard.
- [ ] Every console view displays its exact, backend-verified support tier (`Original Dashboard`, `Partial`, `Direct Launch Only`, `Experimental`, `Unsupported`).
- [ ] "Boot Original System" is enabled only when genuinely supported and user system files are validated.
- [ ] Zero copyrighted firmware assets, BIOS audio, or proprietary console OS dumps are bundled.
- [ ] All hub flows work seamlessly with both mouse/keyboard and gamepad navigation.

## References

- Authentic Console Emulation capability tiers: see `authentic-console-emulation` skill.
- Visual craft & intentional UI design: see `frontend-design` skill.
- Accessibility & interface audit rules: see `web-design-guidelines` skill.
