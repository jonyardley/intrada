use leptos::ev;
use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::{Icon, IconName};

/// Trailing action for top-level list pages — a circular icon-only "+"
/// rendered in a [`PageHeading`]'s trailing slot.
///
/// One pattern across web + iOS, replacing the earlier full-width purple
/// pill on web that collapsed to "+" only on iOS. The 2.75rem circular
/// hit target meets WCAG 2.5.5 (44pt minimum) and matches Apple's
/// trailing-nav add idiom (Calendar / Notes / Reminders / Mail).
///
/// Accepts either an `href` (renders as a router `<A>` link) or
/// `on_click` (renders as a `<button>`) — pick whichever fits the
/// add flow on that page (route navigation vs opening a sheet).
///
/// Always pair with a descriptive `aria_label` — screen readers won't
/// have a visible text fallback to read.
///
/// [`PageHeading`]: crate::components::PageHeading
#[component]
pub fn PageAddButton(
    /// What this button adds — read aloud by screen readers ("Add Item",
    /// "New Session", "New Set"). Keep it short; the visible affordance
    /// is the "+" glyph alone.
    #[prop(into)]
    aria_label: String,
    /// Destination route. Pass this OR `on_click`, not both.
    #[prop(optional, into)]
    href: Option<String>,
    /// Click handler when the button shouldn't navigate (e.g. opens a
    /// bottom sheet). Pass this OR `href`, not both.
    #[prop(optional)]
    on_click: Option<Callback<ev::MouseEvent>>,
) -> impl IntoView {
    let icon = view! { <Icon name=IconName::Plus class="page-add-button-icon" /> };

    match (href, on_click) {
        (Some(href), _) => view! {
            <A
                href=href
                attr:class="page-add-button"
                attr:aria-label=aria_label
            >
                {icon}
            </A>
        }
        .into_any(),
        (None, Some(handler)) => view! {
            <button
                type="button"
                class="page-add-button"
                aria-label=aria_label
                on:click=move |ev| handler.run(ev)
            >
                {icon}
            </button>
        }
        .into_any(),
        (None, None) => {
            // Caller forgot to wire an action — render an inert label so
            // the page heading doesn't crash, but make the mistake visible
            // by colouring it muted instead of the accent.
            view! {
                <span class="page-add-button text-muted" aria-label=aria_label>
                    {icon}
                </span>
            }
            .into_any()
        }
    }
}
