use std::collections::HashMap;

use leptos::prelude::*;

/// Displays an inline validation error for a named form field.
#[component]
pub fn FormFieldError(field: String, errors: RwSignal<HashMap<String, String>>) -> impl IntoView {
    view! {
        {move || {
            errors.get().get(&field).cloned().map(|msg| {
                view! {
                    <p class="mt-1 text-sm text-red-600" role="alert">{msg}</p>
                }
            })
        }}
    }
}
