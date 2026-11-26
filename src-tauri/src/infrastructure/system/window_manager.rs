use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use tauri::{AppHandle, Manager};

use crate::domain::error::DomainError;

/// Coordinates Hub desktop window focus transitions when launching and exiting real emulators.
pub trait WindowLifecycleHook: Send + Sync {
    fn on_emulator_launched(&self) -> Result<(), DomainError>;
    fn on_emulator_exited(&self) -> Result<(), DomainError>;
}

#[derive(Debug, Default, Clone)]
pub struct NoopWindowHook;

impl WindowLifecycleHook for NoopWindowHook {
    fn on_emulator_launched(&self) -> Result<(), DomainError> {
        Ok(())
    }

    fn on_emulator_exited(&self) -> Result<(), DomainError> {
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct RecordingWindowHook {
    launched: Arc<AtomicUsize>,
    exited: Arc<AtomicUsize>,
}

impl RecordingWindowHook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn launched_count(&self) -> usize {
        self.launched.load(Ordering::SeqCst)
    }

    pub fn exited_count(&self) -> usize {
        self.exited.load(Ordering::SeqCst)
    }
}

impl WindowLifecycleHook for RecordingWindowHook {
    fn on_emulator_launched(&self) -> Result<(), DomainError> {
        self.launched.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn on_emulator_exited(&self) -> Result<(), DomainError> {
        self.exited.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

/// Real Windows Tauri 2 window manager: minimizes the Hub cleanly before an emulator
/// takes fullscreen GPU ownership, and restores + focuses the Hub window on exit.
#[derive(Clone)]
pub struct TauriWindowManager {
    app_handle: AppHandle,
    window_label: String,
}

impl TauriWindowManager {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            window_label: "main".to_string(),
        }
    }
}

impl WindowLifecycleHook for TauriWindowManager {
    fn on_emulator_launched(&self) -> Result<(), DomainError> {
        if let Some(window) = self.app_handle.get_webview_window(&self.window_label) {
            window.minimize().map_err(|e| {
                DomainError::Process(format!("failed to minimize Hub window on launch: {e}"))
            })?;
        }
        Ok(())
    }

    fn on_emulator_exited(&self) -> Result<(), DomainError> {
        if let Some(window) = self.app_handle.get_webview_window(&self.window_label) {
            let _ = window.unminimize();
            let _ = window.show();
            window.set_focus().map_err(|e| {
                DomainError::Process(format!("failed to restore Hub window focus on exit: {e}"))
            })?;
        }
        Ok(())
    }
}
