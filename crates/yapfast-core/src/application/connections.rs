//! Connected Apps: confirmed settings, a removal prompt, and a browser login
//! held only until its answer or cancellation. Only one request runs at once.

use super::*;
use crate::chat::{ConnectedAccount, ConnectionAnswer, ConnectionOperation};
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct ConnectionsPage {
    pub accounts: Vec<ConnectedAccount>,
    pub loaded: bool,
    pub busy: bool,
    pub error: Option<String>,
    pub removing: Option<(String, String)>,
    pub authorizing: Option<String>,
    request: u64,
    pending: Option<Pending>,
}

struct Pending {
    state: String,
    started: Instant,
}

impl ConnectionsPage {
    pub(super) fn clear(&mut self) {
        *self = Self {
            request: self.request.wrapping_add(1),
            ..Self::default()
        };
    }
}

pub enum ConnectionInput {
    Refresh,
    Profile {
        kind: String,
        id: String,
        shown: bool,
    },
    Activity {
        kind: String,
        id: String,
        shown: bool,
    },
    AskRemove {
        kind: String,
        id: String,
    },
    Remove,
    CancelRemove,
    Authorize(String),
    Complete(String),
    CancelAuthorization,
}

impl Application {
    pub(super) fn connection_input(&mut self, input: ConnectionInput) -> Vec<Effect> {
        let page = &mut self.state.connections;
        if matches!(&input, ConnectionInput::CancelAuthorization) {
            if page.authorizing.is_some() && !page.busy {
                page.authorizing = None;
                page.pending = None;
                page.error = None;
            }
            return vec![];
        }
        if matches!(&input, ConnectionInput::CancelRemove) {
            if !page.busy {
                page.removing = None;
            }
            return vec![];
        }
        if self.state.preview
            || !self.state.discord_online
            || !self.state.discord_linked
            || page.busy
        {
            return vec![];
        }
        let operation = match input {
            ConnectionInput::Refresh => ConnectionOperation::List,
            ConnectionInput::Profile { kind, id, shown } => {
                if !page
                    .accounts
                    .iter()
                    .any(|a| a.kind == kind && a.id == id && a.visible != shown)
                {
                    return vec![];
                }
                ConnectionOperation::Profile { kind, id, shown }
            }
            ConnectionInput::Activity { kind, id, shown } => {
                if kind != "spotify"
                    || !page
                        .accounts
                        .iter()
                        .any(|a| a.kind == kind && a.id == id && a.activity != shown)
                {
                    return vec![];
                }
                ConnectionOperation::Activity { kind, id, shown }
            }
            ConnectionInput::AskRemove { kind, id } => {
                if page.accounts.iter().any(|a| a.kind == kind && a.id == id) {
                    page.removing = Some((kind, id));
                }
                return vec![];
            }
            ConnectionInput::Remove => {
                let Some((kind, id)) = page.removing.clone() else {
                    return vec![];
                };
                ConnectionOperation::Remove { kind, id }
            }
            ConnectionInput::Authorize(kind) => {
                if !matches!(
                    kind.as_str(),
                    "spotify"
                        | "github"
                        | "steam"
                        | "twitch"
                        | "youtube"
                        | "reddit"
                        | "xbox"
                        | "playstation"
                        | "battlenet"
                        | "epicgames"
                        | "riotgames"
                        | "crunchyroll"
                        | "paypal"
                        | "ebay"
                        | "facebook"
                        | "twitter"
                        | "roblox"
                ) {
                    return vec![];
                }
                page.pending = None;
                page.authorizing = Some(kind.clone());
                ConnectionOperation::Authorize(kind)
            }
            ConnectionInput::Complete(address) => {
                let (Some(kind), Some(pending)) = (&page.authorizing, &page.pending) else {
                    return vec![];
                };
                if pending.started.elapsed() > Duration::from_secs(600) {
                    page.error =
                        Some("This login expired. Cancel it and start a new connection.".into());
                    return vec![];
                }
                #[cfg(feature = "discord")]
                if let Err(error) =
                    yapfast_discord::connections::callback(kind, &pending.state, &address)
                {
                    page.error = Some(error.to_string());
                    return vec![];
                }
                ConnectionOperation::Complete {
                    kind: kind.clone(),
                    state: pending.state.clone(),
                    address,
                }
            }
            ConnectionInput::CancelAuthorization | ConnectionInput::CancelRemove => return vec![],
        };
        page.error = None;
        page.busy = true;
        page.request = page.request.wrapping_add(1);
        vec![Effect::Backend(Command::Discord(
            DiscordCommand::Connections {
                request: page.request,
                operation,
            },
        ))]
    }

