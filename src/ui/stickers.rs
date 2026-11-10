//! A virtualized sticker list; PictureStore requests only instantiated rows.
use super::*;

pub(super) fn show(adapter: &Adapter, window: &StaticWindow) {
    let state = &adapter.app.state;
    let picker = &state.sticker_picker;
    window.set_sticker_picker_open(picker.open);
    window.set_staged_sticker_name(
        state
            .selected
            .as_ref()
            .and_then(|room| state.staged_stickers.get(room))
            .map_or("", |sticker| sticker.form.name.as_str())
            .into(),
    );
    if !picker.open {
        return;
    }
    let query = picker.query.trim().to_lowercase();
    let rows: Vec<StickerRow> = picker
        .entries
        .iter()
        .filter(|entry| {
            query.is_empty()
                || entry.sticker.form.name.to_lowercase().contains(&query)
                || entry.sticker.form.tags.to_lowercase().contains(&query)
                || entry.server.to_lowercase().contains(&query)
        })
        .map(|entry| StickerRow {
            id: entry.sticker.id.clone().into(),
            name: entry.sticker.form.name.clone().into(),
            server: entry.server.clone().into(),
            url: entry.sticker.picture.as_deref().unwrap_or_default().into(),
            available: entry.sticker.available,
        })
        .collect();
    let note = if picker.loading {
        "Loading server stickers..."
    } else if let Some(error) = &picker.error {
        error
    } else if rows.is_empty() {
        "No stickers found."
    } else {
        "Choose a sticker to add it to your message. Other servers' stickers may require Nitro."
    };
    window.set_sticker_note(note.into());
    window.set_sticker_query(picker.query.clone().into());
    window.set_sticker_rows(same_list(
        &window.get_sticker_rows(),
        ModelRc::new(VecModel::from(rows)),
    ));
}

pub(super) fn bind(window: &StaticWindow, adapter: &Rc<RefCell<Adapter>>) {
    macro_rules! bind {
        ($callback:ident, $action:expr) => {{
            let (state, weak) = (adapter.clone(), window.as_weak());
            window.$callback(move || {
                if let Some(window) = weak.upgrade() {
                    state.borrow_mut().action(&window, $action);
                }
            });
        }};
    }
    bind!(on_open_stickers, Action::OpenStickers);
    bind!(on_close_stickers, Action::CloseStickers);
    bind!(on_remove_sticker, Action::RemoveSticker);
    let (state, weak) = (adapter.clone(), window.as_weak());
    window.on_search_stickers(move |query| {
        if let Some(window) = weak.upgrade() {
            state
                .borrow_mut()
                .action(&window, Action::SearchStickers(query.to_string()));
        }
    });
    let (state, weak) = (adapter.clone(), window.as_weak());
    window.on_choose_sticker(move |id| {
        if let Some(window) = weak.upgrade() {
            state
                .borrow_mut()
                .action(&window, Action::ChooseSticker(id.to_string()));
        }
    });
}
