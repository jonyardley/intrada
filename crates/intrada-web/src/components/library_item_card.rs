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
    let has_tags = !tags.is_empty();
    let href = format!("/library/{id}");

    view! {
        <li class="bg-white rounded-xl shadow-sm border border-slate-200 hover:shadow-md transition-shadow">
            <A href=href attr:class="block p-5">
                <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0 flex-1">
                        <h3 class="text-base font-semibold text-slate-900 truncate">{title}</h3>
                        {if has_subtitle {
                            Some(view! {
                                <p class="text-sm text-slate-500 mt-0.5 truncate">{subtitle}</p>
                            })
                        } else {
                            None
                        }}
                        <div class="flex flex-wrap items-center gap-x-4 gap-y-1 mt-2 text-xs text-slate-400">
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
                        {if has_tags {
                            Some(view! {
                                <div class="flex flex-wrap gap-1.5 mt-2">
                                    {tags.into_iter().map(|tag| {
                                        view! {
                                            <span class="inline-flex items-center rounded-md bg-slate-100 px-2 py-0.5 text-xs text-slate-600">
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
