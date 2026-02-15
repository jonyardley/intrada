use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use leptos_router::hooks::use_params_map;
use leptos_router::NavigateOptions;

use intrada_core::{Event, ExerciseEvent, LogSession, PieceEvent, SessionEvent, ViewModel};

use crate::components::{
    BackLink, Button, ButtonVariant, Card, FieldLabel, PracticeTimer, SessionHistory, TypeBadge,
};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::SharedCore;

#[component]
pub fn DetailView() -> impl IntoView {
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let core = expect_context::<SharedCore>();
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
                <p class="text-slate-600 mb-4">"Item not found."</p>
                <A href="/" attr:class="text-indigo-600 hover:text-indigo-800 font-medium">
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
    let id_for_history = item_id.clone();
    let id_for_timer = item_id.clone();
    let id_for_log = item_id.clone();
    let type_for_badge = item_type.clone();
    let type_for_delete = item_type;

    let show_log_form = RwSignal::new(false);
    let log_duration = RwSignal::new(String::new());
    let log_notes = RwSignal::new(String::new());
    let log_error: RwSignal<Option<String>> = RwSignal::new(None);
    let core_for_log = core.clone();

    view! {
        <div>
            // Back link
            <BackLink label="Back to Library" href="/".to_string() />

            // Delete confirmation banner (FR-011)
            {move || {
                if show_delete_confirm.get() {
                    let id_del = id_for_delete.clone();
                    let core_del = core.clone();
                    let item_type_del = type_for_delete.clone();
                    let navigate_del = navigate.clone();
                    Some(view! {
                        <div class="mb-6 rounded-lg bg-red-50 border border-red-200 p-4" role="alert">
                            <p class="text-sm text-red-800 mb-3">
                                "Are you sure you want to delete this item? This action cannot be undone."
                            </p>
                            <div class="flex gap-3">
                                <Button variant=ButtonVariant::Danger on_click=Callback::new(move |_| {
                                        let event = if item_type_del == "piece" {
                                            Event::Piece(PieceEvent::Delete { id: id_del.clone() })
                                        } else {
                                            Event::Exercise(ExerciseEvent::Delete { id: id_del.clone() })
                                        };
                                        let core_ref = core_del.borrow();
                                        let effects = core_ref.process_event(event);
                                        process_effects(&core_ref, effects, &view_model);
                                        navigate_del("/", NavigateOptions { replace: true, ..Default::default() });
                                    })>
                                    "Confirm Delete"
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
                        <h2 class="text-2xl font-bold text-slate-900">{title}</h2>
                        {if !subtitle.is_empty() {
                            Some(view! {
                                <p class="text-lg text-slate-500 mt-1">{subtitle.clone()}</p>
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
                                <dd class="mt-1 text-sm text-slate-700">{cat}</dd>
                            </div>
                        }
                    })}
                    {key.map(|k| {
                        view! {
                            <div>
                                <FieldLabel text="Key" />
                                <dd class="mt-1 text-sm text-slate-700">{k}</dd>
                            </div>
                        }
                    })}
                    {tempo.map(|t| {
                        view! {
                            <div>
                                <FieldLabel text="Tempo" />
                                <dd class="mt-1 text-sm text-slate-700">{t}</dd>
                            </div>
                        }
                    })}
                </dl>

                // Notes
                {notes.map(|n| {
                    view! {
                        <div class="mb-6">
                            <FieldLabel text="Notes" />
                            <dd class="text-sm text-slate-700 whitespace-pre-wrap">{n}</dd>
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
                                        <span class="inline-flex items-center rounded-md bg-slate-100 px-2.5 py-1 text-xs text-slate-600">
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
                <div class="border-t border-slate-100 pt-4 grid grid-cols-1 sm:grid-cols-2 gap-4 text-xs text-slate-400">
                    <div>
                        <span class="font-medium">"Created: "</span>{created_at}
                    </div>
                    <div>
                        <span class="font-medium">"Updated: "</span>{updated_at}
                    </div>
                </div>
            </Card>

            // Practice summary
            {practice.map(|p| {
                view! {
                    <div class="mt-4 rounded-lg bg-indigo-50 border border-indigo-100 px-4 py-3">
                        <p class="text-sm font-medium text-indigo-900">
                            {format!(
                                "{} session{}, {} min total",
                                p.session_count,
                                if p.session_count == 1 { "" } else { "s" },
                                p.total_minutes
                            )}
                        </p>
                    </div>
                }
            })}

            // Action buttons (FR-009, FR-011)
            <div class="mt-6 flex gap-3">
                <A href=edit_href attr:class="inline-flex items-center justify-center rounded-lg bg-indigo-600 px-4 py-2.5 text-sm font-semibold text-white shadow-sm hover:bg-indigo-500 transition-colors">
                    "Edit"
                </A>
                <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                    show_log_form.update(|v| *v = !*v);
                })>
                    "Log Session"
                </Button>
                <Button variant=ButtonVariant::DangerOutline on_click=Callback::new(move |_| { show_delete_confirm.set(true); })>
                    "Delete"
                </Button>
            </div>

            // Manual log session form (T023)
            {move || {
                if show_log_form.get() {
                    let core_log = core_for_log.clone();
                    let item_id_log = id_for_log.clone();
                    Some(view! {
                        <div class="mt-4 rounded-lg border border-slate-200 bg-white p-4 space-y-3">
                            <h4 class="text-sm font-semibold text-slate-900">"Log Practice Session"</h4>
                            <div>
                                <label class="block text-sm font-medium text-slate-700 mb-1" for="log-duration">"Duration (minutes)"</label>
                                <input
                                    id="log-duration"
                                    type="number"
                                    min="1"
                                    max="1440"
                                    placeholder="30"
                                    class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                                    bind:value=log_duration
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-slate-700 mb-1" for="log-notes">"Notes (optional)"</label>
                                <textarea
                                    id="log-notes"
                                    rows="2"
                                    class="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500"
                                    bind:value=log_notes
                                />
                            </div>
                            {move || log_error.get().map(|msg| {
                                view! {
                                    <p class="text-sm text-red-600">{msg}</p>
                                }
                            })}
                            <div class="flex gap-2">
                                <Button variant=ButtonVariant::Primary on_click=Callback::new(move |_| {
                                    let dur_str = log_duration.get();
                                    let dur: u32 = match dur_str.parse() {
                                        Ok(d) => d,
                                        Err(_) => {
                                            log_error.set(Some("Please enter a valid number.".to_string()));
                                            return;
                                        }
                                    };
                                    let notes_val = log_notes.get();
                                    let session_notes = if notes_val.is_empty() { None } else { Some(notes_val) };
                                    let event = Event::Session(SessionEvent::Log(LogSession {
                                        item_id: item_id_log.clone(),
                                        duration_minutes: dur,
                                        notes: session_notes,
                                    }));
                                    let core_ref = core_log.borrow();
                                    let effects = core_ref.process_event(event);
                                    process_effects(&core_ref, effects, &view_model);

                                    let vm = view_model.get_untracked();
                                    if let Some(err) = vm.error {
                                        log_error.set(Some(err));
                                    } else {
                                        show_log_form.set(false);
                                        log_duration.set(String::new());
                                        log_notes.set(String::new());
                                        log_error.set(None);
                                    }
                                })>
                                    "Save"
                                </Button>
                                <Button variant=ButtonVariant::Secondary on_click=Callback::new(move |_| {
                                    show_log_form.set(false);
                                    log_error.set(None);
                                })>
                                    "Cancel"
                                </Button>
                            </div>
                        </div>
                    })
                } else {
                    None
                }
            }}

            // Practice timer (T032-T035)
            <PracticeTimer item_id=id_for_timer />

            // Session history (T030-T031)
            <SessionHistory item_id=id_for_history />
        </div>
    }.into_any()
}
