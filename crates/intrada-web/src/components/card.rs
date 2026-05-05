use leptos::prelude::*;

/// Shared card container — whisper-soft surface with a subtle 1px shadow.
/// Matches the `.detail-group` and `.stat-card-faint` siblings in the 2026
/// design language. No glassmorphism / backdrop-blur — see the `.card`
/// utility comment in `input.css` for why.
#[component]
pub fn Card(children: Children) -> impl IntoView {
    view! {
        <div class="card p-card sm:p-card-comfortable">
            {children()}
        </div>
    }
}
