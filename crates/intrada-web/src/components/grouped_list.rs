use leptos::prelude::*;

/// iOS-style inset-grouped list container.
///
/// On web / Android: renders as a column of distinct cards-with-gaps,
/// visually identical to the prior `<Card>`-per-row pattern.
/// On iOS: the container becomes ONE rounded card and rows are stripped
/// of their own chrome and separated by hairline rules — matches the
/// UITableView `.grouped` idiom from Settings, Mail, Music, etc.
///
/// All visual treatment lives in `.grouped-list` / `.grouped-list-row`
/// in `input.css`, so the same markup adapts across platforms without
/// per-callsite branching.
#[component]
pub fn GroupedList(
    children: Children,
    /// Optional ARIA label describing the list (for screen readers).
    #[prop(optional, into)]
    aria_label: Option<String>,
) -> impl IntoView {
    view! {
        <ul class="grouped-list" role="list" aria-label=aria_label>
            {children()}
        </ul>
    }
}

/// A single row in a `<GroupedList>`. Renders as `<li>` to keep the
/// list semantically a list for assistive tech. Provides the row chrome
/// (background + radius on web, transparent + hairline on iOS) but no
/// inner padding — content is responsible for its own padding so
/// `<SwipeActions>` can fill the full row width.
#[component]
pub fn GroupedListRow(children: Children) -> impl IntoView {
    view! {
        <li class="grouped-list-row">
            {children()}
        </li>
    }
}
