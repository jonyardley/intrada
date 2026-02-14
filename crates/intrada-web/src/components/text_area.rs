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
    field_name: &'static str,
    errors: RwSignal<HashMap<String, String>>,
) -> impl IntoView {
    let rows_str = rows.to_string();
    let error_id: &'static str =
        Box::leak(format!("{id}-error").into_boxed_str());
    let has_error = move || errors.get().contains_key(field_name);

    view! {
        <div>
            <label class="block text-sm font-medium text-slate-700 mb-1" for=id>
                {label}
            </label>
            <textarea
                id=id
                rows=rows_str
                class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                prop:value=move || value.get()
                on:input=move |ev| { value.set(event_target_value(&ev)); }
                aria-describedby=error_id
                aria-invalid=move || if has_error() { "true" } else { "false" }
            />
            <FormFieldError field=field_name.to_string() errors=errors error_id=error_id />
        </div>
    }
}
