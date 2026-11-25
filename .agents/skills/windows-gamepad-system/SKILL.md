---
name: windows-gamepad-system
description: >-
  Use when designing or implementing Windows controller detection, XInput, SDL
  GameController, DualShock/DualSense/generic USB gamepad input, deadzones,
  calibration, button glyph mapping, or emulator input handoff in Universal
  Console Hub.
---

# Windows Gamepad System

## Purpose & Scope

Guides reliable Windows gamepad support for Universal Console Hub's desktop UI and seamless input handoff to external emulators. Covers XInput, SDL2/SDL3 GameController / HID (`gilrs` / `sdl2`), DualShock 4, DualSense, Xbox controllers, generic DirectInput/USB pads, calibration, and focus-aware input suspension.

**Out of scope:** Emulator process launching (use `emulator-integration`), visual design of controller overlays (use `frontend-design` / `console-ui-fidelity`).

## When to Use

- Implementing controller discovery, hot-plug connect/disconnect events, and multi-controller slot assignment on Windows.
- Building 10-foot / couch-friendly gamepad navigation inside the Tauri/React hub UI.
- Handling DualShock 4 / DualSense / Xbox / generic USB button-glyph detection and remapping.
- Implementing input handoff (suspending hub gamepad polling when an emulator is running and resuming cleanly when it exits).
- Designing deadzone, trigger threshold, and stick calibration UI/backend logic.

**When NOT to use:** Keyboard/mouse accessibility audits (use `web-design-guidelines`).

## Windows Input Stack Strategy

| Layer | Best Tooling | Strengths & Caveats |
|---|---|---|
| **Hub UI Navigation (Rust Backend)** | `gilrs` (or `sdl2` GameControllerDB) emitting normalized Tauri events | Native hot-plug, XInput + DirectInput/HID support, works even when WebView2 focus quirks occur. |
| **WebView2 Browser Gamepad API** | `navigator.getGamepads()` (fallback/supplement only) | Easy in React, but stops reporting when window loses focus and lacks reliable DualSense/vendor VID:PID glyph mapping without heuristics. |
| **Emulator Input** | **Delegate to the emulator itself** (SDL/XInput/DualSense native support in RPCS3, PCSX2, Dolphin, xemu, Xenia, shadPS4) | Never inject synthetic keyboard/XInput events unless explicitly needed; modern emulators handle DualSense gyro/adaptive triggers and native XInput directly. |

## Working Procedure

1. **Normalize Controller Events in Rust**:
   - Represent controllers with a canonical `GamepadDevice` (`id`, `name`, `vendor_id`, `product_id`, `backend`, `glyph_family: Xbox | PlayStation | Nintendo | Generic`, `connected`, `battery_percent`).
   - Map raw axes and buttons to a standard virtual layout (South/East/West/North, D-Pad, L1/R1, L2/R2, L3/R3, Start/Select/Guide, Left/Right Stick).
2. **Apply Radial Deadzones & Anti-Repeat Debouncing**:
   - Use **scaled radial deadzones** (default ~15–20% inner deadzone, ~95% outer saturation) for analog sticks to prevent stick drift from scrolling hub menus.
   - Implement clean initial-delay + repeat-rate timers (e.g., 400ms initial delay, 120ms repeat) for D-pad and stick menu navigation.
3. **Implement Strict Emulator Input Handoff**:
   - Subscribe to `session:launching` / `session:started` events from `emulator-integration`.
   - **Immediately pause** hub gamepad navigation and release any exclusive HID handles *before* the emulator process starts.
   - Do **not** process hub navigation inputs while `session_state == Running` (prevents "ghost navigation" where pressing A/Cross in-game clicks buttons behind the emulator).
   - Optionally support a configurable **safe chord** (only if explicitly enabled by the user and non-conflicting) or wait until `session:exited` to re-enable hub navigation with a 300ms input-drain cooldown so the final button press in the game does not immediately re-launch it.
4. **Match Button Glyphs to Active Controller**:
   - Detect controller family via USB VID/PID (Sony `0x054C`, Microsoft `0x045E`, Nintendo `0x057E`) with a manual user override in settings.
   - Update on-screen prompts dynamically when the user presses a button on a different controller or switches to keyboard/mouse.
5. **Handle Calibration & Remapping**:
   - Store per-device calibration (`deadzone_left`, `deadzone_right`, `trigger_threshold`, optional button overrides) in SQLite keyed by device signature (`VID:PID` + name).

## Technical Best Practices

- **Hot-Plug Resilience**: Controllers disconnect frequently (Bluetooth sleep, USB cable bump). Handle connect/disconnect events gracefully without crashing or freezing the UI.
- **No Exclusive HID Locks**: Open controllers in shared/non-exclusive mode in the hub so emulators (especially RPCS3/Dolphin/PCSX2 using SDL or DualSense HID) are never blocked from claiming the device.
- **Input Drain on Focus Return**: After an emulator exits, discard buffered gamepad state for ~250–400ms before accepting new hub `Confirm` presses.
- **Keep Polling Off the Main UI Thread**: Run the gamepad polling/event loop on a dedicated Rust thread or async task and emit only state transitions (not 1000Hz raw noise) over Tauri IPC.

## Common Mistakes to Avoid

- **Ghost Input Behind Fullscreen Emulators**: Leaving hub gamepad listeners active while an emulator is running causes the hub to launch a second game or navigate menus invisibly in the background.
- **Holding Exclusive HID Handles**: Prevents DualSense/DualShock 4 native features (rumble, gyro, lightbar) from working inside RPCS3, PCSX2, or Dolphin.
- **Hardcoding Xbox Prompts Only**: Showing "Press A" to a DualSense user degrades console-hub polish; use semantic actions (`Confirm`, `Back`, `ContextMenu`) mapped to the active glyph family.
- **Flooding Tauri IPC With Raw Axis Frames**: Emitting every tiny analog jitter (`0.001 -> 0.002`) saturates WebView2 IPC; filter with deadzone + epsilon threshold in Rust first.

## Quality Checklist

- [ ] Hub gamepad navigation pauses cleanly before emulator launch and drains input on exit.
- [ ] Controllers are opened non-exclusively so emulators retain full HID/XInput/SDL access.
- [ ] Scaled radial deadzones prevent stick drift in UI navigation.
- [ ] Hot-plug connect and disconnect work without restarting Universal Console Hub.
- [ ] Button prompts adapt to PlayStation, Xbox, Nintendo, and Generic controllers (plus keyboard/mouse).
- [ ] IPC only receives meaningful state changes, never raw unfiltered polling floods.

## References

- Microsoft XInput API Overview: https://learn.microsoft.com/en-us/windows/win32/xinput/getting-started-with-xinput
- SDL GameControllerDB (Community Mapping Database): https://github.com/mdqinc/SDL_GameControllerDB
- `gilrs` Rust Gamepad Input Library: https://docs.rs/gilrs/
- W3C Gamepad API (WebView2 reference): https://www.w3.org/TR/gamepad/
