use leptos::prelude::*;

use crate::components::{Button, ButtonVariant, Card};

/// Inline form for saving a setlist or summary as a named routine.
///
/// When collapsed, shows a "Save as Routine" button. When expanded, shows a
/// name input, Save, and Cancel buttons. Calls `on_save` with the entered name.
#[component]
pub fn RoutineSaveForm(
    /// Callback invoked with the routine name when the user taps Save.
    on_save: Callback<String>,
) -> impl IntoView {
    let expanded = RwSignal::new(false);
    let name = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);

    let try_save = move || {
        let trimmed = name.get_untracked().trim().to_string();
        if trimmed.is_empty() {
            error.set(Some("Name is required".to_string()));
        } else {
            error.set(None);
            on_save.run(trimmed);
            name.set(String::new());
            expanded.set(false);
        }
    };

    view! {
        {move || {
            if expanded.get() {
                let try_save_enter = try_save;
                let try_save_btn = try_save;
                view! {
                    <Card>
                        <h4 class="text-sm font-semibold text-white mb-3">"Save as Routine"</h4>
                        <div class="space-y-3">
                            <div>
                                <label class="sr-only" for="routine-name">"Routine name"</label>
                                <input
                                    id="routine-name"
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
                                <p class="text-xs text-red-400">{msg}</p>
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
            } else {
                view! {
                    <button
                        class="w-full rounded-lg border border-dashed border-white/20 bg-white/5 px-4 py-3 text-sm font-medium text-indigo-300 hover:bg-white/10 hover:border-indigo-400/40 motion-safe:transition-colors motion-safe:duration-150"
                        on:click=move |_| expanded.set(true)
                    >
                        "Save as Routine"
                    </button>
                }.into_any()
            }
        }}
    }
}
