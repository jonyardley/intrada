use leptos::prelude::*;

use intrada_core::ViewModel;

use crate::components::{use_toast, Button, ButtonVariant, Card, Icon};

/// Inline form for saving a setlist or summary as a named set.
///
/// When collapsed, shows a "Save as Set" button. When expanded, shows a
/// name input, Save, and Cancel buttons. Calls `on_save` with the entered name.
/// After the **server confirms** the save (via the `set_saves_committed`
/// counter rising on the ViewModel), the button switches to a disabled
/// "Saved" state. If the save fails, the form stays expanded so the user
/// can retry — no false-success feedback (#449).
///
/// When mounted inside a [`BottomSheet`], pass the sheet's `open` signal as
/// `sheet_open` — the "Saved" state will reset when the sheet closes, so a
/// close→reopen cycle starts fresh. Bottom sheets keep their children
/// mounted (translated off-screen), so without this the button would stay
/// stuck in the "Saved" state forever. Full-screen mounts (e.g. session
/// summary) can omit it.
#[component]
pub fn SetSaveForm(
    /// Callback invoked with the set name when the user taps Save.
    on_save: Callback<String>,
    /// Optional parent-sheet open signal. If provided, the "Saved" state
    /// resets when this transitions to false.
    #[prop(optional, into)]
    sheet_open: Option<Signal<bool>>,
) -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let expanded = RwSignal::new(false);
    let name = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let saved = RwSignal::new(false);
    // Pending = a save was dispatched and we're waiting for server confirmation
    // (counter increment) OR an error to surface. Stops users double-dispatching
    // while the request is in flight.
    let pending = RwSignal::new(false);
    // Snapshot of the success-counter at dispatch time. We promote `saved=true`
    // only when the live counter > this baseline.
    let baseline_counter = RwSignal::new(0u64);
    // Snapshot of `view_model.error` at dispatch time. A pre-existing error
    // (e.g. dismissed-but-not-cleared from an unrelated earlier failure)
    // would otherwise immediately trigger the failure path on the first
    // render after dispatch. We only count a NEW error (post-dispatch) as
    // our save's failure signal.
    let baseline_error = RwSignal::new(Option::<String>::None);
    let toast = use_toast();

    if let Some(open) = sheet_open {
        Effect::new(move |_| {
            if !open.get() {
                saved.set(false);
                pending.set(false);
            }
        });
    }

    // Reactive bridge: pending → confirmed (success) or pending → expanded
    // (failure). Driven by either the counter rising or a NEW
    // `view_model.error` appearing post-dispatch (distinguished from a
    // pre-existing one via `baseline_error`).
    Effect::new(move |_| {
        let vm = view_model.get();
        if !pending.get_untracked() {
            return;
        }
        if vm.set_saves_committed > baseline_counter.get_untracked() {
            // Confirmed: the round-trip succeeded.
            pending.set(false);
            saved.set(true);
            toast.show("Saved as Set");
        } else if vm.error.is_some() && vm.error != baseline_error.get_untracked() {
            // A new error surfaced after dispatch — treat as our save's
            // failure. Re-expand the form so the user can retry; the error
            // banner shows the message.
            pending.set(false);
            expanded.set(true);
        }
    });

    let try_save = move || {
        let trimmed = name.get_untracked().trim().to_string();
        if trimmed.is_empty() {
            error.set(Some("Name is required".to_string()));
            return;
        }
        error.set(None);
        let vm = view_model.get_untracked();
        baseline_counter.set(vm.set_saves_committed);
        baseline_error.set(vm.error.clone());
        pending.set(true);
        on_save.run(trimmed);
        name.set(String::new());
        expanded.set(false);
    };

    view! {
        {move || {
            if expanded.get() {
                let try_save_enter = try_save;
                let try_save_btn = try_save;
                view! {
                    <Card>
                        <h4 class="card-title">"Save as Set"</h4>
                        <div class="space-y-3">
                            <div>
                                <label class="sr-only" for="set-name">"Set name"</label>
                                <input
                                    id="set-name"
                                    type="text"
                                    class="input-base"
                                    placeholder="e.g. Morning Warm-up"
                                    bind:value=name
                                    on:keydown=move |ev: leptos::ev::KeyboardEvent| {
                                        if intrada_web::helpers::keyboard_event_key(ev.as_ref()).as_deref() == Some("Enter") {
                                            try_save_enter();
                                        }
                                    }
                                />
                            </div>
                            {move || error.get().map(|msg| view! {
                                <p class="text-xs text-danger-text">{msg}</p>
                            })}
                            <div class="flex gap-2">
                                <Button
                                    variant=ButtonVariant::Primary
                                    on_click=Callback::new(move |_| {
                                        try_save_btn();
                                    })
                                >
                                    "Save"
                                </Button>
                                <Button
                                    variant=ButtonVariant::Secondary
                                    on_click=Callback::new(move |_| {
                                        name.set(String::new());
                                        error.set(None);
                                        expanded.set(false);
                                    })
                                >
                                    "Cancel"
                                </Button>
                            </div>
                        </div>
                    </Card>
                }.into_any()
            } else if pending.get() {
                // Awaiting server confirmation. Show a disabled "Saving…" so
                // users see feedback but can't double-dispatch (#449).
                view! {
                    <button
                        class="w-full rounded-lg border border-dashed border-border-default bg-surface-secondary px-4 py-3 text-sm font-medium text-muted inline-flex items-center justify-center gap-2 cursor-default"
                        disabled
                    >
                        "Saving\u{2026}"
                    </button>
                }.into_any()
            } else if saved.get() {
                view! {
                    <button
                        class="w-full rounded-lg border border-success/40 bg-success/10 px-4 py-3 text-sm font-medium text-success-text inline-flex items-center justify-center gap-2 cursor-default"
                        disabled
                    >
                        <Icon icon=icondata::LuCheck class="w-4 h-4" />
                        "Saved"
                    </button>
                }.into_any()
            } else {
                view! {
                    <button
                        class="w-full rounded-lg border border-dashed border-border-default bg-surface-secondary px-4 py-3 text-sm font-medium text-accent-text hover:bg-surface-hover hover:border-accent-focus/40 motion-safe:transition-colors motion-safe:duration-150"
                        on:click=move |_| expanded.set(true)
                    >
                        "Save as Set"
                    </button>
                }.into_any()
            }
        }}
    }
}
