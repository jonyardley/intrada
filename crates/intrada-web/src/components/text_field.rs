use std::collections::HashMap;

use leptos::ev;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlElement, ScrollIntoViewOptions, ScrollLogicalPosition};

use crate::components::FormFieldError;

/// Shared text input field with label and validation error display.
///
/// Adds two iOS-native polish behaviours:
/// - **Clear button**: a small "×" appears on the right when the field has
///   content; tapping clears it. Matches the native UITextField clear-mode.
/// - **Focus-scroll**: on focus, scrolls the input into view (centred in
///   its scroll container). Insurance for nested-scroll setups (main scroll
///   container + BottomSheet body) where iOS WebKit's default scroll-into-
///   view-when-keyboard-appears can fail.
#[component]
pub fn TextField(
    id: &'static str,
    label: &'static str,
    value: RwSignal<String>,
    #[prop(default = false)] required: bool,
    #[prop(optional)] placeholder: Option<&'static str>,
    #[prop(optional)] hint: Option<&'static str>,
    field_name: &'static str,
    errors: RwSignal<HashMap<String, String>>,
    #[prop(default = "text")] input_type: &'static str,
    /// Hints to mobile browsers what kind of soft keyboard to show.
    /// e.g. "numeric", "decimal", "tel", "email", "url".
    #[prop(optional)]
    input_mode: Option<&'static str>,
) -> impl IntoView {
    let error_id = format!("{id}-error");
    let has_error = move || errors.get().contains_key(field_name);

    let on_focus = move |ev: ev::FocusEvent| {
        if let Some(target) = ev.target().and_then(|t| t.dyn_into::<HtmlElement>().ok()) {
            let opts = ScrollIntoViewOptions::new();
            opts.set_block(ScrollLogicalPosition::Center);
            opts.set_behavior(web_sys::ScrollBehavior::Smooth);
            target.scroll_into_view_with_scroll_into_view_options(&opts);
        }
    };

    view! {
        <div>
            <label class="form-label" for=id>
                {label}
            </label>
            {hint.map(|h| view! {
                <p class="hint-text">{h}</p>
            })}
            <div class="input-wrapper">
                <input
                    id=id
                    type=input_type
                    inputmode=input_mode.unwrap_or("")
                    class="input-base input-base--with-clear"
                    placeholder=placeholder.unwrap_or("")
                    bind:value=value
                    required=required
                    aria-describedby=error_id.clone()
                    aria-invalid=move || if has_error() { "true" } else { "false" }
                    on:focus=on_focus
                />
                <Show when=move || !value.get().is_empty()>
                    <button
                        type="button"
                        class="input-clear"
                        aria-label="Clear field"
                        // mousedown fires before the input loses focus, so we
                        // can clear without the input blurring (which would
                        // hide the clear button before our click fires on iOS).
                        on:mousedown=move |ev| {
                            ev.prevent_default();
                            value.set(String::new());
                        }
                        on:touchstart=move |_| value.set(String::new())
                    >
                        "×"
                    </button>
                </Show>
            </div>
            <FormFieldError field=field_name errors=errors error_id=error_id />
        </div>
    }
}
