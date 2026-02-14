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
    field_name: &'static str,
    errors: RwSignal<HashMap<String, String>>,
    #[prop(default = "text")] input_type: &'static str,
) -> impl IntoView {
    // Build a static error element ID for aria-describedby linkage.
    // We use a leaked &'static str since Leptos attribute values need 'static.
    let error_id: &'static str =
        Box::leak(format!("{id}-error").into_boxed_str());
    let has_error = move || errors.get().contains_key(field_name);

    view! {
        <div>
            <label class="block text-sm font-medium text-slate-700 mb-1" for=id>
                {label}
            </label>
            <input
                id=id
                type=input_type
                class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                placeholder=placeholder.unwrap_or("")
                prop:value=move || value.get()
                on:input=move |ev| { value.set(event_target_value(&ev)); }
                required=required
                aria-describedby=error_id
                aria-invalid=move || if has_error() { "true" } else { "false" }
            />
            <FormFieldError field=field_name.to_string() errors=errors error_id=error_id />
        </div>
    }
}