    pub(super) fn connections_answer(
        &mut self,
        request: u64,
        result: Result<ConnectionAnswer, String>,
    ) -> Vec<Effect> {
        let page = &mut self.state.connections;
        if request != page.request || !page.busy {
            return vec![];
        }
        page.busy = false;
        match result {
            Ok(ConnectionAnswer::Listed {
                accounts,
                completed,
            }) => {
                page.accounts = accounts;
                page.loaded = true;
                page.removing = None;
                if completed {
                    page.pending = None;
                    page.authorizing = None;
                }
            }
            Ok(ConnectionAnswer::Authorized { kind, url, state }) => {
                if page.authorizing.as_ref() != Some(&kind) {
                    return vec![];
                }
                page.pending = Some(Pending {
                    state,
                    started: Instant::now(),
                });
                return vec![Effect::OpenPage(url)];
            }
            Err(error) => {
                if page.pending.is_none() {
                    page.authorizing = None;
                }
                page.error = Some(error);
            }
        }
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Application {
        let mut app = Application::default();
        app.state.discord_linked = true;
        app.state.discord_online = true;
        app.state.connections.accounts.push(ConnectedAccount {
            kind: "spotify".into(),
            id: "1".into(),
            visible: true,
            activity: true,
            ..ConnectedAccount::default()
        });
        app
    }

    #[test]
    fn waits_for_a_confirmed_change_and_ignores_old_answers() {
        let mut app = fixture();
        let input = || ConnectionInput::Activity {
            kind: "spotify".into(),
            id: "1".into(),
            shown: false,
        };
        assert_eq!(app.connection_input(input()).len(), 1);
        assert!(app.state.connections.accounts[0].activity);
        assert!(app.connection_input(input()).is_empty());
        let request = app.state.connections.request;
        app.connections_answer(request, Err("refused".into()));
        assert!(!app.state.connections.busy);
        assert!(app.state.connections.accounts[0].activity);
        app.connection_input(ConnectionInput::Refresh);
        app.connections_answer(
            request,
            Ok(ConnectionAnswer::Listed {
                accounts: vec![],
                completed: false,
            }),
        );
        assert!(app.state.connections.busy);
        app.state.connections.clear();
        app.connections_answer(request + 1, Err("old account".into()));
        assert!(app.state.connections.error.is_none());
    }

    #[test]
    fn login_is_cancelled_or_expires_without_sending_a_callback() {
        let mut app = fixture();
        app.connection_input(ConnectionInput::Authorize("github".into()));
        let request = app.state.connections.request;
        let effects = app.connections_answer(
            request,
            Ok(ConnectionAnswer::Authorized {
                kind: "github".into(),
                url: "https://github.com/login".into(),
                state: "ours".into(),
            }),
        );
        assert!(matches!(effects.as_slice(), [Effect::OpenPage(_)]));
        app.state.connections.pending.as_mut().unwrap().started =
            Instant::now() - Duration::from_secs(601);
        assert!(
            app.connection_input(ConnectionInput::Complete(
                "https://discord.com/connections/github?state=ours&code=code".into()
            ))
            .is_empty()
        );
        assert!(
            app.state
                .connections
                .error
                .as_deref()
                .unwrap()
                .contains("expired")
        );
        app.connection_input(ConnectionInput::CancelAuthorization);
        assert!(app.state.connections.pending.is_none());
        assert!(app.state.connections.authorizing.is_none());
    }

    #[test]
    fn removal_needs_confirmation_and_preview_never_sends() {
        let mut app = fixture();
        assert!(app.connection_input(ConnectionInput::Remove).is_empty());
        assert!(
            app.connection_input(ConnectionInput::AskRemove {
                kind: "spotify".into(),
                id: "1".into()
            })
            .is_empty()
        );
        assert!(app.state.connections.removing.is_some());
        assert_eq!(app.connection_input(ConnectionInput::Remove).len(), 1);
        let mut preview = fixture();
        preview.state.preview = true;
        assert!(
            preview
                .connection_input(ConnectionInput::Refresh)
                .is_empty()
        );
    }
}
