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
            let adapter = state.borrow();
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
                        window.invoke_show_accounts();
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
                    _ => Ok(false),
                }
            })();
            match result {
                Ok(true) => {
                    drop(adapter);
                    window.invoke_restart();
                }
                Ok(false) => {}
                Err(error) => window.set_accounts_error(error.to_string().into()),
            }
        }
        #[cfg(not(feature = "discord"))]
        let _ = (&state, what, slot, window);
    });
}
