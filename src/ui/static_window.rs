//! Selects the presentation without changing the shared window contract.
use super::*;

pub(super) fn enabled() -> bool {
    !std::env::var("YAPFAST_UI").is_ok_and(|mode| mode == "classic")
}

/// Used at startup and when the tray recreates the window.
pub(super) fn window() -> Result<StaticWindow, slint::PlatformError> {
    let window = StaticWindow::new()?;
    let style = window.global::<ClientStyle>();
    style.set_enabled(enabled());
    Ok(window)
}
