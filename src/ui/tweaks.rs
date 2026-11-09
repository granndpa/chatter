//! Discord Tweaks' Saved panel: bookmarks, message reminders and the Hall of
//! Fame, and drafts kept on disk. The data is `static_core::tweaks`'s; this
//! wires it to the window, per Discord account.

use std::path::PathBuf;

use static_core::tweaks::{
    self, SavedMessage,
    bookmarks::{self, Bookmarks, Sort},
    drafts::{self, Drafts},
    inbox::{self, Inbox},
    pins::{self, Pins},
    reminders::{self, Reminders},
};

use super::*;

/// How often reminders are checked and drafts saved.
const TICK: std::time::Duration = std::time::Duration::from_secs(5);

/// The Saved panel's tabs.
const TAB_HALL_OF_FAME: i32 = 2;

#[derive(Default)]
pub(super) struct Tweaks {
    /// The Discord account whose files are loaded, and their folder.
    account: Option<String>,
    dir: Option<PathBuf>,
    bookmarks: Bookmarks,
    reminders: Reminders,
    /// Pinned servers and friends, which the rail shows first.
    pub(super) pins: Pins,
    inbox: Inbox,
    /// The inbox shows only unread entries; on unless switched.
    inbox_all: bool,
    /// The drafts as last written to disk.
    drafts: Drafts,
    /// The message the last reminder was about, which clicking its
    /// notification opens.
    pub(super) last_fired: Option<SavedMessage>,
    /// The Saved panel's search, collection and order.
    query: String,
    collection: String,
    sort: i32,
    /// Keeps checking reminders while the window lives.
    timer: Option<Rc<slint::Timer>>,
}

fn now() -> u64 {
    static_core::clock::now_ms() / 1000
}

/// Loads the account's files when the signed-in account changes, and brings
/// its drafts back.
pub(super) fn sync_account(adapter: &mut Adapter) {
    let account = adapter.app.state.discord_user.clone();
    if account == adapter.tweaks.account {
        return;
    }
    let dir = account.as_deref().map(|account| tweaks::account_dir(&adapter.root, account));
    let tweaks = &mut adapter.tweaks;
    tweaks.account = account;
    tweaks.dir = dir.clone();
    tweaks.bookmarks = Bookmarks::default();
    tweaks.reminders = Reminders::default();
    tweaks.pins = Pins::default();
    tweaks.inbox = Inbox::default();
    tweaks.drafts = Drafts::default();
    tweaks.collection.clear();
    if let Some(dir) = dir {
        tweaks.bookmarks = tweaks::load(&dir, bookmarks::FILE);
        tweaks.reminders = tweaks::load(&dir, reminders::FILE);
        tweaks.pins = tweaks::load(&dir, pins::FILE);
        tweaks.inbox = tweaks::load(&dir, inbox::FILE);
        let saved: Drafts = tweaks::load(&dir, drafts::FILE);
        adapter.app.state.restore_drafts(saved);
        adapter.tweaks.drafts = adapter.app.state.saved_drafts();
        adapter.restyle = true;
    }
    adapter.rebuild_rail();
}

/// Pins or unpins a server (`server`) or friend, or moves a pin, from its
/// menu: `kind` is `server`, `friend`, `up` or `down`.
pub(super) fn pin(adapter: &mut Adapter, kind: &str, target: &str) {
    sync_account(adapter);
    let state = &adapter.app.state;
    let server = state.servers.iter().any(|server| server.id == target);
    // A DM on the rail stands for its person.
    let user = state
        .rooms
        .iter()
        .find(|room| room.id == target && room.direct)
        .and_then(|room| room.person.clone())
        .unwrap_or_else(|| target.to_owned());
    let tweaks = &mut adapter.tweaks;
    match kind {
        "server" => {
            pins::toggle(&mut tweaks.pins.servers, target);
        }
        "friend" => {
            pins::toggle(&mut tweaks.pins.friends, &user);
        }
        "up" | "down" => {
            let by = if kind == "up" { -1 } else { 1 };
            if server {
                pins::shift(&mut tweaks.pins.servers, target, by);
            } else {
                pins::shift(&mut tweaks.pins.friends, &user, by);
            }
        }
        _ => return,
    }
    if let Some(dir) = &tweaks.dir
        && let Err(error) = tweaks::save(dir, pins::FILE, &tweaks.pins)
    {
        tracing::warn!(%error, "could not save pins");
    }
    adapter.rebuild_rail();
}

