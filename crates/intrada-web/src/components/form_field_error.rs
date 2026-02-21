use std::collections::HashMap;
use std::sync::Arc;

use leptos::prelude::*;

/// Displays an inline validation error for a named form field.
#[component]
pub fn FormFieldError(
    field: &'static str,
    errors: RwSignal<HashMap<String, String>>,
    #[prop(default = String::new())] error_id: String,
) -> impl IntoView {
    let error_id: Arc<str> = error_id.into();
    view! {
        {move || {
            let eid = String::from(&*error_id);
            errors.get().get(field).cloned().map(|msg| {
                view! {
                    <p id=eid class="mt-1 text-sm text-danger-text" role="alert">{msg}</p>
                }
            })
        }}
    }
}
