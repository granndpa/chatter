//! Connected Apps' rows and callbacks; the application holds each request
//! until Discord answers, so refreshes never overwrite a pending switch.

use super::*;
use chatter_core::application::ConnectionInput;

pub(super) fn bind(window: &ChatterWindow, adapter: &Rc<RefCell<Adapter>>) {
    let state = adapter.clone();
    let weak = window.as_weak();
    window.on_connections_action(move |what, kind, id, shown| {
        let (kind, id) = (kind.to_string(), id.to_string());
        let input = match what.as_str() {
            "refresh" => ConnectionInput::Refresh,
            "profile" => ConnectionInput::Profile { kind, id, shown },
            "activity" => ConnectionInput::Activity { kind, id, shown },
            "ask-remove" => ConnectionInput::AskRemove { kind, id },
            "remove" => ConnectionInput::Remove,
            "cancel-remove" => ConnectionInput::CancelRemove,
            "authorize" => ConnectionInput::Authorize(kind),
            "complete" => ConnectionInput::Complete(id),
            "cancel-authorization" => ConnectionInput::CancelAuthorization,
            _ => return,
        };
        if let Some(window) = weak.upgrade() {
            state
                .borrow_mut()
                .action(&window, Action::Connections(input));
        }
    });
}

impl Adapter {
    pub(super) fn show_connections(&mut self, window: &ChatterWindow) {
        let page = &self.app.state.connections;
        window.set_connections_online(
            self.app.state.discord_linked
                && self.app.state.discord_online
                && !self.app.state.preview,
        );
        window.set_connections_busy(page.busy);
        window.set_connections_loaded(page.loaded);
        window.set_connections_error(page.error.as_deref().unwrap_or_default().into());
        window.set_connections_authorizing(
            page.authorizing
                .as_deref()
                .map(provider)
                .unwrap_or_default()
                .into(),
        );
        let (kind, id) = page
            .removing
            .as_ref()
            .map(|(kind, id)| (kind.as_str(), id.as_str()))
            .unwrap_or_default();
        window.set_connections_removing_kind(kind.into());
        window.set_connections_removing_id(id.into());
        let rows = page
            .accounts
            .iter()
            .map(|account| ConnectionRow {
                kind: account.kind.as_str().into(),
                id: account.id.as_str().into(),
                name: account.name.as_str().into(),
                provider: provider(&account.kind).into(),
                visible: account.visible,
                activity: account.activity,
                revoked: account.revoked,
            })
            .collect::<Vec<_>>();
        let rows = same_list(
            &window.get_connections(),
            ModelRc::new(VecModel::from(rows)),
        );
        window.set_connections(rows);
    }
}

fn provider(kind: &str) -> &str {
    match kind {
        "spotify" => "Spotify",
        "github" => "GitHub",
        "steam" => "Steam",
        "twitch" => "Twitch",
        "youtube" => "YouTube",
        "reddit" => "Reddit",
        "xbox" => "Xbox",
        "playstation" => "PlayStation",
        "battlenet" => "Battle.net",
        "epicgames" => "Epic Games",
        "riotgames" => "Riot Games",
        "facebook" => "Facebook",
        "twitter" => "X",
        "domain" => "Domain",
        "crunchyroll" => "Crunchyroll",
        "paypal" => "PayPal",
        "ebay" => "eBay",
        "roblox" => "Roblox",
        _ => kind,
    }
}
