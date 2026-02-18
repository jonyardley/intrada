use std::collections::HashMap;

use leptos::prelude::*;

use crate::components::FormFieldError;

/// Shared textarea field with label and validation error display.
#[component]
pub fn TextArea(
    id: &'static str,
    label: &'static str,
    value: RwSignal<String>,
    #[prop(default = 3)] rows: u32,
    #[prop(optional)] hint: Option<&'static str>,
    field_name: &'static str,
    errors: RwSignal<HashMap<String, String>>,
) -> impl IntoView {
    let rows_str = rows.to_string();
    let error_id = format!("{id}-error");
    let has_error = move || errors.get().contains_key(field_name);

    view! {
        <div>
            <label class="block text-sm font-medium text-gray-200 mb-1" for=id>
                {label}
            </label>
            {hint.map(|h| view! {
                <p class="text-xs text-gray-400 mb-1">{h}</p>
            })}
            <textarea
                id=id
                rows=rows_str
                class="w-full rounded-lg border border-white/20 bg-white/10 px-3 py-2.5 text-sm text-white placeholder-gray-400 focus:border-indigo-400 focus:ring-1 focus:ring-indigo-400"
                bind:value=value
                aria-describedby=error_id.clone()
                aria-invalid=move || if has_error() { "true" } else { "false" }
            />
            <FormFieldError field=field_name errors=errors error_id=error_id />
        </div>
    }
}
