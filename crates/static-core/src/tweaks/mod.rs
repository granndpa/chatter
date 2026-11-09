//! Discord Tweaks' local data: bookmarks (with the Hall of Fame), message
//! reminders, saved drafts and pinned servers and friends. Each Discord account keeps its own files in
//! its own folder beside the session, as JSON, and nothing here leaves this
//! computer.

pub mod bookmarks;
pub mod drafts;
pub mod inbox;
pub mod pins;
pub mod reminders;
pub mod stats;

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// A message a tweak saved: enough to show it without Discord, and to find
/// it again.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SavedMessage {
    /// The conversation's room id, and the message's item in it.
    pub room: String,
    pub item: String,
    /// The server's name, empty for a DM, and the channel's or DM's name.
    pub server: String,
    pub channel: String,
    pub author: String,
    pub body: String,
    /// When it was sent, in seconds since the Unix epoch; zero when unknown.
    pub sent: u64,
    /// The address of its first picture, for a thumbnail.
    pub picture: Option<String>,
}

/// The folder an account's tweak files go in, under the data folder.
pub fn account_dir(root: &Path, account: &str) -> PathBuf {
    let name: String = account
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect();
    root.join("tweaks")
        .join(if name.is_empty() { "default" } else { &name })
}

/// Reads a tweak file, or its default when it's missing or unreadable.
pub fn load<T: DeserializeOwned + Default>(dir: &Path, name: &str) -> T {
    fs::read(dir.join(name))
        .ok()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}

/// Writes a tweak file whole: to a temporary file first, then over the old
/// one, so a crash never leaves half a file.
pub fn save<T: Serialize>(dir: &Path, name: &str, value: &T) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let bytes = serde_json::to_vec_pretty(value).map_err(io::Error::other)?;
    let temporary = dir.join(format!("{name}.tmp"));
    fs::write(&temporary, bytes)?;
    fs::rename(temporary, dir.join(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_account_gets_its_own_folder_with_a_safe_name() {
        let root = Path::new("data");
        assert_eq!(account_dir(root, "1234"), root.join("tweaks").join("1234"));
        assert_eq!(account_dir(root, "../x"), root.join("tweaks").join("x"));
        assert_eq!(account_dir(root, ""), root.join("tweaks").join("default"));
    }

    #[test]
    fn a_saved_file_reads_back_and_a_missing_one_is_empty() {
        let dir = tempfile::tempdir().unwrap();
        let saved = vec!["a".to_owned(), "b".to_owned()];
        save(dir.path(), "list.json", &saved).unwrap();
        assert_eq!(load::<Vec<String>>(dir.path(), "list.json"), saved);
        assert!(load::<Vec<String>>(dir.path(), "missing.json").is_empty());
    }
}
