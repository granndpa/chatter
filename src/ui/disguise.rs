//! Discord Tweaks' "Show as Discord in Windows search": Chatter's Start menu
//! shortcut is named Discord, keeping Chatter's icon, and Discord's own
//! shortcuts are moved into Chatter's data folder, with a note of where they
//! were, so turning it off puts them back. Discord's updater may recreate
//! its shortcuts; they're moved again each time Chatter starts.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use super::*;

/// Chatter's shortcut, by its own name and disguised. The disguised one sits
/// in a folder of its own, so it's never mistaken for Discord's.
const OWN: &str = "Chatter.lnk";
const DISGUISED: &str = "Chatter/Discord.lnk";
/// Discord's own shortcuts, where its installer puts them.
const DISCORD: [&str; 2] = ["Discord.lnk", "Discord Inc/Discord.lnk"];
/// Where Discord's shortcuts are kept in Chatter's data folder, and the note
/// of where each came from.
const KEPT: &str = "hidden-shortcuts";
const NOTE: &str = "hidden-shortcuts/README.txt";

fn programs() -> Option<PathBuf> {
    let appdata = std::env::var_os("APPDATA")?;
    Some(Path::new(&appdata).join("Microsoft/Windows/Start Menu/Programs"))
}

fn moved(from: &Path, to: &Path) -> io::Result<()> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)?;
    }
    if to.exists() {
        fs::remove_file(to)?;
    }
    fs::rename(from, to)
}

/// Names Chatter's shortcut Discord and keeps Discord's own in `data`.
pub(super) fn disguise(programs: &Path, data: &Path) -> io::Result<()> {
    let mut note = String::from(
        "Discord's Start menu shortcuts, hidden by Chatter's \"Show as Discord in Windows search\".\n\
         Turning the switch off puts them back. Each line: where it was.\n",
    );
    for each in DISCORD {
        let original = programs.join(each);
        if original.exists() {
            moved(&original, &data.join(KEPT).join(each))?;
        }
        if data.join(KEPT).join(each).exists() {
            note.push_str(&format!("{}\n", original.display()));
        }
    }
    let _ = fs::remove_dir(programs.join("Discord Inc"));
    if data.join(KEPT).exists() {
        fs::write(data.join(NOTE), note)?;
    }
    let own = programs.join(OWN);
    if own.exists() {
        moved(&own, &programs.join(DISGUISED))?;
    }
    Ok(())
}

/// Names Chatter's shortcut Chatter again and puts Discord's back.
pub(super) fn undisguise(programs: &Path, data: &Path) -> io::Result<()> {
    let disguised = programs.join(DISGUISED);
    if disguised.exists() {
        moved(&disguised, &programs.join(OWN))?;
        let _ = fs::remove_dir(programs.join("Chatter"));
    }
    for each in DISCORD {
        let kept = data.join(KEPT).join(each);
        if kept.exists() && !programs.join(each).exists() {
            moved(&kept, &programs.join(each))?;
        }
    }
    let _ = fs::remove_file(data.join(NOTE));
    let _ = fs::remove_dir(data.join(KEPT).join("Discord Inc"));
    let _ = fs::remove_dir(data.join(KEPT));
    Ok(())
}

fn apply(on: bool, data: &Path) {
    if !cfg!(windows) {
        return;
    }
    let Some(programs) = programs() else { return };
    let done = if on { disguise(&programs, data) } else { undisguise(&programs, data) };
    if let Err(error) = done {
        tracing::warn!(%error, on, "could not change the Start menu shortcuts");
    }
}

pub(super) fn bind(window: &ChatterWindow, adapter: &Rc<RefCell<Adapter>>) {
    {
        let adapter = adapter.borrow();
        window.set_show_as_discord(adapter.layout.show_as_discord);
        if adapter.layout.show_as_discord {
            apply(true, &adapter.root);
        }
    }
    let state = adapter.clone();
    window.on_show_as_discord_toggled(move |on| {
        let mut adapter = state.borrow_mut();
        adapter.layout.show_as_discord = on;
        adapter.layout.write(&adapter.root);
        apply(on, &adapter.root);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discords_shortcuts_are_kept_and_put_back() {
        let programs = tempfile::tempdir().unwrap();
        let data = tempfile::tempdir().unwrap();
        let (p, d) = (programs.path(), data.path());
        fs::create_dir_all(p.join("Discord Inc")).unwrap();
        fs::write(p.join("Chatter.lnk"), "chatter").unwrap();
        fs::write(p.join("Discord.lnk"), "discord").unwrap();
        fs::write(p.join("Discord Inc/Discord.lnk"), "discord inc").unwrap();
        disguise(p, d).unwrap();
        assert!(!p.join("Discord.lnk").exists() && !p.join("Chatter.lnk").exists());
        assert_eq!(fs::read_to_string(p.join("Chatter/Discord.lnk")).unwrap(), "chatter");
        assert!(fs::read_to_string(d.join(NOTE)).unwrap().contains("Discord.lnk"));
        // Discord's updater made its shortcut again: it's hidden again.
        fs::write(p.join("Discord.lnk"), "discord new").unwrap();
        disguise(p, d).unwrap();
        assert!(!p.join("Discord.lnk").exists());
        undisguise(p, d).unwrap();
        assert_eq!(fs::read_to_string(p.join("Chatter.lnk")).unwrap(), "chatter");
        assert_eq!(fs::read_to_string(p.join("Discord.lnk")).unwrap(), "discord new");
        assert_eq!(fs::read_to_string(p.join("Discord Inc/Discord.lnk")).unwrap(), "discord inc");
        assert!(!p.join("Chatter").exists() && !d.join(KEPT).exists());
    }
}
