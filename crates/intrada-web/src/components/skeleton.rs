use leptos::prelude::*;

/// A single pulsing line placeholder for text content.
#[component]
pub fn SkeletonLine(
    /// Width as a Tailwind class, e.g. "w-3/4", "w-1/2", "w-full"
    #[prop(default = "w-3/4")]
    width: &'static str,
    /// Height as a Tailwind class, e.g. "h-4", "h-3"
    #[prop(default = "h-4")]
    height: &'static str,
) -> impl IntoView {
    let class = format!("{width} {height} rounded bg-surface-secondary animate-pulse");
    view! { <div class=class></div> }
}

/// A rectangular pulsing block placeholder for cards, charts, and images.
#[component]
pub fn SkeletonBlock(
    /// Height as a Tailwind class, e.g. "h-24", "h-48"
    #[prop(default = "h-24")]
    height: &'static str,
) -> impl IntoView {
    let class = format!("{height} rounded-xl bg-surface-secondary animate-pulse");
    view! { <div class=class></div> }
}

/// Skeleton matching the 2026-refresh `LibraryItemCard` layout — a 60px
/// AccentRow shape with title + subtitle skeleton on the left and a
/// small type-indicator placeholder on the right.
#[component]
pub fn SkeletonItemCard() -> impl IntoView {
    view! {
        <li class="accent-row accent-row--no-bar animate-pulse">
            <div class="flex flex-col flex-1 min-w-0 gap-1.5">
                <div class="h-3.5 w-2/3 rounded bg-surface-secondary"></div>
                <div class="h-3 w-1/2 rounded bg-surface-secondary"></div>
            </div>
            <div class="h-3 w-12 rounded-full bg-surface-secondary shrink-0"></div>
        </li>
    }
}

/// Generic list-page skeleton — a heading line followed by stacked card blocks.
/// Used for sessions and routines pages.
#[component]
pub fn SkeletonCardList(
    /// Number of card placeholders to render
    #[prop(default = 3)]
    count: usize,
) -> impl IntoView {
    let cards: Vec<_> = (0..count)
        .map(|_| {
            view! { <SkeletonBlock height="h-28" /> }
        })
        .collect();

    view! {
        <div class="space-y-3 animate-pulse">
            {cards}
        </div>
    }
}
