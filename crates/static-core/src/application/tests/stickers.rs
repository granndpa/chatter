use super::*;

fn ready() -> Application {
    let mut app = connected();
    app.state.rooms = vec![Arc::new(preview_room("discord:7:8", "general", false, 3))];
    app.state.selected = Some("discord:7:8".into());
    app.state.room_ready = true;
    app.state.discord_online = true;
    app
}

fn sticker(id: &str) -> chat::ServerSticker {
    chat::ServerSticker {
        id: id.into(),
        available: true,
        form: chat::StickerForm {
            name: "Wave".into(),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[test]
fn sticker_only_messages_use_the_normal_queue_and_keep_the_sticker_on_failure() {
    let mut app = ready();
    app.state
        .staged_stickers
        .insert("discord:7:8".into(), sticker("42"));
    assert!(app.state.can_send());
    let effects = app.dispatch(Action::Send);
    assert!(effects.iter().any(|effect| matches!(effect,
        Effect::Backend(Command::Send { room, body, options, .. })
        if room == "discord:7:8" && body.is_empty() && options.sticker_id == Some(42))));
    app.apply(Event::SendResult {
        room: "discord:7:8".into(),
        body: String::new(),
        result: Err("Try again".into()),
    });
    assert!(app.state.staged_stickers.contains_key("discord:7:8"));
    app.dispatch(Action::SetDraft("Hello".into()));
    let effects = app.dispatch(Action::Send);
    assert!(effects.iter().any(|effect| matches!(effect,
        Effect::Backend(Command::Send { body, options, .. }) if body == "Hello" && options.sticker_id == Some(42))));
    app.apply(Event::SendResult {
        room: "discord:7:8".into(),
        body: "Hello".into(),
        result: Ok(()),
    });
    assert!(!app.state.staged_stickers.contains_key("discord:7:8"));
}

#[test]
fn auxiliary_text_keeps_the_drafts_sticker_and_stale_lists_do_not_replace_new_ones() {
    let mut app = ready();
    app.state
        .staged_stickers
        .insert("discord:7:8".into(), sticker("42"));
    let effects = app.send_text_in("discord:7:8".into(), "Helper message".into());
    assert!(effects.iter().any(|effect| matches!(effect,
        Effect::Backend(Command::Send { options, .. }) if options.sticker_id.is_none())));
    app.apply(Event::SendResult {
        room: "discord:7:8".into(),
        body: "Helper message".into(),
        result: Ok(()),
    });
    assert!(app.state.staged_stickers.contains_key("discord:7:8"));
    app.dispatch(Action::OpenStickers);
    let request = app.state.sticker_picker.request;
    app.apply(Event::PickerStickers {
        request: request.wrapping_sub(1),
        entries: vec![chat::PickerSticker {
            server: "Old".into(),
            sticker: sticker("1"),
        }],
        error: None,
    });
    assert!(app.state.sticker_picker.entries.is_empty());
    let mut unavailable = sticker("43");
    unavailable.available = false;
    app.apply(Event::PickerStickers {
        request,
        entries: vec![chat::PickerSticker {
            server: "Server".into(),
            sticker: unavailable,
        }],
        error: None,
    });
    app.dispatch(Action::ChooseSticker("43".into()));
    assert_eq!(app.state.staged_stickers["discord:7:8"].id, "42");
}
