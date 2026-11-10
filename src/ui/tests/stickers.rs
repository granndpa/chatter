use super::*;

#[test]
fn the_sticker_picker_requests_only_instantiated_previews() {
    i_slint_backend_testing::init_no_event_loop();
    let window = classic_window();
    let requested = Rc::new(RefCell::new(HashSet::new()));
    let requests = requested.clone();
    window
        .global::<PictureStore>()
        .on_picture(|_, _| slint::Image::default());
    window.global::<PictureStore>().on_request(move |url| {
        requests.borrow_mut().insert(url.to_string());
    });
    window.set_sticker_rows(ModelRc::new(VecModel::from(
        (0..1000)
            .map(|index| StickerRow {
                id: index.to_string().into(),
                name: format!("Sticker {index}").into(),
                server: "Server".into(),
                url: format!("sticker/{index}").into(),
                available: true,
            })
            .collect::<Vec<_>>(),
    )));
    window.set_sticker_picker_open(true);
    window
        .window()
        .set_size(slint::LogicalSize::new(1120., 760.));
    window.show().unwrap();
    settle(&window);
    assert!(!requested.borrow().is_empty());
    assert!(
        requested.borrow().len() < 40,
        "the whole sticker pack must not be instantiated"
    );
    window.hide().unwrap();
}
