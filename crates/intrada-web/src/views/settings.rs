use std::collections::HashMap;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use intrada_core::{AccountEvent, AccountPreferences, Event, ViewModel};
use intrada_web::clerk_bindings;
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

use crate::components::{BottomSheet, GroupedList, GroupedListRow, Icon, IconName, TextField};

/// Bottom-sheet Settings surface — account info, practice defaults, sign-out,
/// account deletion entry-point.
#[component]
pub fn SettingsSheet(open: RwSignal<bool>, on_close: Callback<()>) -> impl IntoView {
    let core = expect_context::<SharedCore>();
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    // Trigger preferences load each time the sheet opens so the displayed
    // values match the server (a sign-in or another device may have changed
    // them since the last open).
    Effect::new({
        let core = core.clone();
        move |_| {
            if !open.get() {
                return;
            }
            let core_ref = core.borrow();
            let effects = core_ref.process_event(Event::Account(AccountEvent::LoadPreferences));
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        }
    });

    let email = move || clerk_bindings::email().unwrap_or_else(|| "Signed in".to_string());

    // Local form state for the duration / rep editors. Initialised from the
    // ViewModel (or the local default) on every open.
    let focus_minutes = RwSignal::new(String::new());
    let rep_count = RwSignal::new(String::new());
    let errors = RwSignal::new(HashMap::<String, String>::new());

    Effect::new(move |_| {
        if !open.get() {
            return;
        }
        let prefs = view_model.get().account_preferences.unwrap_or_default();
        focus_minutes.set(prefs.default_focus_minutes.to_string());
        rep_count.set(prefs.default_rep_count.to_string());
        errors.set(HashMap::new());
    });

    // "Done" trailing nav action saves preferences if changed, else closes.
    let dirty = Signal::derive(move || {
        let saved = view_model.get().account_preferences.unwrap_or_default();
        focus_minutes.get().parse::<u32>().ok() != Some(saved.default_focus_minutes)
            || rep_count.get().parse::<u32>().ok() != Some(saved.default_rep_count)
    });

    let core_for_save = core.clone();
    let save = Callback::new(move |_: ()| {
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
        on_close.run(());
    });

    let nav_action = Callback::new(move |_: ()| {
        if dirty.get_untracked() {
            save.run(());
        } else {
            on_close.run(());
        }
    });
    let nav_action_disabled = Signal::derive(move || !errors.get().is_empty());

    let sign_out_click = move |_| {
        let close = on_close;
        leptos::task::spawn_local(async move {
            close.run(());
            clerk_bindings::sign_out().await;
        });
    };

    let navigate = use_navigate();
    let go_delete = Callback::new(move |_: leptos::ev::MouseEvent| {
        on_close.run(());
        navigate("/settings/delete-account", NavigateOptions::default());
    });

    view! {
        <BottomSheet
            open=open
            on_close=on_close
            nav_title="Settings".to_string()
            nav_action_label="Done".to_string()
            on_nav_action=nav_action
            nav_action_disabled=nav_action_disabled
        >
            <div class="space-y-comfortable">
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
        </BottomSheet>
    }
}
