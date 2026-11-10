//! Streamer Mode: on by hand, or while OBS or XSplit runs, it keeps private
//! details, sounds and notifications off a stream, and can hide the window
//! from screen capture.

use std::sync::atomic::{AtomicBool, Ordering::Relaxed};

use super::*;

static ACTIVE: AtomicBool = AtomicBool::new(false);
static SOUNDS_OFF: AtomicBool = AtomicBool::new(true);
static NOTIFICATIONS_OFF: AtomicBool = AtomicBool::new(true);
static HIDE_INFO: AtomicBool = AtomicBool::new(true);
/// What Streamer Mode hides changed, so messages are drawn again.
static CHANGED: AtomicBool = AtomicBool::new(false);
/// OBS or XSplit runs, as the last look found.
static BROADCASTING: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// What looks for OBS and XSplit, for the window shown.
    static WATCH: slint::Timer = slint::Timer::default();
}

/// Streamer Mode is on and keeps sounds quiet.
pub(super) fn quiets_sounds() -> bool {
    ACTIVE.load(Relaxed) && SOUNDS_OFF.load(Relaxed)
}

/// Streamer Mode is on and keeps notifications off the screen.
pub(super) fn quiets_notifications() -> bool {
    ACTIVE.load(Relaxed) && NOTIFICATIONS_OFF.load(Relaxed)
}

/// Streamer Mode is on and hides personal information, invite links among it.
pub(super) fn hides_info() -> bool {
    ACTIVE.load(Relaxed) && HIDE_INFO.load(Relaxed)
}

/// Whether what Streamer Mode hides changed since last asked.
pub(super) fn take_changed() -> bool {
    CHANGED.swap(false, Relaxed)
}

/// `text` with the codes of Discord invite links hidden, while Streamer Mode
/// hides personal information, as Discord's does.
pub(super) fn mask_invites(text: &str) -> std::borrow::Cow<'_, str> {
    const HOSTS: [&str; 3] = ["discord.gg/", "discord.com/invite/", "discordapp.com/invite/"];
    if !hides_info() || !HOSTS.iter().any(|host| text.contains(host)) {
        return std::borrow::Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some((at, host)) = HOSTS
        .iter()
        .filter_map(|host| rest.find(host).map(|at| (at, *host)))
        .min_by_key(|(at, _)| *at)
    {
        let after = at + host.len();
        out.push_str(&rest[..after]);
        let code = rest[after..]
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .unwrap_or(rest.len() - after);
        out.push_str(if code > 0 { "hidden" } else { "" });
        rest = &rest[after + code..];
    }
    out.push_str(rest);
    std::borrow::Cow::Owned(out)
}

/// Shows Streamer Mode as the window's settings and OBS say, and hides the
/// window from capture while it should be.
pub(super) fn apply(window: &StaticWindow) {
    let active = window.get_streamer_mode()
        || (window.get_streamer_auto() && BROADCASTING.load(Relaxed));
    ACTIVE.store(active, Relaxed);
    SOUNDS_OFF.store(window.get_streamer_sounds_off(), Relaxed);
    NOTIFICATIONS_OFF.store(window.get_streamer_notifications_off(), Relaxed);
    let hide = window.get_streamer_hide_info();
    if HIDE_INFO.swap(hide, Relaxed) != hide {
        CHANGED.store(true, Relaxed);
    }
    if window.get_streamer_active() != active {
        window.set_streamer_active(active);
        CHANGED.store(true, Relaxed);
    }
    hide_from_capture(window, active && window.get_streamer_hide_capture());
}

/// Looks for OBS and XSplit every few seconds while the automatic switch is
/// on, for this window from now on.
pub(super) fn watch(window: &StaticWindow) {
    let weak = window.as_weak();
    let look = move || {
        let Some(window) = weak.upgrade() else {
            return;
        };
        BROADCASTING.store(window.get_streamer_auto() && broadcaster_running(), Relaxed);
        // The window may only now exist on screen, so its capture is set again.
        apply(&window);
    };
    WATCH.with(|timer| {
        timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_secs(5),
            look,
        );
    });
}

#[cfg(windows)]
fn broadcaster_running() -> bool {
    use windows_sys::Win32::{
        Foundation::{CloseHandle, INVALID_HANDLE_VALUE},
        System::Diagnostics::ToolHelp::{
            CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
            TH32CS_SNAPPROCESS,
        },
    };
    const NAMES: [&str; 4] = ["obs64.exe", "obs32.exe", "obs.exe", "xsplit.core.exe"];
    // SAFETY: the snapshot is closed before returning, and each entry's size
    // is set before the calls that fill it.
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return false;
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut found = false;
        let mut more = Process32FirstW(snapshot, &mut entry) != 0;
        while more {
            let length = entry
                .szExeFile
                .iter()
                .position(|&unit| unit == 0)
                .unwrap_or(entry.szExeFile.len());
            let name = String::from_utf16_lossy(&entry.szExeFile[..length]).to_ascii_lowercase();
            if NAMES.contains(&name.as_str()) {
                found = true;
                break;
            }
            more = Process32NextW(snapshot, &mut entry) != 0;
        }
        CloseHandle(snapshot);
        found
    }
}

#[cfg(not(windows))]
fn broadcaster_running() -> bool {
    false
}

/// Windows leaves the window out of screenshots, recordings and screen
/// sharing while `hidden`; elsewhere this does nothing.
fn hide_from_capture(window: &StaticWindow, hidden: bool) {
    #[cfg(windows)]
    {
        use slint::winit_030::WinitWindowAccessor as _;
        use slint::winit_030::winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SetWindowDisplayAffinity, WDA_EXCLUDEFROMCAPTURE, WDA_NONE,
        };
        let _ = window.window().with_winit_window(|winit| {
            if let Ok(handle) = winit.window_handle()
                && let RawWindowHandle::Win32(handle) = handle.as_raw()
            {
                let affinity = if hidden { WDA_EXCLUDEFROMCAPTURE } else { WDA_NONE };
                // SAFETY: the handle is this window's, alive while winit has it.
                unsafe {
                    SetWindowDisplayAffinity(handle.hwnd.get() as _, affinity);
                }
            }
        });
    }
    #[cfg(not(windows))]
    let _ = (window, hidden);
}