/// The pin rows a server's, person's or DM's menu offers: their ids and text.
pub(super) fn pin_rows(adapter: &Adapter, id: &str, person: bool) -> Vec<(String, &'static str)> {
    let state = &adapter.app.state;
    let pins = &adapter.tweaks.pins;
    let mut rows = Vec::new();
    let (list, pin_text, unpin_text): (&[String], _, _) =
        if !person && state.servers.iter().any(|server| server.id == id) {
            (&pins.servers, "Pin Server", "Unpin Server")
        } else {
            let user = if person {
                Some(id.to_owned())
            } else {
                state
                    .rooms
                    .iter()
                    .find(|room| room.id == id && room.direct)
                    .and_then(|room| room.person.clone())
            };
            // Only Discord people: their ids are numbers.
            let Some(user) = user.filter(|user| !user.is_empty() && user.bytes().all(|b| b.is_ascii_digit())) else {
                return rows;
            };
            if adapter.app.state.discord_user.as_deref() == Some(user.as_str()) {
                return rows;
            }
            let pinned = pins.friends.contains(&user);
            rows.push(("friend".to_owned(), if pinned { "Unpin Friend" } else { "Pin Friend" }));
            if pinned {
                rows.push(("up".to_owned(), "Move Pin Up"));
                rows.push(("down".to_owned(), "Move Pin Down"));
            }
            return rows;
        };
    let pinned = list.iter().any(|pinned| pinned == id);
    rows.push(("server".to_owned(), if pinned { unpin_text } else { pin_text }));
    if pinned {
        rows.push(("up".to_owned(), "Move Pin Up"));
        rows.push(("down".to_owned(), "Move Pin Down"));
    }
    rows
}

fn save_bookmarks(adapter: &Adapter) {
    if let Some(dir) = &adapter.tweaks.dir
        && let Err(error) = tweaks::save(dir, bookmarks::FILE, &adapter.tweaks.bookmarks)
    {
        tracing::warn!(%error, "could not save bookmarks");
    }
}

fn save_inbox(adapter: &Adapter) {
    if let Some(dir) = &adapter.tweaks.dir
        && let Err(error) = tweaks::save(dir, inbox::FILE, &adapter.tweaks.inbox)
    {
        tracing::warn!(%error, "could not save the inbox");
    }
}

/// Puts a notification's message in the Mentions Inbox when it mentions or
/// replies to this account, or came in a DM.
pub(super) fn record_alert(adapter: &mut Adapter, alert: &static_core::chat::Alert) {
    if alert.call {
        return;
    }
    let Some(item) = alert.item.clone() else {
        return;
    };
    sync_account(adapter);
    let state = &adapter.app.state;
    let room = state.rooms.iter().find(|found| found.id == alert.room);
    if !alert.mention && !room.is_some_and(|room| room.direct) {
        return;
    }
    let message = SavedMessage {
        room: alert.room.clone(),
        item,
        server: state
            .servers
            .iter()
            .find(|server| server.channels.contains(&alert.room))
            .map(|server| server.name.clone())
            .unwrap_or_default(),
        channel: room.map(|room| room.name.clone()).unwrap_or_default(),
        author: alert.author.clone(),
        body: alert.body.clone(),
        sent: now(),
        picture: None,
    };
    if adapter.tweaks.inbox.add(message) {
        save_inbox(adapter);
        if let Some(window) = adapter.window.upgrade() {
            show(adapter, &window);
        }
    }
}

fn save_reminders(adapter: &Adapter) {
    if let Some(dir) = &adapter.tweaks.dir
        && let Err(error) = tweaks::save(dir, reminders::FILE, &adapter.tweaks.reminders)
    {
        tracing::warn!(%error, "could not save reminders");
    }
}

/// Saves drafts that changed, shows reminders that came due, and keeps the
/// panel's lists current.
fn tick(adapter: &mut Adapter, window: &StaticWindow) {
    sync_account(adapter);
    let Some(dir) = adapter.tweaks.dir.clone() else {
        return;
    };
    let drafts = adapter.app.state.saved_drafts();
    if drafts != adapter.tweaks.drafts {
        if let Err(error) = tweaks::save(&dir, drafts::FILE, &drafts) {
            tracing::warn!(%error, "could not save drafts");
        }
        adapter.tweaks.drafts = drafts;
    }
    let due = adapter.tweaks.reminders.take_due(now());
    if let Some(last) = due.last() {
        save_reminders(adapter);
        let message = &last.message;
        let title = if due.len() > 1 {
            format!("{} reminders", due.len())
        } else {
            "Reminder".to_owned()
        };
        let text = format!(
            "{}: {}",
            message.author,
            static_core::chat::plain_snippet(&message.body)
        );
        if let Some(tray) = &adapter.tray {
            tray.balloon(&title, &text);
        }
        adapter.tweaks.last_fired = Some(message.clone());
        show(adapter, window);
    }
}

