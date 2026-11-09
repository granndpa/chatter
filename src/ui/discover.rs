//! Create a Server and Discover, as `static_core`'s application keeps them.

use super::*;

/// `1234567` as `1,234,567`.
fn grouped(count: u64) -> String {
    let digits = count.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// Shows Create a Server and Discover as the application has them.
pub(super) fn show(adapter: &Adapter, window: &StaticWindow) {
    let state = &adapter.app.state;
    match &state.create_server {
        Some(dialog) => {
            window.set_create_server_open(true);
            window.set_create_server_busy(dialog.busy);
            window.set_create_server_error(dialog.error.clone().unwrap_or_default().into());
        }
        None => window.set_create_server_open(false),
    }
    let Some(view) = &state.discover else {
        if window.get_discover_open() {
            window.set_discover_open(false);
            window.set_discover_rows(ModelRc::default());
        }
        return;
    };
    window.set_discover_open(true);
    window.set_discover_loading(view.loading);
    window.set_discover_more(view.more);
    window.set_discover_error(view.error.clone().unwrap_or_default().into());
    if window.get_discover_rows().row_count() != view.servers.len() {
        let rows: Vec<DiscoverRow> = view
            .servers
            .iter()
            .map(|server| DiscoverRow {
                id: server.id.as_str().into(),
                name: server.name.as_str().into(),
                description: server.description.as_str().into(),
                picture_url: server.icon.clone().unwrap_or_default().into(),
                initial: initials(&server.name),
                counts: format!(
                    "{} members, {} online",
                    grouped(server.members),
                    grouped(server.online)
                )
                .into(),
                invite: server.invite.clone().unwrap_or_default().into(),
            })
            .collect();
        window.set_discover_rows(ModelRc::new(VecModel::from(rows)));
    }
}

pub(super) fn bind(window: &StaticWindow, adapter: &Rc<RefCell<Adapter>>) {
    macro_rules! act {
        ($callback:ident, $action:expr) => {{
            let (state, weak) = (adapter.clone(), window.as_weak());
            window.$callback(move || {
                if let Some(window) = weak.upgrade() {
                    state.borrow_mut().action(&window, $action);
                }
            });
        }};
    }
    macro_rules! act_with {
        ($callback:ident, $action:path) => {{
            let (state, weak) = (adapter.clone(), window.as_weak());
            window.$callback(move |text| {
                if let Some(window) = weak.upgrade() {
                    state.borrow_mut().action(&window, $action(text.to_string()));
                }
            });
        }};
    }
    act!(on_open_create_server, Action::OpenCreateServer);
    act!(on_close_create_server, Action::CloseCreateServer);
    act!(on_open_discover, Action::OpenDiscover);
    act!(on_discover_load_more, Action::MoreDiscover);
    act!(on_close_discover, Action::CloseDiscover);
    act_with!(on_create_server, Action::CreateServer);
    act_with!(on_discover_search, Action::SearchDiscover);
    act_with!(on_join_discovered, Action::JoinDiscovered);
}

#[cfg(test)]
mod tests {
    #[test]
    fn counts_are_grouped_in_thousands() {
        assert_eq!(super::grouped(7), "7");
        assert_eq!(super::grouped(1234), "1,234");
        assert_eq!(super::grouped(1234567), "1,234,567");
    }
}
