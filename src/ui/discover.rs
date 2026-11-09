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
            if !window.get_create_server_open() {
                let names: Vec<SharedString> = static_core::application::SERVER_TEMPLATES
                    .iter()
                    .map(|(name, _, _)| SharedString::from(*name))
                    .collect();
                window.set_create_server_templates(ModelRc::new(VecModel::from(names)));
            }
            // The picture loads once per file chosen, not every refresh.
            thread_local! {
                static SHOWN: RefCell<Option<std::path::PathBuf>> = const { RefCell::new(None) };
            }
            if SHOWN.with_borrow(|shown| shown.as_ref() != dialog.icon.as_ref()) {
                SHOWN.set(dialog.icon.clone());
                window.set_create_server_icon(
                    dialog
                        .icon
                        .as_deref()
                        .and_then(|path| slint::Image::load_from_path(path).ok())
                        .unwrap_or_default(),
                );
            }
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
    {
        let (state, weak) = (adapter.clone(), window.as_weak());
        window.on_create_server(move |name, template| {
            if let Some(window) = weak.upgrade() {
                state.borrow_mut().action(
                    &window,
                    Action::CreateServer {
                        name: name.to_string(),
                        template: template.max(0) as usize,
                    },
                );
            }
        });
    }
    act!(on_clear_server_icon, Action::SetServerIcon(None));
    {
        let (state, weak) = (Rc::downgrade(adapter), window.as_weak());
        window.on_pick_server_icon(move || {
            let (state, weak) = (state.clone(), weak.clone());
            let _ = slint::spawn_local(async move {
                let picked = rfd::AsyncFileDialog::new()
                    .set_title("Choose a server icon")
                    .add_filter("Pictures", &["png", "jpg", "jpeg", "gif", "webp"])
                    .pick_file()
                    .await;
                if let (Some(file), Some(adapter), Some(window)) = (picked, state.upgrade(), weak.upgrade()) {
                    adapter
                        .borrow_mut()
                        .action(&window, Action::SetServerIcon(Some(file.path().to_owned())));
                }
            });
        });
    }
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
