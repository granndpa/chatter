//! Discord Tweaks' chat tabs, off unless switched on: each conversation
//! opened gets a tab above the conversation, kept until it's closed, and
//! remembered across restarts. Drafts already stay with each conversation.

use super::*;

/// A tab's name: `#general` for a channel, a DM's own name.
fn tab_name(state: &AppState, room: &str) -> Option<String> {
    let found = state.rooms.iter().find(|found| found.id == room)?;
    Some(if found.direct {
        found.name.clone()
    } else {
        format!("#{}", found.name)
    })
}

/// Adds the conversation open now as a tab, and shows the tabs.
pub(super) fn show(adapter: &mut Adapter, window: &ChatterWindow) {
    let enabled = adapter.layout.chat_tabs;
    window.set_chat_tabs_enabled(enabled);
    if !enabled {
        return;
    }
    let state = &adapter.app.state;
    let selected = state.selected.clone();
    if let Some(room) = &selected
        && !adapter.layout.open_tabs.contains(room)
        && tab_name(state, room).is_some()
    {
        adapter.layout.open_tabs.push(room.clone());
        adapter.layout.write(&adapter.root);
    }
    let state = &adapter.app.state;
    let tabs: Vec<ChatTab> = adapter
        .layout
        .open_tabs
        .iter()
        .filter_map(|room| {
            let name = tab_name(state, room)?;
            let unread = state
                .rooms
                .iter()
                .any(|found| &found.id == room && (found.unread > 0 || found.mentions > 0));
            Some(ChatTab {
                id: room.as_str().into(),
                name: name.into(),
                current: selected.as_ref() == Some(room),
                unread,
            })
        })
        .collect();
    let model = window.get_chat_tabs();
    let same = model.row_count() == tabs.len()
        && tabs
            .iter()
            .enumerate()
            .all(|(index, tab)| model.row_data(index).as_ref() == Some(tab));
    if !same {
        window.set_chat_tabs(ModelRc::new(VecModel::from(tabs)));
    }
}

pub(super) fn bind(window: &ChatterWindow, adapter: &Rc<RefCell<Adapter>>) {
    {
        let (state, weak) = (adapter.clone(), window.as_weak());
        window.on_chat_tab_open(move |room| {
            if let Some(window) = weak.upgrade() {
                state.borrow_mut().action(&window, Action::Open(room.into()));
            }
        });
    }
    {
        let (state, weak) = (adapter.clone(), window.as_weak());
        window.on_chat_tab_close(move |room| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            let tabs = &mut adapter.layout.open_tabs;
            let Some(at) = tabs.iter().position(|open| open.as_str() == room.as_str()) else {
                return;
            };
            tabs.remove(at);
            // Closing the tab in front moves to its neighbour, as a browser does.
            let next = (adapter.app.state.selected.as_deref() == Some(room.as_str()))
                .then(|| {
                    let tabs = &adapter.layout.open_tabs;
                    tabs.get(at).or_else(|| tabs.get(at.wrapping_sub(1))).cloned()
                })
                .flatten();
            adapter.layout.write(&adapter.root);
            match next {
                Some(next) => adapter.action(&window, Action::Open(next)),
                None => show(&mut adapter, &window),
            }
        });
    }
    {
        let (state, weak) = (adapter.clone(), window.as_weak());
        window.on_chat_tab_shift(move |room, by| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            let tabs = &mut adapter.layout.open_tabs;
            if let Some(from) = tabs.iter().position(|open| open.as_str() == room.as_str()) {
                let to = from as i64 + i64::from(by);
                if (0..tabs.len() as i64).contains(&to) {
                    tabs.swap(from, to as usize);
                    adapter.layout.write(&adapter.root);
                }
            }
            show(&mut adapter, &window);
        });
    }
    {
        let (state, weak) = (adapter.clone(), window.as_weak());
        window.on_chat_tabs_toggled(move |on| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            adapter.layout.chat_tabs = on;
            if !on {
                adapter.layout.open_tabs.clear();
            }
            adapter.layout.write(&adapter.root);
            show(&mut adapter, &window);
        });
    }
}
