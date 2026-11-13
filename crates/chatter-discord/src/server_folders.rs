//! Edits the account's ordered server groups, preserving every server and
//! the names and colours of folders that aren't being changed.

use crate::settings::{Folder, ServerList};
use anyhow::{Result, bail};

pub fn entries(list: &ServerList) -> Vec<Folder> {
    let mut entries = Vec::new();
    for id in &list.order {
        if entries
            .iter()
            .any(|entry: &Folder| entry.servers.contains(id))
        {
            continue;
        }
        entries.push(
            list.folders
                .iter()
                .find(|f| f.servers.contains(id))
                .cloned()
                .unwrap_or_else(|| Folder {
                    servers: vec![id.clone()],
                    ..Folder::default()
                }),
        );
    }
    entries
}

fn locate(entries: &[Folder], key: &str) -> Option<(usize, Option<usize>)> {
    if let Some(id) = key.strip_prefix("folder:") {
        return entries.iter().position(|f| f.id == id).map(|at| (at, None));
    }
    entries.iter().enumerate().find_map(|(at, f)| {
        f.servers
            .iter()
            .position(|s| s == key)
            .map(|index| (at, Some(index)))
    })
}

fn folder_id(entries: &[Folder], seed: u64) -> String {
    let mut id = (seed as u32).max(1);
    while entries.iter().any(|f| f.id == id.to_string()) {
        id = id.wrapping_add(1).max(1);
    }
    id.to_string()
}

/// 0 before, 1 after, 2 combine, 3 create, 4 rename, 5 ungroup.
pub fn edit(
    entries: &[Folder],
    source: &str,
    target: &str,
    mode: i32,
    name: &str,
    seed: u64,
) -> Result<Vec<Folder>> {
    let mut changed = entries.to_vec();
    // Creating or editing a folder may carry its colour after the name, as
    // `name\u{1f}rrggbb`; an empty colour is Discord's default.
    let (name, color) = match name.split_once('\u{1f}') {
        Some((name, color)) => (name, Some(u32::from_str_radix(color.trim_start_matches('#'), 16).ok())),
        None => (name, None),
    };
    let Some((from, inside)) = locate(&changed, source) else {
        bail!("That server or folder is no longer in the list.");
    };
    if mode == 4 {
        if inside.is_some() {
            bail!("Choose a folder to rename.");
        }
        changed[from].name = Some(folder_name(name)?);
        if let Some(color) = color {
            changed[from].color = color;
        }
    } else if mode == 5 {
        if inside.is_some() {
            bail!("Choose a folder to ungroup.");
        }
        let folder = changed.remove(from);
        for (offset, server) in folder.servers.into_iter().enumerate() {
            changed.insert(
                from + offset,
                Folder {
                    servers: vec![server],
                    ..Folder::default()
                },
            );
        }
    } else if mode == 3 {
        let Some(index) = inside else {
            bail!("Choose a server for the new folder.");
        };
        let server = changed[from].servers.remove(index);
        let folder = Folder {
            id: folder_id(&changed, seed),
            name: Some(folder_name(name)?),
            color: color.flatten(),
            servers: vec![server],
        };
        changed.insert(from + 1, folder);
    } else {
        if !(0..=2).contains(&mode) || source == target {
            bail!("Choose another place in the server list.");
        }
        let Some((onto, _)) = locate(&changed, target) else {
            bail!("That destination is no longer in the list.");
        };
        if inside.is_none() && onto == from {
            bail!("A folder cannot be moved into itself.");
        }
        if mode == 2 && inside.is_none() {
            bail!("Folders cannot be nested. Drop above or below a folder to move it.");
        }
        let moving = match inside {
            Some(index) => Folder {
                servers: vec![changed[from].servers.remove(index)],
                ..Folder::default()
            },
            None => changed.remove(from),
        };
        let Some((onto, target_inside)) = locate(&changed, target) else {
            bail!("That destination is no longer in the list.");
        };
        if mode == 2 {
            if changed[onto].id.is_empty() {
                changed[onto].id = folder_id(&changed, seed);
            }
            changed[onto].servers.extend(moving.servers);
        } else if moving.id.is_empty() && target_inside.is_some() && !changed[onto].id.is_empty() {
            let index = target_inside.unwrap() + usize::from(mode == 1);
            changed[onto].servers.splice(index..index, moving.servers);
        } else {
            changed.insert(onto + usize::from(mode == 1), moving);
        }
    }
    changed.retain(|f| !f.servers.is_empty());
    // A failed edit never loses or duplicates an account's servers.
    let ids = |entries: &[Folder]| {
        let mut ids = entries
            .iter()
            .flat_map(|f| f.servers.clone())
            .collect::<Vec<_>>();
        ids.sort();
        ids
    };
    if ids(&changed) != ids(entries) {
        bail!("The server list changed. Refresh it and try again.");
    }
    Ok(changed)
}

fn folder_name(name: &str) -> Result<String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 100 || name.chars().any(char::is_control) {
        bail!("Use a folder name of 1 to 100 characters.");
    }
    Ok(name.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn single(id: &str) -> Folder {
        Folder {
            servers: vec![id.into()],
            ..Folder::default()
        }
    }
    #[test]
    fn edits_a_folders_name_and_colour() {
        let mut folder = single("1");
        folder.id = "5".into();
        let edited = edit(&[folder], "folder:5", "", 4, "Games\u{1f}#5865f2", 1).unwrap();
        assert_eq!(edited[0].name.as_deref(), Some("Games"));
        assert_eq!(edited[0].color, Some(0x5865f2));
        let reset = edit(&edited, "folder:5", "", 4, "Games\u{1f}", 1).unwrap();
        assert_eq!(reset[0].color, None);
    }

    #[test]
    fn moves_servers_into_out_of_and_between_folders() {
        let initial = vec![single("1"), single("2"), single("3")];
        let grouped = edit(&initial, "1", "2", 2, "", 7).unwrap();
        assert_eq!(grouped[0].id, "7");
        assert_eq!(grouped[0].servers, ["2", "1"]);
        let moved = edit(&grouped, "1", "3", 1, "", 9).unwrap();
        assert_eq!(
            moved
                .iter()
                .flat_map(|f| f.servers.clone())
                .collect::<Vec<_>>(),
            ["2", "3", "1"]
        );
        let moved = edit(&grouped, "folder:7", "3", 1, "", 9).unwrap();
        assert_eq!(moved[1].id, "7");
        assert!(edit(&grouped, "folder:7", "1", 2, "", 9).is_err());
    }
    #[test]
    fn creates_renames_and_ungroups_without_losing_a_server() {
        let initial = vec![single("1"), single("2")];
        let grouped = edit(&initial, "1", "", 3, "Games", 7).unwrap();
        let renamed = edit(&grouped, "folder:7", "", 4, "Friends", 8).unwrap();
        assert_eq!(renamed[0].name.as_deref(), Some("Friends"));
        assert_eq!(edit(&renamed, "folder:7", "", 5, "", 8).unwrap(), initial);
    }
}
