use leptos::prelude::*;

use crate::components::{Button, ButtonVariant, Card, Icon, IconName};

/// Inline form for saving a setlist or summary as a named set.
///
/// When collapsed, shows a "Save as Set" button. When expanded, shows a
/// name input, Save, and Cancel buttons. Calls `on_save` with the entered name.
/// After a successful save the button switches to a disabled "Saved" state to
/// prevent duplicate Set creation.
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
    let expanded = RwSignal::new(false);
    let name = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let saved = RwSignal::new(false);

    if let Some(open) = sheet_open {
        Effect::new(move |_| {
            if !open.get() {
                saved.set(false);
            }
        });
    }

    let try_save = move || {
        let trimmed = name.get_untracked().trim().to_string();
        if trimmed.is_empty() {
            error.set(Some("Name is required".to_string()));
        } else {
            error.set(None);
            on_save.run(trimmed);
            name.set(String::new());
            expanded.set(false);
            saved.set(true);
        }
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
                                        if ev.key() == "Enter" {
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
            } else if saved.get() {
                view! {
                    <button
                        class="w-full rounded-lg border border-success/40 bg-success/10 px-4 py-3 text-sm font-medium text-success-text inline-flex items-center justify-center gap-2 cursor-default"
                        disabled
                    >
                        <Icon name=IconName::Check class="w-4 h-4" />
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
