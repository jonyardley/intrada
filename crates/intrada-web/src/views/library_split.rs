use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use leptos_router::NavigateOptions;

use intrada_core::{Event, ItemEvent, ViewModel};

use crate::components::{
    parse_target_bpm, BackLink, Button, ButtonVariant, Card, FieldLabel, LibraryListRow,
    SkeletonBlock, SkeletonLine, TempoProgressChart, TypeBadge,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::helpers::{format_date_short, format_datetime_short};
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// Split-view library: sidebar list + detail pane on desktop, stacked on mobile.
///
/// Route: `/library` (list) or `/library/:id` (detail)
/// On desktop (≥768px): both panes visible side-by-side
/// On mobile (<768px): either list OR detail based on whether :id is present
#[component]
pub fn LibrarySplitView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let params = use_params_map();
    let navigate = use_navigate();

    // Get selected item ID from route params (if any)
    let selected_id = Signal::derive(move || params.read().get("id").unwrap_or_default());

    let has_selected = Signal::derive(move || !selected_id.get().is_empty());

    // Auto-select first item on desktop when no :id param and items exist.
    // We do this via an effect that fires once after data loads.
    Effect::new(move |prev_ran: Option<bool>| {
        let vm = view_model.get();
        let id = selected_id.get();
        let loading = is_loading.get();

        // Only auto-select once: when loading finishes, items exist, and no ID selected
        if !loading && id.is_empty() && !vm.items.is_empty() && prev_ran != Some(true) {
            let first_id = vm.items[0].id.clone();
            // Only navigate on desktop — use js to check viewport width
            if let Some(window) = web_sys::window() {
                if let Ok(width) = window.inner_width() {
                    if let Some(w) = width.as_f64() {
                        if w >= 768.0 {
                            navigate(
                                &format!("/library/{first_id}"),
                                NavigateOptions {
                                    replace: true,
                                    ..Default::default()
                                },
                            );
                        }
                    }
                }
            }
            return true;
        }
        prev_ran.unwrap_or(false)
    });

    view! {
        <div class="flex h-full -mx-4 sm:-mx-6 -my-6 sm:-my-10">
            // ── Sidebar (library list) ──
            // Visible on desktop always. On mobile: visible when no item selected.
            <div class={move || {
                if has_selected.get() {
                    "hidden md:flex md:flex-col md:w-80 md:shrink-0 md:border-r md:border-border-default md:overflow-y-auto"
                } else {
                    "flex flex-col w-full md:w-80 md:shrink-0 md:border-r md:border-border-default md:overflow-y-auto"
                }
            }}>
                <div class="p-4 space-y-4">
                    <div class="flex items-start justify-between gap-3">
                        <div>
                            <h1 class="text-lg font-semibold text-primary">"Library"</h1>
                            <span class="text-xs text-muted">
                                {move || {
                                    let count = view_model.get().items.len();
                                    if count == 1 { "1 item".to_string() } else { format!("{count} items") }
                                }}
                            </span>
                        </div>
                        <A href="/library/new" attr:class="cta-link text-sm shrink-0">
                            "Add Item"
                        </A>
                    </div>
                </div>

                // Item list
                <div class="flex-1 overflow-y-auto">
                    {move || {
                        if is_loading.get() {
                            view! {
                                <div class="px-4 space-y-3 animate-pulse">
                                    <SkeletonLine width="w-full" height="h-12" />
                                    <SkeletonLine width="w-full" height="h-12" />
                                    <SkeletonLine width="w-full" height="h-12" />
                                    <SkeletonLine width="w-full" height="h-12" />
                                    <SkeletonLine width="w-full" height="h-12" />
                                </div>
                            }.into_any()
                        } else {
                            let vm = view_model.get();
                            let current_id = selected_id.get();
                            if vm.items.is_empty() {
                                view! {
                                    <div class="text-center py-12 px-4">
                                        <p class="text-muted">"No items in your library yet."</p>
                                        <p class="text-sm text-faint mt-2">"Add a piece or exercise to get started."</p>
                                        <div class="mt-6">
                                            <A href="/library/new" attr:class="cta-link">"Add Item"</A>
                                        </div>
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <ul role="list" aria-label="Library items">
                                        {vm.items.into_iter().map(|item| {
                                            let item_id = item.id.clone();
                                            let is_current = item_id == current_id;
                                            view! {
                                                <LibraryListRow
                                                    item=item
                                                    href=format!("/library/{item_id}")
                                                    is_selected=is_current
                                                />
                                            }
                                        }).collect::<Vec<_>>()}
                                    </ul>
                                }.into_any()
                            }
                        }
                    }}
                </div>
            </div>

            // ── Detail pane ──
            // Visible on desktop always. On mobile: visible when an item is selected.
            <div class={move || {
                if has_selected.get() {
                    "flex flex-col flex-1 overflow-y-auto w-full"
                } else {
                    "hidden md:flex md:flex-col md:flex-1 md:overflow-y-auto"
                }
            }}>
                <div class="p-4 sm:p-6 space-y-4">
                    {move || {
                        let id = selected_id.get();
                        if id.is_empty() {
                            // No item selected — show empty state (desktop only)
                            view! {
                                <div class="flex items-center justify-center h-64">
                                    <p class="text-muted">"Select an item from the library"</p>
                                </div>
                            }.into_any()
                        } else {
                            // Render detail for selected item
                            view! {
                                <LibraryDetailPane item_id=id.clone() />
                            }.into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

/// Detail pane content — extracted so it can be rendered inside the split-view.
/// Shows back link on mobile, item detail, and action buttons.
#[component]
fn LibraryDetailPane(item_id: String) -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let navigate = use_navigate();

    let show_delete_confirm = RwSignal::new(false);

    view! {
        // Back link (mobile only)
        <div class="md:hidden">
            <BackLink label="Back to Library" href="/library".to_string() />
        </div>

        {move || {
            let item = view_model
                .get()
                .items
                .into_iter()
                .find(|i| i.id == item_id);

            if let Some(item) = item {
                let intrada_core::LibraryItemView {
                    id: id_val,
                    title,
                    subtitle,
                    item_type,
                    key,
                    tempo,
                    notes,
                    tags,
                    created_at,
                    updated_at,
                    practice,
                    latest_achieved_tempo: _,
                } = item;

                let tempo_for_history = tempo.clone();
                let edit_href = format!("/library/{}/edit", id_val);
                let id_for_delete = id_val.clone();
                let type_for_badge = item_type;
                let core_for_delete = core.clone();
                let navigate_for_delete = navigate.clone();

                view! {
                    // Delete confirmation
                    {move || {
                        if show_delete_confirm.get() {
                            let id_del = id_for_delete.clone();
                            let core_del = core_for_delete.clone();
                            let navigate_del = navigate_for_delete.clone();
                            Some(view! {
                                <div class="rounded-lg bg-danger-surface border border-danger/20 p-4 mb-4" role="alert">
                                    <p class="text-sm text-danger-text mb-3">
                                        "Are you sure you want to delete this item? This action cannot be undone."
                                    </p>
                                    <div class="flex gap-3">
                                        <Button
                                            variant=ButtonVariant::Danger
                                            loading=Signal::derive(move || is_submitting.get())
                                            on_click=Callback::new(move |_| {
                                                let event = Event::Item(ItemEvent::Delete { id: id_del.clone() });
                                                let core_ref = core_del.borrow();
                                                let effects = core_ref.process_event(event);
                                                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
                                                navigate_del("/library", NavigateOptions { replace: true, ..Default::default() });
                                            })>
                                            {move || if is_submitting.get() { "Deleting\u{2026}" } else { "Confirm Delete" }}
                                        </Button>
                                        <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| { show_delete_confirm.set(false); })>
                                            "Cancel"
                                        </Button>
                                    </div>
                                </div>
                            })
                        } else {
                            None
                        }
                    }}

                    // Detail card
                    <Card>
                        <div class="flex items-start justify-between gap-3 mb-6">
                            <div>
                                <h2 class="text-2xl font-bold text-primary">{title}</h2>
                                {if !subtitle.is_empty() {
                                    Some(view! {
                                        <p class="text-lg text-muted mt-1">{subtitle.clone()}</p>
                                    })
                                } else {
                                    None
                                }}
                            </div>
                            <TypeBadge item_type=type_for_badge.clone() />
                        </div>

                        <dl class="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-4 mb-6">
                            {key.map(|k| view! {
                                <div>
                                    <FieldLabel text="Key" />
                                    <dd class="mt-1 text-sm text-secondary">{k}</dd>
                                </div>
                            })}
                            {tempo.map(|t| view! {
                                <div>
                                    <FieldLabel text="Tempo" />
                                    <dd class="mt-1 text-sm text-secondary">{t}</dd>
                                </div>
                            })}
                        </dl>

                        {notes.map(|n| view! {
                            <div class="mb-6">
                                <FieldLabel text="Notes" />
                                <dd class="text-sm text-secondary whitespace-pre-wrap">{n}</dd>
                            </div>
                        })}

                        {if !tags.is_empty() {
                            Some(view! {
                                <div class="mb-6">
                                    <FieldLabel text="Tags" />
                                    <dd class="flex flex-wrap gap-1.5">
                                        {tags.into_iter().map(|tag| view! {
                                            <span class="inline-flex items-center rounded-full border border-border-default px-2.5 py-0.5 text-xs text-muted">
                                                {tag}
                                            </span>
                                        }).collect::<Vec<_>>()}
                                    </dd>
                                </div>
                            })
                        } else {
                            None
                        }}

                        <div class="mt-2 pt-4 grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs text-faint">
                            <div>
                                <span class="font-medium">"Created: "</span>{format_datetime_short(&created_at)}
                            </div>
                            <div>
                                <span class="font-medium">"Updated: "</span>{format_datetime_short(&updated_at)}
                            </div>
                        </div>
                    </Card>

                    // Practice summary
                    {practice.map(|p| {
                        let has_scores = !p.score_history.is_empty();
                        let has_tempo_history = !p.tempo_history.is_empty();
                        let target_tempo = tempo_for_history.clone();
                        view! {
                            <Card>
                                <div class="space-y-4">
                                    <div>
                                        <h3 class="text-sm font-semibold text-primary mb-1">"Practice Summary"</h3>
                                        <p class="text-sm text-secondary">
                                            {format!(
                                                "{} session{}, {} min total",
                                                p.session_count,
                                                if p.session_count == 1 { "" } else { "s" },
                                                p.total_minutes
                                            )}
                                        </p>
                                    </div>

                                    {p.latest_score.map(|score| view! {
                                        <div class="flex items-center gap-3">
                                            <span class="text-sm text-muted">"Current confidence:"</span>
                                            <span class="text-2xl font-bold text-accent-text">
                                                {format!("{}/5", score)}
                                            </span>
                                        </div>
                                    })}

                                    {if has_scores {
                                        let history = p.score_history;
                                        view! {
                                            <div>
                                                <h4 class="field-label mb-2">"Score History"</h4>
                                                <div class="space-y-1.5">
                                                    {history.into_iter().map(|entry| {
                                                        let display_date = format_date_short(&entry.session_date);
                                                        view! {
                                                            <div class="flex items-center justify-between text-sm">
                                                                <span class="text-muted">{display_date}</span>
                                                                <span class="inline-flex items-center rounded-md bg-badge-piece-bg px-1.5 py-0.5 text-xs font-medium text-accent-text ring-1 ring-accent-focus/20 ring-inset">
                                                                    {format!("{}/5", entry.score)}
                                                                </span>
                                                            </div>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            </div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <p class="text-xs text-faint">"No confidence scores recorded yet"</p>
                                        }.into_any()
                                    }}

                                    {if has_tempo_history {
                                        let target = parse_target_bpm(&target_tempo);
                                        view! {
                                            <div>
                                                <h4 class="field-label mb-2">"Tempo Progress"</h4>
                                                <TempoProgressChart
                                                    entries=p.tempo_history
                                                    target_bpm=target
                                                    latest_tempo=p.latest_tempo
                                                />
                                            </div>
                                        }.into_any()
                                    } else {
                                        ().into_any()
                                    }}
                                </div>
                            </Card>
                        }
                    })}

                    // Action buttons
                    <div class="flex flex-col sm:flex-row gap-3">
                        <A href=edit_href attr:class="cta-link">"Edit"</A>
                        <Button
                            variant=ButtonVariant::DangerOutline
                            disabled=Signal::derive(move || is_submitting.get())
                            on_click=Callback::new(move |_| { show_delete_confirm.set(true); })
                        >
                            "Delete"
                        </Button>
                    </div>
                }.into_any()
            } else if is_loading.get() {
                view! {
                    <Card>
                        <div class="space-y-4 animate-pulse">
                            <div class="flex items-start justify-between gap-3">
                                <div class="flex-1 space-y-3">
                                    <SkeletonLine width="w-2/3" height="h-7" />
                                    <SkeletonLine width="w-1/2" height="h-5" />
                                </div>
                                <SkeletonLine width="w-16" height="h-6" />
                            </div>
                            <div class="grid grid-cols-2 gap-4">
                                <SkeletonLine width="w-3/4" />
                                <SkeletonLine width="w-1/2" />
                            </div>
                            <SkeletonBlock height="h-20" />
                        </div>
                    </Card>
                }.into_any()
            } else {
                view! {
                    <div class="text-center py-8">
                        <p class="text-secondary mb-4">"Item not found."</p>
                        <A href="/library" attr:class="text-accent-text hover:text-accent-hover font-medium">
                            "← Back to Library"
                        </A>
                    </div>
                }.into_any()
            }
        }}
    }
}