/// The message with this item, as a tweak saves it, from the open
/// conversation or the one beside it.
fn saved_message(state: &AppState, item: &str) -> Option<SavedMessage> {
    let message = state.message(item)?;
    let room = if state.messages.iter().any(|found| found.item == item) {
        state.selected.clone()?
    } else {
        state.aside.as_ref()?.room.clone()
    };
    let channel = state
        .rooms
        .iter()
        .find(|found| found.id == room)
        .map(|found| found.name.clone())
        .unwrap_or_default();
    let server = state
        .servers
        .iter()
        .find(|server| server.channels.contains(&room))
        .map(|server| server.name.clone())
        .unwrap_or_default();
    let picture = message
        .attachments
        .iter()
        .find(|attachment| attachment.image)
        .and_then(|attachment| attachment.url.as_deref().map(str::to_owned));
    Some(SavedMessage {
        room,
        item: item.to_owned(),
        server,
        channel,
        author: message.author.clone(),
        body: message.body.clone(),
        sent: message.timestamp,
        picture,
    })
}

/// Where a saved message was: `#general in Server`, or the DM's name.
fn place(message: &SavedMessage) -> String {
    match (message.server.is_empty(), message.channel.is_empty()) {
        (true, true) => "Direct message".into(),
        (true, false) => message.channel.clone(),
        (false, true) => message.server.clone(),
        (false, false) => format!("#{} in {}", message.channel, message.server),
    }
}

/// Fills the Saved panel's lists.
fn show(adapter: &Adapter, window: &StaticWindow) {
    let state = &adapter.app.state;
    let tweaks = &adapter.tweaks;
    let twelve_hour = adapter.layout.twelve_hour_clock;
    let hall = window.get_saved_tab() == TAB_HALL_OF_FAME;
    let available = |room: &str| state.rooms.iter().any(|found| found.id == room);
    let rows: Vec<SavedRow> = tweaks
        .bookmarks
        .search(hall, &tweaks.query, &tweaks.collection, Sort::from_index(tweaks.sort))
        .into_iter()
        .map(|bookmark| {
            let message = &bookmark.message;
            SavedRow {
                id: bookmark.id as i32,
                author: message.author.as_str().into(),
                place: place(message).into(),
                time: static_core::clock::full(message.sent, twelve_hour).into(),
                body: message.body.as_str().into(),
                collection: bookmark.collection.as_str().into(),
                caption: bookmark.caption.as_str().into(),
                picture_url: message.picture.clone().unwrap_or_default().into(),
                available: available(&message.room),
            }
        })
        .collect();
    window.set_saved_rows(ModelRc::new(VecModel::from(rows)));
    let collections: Vec<SharedString> = tweaks
        .bookmarks
        .collections(hall)
        .into_iter()
        .map(SharedString::from)
        .collect();
    window.set_saved_collections(ModelRc::new(VecModel::from(collections)));
    let now = now();
    let reminders: Vec<ReminderRow> = tweaks
        .reminders
        .listed()
        .into_iter()
        .map(|reminder| ReminderRow {
            id: reminder.id as i32,
            author: reminder.message.author.as_str().into(),
            place: place(&reminder.message).into(),
            body: reminder.message.body.as_str().into(),
            due: if reminder.fired {
                "Due now".into()
            } else {
                reminders::describe(reminder.due, now, twelve_hour).into()
            },
            fired: reminder.fired,
        })
        .collect();
    window.set_reminder_rows(ModelRc::new(VecModel::from(reminders)));
    let unread_only = !tweaks.inbox_all;
    let inbox: Vec<InboxRow> = tweaks
        .inbox
        .listed(unread_only)
        .map(|entry| InboxRow {
            id: entry.id as i32,
            author: entry.message.author.as_str().into(),
            place: place(&entry.message).into(),
            time: static_core::clock::full(entry.message.sent, twelve_hour).into(),
            body: entry.message.body.as_str().into(),
            read: entry.read,
        })
        .collect();
    window.set_inbox_rows(ModelRc::new(VecModel::from(inbox)));
    window.set_inbox_unread_only(unread_only);
    let unread = tweaks.inbox.unread();
    window.set_saved_badge(if unread == 0 {
        SharedString::default()
    } else if unread > 99 {
        "99+".into()
    } else {
        unread.to_string().into()
    });
}

