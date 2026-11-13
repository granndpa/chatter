//! Making a Discord server of one's own, and Discover: the public servers
//! Discord lists, searched and paged a few at a time. Joining one goes
//! through Join a Server with its public invite.

use super::*;
use crate::chat::DiscoverServer;

/// How many servers Discover asks Discord for at a time.
pub const DISCOVER_PAGE: usize = 24;

/// Create a Server, while it's open.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CreateServerDialog {
    /// Discord is making it.
    pub busy: bool,
    /// The picture chosen for its icon.
    pub icon: Option<std::path::PathBuf>,
    pub error: Option<String>,
}

/// Discover, while it's open.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DiscoverView {
    /// What was searched for; empty for Discord's own picks.
    pub query: String,
    pub servers: Vec<DiscoverServer>,
    pub loading: bool,
    /// Discord may have more after these.
    pub more: bool,
    pub error: Option<String>,
    /// The server being joined, by id.
    pub joining: Option<String>,
}

impl Application {
    pub(super) fn open_create_server(&mut self) -> Vec<Effect> {
        if self.state.discord_online {
            self.state.join = None;
            self.state.create_server = Some(CreateServerDialog::default());
        }
        vec![]
    }

    pub(super) fn set_server_icon(&mut self, icon: Option<std::path::PathBuf>) -> Vec<Effect> {
        if let Some(dialog) = &mut self.state.create_server {
            dialog.icon = icon;
        }
        vec![]
    }

    pub(super) fn create_server(&mut self, name: String) -> Vec<Effect> {
        let Some(dialog) = self.state.create_server.as_mut().filter(|dialog| !dialog.busy) else {
            return vec![];
        };
        let name = name.trim().to_owned();
        if name.is_empty() || name.chars().count() > 100 {
            dialog.error = Some("Give it a name, up to 100 characters.".into());
            return vec![];
        }
        dialog.busy = true;
        dialog.error = None;
        let icon = dialog.icon.clone();
        vec![Effect::Backend(Command::Discord(DiscordCommand::CreateServer { name, icon }))]
    }

    /// The server made, which Discord lists on the rail as it arrives.
    pub(super) fn server_created(&mut self, result: Result<String, String>) -> Vec<Effect> {
        match result {
            Ok(_) => self.state.create_server = None,
            Err(error) => {
                if let Some(dialog) = &mut self.state.create_server {
                    dialog.busy = false;
                    dialog.error = Some(error);
                }
            }
        }
        vec![]
    }

    pub(super) fn open_discover(&mut self) -> Vec<Effect> {
        if !self.state.discord_online {
            return vec![];
        }
        self.state.join = None;
        self.state.discover = Some(DiscoverView {
            loading: true,
            ..DiscoverView::default()
        });
        discover_page(String::new(), 0)
    }

    pub(super) fn search_discover(&mut self, query: String) -> Vec<Effect> {
        let Some(view) = &mut self.state.discover else {
            return vec![];
        };
        let query = query.trim().to_owned();
        *view = DiscoverView {
            query: query.clone(),
            loading: true,
            ..DiscoverView::default()
        };
        discover_page(query, 0)
    }

    pub(super) fn more_discover(&mut self) -> Vec<Effect> {
        let Some(view) = self
            .state
            .discover
            .as_mut()
            .filter(|view| view.more && !view.loading)
        else {
            return vec![];
        };
        view.loading = true;
        discover_page(view.query.clone(), view.servers.len() as u32)
    }

    /// A page Discord answered, unless the search moved on since.
    pub(super) fn discovered(
        &mut self,
        query: String,
        offset: u32,
        result: Result<Vec<DiscoverServer>, String>,
    ) -> Vec<Effect> {
        let Some(view) = &mut self.state.discover else {
            return vec![];
        };
        if view.query != query || view.servers.len() as u32 != offset {
            return vec![];
        }
        view.loading = false;
        match result {
            Ok(servers) => {
                view.more = servers.len() >= DISCOVER_PAGE;
                view.servers.extend(servers);
            }
            Err(error) => view.error = Some(error),
        }
        vec![]
    }

    /// Joins a server Discover listed, by its id, as Discord's own Join does.
    pub(super) fn join_discovered(&mut self, guild: String) -> Vec<Effect> {
        let Some(view) = self.state.discover.as_mut().filter(|view| view.joining.is_none()) else {
            return vec![];
        };
        view.joining = Some(guild.clone());
        view.error = None;
        vec![Effect::Backend(Command::Discord(DiscordCommand::JoinDiscoverable { guild }))]
    }

    /// Joined: Discover closes, and the server arrives on the rail.
    pub(super) fn discover_joined(&mut self, result: Result<(), String>) -> Vec<Effect> {
        match result {
            Ok(()) => self.state.discover = None,
            Err(error) => {
                if let Some(view) = &mut self.state.discover {
                    view.joining = None;
                    view.error = Some(error);
                }
            }
        }
        vec![]
    }
}

fn discover_page(query: String, offset: u32) -> Vec<Effect> {
    vec![Effect::Backend(Command::Discord(DiscordCommand::Discover { query, offset }))]
}
