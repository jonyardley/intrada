use leptos::prelude::*;
use leptos_router::components::A;

use intrada_core::LibraryItemView;

use crate::components::TypeBadge;

/// Compact library list row — replaces `LibraryItemCard` in list contexts.
///
/// Renders a horizontal row with title, subtitle, metadata, and type badge.
/// Supports an optional `is_selected` state for session builder context
/// (accent left bar + check icon when selected).
#[component]
pub fn LibraryListRow(
    item: LibraryItemView,
    /// Optional URL to navigate to on click. If None, row is non-navigable
    /// (used in session builder where click toggles selection instead).
    #[prop(optional)]
    href: Option<String>,
    /// Whether this item is selected (e.g. in the session builder setlist).
    #[prop(optional, default = false)]
    is_selected: bool,
    /// Whether to show the selected state indicator (accent bar + check icon).
    /// When false, the row is a plain list item with no selection UI.
    #[prop(optional, default = false)]
    show_selection: bool,
    /// Optional click handler (for tap-to-queue in session builder).
    #[prop(optional)]
    on_click: Option<Callback<()>>,
) -> impl IntoView {
    let LibraryItemView {
        id: _,
        title,
        subtitle,
        item_type,
        key,
        tempo,
        latest_achieved_tempo,
        ..
    } = item;

    let has_subtitle = !subtitle.is_empty();
    let has_meta = key.is_some() || tempo.is_some() || latest_achieved_tempo.is_some();

    // Build tempo display
    let tempo_display = match (latest_achieved_tempo, &tempo) {
        (Some(achieved), Some(target)) => Some(format!("{achieved} / {target}")),
        (Some(achieved), None) => Some(format!("{achieved} BPM")),
        (None, Some(_)) => None,
        (None, None) => None,
    };

    let row_content = view! {
        <div class="flex items-center gap-3 w-full">
            // Selection indicator (accent bar)
            {if show_selection {
                Some(view! {
                    <div class={if is_selected { "w-1 self-stretch rounded-full bg-accent shrink-0" } else { "w-1 self-stretch rounded-full bg-transparent shrink-0" }} />
                })
            } else {
                None
            }}

            // Item info
            <div class="min-w-0 flex-1">
                <p class={if is_selected { "text-sm font-semibold text-primary truncate" } else { "text-sm font-medium text-primary truncate" }}>
                    {title}
                </p>
                {if has_subtitle {
                    Some(view! {
                        <p class="text-xs text-muted truncate">{subtitle}</p>
                    })
                } else {
                    None
                }}
                {if has_meta {
                    Some(view! {
                        <div class="flex items-center gap-2 mt-0.5 text-xs text-faint">
                            {key.clone().map(|k| view! { <span>{k}</span> })}
                            {if key.is_some() && (tempo.is_some() || tempo_display.is_some()) {
                                Some(view! { <span>"·"</span> })
                            } else {
                                None
                            }}
                            {if let Some(combined) = tempo_display {
                                Some(view! { <span>"♩ "{combined}</span> })
                            } else {
                                tempo.map(|t| view! { <span>"♩ "{t}</span> })
                            }}
                        </div>
                    })
                } else {
                    None
                }}
            </div>

            // Type badge
            <TypeBadge item_type=item_type />

            // Selection toggle icon
            {if show_selection {
                Some(if is_selected {
                    view! {
                        <span class="text-accent-text text-sm shrink-0">"✓"</span>
                    }.into_any()
                } else {
                    view! {
                        <span class="text-muted text-sm shrink-0">"+"</span>
                    }.into_any()
                })
            } else {
                None
            }}
        </div>
    };

    // Wrap in link or clickable div
    let selected_class = if is_selected {
        "block px-3 py-2.5 bg-surface-hover transition-colors"
    } else {
        "block px-3 py-2.5 hover:bg-surface-hover transition-colors"
    };

    if let Some(url) = href {
        view! {
            <li class="border-b border-border-default last:border-0">
                <A href=url attr:class=selected_class>
                    {row_content}
                </A>
            </li>
        }.into_any()
    } else if let Some(cb) = on_click {
        view! {
            <li class="border-b border-border-default last:border-0">
                <button
                    class="block w-full text-left px-3 py-2.5 hover:bg-surface-hover transition-colors"
                    on:click=move |_| cb.run(())
                >
                    {row_content}
                </button>
            </li>
        }.into_any()
    } else {
        view! {
            <li class="border-b border-border-default last:border-0 px-3 py-2.5">
                {row_content}
            </li>
        }.into_any()
    }
}
