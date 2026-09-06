//! Account preferences and account-deletion events.

use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};

use crate::app::{Effect, Event};
use crate::model::Model;

/// Per-user practice defaults. Mirrors the API's `AccountPreferences`
/// (`intrada-api/src/db/account.rs`) so the same JSON deserialises both sides.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
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
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum AccountEvent {
    LoadPreferences,
    PreferencesLoaded(AccountPreferences),
    SavePreferences(AccountPreferences),
    PreferencesSaved(AccountPreferences),
    /// On failure the model rolls back to the pre-edit value carried here.
    SavePreferencesFailed {
        previous: Option<AccountPreferences>,
        message: String,
        /// Resolved from a 429's `Retry-After` header, if the rejection was
        /// one and named a resolvable wait (#1273).
        retry_after: Option<DateTime<Utc>>,
    },
    DeleteAccount,
    /// The shell watches `account_deleted` to sign out + route home.
    AccountDeleted,
    DeleteAccountFailed {
        message: String,
        /// Resolved from a 429's `Retry-After` header, if the rejection was
        /// one and named a resolvable wait (#1273).
        retry_after: Option<DateTime<Utc>>,
    },
}

pub fn handle_account_event(event: AccountEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        AccountEvent::LoadPreferences => crate::http::get_account_preferences(&model.api_base_url),

        AccountEvent::PreferencesLoaded(prefs) => {
            model.account_preferences = Some(prefs);
            model.record_success();
            crux_core::render::render()
        }

        AccountEvent::SavePreferences(prefs) => {
            let now = chrono::Utc::now();
            if let Some(msg) = model.rate_limit_message(now) {
                model.surface_error(msg);
                return crux_core::render::render();
            }
            // Optimistic update; carry the prior value so a failure can roll back.
            let previous = model.account_preferences.clone();
            model.account_preferences = Some(prefs.clone());
            Command::all([
                crate::http::save_account_preferences(&model.api_base_url, &prefs, previous),
                crux_core::render::render(),
            ])
        }

        AccountEvent::PreferencesSaved(prefs) => {
            model.account_preferences = Some(prefs);
            model.record_success();
            crux_core::render::render()
        }

        AccountEvent::SavePreferencesFailed {
            previous,
            message,
            retry_after,
        } => {
            model.account_preferences = previous;
            model.note_retry_after(retry_after);
            model.surface_error(message);
            crux_core::render::render()
        }

        AccountEvent::DeleteAccount => {
            let now = chrono::Utc::now();
            if let Some(msg) = model.rate_limit_message(now) {
                model.surface_error(msg);
                return crux_core::render::render();
            }
            model.delete_in_flight = true;
            Command::all([
                crate::http::delete_account(&model.api_base_url),
                crux_core::render::render(),
            ])
        }

        AccountEvent::AccountDeleted => {
            model.delete_in_flight = false;
            model.account_deleted = true;
            model.record_success();
            crux_core::render::render()
        }

        AccountEvent::DeleteAccountFailed {
            message,
            retry_after,
        } => {
            model.delete_in_flight = false;
            model.note_retry_after(retry_after);
            model.surface_error(message);
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
        assert_eq!(model.account_preferences, Some(prefs));
    }

    #[test]
    fn save_preferences_failed_rolls_back_to_previous() {
        let mut model = fresh_model();
        let original = AccountPreferences {
            default_focus_minutes: 5,
            default_rep_count: 4,
        };
        model.account_preferences = Some(AccountPreferences {
            default_focus_minutes: 99,
            default_rep_count: 99,
        });
        let _cmd = handle_account_event(
            AccountEvent::SavePreferencesFailed {
                previous: Some(original.clone()),
                message: "oops".to_string(),
                retry_after: None,
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
            AccountEvent::DeleteAccountFailed {
                message: "network".to_string(),
                retry_after: None,
            },
            &mut model,
        );
        assert!(!model.delete_in_flight);
        assert!(!model.account_deleted);
        assert_eq!(model.last_error.as_deref(), Some("network"));
    }

    #[test]
    fn save_preferences_failed_notes_retry_after() {
        let mut model = fresh_model();
        let until = Utc::now() + chrono::Duration::seconds(30);
        let _cmd = handle_account_event(
            AccountEvent::SavePreferencesFailed {
                previous: None,
                message: "rate limited".to_string(),
                retry_after: Some(until),
            },
            &mut model,
        );
        assert_eq!(model.rate_limited_until, Some(until));
    }

    #[test]
    fn delete_account_failed_notes_retry_after() {
        let mut model = fresh_model();
        let until = Utc::now() + chrono::Duration::seconds(30);
        let _cmd = handle_account_event(
            AccountEvent::DeleteAccountFailed {
                message: "rate limited".to_string(),
                retry_after: Some(until),
            },
            &mut model,
        );
        assert_eq!(model.rate_limited_until, Some(until));
    }

    #[test]
    fn delete_account_declines_to_fire_while_rate_limited() {
        let mut model = fresh_model();
        model.rate_limited_until = Some(Utc::now() + chrono::Duration::seconds(30));
        let mut cmd = handle_account_event(AccountEvent::DeleteAccount, &mut model);
        assert!(!cmd.effects().any(|e| matches!(e, Effect::Http(_))));
        assert!(!model.delete_in_flight);
        assert!(model.last_error.is_some());
    }

    #[test]
    fn save_preferences_declines_to_fire_while_rate_limited() {
        let mut model = fresh_model();
        model.rate_limited_until = Some(Utc::now() + chrono::Duration::seconds(30));
        let original = model.account_preferences.clone();
        let mut cmd = handle_account_event(
            AccountEvent::SavePreferences(AccountPreferences {
                default_focus_minutes: 10,
                default_rep_count: 3,
            }),
            &mut model,
        );
        assert!(!cmd.effects().any(|e| matches!(e, Effect::Http(_))));
        assert_eq!(model.account_preferences, original);
    }
}
