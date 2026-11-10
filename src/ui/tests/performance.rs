use super::*;
use i_slint_core::model::ModelChangeListenerContainer;

#[test]
fn switching_unrelated_timelines_emits_one_reset_instead_of_thousands_of_changes() {
    init_backend();
    let model = VecModel::default();
    let mut rows = Vec::new();
    let messages = |prefix: &str| {
        (0..2000)
            .map(|index| {
                Arc::new(ChatMessage {
                    id: format!("{prefix}{index}"),
                    item: format!("{prefix}{index}"),
                    body: "A message".into(),
                    ..Default::default()
                })
            })
            .collect::<Vec<_>>()
    };
    let first = messages("a");
    let next = messages("b");
    let revealed = HashSet::new();
    let marks = Marks {
        first_new: usize::MAX,
        first_unread: None,
        revealed: &revealed,
        restyle: false,
    };
    show_timeline(&model, &mut rows, &[], &first, &marks, message_row_plain);
    let notices = Box::pin(ModelChangeListenerContainer::new(Notices::default()));
    model
        .model_tracker()
        .attach_peer(notices.as_ref().model_peer());
    show_timeline(&model, &mut rows, &first, &next, &marks, message_row_plain);
    assert_eq!(&*notices.0.borrow(), &["reset"]);
    assert_eq!(model.row_count(), 2000);
    notices.0.borrow_mut().clear();
    show_timeline(&model, &mut rows, &next, &[], &marks, message_row_plain);
    assert_eq!(&*notices.0.borrow(), &["reset"]);
    assert_eq!(model.row_count(), 0);
}

#[test]
fn unrelated_sidebar_rows_are_replaced_in_one_notification() {
    let model = VecModel::from((0..2000).map(|n| format!("a{n}")).collect::<Vec<_>>());
    let notices = Box::pin(ModelChangeListenerContainer::new(Notices::default()));
    model
        .model_tracker()
        .attach_peer(notices.as_ref().model_peer());
    update_model_by_id(
        &model,
        (0..2000).map(|n| format!("b{n}")).collect(),
        |row| row.as_str().into(),
    );
    assert_eq!(&*notices.0.borrow(), &["reset"]);
    assert_eq!(model.row_count(), 2000);
}

#[test]
fn a_failed_viewport_request_still_shows_the_backend_error() {
    init_backend();
    let directory = tempfile::tempdir().unwrap();
    let window = classic_window();
    let adapter = Adapter::new(
        Arc::new(BackgroundRuntime::new().unwrap()),
        directory.path().into(),
        &window,
    )
    .unwrap();
    let mut adapter = adapter.borrow_mut();
    adapter.app.state.selected = Some("discord:7:8".into());
    adapter.action(
        &window,
        Action::VisibleMessages {
            room: "discord:7:8".into(),
            items: vec!["d:1".into()],
            at_end: true,
        },
    );
    assert!(adapter.app.state.notice.is_some());
    assert_eq!(
        window.get_notice().as_str(),
        adapter.app.state.notice.as_deref().unwrap()
    );
}
