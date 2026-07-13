use leptos::prelude::*;

use intrada_core::ViewModel;

use crate::components::{use_toast, Button, ButtonVariant, Card, Icon};

/// Inline "Save as Set" form. Per-dispatch `request_id` isolates concurrent
/// instances (#663); `sheet_open` is required for bottom-sheet mounts since
/// sheets keep children mounted off-screen.
#[component]
pub fn SetSaveForm(
    on_save: Callback<(String, String)>,
    #[prop(optional, into)] sheet_open: Option<Signal<bool>>,
) -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let expanded = RwSignal::new(false);
    let name = RwSignal::new(String::new());
    let error = RwSignal::new(Option::<String>::None);
    let saved = RwSignal::new(false);
    let pending = RwSignal::new(false);
    let my_request_id = RwSignal::new(Option::<String>::None);
    // Avoid a dismissed earlier failure tripping our failure path (#449).
    let error_before_dispatch = RwSignal::new(Option::<String>::None);
    let toast = use_toast();

    if let Some(open) = sheet_open {
        Effect::new(move |_| {
            if !open.get() {
                saved.set(false);
                pending.set(false);
                my_request_id.set(None);
            }
        });
    }

    Effect::new(move |_| {
        let vm = view_model.get();
        if !pending.get_untracked() {
            return;
        }
        let my_id = my_request_id.get_untracked();
        if my_id.is_some() && vm.last_set_save_request_id == my_id {
            pending.set(false);
            my_request_id.set(None);
            saved.set(true);
            toast.show("Saved as Set");
        } else if vm.error.is_some() && vm.error != error_before_dispatch.get_untracked() {
            pending.set(false);
            my_request_id.set(None);
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
        let request_id = ulid::Ulid::gen().to_string();
        // Must precede `pending.set(true)` — Effect guards on `my_id.is_some()`.
        my_request_id.set(Some(request_id.clone()));
        error_before_dispatch.set(vm.error.clone());
        pending.set(true);
        on_save.run((trimmed, request_id));
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
