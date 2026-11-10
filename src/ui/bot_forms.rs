//! Render bot forms without replacing field models while typing.
use super::*;

pub(super) fn show(adapter: &Adapter, window: &StaticWindow) {
    let Some(form) = &adapter.app.state.bot_form else {
        window.set_bot_form_open(false);
        return;
    };
    let fields: Vec<BotFormField> = form
        .fields
        .iter()
        .map(|field| BotFormField {
            label: field.label.clone().into(),
            placeholder: field.placeholder.clone().into(),
            value: field.value.clone().into(),
            multiline: field.multiline,
            required: field.required,
        })
        .collect();
    let old = window.get_bot_form_fields();
    if old.row_count() == fields.len() {
        for (index, field) in fields.into_iter().enumerate() {
            if old.row_data(index).as_ref() != Some(&field) {
                old.set_row_data(index, field);
            }
        }
    } else {
        window.set_bot_form_fields(ModelRc::new(VecModel::from(fields)));
    }
    window.set_bot_form_title(form.title.clone().into());
    window.set_bot_form_busy(form.busy);
    window.set_bot_form_error(form.error.as_deref().unwrap_or_default().into());
    window.set_bot_form_open(true);
}

pub(super) fn bind(window: &StaticWindow, adapter: &Rc<RefCell<Adapter>>) {
    let (state, weak) = (adapter.clone(), window.as_weak());
    window.on_write_bot_form(move |index, value| {
        if let Some(window) = weak.upgrade() {
            state.borrow_mut().action(
                &window,
                Action::WriteBotForm {
                    index: index.max(0) as usize,
                    value: value.to_string(),
                },
            );
        }
    });
    let (state, weak) = (adapter.clone(), window.as_weak());
    window.on_submit_bot_form(move || {
        if let Some(window) = weak.upgrade() {
            state.borrow_mut().action(&window, Action::SubmitBotForm);
        }
    });
    let (state, weak) = (adapter.clone(), window.as_weak());
    window.on_close_bot_form(move || {
        if let Some(window) = weak.upgrade() {
            state.borrow_mut().action(&window, Action::CloseBotForm);
        }
    });
}
