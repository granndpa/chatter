//! Local account choices. Selecting one restarts after the old backend has
//! stopped, so requests and cached data never cross account boundaries.

use super::*;

pub(super) fn bind(window: &StaticWindow, adapter: &Rc<RefCell<Adapter>>) {
    let state = adapter.clone();
    let weak = window.as_weak();
    window.on_accounts_action(move |what, slot| {
        let Some(window) = weak.upgrade() else {
            return;
        };
        #[cfg(feature = "discord")]
        {
            use static_core::static_discord::accounts;
            let mut adapter = state.borrow_mut();
            if adapter.app.state.preview {
                return;
            }
            let result = (|| -> Result<bool> {
                let app = &adapter.app.state;
                accounts::remember(
                    &adapter.root,
                    app.discord_user.as_deref().unwrap_or_default(),
                    app.account_name.as_deref().unwrap_or_default(),
                    app.account_username.as_deref().unwrap_or_default(),
                )?;
                match what.as_str() {
                    "open" => {
                        let rows = accounts::list(&adapter.root)?
                            .into_iter()
                            .map(|a| SavedAccountRow {
                                current: a.slot == accounts::active_slot(),
                                slot: a.slot.into(),
                                name: a.name.into(),
                                username: a.username.into(),
                            })
                            .collect::<Vec<_>>();
                        window.set_saved_accounts(ModelRc::new(VecModel::from(rows)));
                        window.set_accounts_error(SharedString::default());
                        let weak = window.as_weak();
                        slint::Timer::single_shot(std::time::Duration::ZERO, move || {
                            if let Some(window) = weak.upgrade() {
                                window.invoke_show_accounts();
                            }
                        });
                        Ok(false)
                    }
                    "add" => {
                        accounts::add(&adapter.root)?;
                        Ok(true)
                    }
                    "select" if slot != accounts::active_slot() => {
                        accounts::select(&adapter.root, slot.as_str())?;
                        Ok(true)
                    }
                    "select" if slot == accounts::active_slot() && !app.discord_linked => Ok(false),
                    _ => Ok(false),
                }
            })();
            match result {
                Ok(true) => {
                    drop(adapter);
                    window.invoke_restart();
                }
                Ok(false) => {
                    if what == "select" && !adapter.app.state.discord_linked {
                        let effects = adapter.app.dispatch(Action::LinkDiscord);
                        adapter.execute(effects);
                        adapter.refresh(&window);
                    }
                }
                Err(error) => {
                    window.set_accounts_error(error.to_string().into());
                    let weak = window.as_weak();
                    slint::Timer::single_shot(std::time::Duration::ZERO, move || {
                        if let Some(window) = weak.upgrade() {
                            window.invoke_show_accounts();
                        }
                    });
                }
            }
        }
        #[cfg(not(feature = "discord"))]
        let _ = (&state, what, slot, window);
    });
}
