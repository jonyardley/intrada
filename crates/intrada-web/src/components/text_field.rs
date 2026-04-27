use std::collections::HashMap;

use leptos::prelude::*;

use crate::components::FormFieldError;

/// Shared text input field with label and validation error display.
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

    view! {
        <div>
            <label class="form-label" for=id>
                {label}
            </label>
            {hint.map(|h| view! {
                <p class="hint-text">{h}</p>
            })}
            <input
                id=id
                type=input_type
                inputmode=input_mode.unwrap_or("")
                class="input-base"
                placeholder=placeholder.unwrap_or("")
                bind:value=value
                required=required
                aria-describedby=error_id.clone()
                aria-invalid=move || if has_error() { "true" } else { "false" }
            />
            <FormFieldError field=field_name errors=errors error_id=error_id />
        </div>
    }
}
