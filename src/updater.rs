//! Updates from Chatter's GitHub releases: asks GitHub for the latest
//! release, and on Windows downloads its ZIP, puts the new executable in
//! place, and starts it once Chatter has quit.

use std::{path::PathBuf, process::Command};

/// Where releases are published.
const REPO: &str = "granndpa/chatter-beta";
/// The release's file for Windows.
const ASSET: &str = "Chatter-windows-x64.zip";

/// A newer release than this build.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    /// Such as `0.5.1`.
    pub version: String,
    /// The Windows ZIP's address, when the release has one.
    pub download: Option<String>,
    /// The release's page.
    pub page: String,
}

fn curl() -> Command {
    let mut curl = Command::new("curl");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // No console window flashes up.
        curl.creation_flags(0x0800_0000);
    }
    curl
}

/// The latest release, when it's newer than this build.
pub fn check() -> Result<Option<Release>, String> {
    let output = curl()
        .args(["-sSfL", "--max-time", "20", "-H", "Accept: application/vnd.github+json", "-H"])
        .arg(format!("User-Agent: Chatter/{}", env!("CARGO_PKG_VERSION")))
        .arg(format!("https://api.github.com/repos/{REPO}/releases/latest"))
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err("GitHub didn't answer".into());
    }
    let release = parse(&output.stdout)?;
    Ok(newer(&release.version, env!("CARGO_PKG_VERSION")).then_some(release))
}

fn parse(body: &[u8]) -> Result<Release, String> {
    let value: serde_json::Value =
        serde_json::from_slice(body).map_err(|_| "GitHub's answer was unreadable".to_owned())?;
    let version = value["tag_name"]
        .as_str()
        .ok_or("the release has no version")?
        .trim_start_matches('v')
        .to_owned();
    let download = value["assets"].as_array().and_then(|assets| {
        assets
            .iter()
            .find(|asset| asset["name"] == ASSET)
            .and_then(|asset| asset["browser_download_url"].as_str())
            .map(str::to_owned)
    });
    let page = value["html_url"].as_str().unwrap_or_default().to_owned();
    Ok(Release { version, download, page })
}

/// Whether `latest` is a higher version than `current`, part by part.
fn newer(latest: &str, current: &str) -> bool {
    let parts = |version: &str| -> Vec<u64> {
        version
            .split(['.', '-'])
            .map_while(|part| part.parse().ok())
            .collect()
    };
    parts(latest) > parts(current)
}

/// Downloads the release and puts its executable in place of this one,
/// which Windows lets a running program rename out of the way. Chatter
/// quits right after, and a hidden PowerShell starts the new one once it's
/// gone. Nothing shows a window.
#[cfg(windows)]
pub fn install(release: &Release) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const NO_WINDOW: u32 = 0x0800_0000;
    let url = release.download.as_deref().ok_or("this release has no Windows download")?;
    let work: PathBuf = std::env::temp_dir().join("chatter-update");
    let _ = std::fs::remove_dir_all(&work);
    std::fs::create_dir_all(&work).map_err(|error| error.to_string())?;
    let zip = work.join(ASSET);
    let downloaded = curl()
        .args(["-sSfL", "--max-time", "300", "-o"])
        .arg(&zip)
        .arg(url)
        .status()
        .map_err(|error| error.to_string())?;
    if !downloaded.success() {
        return Err("the download failed".into());
    }
    let unpacked = Command::new("tar")
        .creation_flags(NO_WINDOW)
        .arg("-xf")
        .arg(&zip)
        .arg("-C")
        .arg(&work)
        .status()
        .map_err(|error| error.to_string())?;
    let fresh = work.join("chatter.exe");
    if !unpacked.success() || !fresh.exists() {
        return Err("the download had no chatter.exe".into());
    }
    let current = std::env::current_exe().map_err(|error| error.to_string())?;
    let old = old_exe(&current);
    let _ = std::fs::remove_file(&old);
    std::fs::rename(&current, &old).map_err(|error| format!("couldn't move the old version: {error}"))?;
    if let Err(error) = std::fs::copy(&fresh, &current) {
        let _ = std::fs::rename(&old, &current);
        return Err(format!("couldn't put the new version in place: {error}"));
    }
    // Starts the new version once this one has quit, so it doesn't find
    // this one still running and hand over to it.
    let path = current.display().to_string().replace('\'', "''");
    let started = Command::new("powershell")
        .creation_flags(NO_WINDOW)
        .args(["-NoProfile", "-NonInteractive", "-WindowStyle", "Hidden", "-Command"])
        .arg(format!(
            "Wait-Process -Id {} -ErrorAction SilentlyContinue; Start-Process -FilePath '{path}'",
            std::process::id()
        ))
        .spawn();
    if let Err(error) = started {
        tracing::warn!(%error, "could not schedule the restart; start Chatter again by hand");
    }
    Ok(())
}

/// Where the replaced executable waits until the next start deletes it.
#[cfg(windows)]
fn old_exe(current: &std::path::Path) -> PathBuf {
    current.with_file_name("chatter.old.exe")
}

/// Deletes what the last update left: the old executable and the download.
pub fn clean_up() {
    #[cfg(windows)]
    if let Ok(current) = std::env::current_exe() {
        let _ = std::fs::remove_file(old_exe(&current));
    }
    let _ = std::fs::remove_dir_all(std::env::temp_dir().join("chatter-update"));
    let _ = std::fs::remove_file(std::env::temp_dir().join("chatter-update.cmd"));
}

/// Elsewhere, updating is the package manager's or the AppImage's job: the
/// release's page opens instead.
#[cfg(not(windows))]
pub fn install(release: &Release) -> Result<(), String> {
    webbrowser::open(&release.page).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_part_by_part() {
        assert!(newer("0.5.1", "0.5.0"));
        assert!(newer("0.10.0", "0.9.9"));
        assert!(!newer("0.5.0", "0.5.0"));
        assert!(!newer("0.4.9", "0.5.0"));
    }

    #[test]
    fn a_release_gives_its_version_and_windows_download() {
        let body = br#"{"tag_name":"v0.6.0","html_url":"https://github.com/granndpa/chatter/releases/tag/v0.6.0",
            "assets":[{"name":"other.zip","browser_download_url":"x"},
                      {"name":"Chatter-windows-x64.zip","browser_download_url":"https://example.com/c.zip"}]}"#;
        let release = parse(body).unwrap();
        assert_eq!(release.version, "0.6.0");
        assert_eq!(release.download.as_deref(), Some("https://example.com/c.zip"));
    }
}
