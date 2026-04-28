use leptos::prelude::*;
use leptos_router::components::A;

use intrada_core::LibraryItemView;

use crate::components::{SwipeActions, TypeBadge};

#[component]
pub fn LibraryItemCard(
    item: LibraryItemView,
    /// Optional swipe-to-delete callback. When provided (typically in the
    /// library list on iOS), wraps the card in a SwipeActions container
    /// that reveals a trailing Delete action on left-swipe.
    #[prop(optional, into)]
    on_delete: Option<Callback<String>>,
) -> impl IntoView {
    let LibraryItemView {
        id,
        title,
        subtitle,
        item_type,
        key,
        tempo,
        tags,
        latest_achieved_tempo,
        ..
    } = item;

    let has_subtitle = !subtitle.is_empty();
    let has_key_or_tempo = key.is_some() || tempo.is_some() || latest_achieved_tempo.is_some();
    let has_tags = !tags.is_empty();
    let href = format!("/library/{id}");

    // Build combined tempo display: "♩ 108 / 120 BPM" (achieved / target),
    // "♩ 108 BPM" (achieved only), or "♩ 120 BPM" (target only)
    let tempo_display = match (latest_achieved_tempo, &tempo) {
        (Some(achieved), Some(target)) => Some(format!("{achieved} / {target}")),
        (Some(achieved), None) => Some(format!("{achieved} BPM")),
        (None, Some(_)) => None, // handled by existing tempo.map below
        (None, None) => None,
    };

    let id_for_delete = id.clone();
    let body = view! {
        <A href=href attr:class="block p-card sm:p-card-comfortable">
            <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0 flex-1">
                        // Identity cluster: title + composer tightly grouped (audit #12)
                        <h3 class="text-base font-semibold text-primary truncate">{title}</h3>
                        {if has_subtitle {
                            Some(view! {
                                <p class="text-sm text-muted mt-1 truncate">{subtitle}</p>
                            })
                        } else {
                            None
                        }}
                        // Metadata: key/tempo with larger gap from identity cluster
                        {if has_key_or_tempo {
                            Some(view! {
                                <div class="flex flex-wrap items-center gap-x-4 gap-y-1 mt-3 text-xs text-faint">
                                    {key.map(|k| {
                                        view! {
                                            <span class="flex items-center gap-1">
                                                <span aria-hidden="true">"♯"</span>{k}
                                            </span>
                                        }
                                    })}
                                    {if let Some(combined) = tempo_display {
                                        // Achieved tempo exists — show combined display
                                        Some(view! {
                                            <span class="flex items-center gap-1">
                                                <span aria-hidden="true">"♩"</span>{combined}
                                            </span>
                                        })
                                    } else {
                                        // No achieved tempo — show target only (existing behaviour)
                                        tempo.map(|t| {
                                            view! {
                                                <span class="flex items-center gap-1">
                                                    <span aria-hidden="true">"♩"</span>{t}
                                                </span>
                                            }
                                        })
                                    }}
                                </div>
                            })
                        } else {
                            None
                        }}
                        // Tags: consistent gap from metadata
                        {if has_tags {
                            Some(view! {
                                <div class="flex flex-wrap gap-1.5 mt-2">
                                    {tags.into_iter().map(|tag| {
                                        view! {
                                            <span class="inline-flex items-center rounded-full border border-border-default px-2 py-0.5 text-xs text-muted">
                                                {tag}
                                            </span>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            })
                        } else {
                            None
                        }}
                    </div>
                    <TypeBadge item_type=item_type />
                </div>
            </A>
    };

    if let Some(cb) = on_delete {
        let id = id_for_delete;
        view! {
            <li class="glass-card hover:bg-surface-hover motion-safe:transition-colors">
                <SwipeActions on_delete=Callback::new(move |_| cb.run(id.clone()))>
                    {body}
                </SwipeActions>
            </li>
        }
        .into_any()
    } else {
        view! {
            <li class="glass-card hover:bg-surface-hover motion-safe:transition-colors">
                {body}
            </li>
        }
        .into_any()
    }
}
