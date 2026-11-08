//! Updates from Chatter's GitHub releases: asks GitHub for the latest
//! release, and on Windows downloads its ZIP, swaps the executable once
//! Chatter has quit, and starts the new one.

use std::{path::PathBuf, process::Command};

/// Where releases are published.
const REPO: &str = "granndpa/chatter";
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

/// Downloads the release and readies the swap; Chatter must quit right
/// after, which lets the swap replace its executable and start the new one.
#[cfg(windows)]
pub fn install(release: &Release) -> Result<(), String> {
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
    let mut tar = Command::new("tar");
    {
        use std::os::windows::process::CommandExt;
        tar.creation_flags(0x0800_0000);
    }
    let unpacked = tar
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
    let pid = std::process::id();
    // Waits for this Chatter to quit, swaps the executable, starts it again.
    let script = format!(
        "@echo off\r\n\
         :wait\r\n\
         tasklist /FI \"PID eq {pid}\" | find \"{pid}\" >nul && (timeout /t 1 /nobreak >nul & goto wait)\r\n\
         copy /y \"{fresh}\" \"{current}\" >nul\r\n\
         start \"\" \"{current}\"\r\n\
         rmdir /s /q \"{work}\"\r\n",
        fresh = fresh.display(),
        current = current.display(),
        work = work.display(),
    );
    let runner = std::env::temp_dir().join("chatter-update.cmd");
    std::fs::write(&runner, script).map_err(|error| error.to_string())?;
    let mut swap = Command::new("cmd");
    {
        use std::os::windows::process::CommandExt;
        // Detached and without a window, so it outlives Chatter.
        swap.creation_flags(0x0800_0000 | 0x0000_0008);
    }
    swap.arg("/c")
        .arg(&runner)
        .spawn()
        .map_err(|error| error.to_string())?;
    Ok(())
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
