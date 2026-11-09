//! Discord Tweaks' voice overlay: a small window of its own, always on top
//! and letting clicks through, that lists who's in the call and who's
//! speaking while Chatter isn't in front, such as over a game.
//!
//! It's an ordinary window, never anything injected into a game, so it shows
//! over borderless and windowed games but not exclusive fullscreen ones,
//! which Windows draws over everything else.

use super::*;

#[derive(Default)]
pub(super) struct Overlay {
    window: Option<VoiceOverlay>,
    /// Click-through was set on the window Windows made for it.
    through: bool,
}

/// Where the overlay sits, in screen pixels: corner 0 top left, 1 top
/// right, 2 bottom left, 3 bottom right; `width` and `height` in pixels too.
fn place(corner: i32, width: f32, height: f32, scale_factor: f32) -> slint::PhysicalPosition {
    let (screen_width, screen_height) = screen();
    let margin = 16.0 * scale_factor;
    let x = if corner % 2 == 1 { screen_width - width - margin } else { margin };
    let y = if corner >= 2 {
        screen_height - height - 64.0 * scale_factor
    } else {
        margin + 24.0 * scale_factor
    };
    slint::PhysicalPosition::new(x.max(0.0) as i32, y.max(0.0) as i32)
}

#[cfg(windows)]
fn screen() -> (f32, f32) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    // SAFETY: only reads a setting.
    let (width, height) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    (width as f32, height as f32)
}

#[cfg(not(windows))]
fn screen() -> (f32, f32) {
    (1920.0, 1080.0)
}

/// Lets clicks fall through to the game, keeps it off the taskbar and out
/// of Alt+Tab, and never takes the focus.
#[cfg(windows)]
fn click_through(window: &VoiceOverlay) -> bool {
    use slint::winit_030::WinitWindowAccessor as _;
    use slint::winit_030::winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetWindowLongPtrW, LWA_ALPHA, SetLayeredWindowAttributes, SetWindowLongPtrW,
        WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
    };
    window
        .window()
        .with_winit_window(|winit| {
            let Ok(handle) = winit.window_handle() else { return false };
            let RawWindowHandle::Win32(handle) = handle.as_raw() else { return false };
            let hwnd = handle.hwnd.get() as windows_sys::Win32::Foundation::HWND;
            // SAFETY: the window is this overlay's, alive while it's shown.
            unsafe {
                let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                SetWindowLongPtrW(
                    hwnd,
                    GWL_EXSTYLE,
                    style | (WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE) as isize,
                );
                SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA);
            }
            true
        })
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn click_through(_: &VoiceOverlay) -> bool {
    true
}

/// Shows the overlay while in a call with Chatter not in front, and hides it
/// otherwise.
pub(super) fn show(adapter: &mut Adapter) {
    let voice = &adapter.app.state.voice;
    let wanted = adapter.layout.overlay && voice.active() && !FOCUSED.get();
    if !wanted {
        if let Some(window) = &adapter.overlay.window {
            let _ = window.hide();
        }
        return;
    }
    let people: Vec<OverlayPerson> = voice
        .participants
        .iter()
        .map(|person| {
            let name = display_part(&person.user);
            OverlayPerson {
                name: name.into(),
                initial: initial(name),
                picture_url: person.avatar.clone().unwrap_or_default().into(),
                speaking: person.speaking && !person.muted && person.connected,
                muted: person.muted,
                deafened: person.deafened,
            }
        })
        .collect();
    let count = people.len().max(1) as f32;
    if adapter.overlay.window.is_none() {
        match VoiceOverlay::new() {
            Ok(window) => adapter.overlay.window = Some(window),
            Err(error) => {
                tracing::warn!(%error, "could not make the voice overlay");
                return;
            }
        }
    }
    let layout = &adapter.layout;
    let scale = [0.8, 1.0, 1.25][layout.overlay_scale.unwrap_or(1).clamp(0, 2) as usize];
    let Some(window) = &adapter.overlay.window else { return };
    window.set_people(ModelRc::new(VecModel::from(people)));
    window.set_avatars(layout.overlay_avatars.unwrap_or(true));
    window.set_scale(scale);
    window.set_backing([0.75, 0.55, 0.3][layout.overlay_opacity.unwrap_or(1).clamp(0, 2) as usize]);
    let factor = window.window().scale_factor();
    let (width, height) = (240.0 * scale * factor, (count * 34.0 + 8.0) * scale * factor);
    window
        .window()
        .set_position(place(layout.overlay_corner.unwrap_or(1), width, height, factor));
    if !window.window().is_visible() {
        let _ = window.show();
    }
    if !adapter.overlay.through {
        adapter.overlay.through = click_through(window);
    }
}
