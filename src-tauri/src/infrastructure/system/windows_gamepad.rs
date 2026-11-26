use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GamepadGlyphFamily {
    Xbox,
    PlayStation,
    Nintendo,
    Generic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedGamepadInfo {
    pub slot: u32,
    pub name: String,
    pub backend: String,
    pub glyph_family: GamepadGlyphFamily,
    pub connected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GamepadNavAction {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Back,
    Secondary,
    ContextMenu,
    PrevTab,
    NextTab,
    PagePrev,
    PageNext,
    QuickMenu,
    ToggleMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GamepadActionEvent {
    pub slot: u32,
    pub action: GamepadNavAction,
    pub glyph_family: GamepadGlyphFamily,
    pub repeated: bool,
}

/// Applies a scaled radial deadzone to a normalized analog stick pair `[-1.0, 1.0]`.
/// Values inside `inner_deadzone` return `(0.0, 0.0)` cleanly so stick drift never scrolls UI.
pub fn apply_scaled_radial_deadzone(x: f32, y: f32, inner_deadzone: f32) -> (f32, f32) {
    let dz = inner_deadzone.clamp(0.05, 0.75);
    let mag = (x * x + y * y).sqrt();
    if mag < dz || mag == 0.0 {
        return (0.0, 0.0);
    }
    let scaled = ((mag - dz) / (1.0 - dz)).clamp(0.0, 1.0);
    let nx = (x / mag) * scaled;
    let ny = (y / mag) * scaled;
    (nx.clamp(-1.0, 1.0), ny.clamp(-1.0, 1.0))
}

#[derive(Debug, Clone)]
pub struct RepeatGate {
    initial_delay: Duration,
    repeat_interval: Duration,
    states: HashMap<GamepadNavAction, (Instant, Instant)>,
}

impl RepeatGate {
    pub fn new(initial_delay: Duration, repeat_interval: Duration) -> Self {
        Self {
            initial_delay,
            repeat_interval,
            states: HashMap::new(),
        }
    }

    /// Evaluates whether a held action should fire right now (`Some(repeated)` or `None`).
    /// Non-repeatable actions (`repeatable = false`) fire strictly once per press (rising edge).
    pub fn update(
        &mut self,
        action: GamepadNavAction,
        pressed: bool,
        repeatable: bool,
        now: Instant,
    ) -> Option<bool> {
        if !pressed {
            self.states.remove(&action);
            return None;
        }
        match self.states.get_mut(&action) {
            None => {
                self.states.insert(action, (now, now));
                Some(false)
            }
            Some((first_at, last_fired_at)) => {
                if !repeatable {
                    return None;
                }
                if now.duration_since(*first_at) >= self.initial_delay
                    && now.duration_since(*last_fired_at) >= self.repeat_interval
                {
                    *last_fired_at = now;
                    Some(true)
                } else {
                    None
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.states.clear();
    }
}

/// Non-exclusive Windows XInput poller with graceful emulator handoff (`pause` / `resume_with_drain`).
pub struct WindowsGamepadMonitor {
    paused: AtomicBool,
    drain_until: Mutex<Option<Instant>>,
    connected: Mutex<Vec<ConnectedGamepadInfo>>,
    running: AtomicBool,
}

impl Default for WindowsGamepadMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowsGamepadMonitor {
    pub fn new() -> Self {
        Self {
            paused: AtomicBool::new(false),
            drain_until: Mutex::new(None),
            connected: Mutex::new(Vec::new()),
            running: AtomicBool::new(false),
        }
    }

    /// Suspends hub gamepad processing immediately before launching an emulator (zero ghost input).
    pub fn pause(&self) {
        self.paused.store(true, Ordering::SeqCst);
    }

    /// Resumes hub gamepad processing after an emulator exits with an input-drain cooldown so
    /// trailing presses inside the game never trigger accidental Hub confirmations.
    pub fn resume_with_drain(&self, cooldown: Duration) {
        if let Ok(mut guard) = self.drain_until.lock() {
            *guard = Some(Instant::now() + cooldown);
        }
        self.paused.store(false, Ordering::SeqCst);
    }

    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::SeqCst)
    }

    pub fn list_connected_gamepads(&self) -> Vec<ConnectedGamepadInfo> {
        self.connected.lock().map(|g| g.clone()).unwrap_or_default()
    }

    pub fn start_with_tauri(self: &Arc<Self>, app_handle: AppHandle) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }
        let monitor = Arc::clone(self);
        std::thread::Builder::new()
            .name("uch-xinput-poller".to_string())
            .spawn(move || {
                monitor.poll_loop(app_handle);
            })
            .ok();
    }

    #[cfg(target_os = "windows")]
    fn poll_loop(&self, app_handle: AppHandle) {
        let Some(xinput_fn) = load_xinput_get_state() else {
            return;
        };
        let mut gates: [RepeatGate; 4] = std::array::from_fn(|_| {
            RepeatGate::new(Duration::from_millis(380), Duration::from_millis(115))
        });
        let mut last_devices: Vec<ConnectedGamepadInfo> = Vec::new();

        loop {
            std::thread::sleep(Duration::from_millis(16));
            let now = Instant::now();

            let draining = self
                .drain_until
                .lock()
                .ok()
                .and_then(|g| *g)
                .is_some_and(|until| now < until);

            if self.paused.load(Ordering::SeqCst) || draining {
                for g in &mut gates {
                    g.clear();
                }
                continue;
            }

            let mut current_devices = Vec::new();
            for slot in 0..4u32 {
                let mut state = XInputState::default();
                // SAFETY: `xinput_fn` was resolved from a loaded system XInput DLL with matching ABI.
                let code = unsafe { xinput_fn(slot, &mut state as *mut XInputState) };
                if code == 0 {
                    current_devices.push(ConnectedGamepadInfo {
                        slot,
                        name: format!("XInput Controller #{}", slot + 1),
                        backend: "xinput".to_string(),
                        glyph_family: GamepadGlyphFamily::Xbox,
                        connected: true,
                    });

                    let pad = &state.gamepad;
                    let lx = f32::from(pad.thumb_lx) / 32767.0;
                    let ly = f32::from(pad.thumb_ly) / 32767.0;
                    let (sx, sy) = apply_scaled_radial_deadzone(lx, ly, 0.24);
                    let buttons = pad.buttons;

                    let checks: &[(GamepadNavAction, bool, bool)] = &[
                        (
                            GamepadNavAction::Up,
                            (buttons & 0x0001) != 0 || sy > 0.55,
                            true,
                        ),
                        (
                            GamepadNavAction::Down,
                            (buttons & 0x0002) != 0 || sy < -0.55,
                            true,
                        ),
                        (
                            GamepadNavAction::Left,
                            (buttons & 0x0004) != 0 || sx < -0.55,
                            true,
                        ),
                        (
                            GamepadNavAction::Right,
                            (buttons & 0x0008) != 0 || sx > 0.55,
                            true,
                        ),
                        (GamepadNavAction::QuickMenu, (buttons & 0x0010) != 0, false),
                        (GamepadNavAction::ToggleMode, (buttons & 0x0020) != 0, false),
                        (GamepadNavAction::PrevTab, (buttons & 0x0100) != 0, true),
                        (GamepadNavAction::NextTab, (buttons & 0x0200) != 0, true),
                        (GamepadNavAction::Confirm, (buttons & 0x1000) != 0, false),
                        (GamepadNavAction::Back, (buttons & 0x2000) != 0, false),
                        (GamepadNavAction::Secondary, (buttons & 0x4000) != 0, false),
                        (
                            GamepadNavAction::ContextMenu,
                            (buttons & 0x8000) != 0,
                            false,
                        ),
                        (GamepadNavAction::PagePrev, pad.left_trigger > 120, true),
                        (GamepadNavAction::PageNext, pad.right_trigger > 120, true),
                    ];

                    let gate = &mut gates[slot as usize];
                    for &(action, pressed, repeatable) in checks {
                        if let Some(repeated) = gate.update(action, pressed, repeatable, now) {
                            let _ = app_handle.emit(
                                "gamepad:action",
                                GamepadActionEvent {
                                    slot,
                                    action,
                                    glyph_family: GamepadGlyphFamily::Xbox,
                                    repeated,
                                },
                            );
                        }
                    }
                } else {
                    gates[slot as usize].clear();
                }
            }

            if current_devices != last_devices {
                if let Ok(mut guard) = self.connected.lock() {
                    *guard = current_devices.clone();
                }
                let _ = app_handle.emit("gamepad:devices", &current_devices);
                last_devices = current_devices;
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn poll_loop(&self, _app_handle: AppHandle) {}
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Default)]
struct XInputGamepad {
    buttons: u16,
    left_trigger: u8,
    right_trigger: u8,
    thumb_lx: i16,
    thumb_ly: i16,
    thumb_rx: i16,
    thumb_ry: i16,
}

#[cfg(target_os = "windows")]
#[repr(C)]
#[derive(Default)]
struct XInputState {
    packet_number: u32,
    gamepad: XInputGamepad,
}

#[cfg(target_os = "windows")]
type XInputGetStateFn = unsafe extern "system" fn(u32, *mut XInputState) -> u32;

#[cfg(target_os = "windows")]
fn load_xinput_get_state() -> Option<XInputGetStateFn> {
    use std::ffi::c_void;

    #[link(name = "kernel32")]
    extern "system" {
        fn LoadLibraryW(lp_lib_file_name: *const u16) -> *mut c_void;
        fn GetProcAddress(h_module: *mut c_void, lp_proc_name: *const u8) -> *const c_void;
    }

    for dll in ["xinput1_4.dll", "xinput1_3.dll", "xinput9_1_0.dll"] {
        let wide: Vec<u16> = dll.encode_utf16().chain(std::iter::once(0)).collect();
        // SAFETY: Null-terminated UTF-16 DLL name passed to Win32 `LoadLibraryW`.
        let handle = unsafe { LoadLibraryW(wide.as_ptr()) };
        if handle.is_null() {
            continue;
        }
        // SAFETY: Null-terminated ASCII symbol name on valid HMODULE returned by `LoadLibraryW`.
        let proc = unsafe { GetProcAddress(handle, c"XInputGetState".as_ptr().cast::<u8>()) };
        if !proc.is_null() {
            // SAFETY: Symbol `XInputGetState` has exact Win32 ABI `extern "system" fn(u32, *mut XInputState) -> u32`.
            let f: XInputGetStateFn = unsafe { std::mem::transmute(proc) };
            return Some(f);
        }
    }
    None
}