/// Opens a saved message where it was, unless that conversation is gone.
fn open(adapter: &mut Adapter, window: &StaticWindow, message: SavedMessage) {
    if !adapter.app.state.rooms.iter().any(|found| found.id == message.room) {
        window.set_saved_note("That conversation isn't available anymore.".into());
        return;
    }
    window.set_saved_open(false);
    adapter.action(
        window,
        Action::JumpInRoom {
            room: message.room,
            item: message.item,
        },
    );
}

/// Opens the message a reminder notification was about.
pub(super) fn open_last_reminder(adapter: &mut Adapter, window: &StaticWindow) {
    if let Some(message) = adapter.tweaks.last_fired.take() {
        open(adapter, window, message);
    }
}

/// Saves the part of the window a Hall of Fame card covers as a picture.
fn export(window: &StaticWindow, x: f32, y: f32, width: f32, height: f32) {
    let Some(shot) = window.window().take_snapshot().ok() else {
        return;
    };
    let scale = window.window().scale_factor();
    let (full_width, full_height) = (shot.width(), shot.height());
    let left = ((x * scale).max(0.0) as u32).min(full_width);
    let top = ((y * scale).max(0.0) as u32).min(full_height);
    let right = (((x + width) * scale) as u32).min(full_width);
    let bottom = (((y + height) * scale) as u32).min(full_height);
    if right <= left || bottom <= top {
        return;
    }
    let pixels = shot.as_slice();
    let mut card = Vec::with_capacity(((right - left) * (bottom - top) * 4) as usize);
    for row in top..bottom {
        for column in left..right {
            let pixel = pixels[(row * full_width + column) as usize];
            card.extend_from_slice(&[pixel.r, pixel.g, pixel.b, pixel.a]);
        }
    }
    let (card_width, card_height) = (right - left, bottom - top);
    std::thread::spawn(move || {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Export Hall of Fame card")
            .set_file_name("hall-of-fame.png")
            .save_file()
        else {
            return;
        };
        match image::RgbaImage::from_raw(card_width, card_height, card) {
            Some(picture) => {
                if let Err(error) = picture.save_with_format(&path, image::ImageFormat::Png) {
                    tracing::warn!(%error, "could not export the card");
                }
            }
            None => tracing::warn!("the card's picture was the wrong size"),
        }
    });
}

