//! Server rail moves and the folder name prompt, saved through Discord's
//! own settings rather than a local ordering that other clients cannot see.

use super::*;

fn native(id: &str) -> Option<String> {
    if let Some(folder) = id.strip_prefix("discord-folder:") {
        return Some(format!("folder:{folder}"));
    }
    discord::parse_server(id).map(str::to_owned)
}

pub(super) fn arrange(
    adapter: &mut Adapter,
    window: &ChatterWindow,
    source: &str,
    target: &str,
    mode: i32,
    name: String,
) {
    let Some(source) = native(source) else {
        return;
    };
    let target = if target.is_empty() {
        String::new()
    } else {
        let Some(target) = native(target) else {
            return;
        };
        target
    };
    adapter.action(
        window,
        Action::ArrangeServers {
            source,
            target,
            mode,
            name,
        },
    );
}

pub(super) fn bind(window: &ChatterWindow, adapter: &Rc<RefCell<Adapter>>) {
    let state = adapter.clone();
    let weak = window.as_weak();
    window.on_arrange_servers(move |source, target, mode, name| {
        if let Some(window) = weak.upgrade() {
            arrange(
                &mut state.borrow_mut(),
                &window,
                source.as_str(),
                target.as_str(),
                mode,
                name.to_string(),
            );
        }
    });
}
