use super::*;
use serde_json::json;

fn open(app: &mut Application, id: &str, nonce: &str) {
    app.apply(Event::BotFormOpened(json!({
        "id": id, "nonce": nonce, "application":{"id":"9"}, "channel_id":"8",
        "title":"Ticket", "custom_id":"ticket", "components":[
            {"type":1,"components":[{"type":4,"style":1,"custom_id":"answer","label":"Answer"}]}
        ]
    })));
}

fn context(app: &mut Application, nonce: &str) {
    app.state.bot_form_contexts.insert(
        nonce.into(),
        json!({"application_id":"9","channel_id":"8","guild_id":"7"}),
    );
}

#[test]
fn bot_form_requires_a_matching_interaction_and_keeps_its_origin() {
    let mut app = Application::default();
    open(&mut app, "100", "1");
    assert!(app.state.bot_form.is_none());
    context(&mut app, "1");
    app.state.bot_form_contexts.get_mut("1").unwrap()["application_id"] = json!("different-bot");
    open(&mut app, "100", "1");
    assert!(app.state.bot_form.is_none());
    context(&mut app, "2");
    open(&mut app, "100", "2");
    assert!(app.dispatch(Action::SubmitBotForm).is_empty());
    assert!(app.state.bot_form.as_ref().unwrap().error.is_some());
    app.dispatch(Action::WriteBotForm {
        index: 0,
        value: "Help".into(),
    });
    // Navigation after opening must not move the submission to another room.
    app.state.selected = Some(discord::room_id("other", "another"));
    let effects = app.dispatch(Action::SubmitBotForm);
    assert!(effects.iter().any(|effect| matches!(effect,
        Effect::Backend(Command::Discord(DiscordCommand::SubmitBotForm { body, .. }))
        if body["type"] == 5 && body["channel_id"] == "8" && body["guild_id"] == "7")));
    assert!(app.dispatch(Action::SubmitBotForm).is_empty());
    app.apply(Event::BotFormSubmitted {
        id: "100".into(),
        result: Err("Try again".into()),
    });
    let form = app.state.bot_form.as_ref().unwrap();
    assert!(!form.busy);
    assert_eq!(form.fields[0].value, "Help");
    assert_eq!(form.error.as_deref(), Some("Try again"));
}

#[test]
fn a_late_submit_does_not_close_a_new_form_and_cancel_sends_nothing() {
    let mut app = Application::default();
    context(&mut app, "1");
    open(&mut app, "100", "1");
    context(&mut app, "2");
    open(&mut app, "200", "2");
    app.apply(Event::BotFormSubmitted {
        id: "100".into(),
        result: Ok(()),
    });
    assert_eq!(app.state.bot_form.as_ref().unwrap().id, "200");
    assert!(app.dispatch(Action::CloseBotForm).is_empty());
    assert!(app.state.bot_form.is_none());
}
