use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use leptos_router::hooks::use_params_map;
use leptos_router::NavigateOptions;

use intrada_core::{Event, ItemEvent, ViewModel};

use crate::components::{BackLink, Button, ButtonVariant, Card, FieldLabel, TypeBadge};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

#[component]
pub fn DetailView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();
    let params = use_params_map();
    let id = params.read().get("id").unwrap_or_default();
    let navigate = use_navigate();

    let show_delete_confirm = RwSignal::new(false);

    // Find the item in the current ViewModel
    let item = view_model
        .get_untracked()
        .items
        .into_iter()
        .find(|i| i.id == id);

    let Some(item) = item else {
        // Item not found — show message with link back to list
        return view! {
            <div class="text-center py-8">
                <p class="text-secondary mb-4">"Item not found."</p>
                <A href="/" attr:class="text-accent-text hover:text-accent-hover font-medium">
                    "← Back to Library"
                </A>
            </div>
        }
        .into_any();
    };

    // Destructure item fields to avoid excessive cloning
    let intrada_core::LibraryItemView {
        id: item_id,
        title,
        subtitle,
        item_type,
        category,
        key,
        tempo,
        notes,
        tags,
        created_at,
        updated_at,
        practice,
    } = item;

    let edit_href = format!("/library/{}/edit", item_id);
    let id_for_delete = item_id.clone();
    let type_for_badge = item_type;

    view! {
        <div class="space-y-4">
            // Back link
            <BackLink label="Back to Library" href="/".to_string() />

            // Delete confirmation banner (FR-011)
            {move || {
                if show_delete_confirm.get() {
                    let id_del = id_for_delete.clone();
                    let core_del = core.clone();
                    let navigate_del = navigate.clone();
                    Some(view! {
                        <div class="rounded-lg bg-danger-surface border border-danger/20 p-4" role="alert">
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
                                        navigate_del("/", NavigateOptions { replace: true, ..Default::default() });
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
                // Header: title + type badge
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

                // Fields grid (FR-007, FR-008: omit empty optional fields)
                <dl class="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-4 mb-6">
                    {category.map(|cat| {
                        view! {
                            <div>
                                <FieldLabel text="Category" />
                                <dd class="mt-1 text-sm text-secondary">{cat}</dd>
                            </div>
                        }
                    })}
                    {key.map(|k| {
                        view! {
                            <div>
                                <FieldLabel text="Key" />
                                <dd class="mt-1 text-sm text-secondary">{k}</dd>
                            </div>
                        }
                    })}
                    {tempo.map(|t| {
                        view! {
                            <div>
                                <FieldLabel text="Tempo" />
                                <dd class="mt-1 text-sm text-secondary">{t}</dd>
                            </div>
                        }
                    })}
                </dl>

                // Notes
                {notes.map(|n| {
                    view! {
                        <div class="mb-6">
                            <FieldLabel text="Notes" />
                            <dd class="text-sm text-secondary whitespace-pre-wrap">{n}</dd>
                        </div>
                    }
                })}

                // Tags
                {if !tags.is_empty() {
                    Some(view! {
                        <div class="mb-6">
                            <FieldLabel text="Tags" />
                            <dd class="flex flex-wrap gap-1.5">
                                {tags.into_iter().map(|tag| {
                                    view! {
                                        <span class="inline-flex items-center rounded-full border border-border-default px-2.5 py-0.5 text-xs text-muted">
                                            {tag}
                                        </span>
                                    }
                                }).collect::<Vec<_>>()}
                            </dd>
                        </div>
                    })
                } else {
                    None
                }}

                // Timestamps
                <div class="mt-2 pt-4 grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs text-faint">
                    <div>
                        <span class="font-medium">"Created: "</span>{created_at}
                    </div>
                    <div>
                        <span class="font-medium">"Updated: "</span>{updated_at}
                    </div>
                </div>
            </Card>

            // Practice summary (spacing between stacked cards)
            {practice.map(|p| {
                let has_scores = !p.score_history.is_empty();
                view! {
                    <Card>
                        <div class="space-y-4">
                            // Practice stats
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

                            // Latest confidence score
                            {p.latest_score.map(|score| {
                                view! {
                                    <div class="flex items-center gap-3">
                                        <span class="text-sm text-muted">"Current confidence:"</span>
                                        <span class="text-2xl font-bold text-accent-text">
                                            {format!("{}/5", score)}
                                        </span>
                                    </div>
                                }
                            })}

                            // Score history
                            {if has_scores {
                                let history = p.score_history;
                                view! {
                                    <div>
                                        <h4 class="field-label mb-2">"Score History"</h4>
                                        <div class="space-y-1.5">
                                            {history.into_iter().map(|entry| {
                                                // Format date for display (extract date portion from RFC3339)
                                                let display_date = entry.session_date.split('T').next().unwrap_or(&entry.session_date).to_string();
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
                        </div>
                    </Card>
                }
            })}

            // Action buttons (FR-009, FR-011)
            <div class="flex flex-col sm:flex-row gap-3">
                <A href=edit_href attr:class="cta-link">
                    "Edit"
                </A>
                <Button
                    variant=ButtonVariant::DangerOutline
                    disabled=Signal::derive(move || is_submitting.get())
                    on_click=Callback::new(move |_| { show_delete_confirm.set(true); })
                >
                    "Delete"
                </Button>
            </div>
        </div>
    }.into_any()
}
