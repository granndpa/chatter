//! Server sticker selection, kept with the draft until its send succeeds.
use super::*;

#[derive(Default)]
pub struct StickerPicker {
    pub open: bool,
    pub loading: bool,
    pub query: String,
    pub entries: Vec<chat::PickerSticker>,
    pub error: Option<String>,
    pub request: u64,
}

impl Application {
    pub(super) fn open_stickers(&mut self) -> Vec<Effect> {
        if self.state.preview
            || !self.state.can_write()
            || self.state.editing()
            || self
                .state
                .selected
                .as_deref()
                .and_then(discord::parse_room)
                .is_none()
        {
            return vec![];
        }
        let picker = &mut self.state.sticker_picker;
        picker.open = true;
        picker.loading = true;
        picker.error = None;
        picker.request = picker.request.wrapping_add(1);
        let guilds = self
            .state
            .servers
            .iter()
            .filter_map(|server| {
                Some((
                    discord::parse_server(&server.id)?.to_owned(),
                    server.name.clone(),
                ))
            })
            .collect();
        vec![Effect::Backend(Command::Discord(
            DiscordCommand::ListPickerStickers {
                request: picker.request,
                guilds,
            },
        ))]
    }

    pub(super) fn choose_sticker(&mut self, id: String) -> Vec<Effect> {
        if !self.state.sticker_picker.open || self.state.editing() || !self.state.can_write() {
            return vec![];
        }
        let Some(room) = self.state.selected.clone() else {
            return vec![];
        };
        if let Some(entry) = self.state.sticker_picker.entries.iter().find(|entry| {
            entry.sticker.id == id && entry.sticker.available && id.parse::<u64>().is_ok()
        }) {
            self.state
                .staged_stickers
                .insert(room, entry.sticker.clone());
            self.state.sticker_picker.open = false;
        }
        vec![]
    }
}
