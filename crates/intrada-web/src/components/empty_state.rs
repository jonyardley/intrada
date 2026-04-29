use leptos::prelude::*;

use crate::components::{Icon, IconName};

/// Empty-state primitive — large icon, heading, body, optional CTA slot.
///
/// Used wherever a list/section can legitimately have no rows: library,
/// routines, sessions list, analytics. Centralises the iOS-shaped layout
/// (large glyph + heading + body) so every empty screen reads the same
/// way. The CTA is passed via `children` so callers can drop in a link,
/// button, or nothing at all.
#[component]
pub fn EmptyState(
    /// Icon shown above the title.
    icon: IconName,
    /// Short headline — what the screen is currently showing none of.
    #[prop(into)]
    title: String,
    /// One-line follow-up — what the user can do or what to expect.
    #[prop(into)]
    body: String,
    /// Optional CTA slot. Pass an `<A>`, `<button>`, or omit for no CTA.
    #[prop(optional)]
    children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="empty-state text-center py-12 px-4 sm:px-6 lg:px-0">
            <div class="empty-state-icon mx-auto mb-4 text-faint">
                <Icon name=icon class="w-full h-full" />
            </div>
            <p class="empty-state-title text-base font-semibold text-secondary">{title}</p>
            <p class="empty-state-body text-sm text-faint mt-2 max-w-xs mx-auto">{body}</p>
            {children.map(|c| view! {
                <div class="empty-state-cta mt-6">
                    {c()}
                </div>
            })}
        </div>
    }
}
