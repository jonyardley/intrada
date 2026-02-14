use leptos::prelude::*;

/// Shared card container with consistent white background, shadow, and border.
#[component]
pub fn Card(children: Children) -> impl IntoView {
    view! {
        <div class="bg-white rounded-xl shadow-sm border border-slate-200 p-6">
            {children()}
        </div>
    }
}
