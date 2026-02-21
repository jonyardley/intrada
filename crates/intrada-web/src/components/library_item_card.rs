use leptos::prelude::*;
use leptos_router::components::A;

use intrada_core::LibraryItemView;

use crate::components::TypeBadge;

#[component]
pub fn LibraryItemCard(item: LibraryItemView) -> impl IntoView {
    let LibraryItemView {
        id,
        title,
        subtitle,
        item_type,
        key,
        tempo,
        tags,
        ..
    } = item;

    let has_subtitle = !subtitle.is_empty();
    let has_key_or_tempo = key.is_some() || tempo.is_some();
    let has_tags = !tags.is_empty();
    let href = format!("/library/{id}");

    view! {
        <li class="bg-surface-fallback supports-backdrop:bg-surface-secondary rounded-xl border border-border-default hover:bg-surface-primary motion-safe:transition-colors">
            <A href=href attr:class="block p-5">
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
                                    {tempo.map(|t| {
                                        view! {
                                            <span class="flex items-center gap-1">
                                                <span aria-hidden="true">"♩"</span>{t}
                                            </span>
                                        }
                                    })}
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
        </li>
    }
}