/// Connects the Saved panel and the message menu's Bookmark, Hall of Fame and
/// Remind Me, and starts checking reminders.
pub(super) fn bind(window: &StaticWindow, adapter: &Rc<RefCell<Adapter>>) {
    let with = |state: &Rc<RefCell<Adapter>>, weak: &slint::Weak<StaticWindow>| {
        (state.clone(), weak.clone())
    };
    let weak = window.as_weak();
    {
        let (state, weak) = with(adapter, &weak);
        window.on_saved_show(move |tab| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            sync_account(&mut adapter);
            if window.get_saved_tab() != tab {
                adapter.tweaks.collection.clear();
            }
            window.set_saved_tab(tab);
            window.set_saved_note(SharedString::default());
            window.set_saved_open(true);
            show(&adapter, &window);
        });
    }
    {
        let (state, weak) = with(adapter, &weak);
        window.on_saved_search(move |query, collection, sort| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            adapter.tweaks.query = query.to_string();
            adapter.tweaks.collection = collection.to_string();
            adapter.tweaks.sort = sort;
            show(&adapter, &window);
        });
    }
    {
        let (state, weak) = with(adapter, &weak);
        window.on_bookmark_message(move |item, hall| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            sync_account(&mut adapter);
            let Some(message) = saved_message(&adapter.app.state, &item) else {
                return;
            };
            adapter.tweaks.bookmarks.add(message, hall, now());
            save_bookmarks(&adapter);
            show(&adapter, &window);
        });
    }
    {
        let (state, weak) = with(adapter, &weak);
        window.on_saved_activate(move |id| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            let message = adapter.tweaks.bookmarks.get(id as u64).map(|found| found.message.clone());
            if let Some(message) = message {
                open(&mut adapter, &window, message);
            }
        });
    }
    {
        let (state, weak) = with(adapter, &weak);
        window.on_saved_remove(move |id| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            adapter.tweaks.bookmarks.remove(id as u64);
            save_bookmarks(&adapter);
            show(&adapter, &window);
        });
    }
    {
        let (state, weak) = with(adapter, &weak);
        window.on_saved_shift(move |id, by| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            adapter.tweaks.bookmarks.shift(id as u64, by);
            save_bookmarks(&adapter);
            show(&adapter, &window);
        });
    }
    {
        let (state, weak) = with(adapter, &weak);
        window.on_saved_set_collection(move |id, collection| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            adapter.tweaks.bookmarks.set_collection(id as u64, &collection);
            save_bookmarks(&adapter);
            show(&adapter, &window);
        });
    }
    {
        let (state, weak) = with(adapter, &weak);
        window.on_saved_set_caption(move |id, caption| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            adapter.tweaks.bookmarks.set_caption(id as u64, &caption);
            save_bookmarks(&adapter);
            show(&adapter, &window);
        });
    }
    {
        let weak = weak.clone();
        window.on_saved_export(move |x, y, width, height| {
            if let Some(window) = weak.upgrade() {
                export(&window, x, y, width, height);
            }
        });
    }
    {
        let (state, weak) = with(adapter, &weak);
        window.on_remind_at(move |item, choice, typed| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            sync_account(&mut adapter);
            let now = now();
            let due = if choice >= 0 {
                reminders::preset(choice, now)
            } else {
                reminders::parse(&typed, now)
            };
            let Some(due) = due else {
                window.set_remind_error("Type a time ahead, like 2026-11-09 18:30 or 18:30.".into());
                return;
            };
            let Some(message) = saved_message(&adapter.app.state, &item) else {
                window.set_remind_item(SharedString::default());
                return;
            };
            adapter.tweaks.reminders.add(message, due);
            save_reminders(&adapter);
            window.set_remind_error(SharedString::default());
            window.set_remind_item(SharedString::default());
            show(&adapter, &window);
        });
    }
    {
        let (state, weak) = with(adapter, &weak);
        window.on_reminder_activate(move |id| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            let message = adapter
                .tweaks
                .reminders
                .list
                .iter()
                .find(|reminder| reminder.id == id as u64)
                .map(|reminder| reminder.message.clone());
            if let Some(message) = message {
                open(&mut adapter, &window, message);
            }
        });
    }
    {
        let (state, weak) = with(adapter, &weak);
        window.on_reminder_cancel(move |id| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            adapter.tweaks.reminders.remove(id as u64);
            save_reminders(&adapter);
            show(&adapter, &window);
        });
    }
    {
        let (state, weak) = with(adapter, &weak);
        window.on_reminder_reschedule(move |id, typed| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            let now = now();
            // Empty is Snooze.
            let due = if typed.trim().is_empty() {
                Some(now + reminders::SNOOZE_SECONDS)
            } else {
                reminders::parse(&typed, now)
            };
            let Some(due) = due else {
                window.set_saved_note("Type a time ahead, like 2026-11-09 18:30 or 18:30.".into());
                return;
            };
            adapter.tweaks.reminders.set_due(id as u64, due);
            save_reminders(&adapter);
            window.set_saved_note(SharedString::default());
            show(&adapter, &window);
        });
    }
    {
        let (state, weak) = with(adapter, &weak);
        window.on_inbox_filter(move |unread_only| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            adapter.tweaks.inbox_all = !unread_only;
            show(&adapter, &window);
        });
    }
    {
        let (state, weak) = with(adapter, &weak);
        window.on_inbox_activate(move |id| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            let message = adapter.tweaks.inbox.get(id as u64).map(|entry| entry.message.clone());
            adapter.tweaks.inbox.mark(id as u64, true);
            save_inbox(&adapter);
            show(&adapter, &window);
            if let Some(message) = message {
                open(&mut adapter, &window, message);
            }
        });
    }
    {
        let (state, weak) = with(adapter, &weak);
        window.on_inbox_mark(move |id, read| {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            adapter.tweaks.inbox.mark(id as u64, read);
            save_inbox(&adapter);
            show(&adapter, &window);
        });
    }
    {
        let (state, weak) = with(adapter, &weak);
        window.on_inbox_mark_all(move || {
            let Some(window) = weak.upgrade() else { return };
            let mut adapter = state.borrow_mut();
            adapter.tweaks.inbox.mark_all();
            save_inbox(&adapter);
            show(&adapter, &window);
        });
    }
    let timer = Rc::new(slint::Timer::default());
    let (state, weak) = (Rc::downgrade(adapter), window.as_weak());
    timer.start(slint::TimerMode::Repeated, TICK, move || {
        if let (Some(adapter), Some(window)) = (state.upgrade(), weak.upgrade())
            && let Ok(mut adapter) = adapter.try_borrow_mut()
        {
            tick(&mut adapter, &window);
        }
    });
    adapter.borrow_mut().tweaks.timer = Some(timer);
}
