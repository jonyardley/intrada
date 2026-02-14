use leptos::ev;
use leptos::prelude::*;

/// Shared back-navigation link with left arrow icon.
#[component]
pub fn BackLink(label: &'static str, on_click: Callback<ev::MouseEvent>) -> impl IntoView {
    view! {
        <button
            type="button"
            class="mb-6 inline-flex items-center gap-1 text-sm text-slate-500 hover:text-slate-700 transition-colors"
            on:click=move |ev| { on_click.run(ev); }
        >
            "\u{2190} "{label}
        </button>
    }
}
