use std::collections::HashMap;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{AccountEvent, AccountPreferences, Event, ViewModel};
use intrada_web::clerk_bindings;
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

use crate::components::{
    Button, ButtonVariant, GroupedList, GroupedListRow, Icon, IconName, TextField,
};

/// Settings route — account info, practice defaults, sign-out, and the
/// entry-point to account deletion. Reached via the Account tab on mobile
/// (bottom tab bar) and the profile button in the header at `sm:` and wider.
#[component]
pub fn SettingsView() -> impl IntoView {
    let core = expect_context::<SharedCore>();
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    // Refresh preferences on mount — server may have newer values from
    // another device.
    {
        let core = core.clone();
        Effect::new(move |_| {
            let core_ref = core.borrow();
            let effects = core_ref.process_event(Event::Account(AccountEvent::LoadPreferences));
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        });
    }

    let email = move || clerk_bindings::email().unwrap_or_else(|| "Signed in".to_string());

    // Seed with defaults so the form is never blank — falls through to
    // these values whenever the API hasn't returned yet, including the
    // dev shell with the API offline.
    let defaults = AccountPreferences::default();
    let focus_minutes = RwSignal::new(defaults.default_focus_minutes.to_string());
    let rep_count = RwSignal::new(defaults.default_rep_count.to_string());
    let errors = RwSignal::new(HashMap::<String, String>::new());

    // Replace defaults with real values once preferences load. The
    // `primed` guard locks the form after that — subsequent view_model
    // updates (e.g. our own save round-tripping back) must not clobber
    // the user's edits.
    let primed = RwSignal::new(false);
    Effect::new(move |_| {
        if primed.get_untracked() {
            return;
        }
        if let Some(prefs) = view_model.get().account_preferences {
            focus_minutes.set(prefs.default_focus_minutes.to_string());
            rep_count.set(prefs.default_rep_count.to_string());
            primed.set(true);
        }
    });

    let dirty = Signal::derive(move || {
        let saved = view_model.get().account_preferences.unwrap_or_default();
        focus_minutes.get().parse::<u32>().ok() != Some(saved.default_focus_minutes)
            || rep_count.get().parse::<u32>().ok() != Some(saved.default_rep_count)
    });

    let core_for_save = core.clone();
    let save = Callback::new(move |_: leptos::ev::MouseEvent| {
        let mut errs = HashMap::new();
        let focus = focus_minutes.get().trim().parse::<u32>().ok();
        let reps = rep_count.get().trim().parse::<u32>().ok();
        match focus {
            Some(v) if (1..=600).contains(&v) => {}
            _ => {
                errs.insert("focus".to_string(), "Enter 1–600 minutes".to_string());
            }
        }
        match reps {
            Some(v) if (1..=999).contains(&v) => {}
            _ => {
                errs.insert("reps".to_string(), "Enter 1–999".to_string());
            }
        }
        if !errs.is_empty() {
            errors.set(errs);
            return;
        }
        errors.set(HashMap::new());
        let prefs = AccountPreferences {
            default_focus_minutes: focus.unwrap(),
            default_rep_count: reps.unwrap(),
        };
        let core_ref = core_for_save.borrow();
        let effects = core_ref.process_event(Event::Account(AccountEvent::SavePreferences(prefs)));
        process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
    });

    let sign_out_click = move |_| {
        leptos::task::spawn_local(async move {
            clerk_bindings::sign_out().await;
        });
    };

    let navigate = use_navigate();
    let go_delete = Callback::new(move |_: leptos::ev::MouseEvent| {
        navigate("/settings/delete-account", NavigateOptions::default());
    });

    view! {
        <div class="max-w-md mx-auto py-comfortable space-y-comfortable pb-[env(safe-area-inset-bottom)]">
            <h1 class="page-title">"Settings"</h1>

            // Account header
            <div class="flex items-center gap-card">
                <div class="flex items-center justify-center h-10 w-10 rounded-full bg-surface-primary border border-border-default text-base font-medium text-primary">
                    {move || {
                        email()
                            .chars()
                            .next()
                            .map(|c| c.to_ascii_uppercase().to_string())
                            .unwrap_or_default()
                    }}
                </div>
                <div class="flex flex-col">
                    <span class="text-sm font-medium text-primary">"Signed in as"</span>
                    <span class="text-sm text-secondary">{email}</span>
                </div>
            </div>

            // Practice defaults section
            <div>
                <h3 class="section-title mb-card-compact">"Defaults"</h3>
                <p class="hint-text">
                    "Used to pre-fill new custom practice sessions."
                </p>
                <div class="space-y-card mt-card-compact">
                    <TextField
                        id="settings-focus-minutes"
                        label="Default focus duration (minutes)"
                        value=focus_minutes
                        field_name="focus"
                        errors=errors
                        input_type="text"
                        input_mode="numeric"
                    />
                    <TextField
                        id="settings-rep-count"
                        label="Default rep count"
                        value=rep_count
                        field_name="reps"
                        errors=errors
                        input_type="text"
                        input_mode="numeric"
                    />
                </div>
                <Show when=move || dirty.get()>
                    <div class="mt-card">
                        <Button variant=ButtonVariant::Primary on_click=save full_width=true>
                            "Save changes"
                        </Button>
                    </div>
                </Show>
            </div>

            // Account section
            <div>
                <h3 class="section-title mb-card-compact">"Account"</h3>
                <GroupedList aria_label="Account actions".to_string()>
                    <GroupedListRow>
                        <button
                            type="button"
                            class="w-full flex items-center justify-between px-card-compact py-card-compact text-left text-sm font-medium text-primary hover:bg-surface-hover motion-safe:transition-colors min-h-[44px]"
                            on:click=sign_out_click
                        >
                            <span>"Sign out"</span>
                            <Icon name=IconName::ChevronRight class="w-4 h-4 text-muted" />
                        </button>
                    </GroupedListRow>
                    <GroupedListRow>
                        <button
                            type="button"
                            class="w-full flex items-center justify-between px-card-compact py-card-compact text-left text-sm font-medium text-danger-text hover:bg-surface-hover motion-safe:transition-colors min-h-[44px]"
                            on:click=move |ev| go_delete.run(ev)
                        >
                            <span>"Delete account"</span>
                            <Icon name=IconName::ChevronRight class="w-4 h-4 text-danger-text" />
                        </button>
                    </GroupedListRow>
                </GroupedList>
            </div>
        </div>
    }
}
