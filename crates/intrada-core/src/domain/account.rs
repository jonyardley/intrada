//! Account preferences and account-deletion events.
//!
//! All HTTP shape lives in `crate::http`; this module only knows about
//! the in-memory `Model` and what events it consumes/produces.

use crux_core::Command;
use serde::{Deserialize, Serialize};

use crate::app::{Effect, Event};
use crate::model::Model;

/// Per-user practice defaults surfaced in the Settings sheet.
///
/// Values mirror the API shape (`AccountPreferences` in
/// `intrada-api/src/db/account.rs`) so the same JSON deserialises on both
/// sides.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AccountPreferences {
    pub default_focus_minutes: u32,
    pub default_rep_count: u32,
}

impl Default for AccountPreferences {
    fn default() -> Self {
        Self {
            default_focus_minutes: 15,
            default_rep_count: 10,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum AccountEvent {
    /// Fetch the current user's preferences from the API.
    LoadPreferences,
    /// Server returned the preferences.
    PreferencesLoaded(AccountPreferences),
    /// Persist new preference values.
    SavePreferences(AccountPreferences),
    /// Server confirmed the save with the canonical row.
    PreferencesSaved(AccountPreferences),
    /// Network/server failure on save — model rolls back to the
    /// pre-edit value carried by the event.
    SavePreferencesFailed {
        previous: Option<AccountPreferences>,
        message: String,
    },
    /// Begin a hard account-delete request.
    DeleteAccount,
    /// Server returned 204 No Content. The handler flips
    /// `account_deleted = true` and clears `delete_in_flight`; the shell
    /// watches `account_deleted` to sign out + route home.
    AccountDeleted,
    /// Anything that prevented the delete (network, server). Shell may retry.
    DeleteAccountFailed(String),
}

pub fn handle_account_event(event: AccountEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        AccountEvent::LoadPreferences => crate::http::get_account_preferences(&model.api_base_url),

        AccountEvent::PreferencesLoaded(prefs) => {
            model.account_preferences = Some(prefs);
            crux_core::render::render()
        }

        AccountEvent::SavePreferences(prefs) => {
            // Optimistic: reflect in model immediately so the UI doesn't
            // bounce back to the prior value. Carry the prior value into
            // the HTTP builder so a failure can roll us back.
            let previous = model.account_preferences.clone();
            model.account_preferences = Some(prefs.clone());
            Command::all([
                crate::http::save_account_preferences(&model.api_base_url, &prefs, previous),
                crux_core::render::render(),
            ])
        }

        AccountEvent::PreferencesSaved(prefs) => {
            model.account_preferences = Some(prefs);
            crux_core::render::render()
        }

        AccountEvent::SavePreferencesFailed { previous, message } => {
            model.account_preferences = previous;
            model.last_error = Some(message);
            crux_core::render::render()
        }

        AccountEvent::DeleteAccount => {
            model.delete_in_flight = true;
            Command::all([
                crate::http::delete_account(&model.api_base_url),
                crux_core::render::render(),
            ])
        }

        AccountEvent::AccountDeleted => {
            model.delete_in_flight = false;
            model.account_deleted = true;
            crux_core::render::render()
        }

        AccountEvent::DeleteAccountFailed(msg) => {
            model.delete_in_flight = false;
            model.last_error = Some(msg);
            crux_core::render::render()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fresh_model() -> Model {
        Model::test_default()
    }

    #[test]
    fn load_preferences_does_not_mutate_model() {
        let mut model = fresh_model();
        let _cmd = handle_account_event(AccountEvent::LoadPreferences, &mut model);
        assert!(model.account_preferences.is_none());
    }

    #[test]
    fn preferences_loaded_updates_model() {
        let mut model = fresh_model();
        let prefs = AccountPreferences {
            default_focus_minutes: 25,
            default_rep_count: 8,
        };
        let _cmd = handle_account_event(AccountEvent::PreferencesLoaded(prefs.clone()), &mut model);
        assert_eq!(model.account_preferences, Some(prefs));
    }

    #[test]
    fn save_preferences_optimistically_updates_model() {
        let mut model = fresh_model();
        let prefs = AccountPreferences {
            default_focus_minutes: 30,
            default_rep_count: 12,
        };
        let _cmd = handle_account_event(AccountEvent::SavePreferences(prefs.clone()), &mut model);
        // Optimistic update happens immediately.
        assert_eq!(model.account_preferences, Some(prefs));
    }

    #[test]
    fn save_preferences_failed_rolls_back_to_previous() {
        let mut model = fresh_model();
        let original = AccountPreferences {
            default_focus_minutes: 5,
            default_rep_count: 4,
        };
        // Simulate optimistic update already applied.
        model.account_preferences = Some(AccountPreferences {
            default_focus_minutes: 99,
            default_rep_count: 99,
        });
        let _cmd = handle_account_event(
            AccountEvent::SavePreferencesFailed {
                previous: Some(original.clone()),
                message: "oops".to_string(),
            },
            &mut model,
        );
        assert_eq!(model.account_preferences, Some(original));
        assert_eq!(model.last_error.as_deref(), Some("oops"));
    }

    #[test]
    fn delete_account_sets_in_flight() {
        let mut model = fresh_model();
        let _cmd = handle_account_event(AccountEvent::DeleteAccount, &mut model);
        assert!(model.delete_in_flight);
        assert!(!model.account_deleted);
    }

    #[test]
    fn account_deleted_flips_terminal_flag_and_clears_in_flight() {
        let mut model = fresh_model();
        model.delete_in_flight = true;
        let _cmd = handle_account_event(AccountEvent::AccountDeleted, &mut model);
        assert!(model.account_deleted);
        assert!(!model.delete_in_flight);
    }

    #[test]
    fn delete_account_failed_clears_flag_and_records_error() {
        let mut model = fresh_model();
        model.delete_in_flight = true;
        let _cmd = handle_account_event(
            AccountEvent::DeleteAccountFailed("network".to_string()),
            &mut model,
        );
        assert!(!model.delete_in_flight);
        assert!(!model.account_deleted);
        assert_eq!(model.last_error.as_deref(), Some("network"));
    }
}
