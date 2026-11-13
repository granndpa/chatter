//! Discord Tweaks' Media Gallery: a channel's or a server's pictures,
//! videos, files and links, found with Discord's own search (`has:image`
//! and the like), a page at a time, so nothing loads the whole history.

use super::*;

/// What each tab searches for, as Discord's `has:` names it.
const KINDS: [&str; 4] = ["image", "video", "file", "link"];

#[derive(Default)]
pub(super) struct Gallery {
    open: bool,
    /// Images, Videos, Files or Links, by index into [`KINDS`].
    kind: usize,
    /// The channel it's about, by room id; none for the whole server.
    channel: Option<String>,
    oldest_first: bool,
    /// Only items whose name or sender has this in it.
    filter: String,
}

/// Searches for the tab's kind in the place the gallery is about.
fn search(adapter: &mut Adapter, window: &ChatterWindow) {
    let has = KINDS[adapter.gallery.kind];
    let channel = adapter.gallery.channel.as_ref().and_then(|room| {
        adapter
            .app
            .state
            .rooms
            .iter()
            .find(|found| &found.id == room)
            .map(|found| found.name.clone())
    });
    let query = match channel {
        Some(name) if name.contains(' ') => format!("has:{has} in:\"#{name}\""),
        Some(name) => format!("has:{has} in:#{name}"),
        None => format!("has:{has}"),
    };
    adapter.action(window, Action::Search(query));
}

/// Opens the gallery for a server, by its id, or a channel or DM, by its
/// room id.
pub(super) fn open(adapter: &mut Adapter, window: &ChatterWindow, place: &str) {
    let server = adapter.app.state.servers.iter().any(|server| server.id == place);
    if server {
        adapter.action(window, Action::Navigate(Navigation::Server(place.to_owned())));
        adapter.gallery.channel = None;
    } else {
        adapter.action(window, Action::Open(place.to_owned()));
        let direct = adapter
            .app
            .state
            .rooms
            .iter()
            .any(|room| room.id == place && room.direct);
        // A DM's search is the DM already.
        adapter.gallery.channel = (!direct).then(|| place.to_owned());
    }
    adapter.gallery.open = true;
    adapter.gallery.kind = 0;
    adapter.gallery.filter.clear();
    window.set_gallery_open(true);
    search(adapter, window);
    show(adapter, window);
}

/// Whether a place's menu offers the gallery: servers, text channels, DMs.
pub(super) fn offered(state: &AppState, place: &str) -> bool {
    state.servers.iter().any(|server| server.id == place && server.discord)
        || state
            .rooms
            .iter()
            .any(|room| room.id == place && !room.voice && room.id.starts_with("discord"))
}

/// The links in a message's text.
fn links(body: &str) -> impl Iterator<Item = &str> {
    body.split(|c: char| c.is_whitespace() || c == '<' || c == '>' || c == '(' || c == ')')
        .filter(|word| word.starts_with("https://") || word.starts_with("http://"))
}

/// Fills the gallery from the search's results.
pub(super) fn show(adapter: &Adapter, window: &ChatterWindow) {
    if !adapter.gallery.open {
        return;
    }
    let gallery = &adapter.gallery;
    let Some(found) = &adapter.app.state.search else {
        window.set_gallery_items(ModelRc::default());
        return;
    };
    let filter = gallery.filter.to_lowercase();
    let twelve_hour = adapter.layout.twelve_hour_clock;
    let mut items: Vec<(u64, GalleryItem)> = Vec::new();
    for hit in &found.results {
        let message = &hit.message;
        let date = chatter_core::clock::full(message.timestamp, twelve_hour);
        let mut push = |name: String, url: String, picture: bool| {
            if !filter.is_empty()
                && !name.to_lowercase().contains(&filter)
                && !message.author.to_lowercase().contains(&filter)
            {
                return;
            }
            items.push((
                message.timestamp,
                GalleryItem {
                    room: hit.room.as_str().into(),
                    item: message.item.as_str().into(),
                    name: name.into(),
                    sender: message.author.as_str().into(),
                    date: date.as_str().into(),
                    picture_url: if picture { url.as_str().into() } else { SharedString::default() },
                    url: url.into(),
                },
            ));
        };
        if gallery.kind == 3 {
            for link in links(&message.body) {
                push(link.to_owned(), link.to_owned(), false);
            }
            continue;
        }
        for attachment in &message.attachments {
            let wanted = match gallery.kind {
                0 => attachment.image && !attachment.video,
                1 => attachment.video,
                _ => !attachment.image && !attachment.video,
            };
            let Some(url) = attachment.url.as_deref() else { continue };
            if wanted {
                push(attachment.name.clone(), url.to_owned(), gallery.kind == 0);
            }
        }
    }
    items.sort_by_key(|(sent, _)| *sent);
    if !gallery.oldest_first {
        items.reverse();
    }
    let items: Vec<GalleryItem> = items.into_iter().map(|(_, item)| item).collect();
    window.set_gallery_items(ModelRc::new(VecModel::from(items)));
    window.set_gallery_kind(gallery.kind as i32);
    window.set_gallery_oldest_first(gallery.oldest_first);
    window.set_gallery_loading(found.loading);
    window.set_gallery_more(found.more());
    window.set_gallery_error(found.error.clone().unwrap_or_default().into());
    let place = match &gallery.channel {
        Some(room) => adapter
            .app
            .state
            .rooms
            .iter()
            .find(|found| &found.id == room)
            .map(|found| format!("#{}", found.name))
            .unwrap_or_default(),
        None => "Whole server".into(),
    };
    window.set_gallery_place(place.into());
}

pub(super) fn bind(window: &ChatterWindow, adapter: &Rc<RefCell<Adapter>>) {
    {
        let (state, weak) = (adapter.clone(), window.as_weak());
        window.on_gallery_kind_chosen(move |kind| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            adapter.gallery.kind = (kind.max(0) as usize).min(KINDS.len() - 1);
            search(&mut adapter, &window);
            show(&adapter, &window);
        });
    }
    {
        let (state, weak) = (adapter.clone(), window.as_weak());
        window.on_gallery_order(move |oldest_first| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            adapter.gallery.oldest_first = oldest_first;
            show(&adapter, &window);
        });
    }
    {
        let (state, weak) = (adapter.clone(), window.as_weak());
        window.on_gallery_filter(move |text| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            adapter.gallery.filter = text.to_string();
            show(&adapter, &window);
        });
    }
    {
        let (state, weak) = (adapter.clone(), window.as_weak());
        window.on_gallery_load_more(move || {
            if let Some(window) = weak.upgrade() {
                state.borrow_mut().action(&window, Action::MoreResults);
            }
        });
    }
    {
        let (state, weak) = (adapter.clone(), window.as_weak());
        window.on_gallery_open_message(move |room, item| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            adapter.gallery.open = false;
            window.set_gallery_open(false);
            adapter.action(
                &window,
                Action::JumpInRoom {
                    room: room.into(),
                    item: item.into(),
                },
            );
        });
    }
    window.on_gallery_save(|url, name| {
        super::menus::save_file(&url, &name);
    });
    {
        let (state, weak) = (adapter.clone(), window.as_weak());
        window.on_gallery_close(move || {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            adapter.gallery.open = false;
            window.set_gallery_open(false);
            adapter.action(&window, Action::Search(String::new()));
        });
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn links_come_out_of_text() {
        let found: Vec<&str> = super::links("see https://a.com/x and <https://b.org> or http://c.net").collect();
        assert_eq!(found, ["https://a.com/x", "https://b.org", "http://c.net"]);
    }
}
