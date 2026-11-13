//! Local account choices contain names and ids only. Each login and cache
//! key stays in its own OS credential-store entry.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

static ACTIVE: OnceLock<String> = OnceLock::new();
const FILE: &str = "discord-accounts.json";

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    pub slot: String,
    pub user_id: String,
    pub name: String,
    pub username: String,
    /// The account's avatar URL, so the switcher shows it without a login.
    #[serde(default)]
    pub avatar: String,
}

#[derive(Default, Serialize, Deserialize)]
struct Accounts {
    #[serde(default = "default_slot")]
    selected: String,
    entries: Vec<Account>,
}

fn default_slot() -> String {
    "default".into()
}
fn valid(slot: &str) -> bool {
    !slot.is_empty()
        && slot.len() <= 80
        && slot
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

/// Called before restoring a login, without reading a credential.
pub fn configure(root: &Path) -> Result<()> {
    let accounts = read(root)?;
    let _ = ACTIVE.set(accounts.selected);
    Ok(())
}

pub fn active_slot() -> &'static str {
    ACTIVE.get().map(String::as_str).unwrap_or("default")
}

fn read(root: &Path) -> Result<Accounts> {
    let bytes = match std::fs::read(root.join(FILE)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Accounts {
                selected: default_slot(),
                entries: vec![],
            });
        }
        Err(error) => return Err(error).context("could not read the saved accounts"),
    };
    let accounts: Accounts =
        serde_json::from_slice(&bytes).context("could not understand the saved accounts")?;
    if !valid(&accounts.selected) || accounts.entries.iter().any(|a| !valid(&a.slot)) {
        bail!("The saved account list has an invalid account id.");
    }
    Ok(accounts)
}

fn write(root: &Path, accounts: &Accounts) -> Result<()> {
    std::fs::create_dir_all(root).context("could not create the accounts folder")?;
    let mut file =
        tempfile::NamedTempFile::new_in(root).context("could not create the account choices")?;
    serde_json::to_writer_pretty(file.as_file_mut(), accounts)?;
    file.as_file()
        .sync_all()
        .context("could not flush the account choices")?;
    file.persist(root.join(FILE))
        .map_err(|_| anyhow::anyhow!("could not save the account choices"))?;
    Ok(())
}

pub fn list(root: &Path) -> Result<Vec<Account>> {
    Ok(read(root)?.entries)
}

/// Updates the current account's public identity without reading its login.
pub fn remember(
    root: &Path,
    user_id: &str,
    name: &str,
    username: &str,
    avatar: &str,
) -> Result<()> {
    if user_id.is_empty() {
        return Ok(());
    }
    let mut accounts = read(root)?;
    let account = Account {
        slot: active_slot().into(),
        user_id: user_id.into(),
        name: name.into(),
        username: username.into(),
        avatar: avatar.into(),
    };
    if let Some(old) = accounts.entries.iter_mut().find(|a| a.slot == account.slot) {
        if *old == account {
            return Ok(());
        }
        *old = account;
    } else {
        accounts.entries.push(account);
    }
    write(root, &accounts)
}

/// Selects a saved namespace for the next process. The running process
/// keeps its namespace until it and its network workers have stopped.
pub fn select(root: &Path, slot: &str) -> Result<()> {
    let mut accounts = read(root)?;
    if !accounts.entries.iter().any(|a| a.slot == slot) {
        bail!("That account is no longer saved.");
    }
    accounts.selected = slot.into();
    write(root, &accounts)
}

/// Forgets a saved account that isn't running, with its login.
pub fn remove(root: &Path, slot: &str) -> Result<()> {
    if slot == active_slot() {
        bail!("Switch to another account before removing this one.");
    }
    let mut accounts = read(root)?;
    let before = accounts.entries.len();
    accounts.entries.retain(|a| a.slot != slot);
    if accounts.entries.len() == before {
        bail!("That account is no longer saved.");
    }
    crate::token::forget_slot(slot)?;
    if accounts.selected == slot {
        accounts.selected = active_slot().into();
    }
    write(root, &accounts)
}

pub fn add(root: &Path) -> Result<()> {
    let mut accounts = read(root)?;
    let slot = format!(
        "account-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
    );
    accounts.entries.push(Account {
        slot: slot.clone(),
        name: "New account".into(),
        ..Account::default()
    });
    accounts.selected = slot;
    write(root, &accounts)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn invalid_namespaces_fail_before_a_credential_can_be_read() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(
            root.path().join(FILE),
            br#"{"selected":"../another-account","entries":[]}"#,
        )
        .unwrap();
        assert!(list(root.path()).is_err());
        assert!(select(root.path(), "default").is_err());
    }
    #[test]
    fn choices_keep_public_identity_and_do_not_change_the_running_scope() {
        let root = tempfile::tempdir().unwrap();
        remember(root.path(), "1", "Alice", "alice", "").unwrap();
        add(root.path()).unwrap();
        assert_eq!(list(root.path()).unwrap().len(), 2);
        assert_ne!(read(root.path()).unwrap().selected, active_slot());
        assert!(select(root.path(), "missing").is_err());
        select(root.path(), "default").unwrap();
        let text = std::fs::read_to_string(root.path().join(FILE)).unwrap();
        assert!(!text.contains("token"));
        assert!(!text.contains("cache_key"));
    }
    #[test]
    fn the_running_account_cannot_be_removed() {
        let root = tempfile::tempdir().unwrap();
        remember(root.path(), "1", "Alice", "alice", "").unwrap();
        assert!(remove(root.path(), active_slot()).is_err());
        assert!(remove(root.path(), "missing").is_err());
        assert_eq!(list(root.path()).unwrap().len(), 1);
    }
}
