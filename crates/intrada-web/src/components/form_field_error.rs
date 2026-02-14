use std::collections::HashMap;

use leptos::prelude::*;

/// Displays an inline validation error for a named form field.
#[component]
pub fn FormFieldError(
    field: &'static str,
    errors: RwSignal<HashMap<String, String>>,
    #[prop(default = String::new())] error_id: String,
) -> impl IntoView {
    view! {
        {move || {
            let eid = error_id.clone();
            errors.get().get(field).cloned().map(|msg| {
                view! {
                    <p id=eid class="mt-1 text-sm text-red-600" role="alert">{msg}</p>
                }
            })
        }}
    }
}
