//! Watches whether this computer is online, so Discord's connection drops as
//! soon as the network goes, and comes back as soon as it returns, rather than
//! waiting for a missed heartbeat or a retry timer.

use std::{sync::Once, time::Duration};

/// How often Windows is asked.
const EVERY: Duration = Duration::from_millis(1500);

/// Starts watching, once per run.
pub(super) fn start() {
    static STARTED: Once = Once::new();
    if cfg!(test) {
        return;
    }
    STARTED.call_once(|| {
        let started = std::thread::Builder::new()
            .name("chatter-network".into())
            .spawn(|| {
                loop {
                    #[cfg(feature = "discord")]
                    static_core::discord_network::set(online());
                    std::thread::sleep(EVERY);
                }
            });
        if let Err(error) = started {
            tracing::warn!(%error, "could not watch the network");
        }
    });
}

/// Whether Windows has a network connection.
#[cfg(windows)]
fn online() -> bool {
    use windows_sys::Win32::Networking::WinInet::InternetGetConnectedState;
    let mut flags = 0;
    // SAFETY: only writes the flags it's given.
    unsafe { InternetGetConnectedState(&mut flags, 0) != 0 }
}

/// Elsewhere the connection's own heartbeats decide.
#[cfg(not(windows))]
fn online() -> bool {
    true
}
