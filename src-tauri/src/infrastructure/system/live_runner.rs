use std::collections::VecDeque;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use crate::domain::emulator::LaunchCommand;
use crate::domain::error::DomainError;

const MAX_TAIL_LINES: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveProcessExit {
    pub exit_code: Option<i32>,
    pub stdout_tail: Vec<String>,
    pub stderr_tail: Vec<String>,
    pub stopped_by_user: bool,
    pub graceful_close_attempted: bool,
}

#[derive(Clone)]
pub struct LiveProcessStopHandle {
    tx: Arc<Mutex<Option<oneshot::Sender<Duration>>>>,
}

impl LiveProcessStopHandle {
    /// Requests a graceful close (Win32 `WM_CLOSE` on Windows) followed by forced
    /// termination only if the process does not exit within `grace_period`.
    pub fn request_stop(&self, grace_period: Duration) -> bool {
        if let Ok(mut guard) = self.tx.lock() {
            if let Some(sender) = guard.take() {
                return sender.send(grace_period).is_ok();
            }
        }
        false
    }
}

pub struct LiveProcessHandle {
    pid: Option<u32>,
    child: Child,
    stdout_task: Option<JoinHandle<Vec<String>>>,
    stderr_task: Option<JoinHandle<Vec<String>>>,
    stop_rx: oneshot::Receiver<Duration>,
    stop_handle: LiveProcessStopHandle,
}

impl LiveProcessHandle {
    pub fn pid(&self) -> Option<u32> {
        self.pid
    }

    pub fn stop_handle(&self) -> LiveProcessStopHandle {
        self.stop_handle.clone()
    }

    pub async fn wait(mut self) -> Result<LiveProcessExit, DomainError> {
        let mut stopped_by_user = false;
        let mut graceful_close_attempted = false;

        let exit_code = tokio::select! {
            wait_res = self.child.wait() => {
                let status = wait_res.map_err(|e| {
                    DomainError::Process(format!("failed waiting on live emulator process: {e}"))
                })?;
                status.code()
            }
            stop_req = &mut self.stop_rx => {
                let grace_period = stop_req.unwrap_or(Duration::from_secs(3));
                stopped_by_user = true;
                graceful_close_attempted = true;

                if let Some(pid) = self.pid {
                    request_graceful_window_close(pid);
                }

                match tokio::time::timeout(grace_period, self.child.wait()).await {
                    Ok(Ok(status)) => status.code(),
                    Ok(Err(e)) => {
                        return Err(DomainError::Process(format!(
                            "failed waiting during graceful close: {e}"
                        )));
                    }
                    Err(_) => {
                        let _ = self.child.kill().await;
                        self.child.wait().await.ok().and_then(|s| s.code())
                    }
                }
            }
        };

        let stdout_tail = match self.stdout_task {
            Some(handle) => handle.await.unwrap_or_default(),
            None => Vec::new(),
        };
        let stderr_tail = match self.stderr_task {
            Some(handle) => handle.await.unwrap_or_default(),
            None => Vec::new(),
        };

        Ok(LiveProcessExit {
            exit_code,
            stdout_tail,
            stderr_tail,
            stopped_by_user,
            graceful_close_attempted,
        })
    }
}

async fn drain_bounded_lines<R: AsyncRead + Unpin>(reader: R) -> Vec<String> {
    let mut buf_reader = BufReader::new(reader);
    let mut ring = VecDeque::with_capacity(MAX_TAIL_LINES);
    let mut raw = Vec::new();
    loop {
        raw.clear();
        match buf_reader.read_until(b'\n', &mut raw).await {
            Ok(0) => break,
            Ok(_) => {
                let line = String::from_utf8_lossy(&raw)
                    .trim_end_matches(['\r', '\n'])
                    .to_string();
                if ring.len() == MAX_TAIL_LINES {
                    ring.pop_front();
                }
                ring.push_back(line);
            }
            Err(_) => break,
        }
    }
    ring.into_iter().collect()
}

pub struct LiveEmulatorRunner;

impl LiveEmulatorRunner {
    /// Spawns a real emulator process via direct argument array (`Command::new(exe).args(...)`),
    /// never invoking a shell (`cmd.exe` / PowerShell), and starts bounded async stdout/stderr draining.
    pub fn spawn(cmd: &LaunchCommand) -> Result<LiveProcessHandle, DomainError> {
        if !cmd.executable.is_file() {
            return Err(DomainError::NotFound(format!(
                "executable not found: '{}'",
                cmd.executable.display()
            )));
        }

        let mut command = Command::new(&cmd.executable);
        command
            .args(&cmd.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(false);

        if let Some(dir) = &cmd.working_dir {
            command.current_dir(dir);
        }

        let mut child = command.spawn().map_err(|e| {
            DomainError::Process(format!(
                "failed to spawn '{}': {e}",
                cmd.executable.display()
            ))
        })?;

        let pid = child.id();
        let stdout_task = child
            .stdout
            .take()
            .map(|out| tokio::spawn(drain_bounded_lines(out)));
        let stderr_task = child
            .stderr
            .take()
            .map(|err| tokio::spawn(drain_bounded_lines(err)));

        let (tx, stop_rx) = oneshot::channel();
        let stop_handle = LiveProcessStopHandle {
            tx: Arc::new(Mutex::new(Some(tx))),
        };

        Ok(LiveProcessHandle {
            pid,
            child,
            stdout_task,
            stderr_task,
            stop_rx,
            stop_handle,
        })
    }
}

#[cfg(windows)]
fn request_graceful_window_close(target_pid: u32) {
    type Hwnd = isize;
    const WM_CLOSE: u32 = 0x0010;

    #[link(name = "user32")]
    extern "system" {
        fn EnumWindows(lp_enum_func: extern "system" fn(Hwnd, isize) -> i32, lparam: isize) -> i32;
        fn GetWindowThreadProcessId(hwnd: Hwnd, lpdw_process_id: *mut u32) -> u32;
        fn PostMessageW(hwnd: Hwnd, msg: u32, wparam: usize, lparam: isize) -> i32;
    }

    extern "system" fn enum_callback(hwnd: Hwnd, lparam: isize) -> i32 {
        let wanted_pid = lparam as u32;
        let mut window_pid: u32 = 0;
        // SAFETY: `hwnd` is supplied by Win32 EnumWindows and `window_pid` is a valid stack pointer.
        unsafe {
            GetWindowThreadProcessId(hwnd, &mut window_pid);
            if window_pid == wanted_pid {
                let _ = PostMessageW(hwnd, WM_CLOSE, 0, 0);
            }
        }
        1
    }

    // SAFETY: Standard Win32 EnumWindows invocation passing target PID by value in lparam.
    unsafe {
        let _ = EnumWindows(enum_callback, target_pid as isize);
    }
}

#[cfg(not(windows))]
fn request_graceful_window_close(_target_pid: u32) {}
