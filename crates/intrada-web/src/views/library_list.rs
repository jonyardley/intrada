use leptos::ev;
use leptos::prelude::*;

use intrada_core::domain::piece::PieceEvent;
use intrada_core::domain::types::CreatePiece;
use intrada_core::{Event, ViewModel};

use crate::components::{Button, ButtonVariant, LibraryItemCard};
use crate::core_bridge::process_effects;
use crate::data::SAMPLE_PIECES;
use crate::types::{SharedCore, ViewState};

#[component]
pub fn LibraryListView(
    view_model: RwSignal<ViewModel>,
    view_state: RwSignal<ViewState>,
    core: SharedCore,
    sample_counter: RwSignal<usize>,
) -> impl IntoView {
    let show_add_menu = RwSignal::new(false);
    let core_for_sample = core.clone();

    view! {
        // Hero section
        <section class="mb-10" aria-labelledby="welcome-heading">
            <h2 id="welcome-heading" class="text-2xl font-semibold text-slate-800 mb-3">
                "Welcome to Intrada"
            </h2>
            <p class="text-slate-600 leading-relaxed max-w-2xl">
                "Organize your music library, track your practice pieces and exercises, "
                "and build better practice habits. Intrada helps musicians stay focused "
                "on what matters \u{2014} making music."
            </p>
        </section>

        // Error banner
        {move || {
            view_model.get().error.map(|err| {
                view! {
                    <div class="mb-6 rounded-lg bg-red-50 border border-red-200 p-4" role="alert">
                        <p class="text-sm text-red-800">
                            <span class="font-medium">"Error: "</span>{err}
                        </p>
                    </div>
                }
            })
        }}

        // Status message
        {move || {
            view_model.get().status.map(|status| {
                view! {
                    <div class="mb-6 rounded-lg bg-blue-50 border border-blue-200 p-4" role="status">
                        <p class="text-sm text-blue-800">{status}</p>
                    </div>
                }
            })
        }}

        // Library section header
        <section class="mb-10" aria-labelledby="library-heading">
            <div class="flex items-center justify-between mb-4">
                <h2 id="library-heading" class="text-lg font-semibold text-slate-700">"Library"</h2>
                <div class="flex items-center gap-3">
                    <span class="text-sm text-slate-500">
                        {move || {
                            let count = view_model.get().item_count;
                            format!("{count} item(s)")
                        }}
                    </span>

                    // Add dropdown (FR-001)
                    <div class="relative">
                        <Button variant=ButtonVariant::Primary on_click=Callback::new(move |_| { show_add_menu.set(!show_add_menu.get()); })>
                            <span aria-hidden="true">"+"</span>
                            " Add"
                        </Button>
                        {move || {
                            if show_add_menu.get() {
                                Some(view! {
                                    <div class="absolute right-0 mt-2 w-48 rounded-lg bg-white shadow-lg border border-slate-200 z-10">
                                        <button
                                            class="w-full text-left px-4 py-2.5 text-sm text-slate-700 hover:bg-slate-50 rounded-t-lg"
                                            on:click=move |_| {
                                                show_add_menu.set(false);
                                                view_state.set(ViewState::AddPiece);
                                            }
                                        >
                                            "Piece"
                                        </button>
                                        <button
                                            class="w-full text-left px-4 py-2.5 text-sm text-slate-700 hover:bg-slate-50 rounded-b-lg"
                                            on:click=move |_| {
                                                show_add_menu.set(false);
                                                view_state.set(ViewState::AddExercise);
                                            }
                                        >
                                            "Exercise"
                                        </button>
                                    </div>
                                })
                            } else {
                                None
                            }
                        }}
                    </div>

                    // "Add Sample Item" button (retained from MVP)
                    <button
                        class="inline-flex items-center gap-1.5 rounded-lg bg-slate-200 px-3.5 py-2 text-sm font-medium text-slate-700 shadow-sm hover:bg-slate-300 transition-colors"
                        aria-label="Add a sample piece to the library"
                        on:click=move |_| {
                            let idx = sample_counter.get() % SAMPLE_PIECES.len();
                            let (title, composer) = SAMPLE_PIECES[idx];
                            sample_counter.set(sample_counter.get() + 1);

                            let event = Event::Piece(PieceEvent::Add(CreatePiece {
                                title: title.to_string(),
                                composer: composer.to_string(),
                                key: None,
                                tempo: None,
                                notes: None,
                                tags: vec!["sample".to_string()],
                            }));

                            let core_ref = core_for_sample.borrow();
                            let effects = core_ref.process_event(event);
                            process_effects(&core_ref, effects, &view_model);
                        }
                    >
                        "Add Sample"
                    </button>
                </div>
            </div>

            // Items list (FR-006: clickable items)
            <div class="space-y-3">
                {move || {
                    let vm = view_model.get();
                    if vm.items.is_empty() {
                        view! {
                            <div class="bg-white rounded-xl border border-slate-200 p-8 text-center">
                                <p class="text-slate-400">"No items in your library yet."</p>
                            </div>
                        }.into_any()
                    } else {
                        view! {
                            <ul class="space-y-3" role="list" aria-label="Library items">
                                {vm.items.into_iter().map(|item| {
                                    let item_id = item.id.clone();
                                    let vs = view_state;
                                    view! {
                                        <LibraryItemCard
                                            item=item
                                            on_click=move |_: ev::MouseEvent| {
                                                vs.set(ViewState::Detail(item_id.clone()));
                                            }
                                        />
                                    }
                                }).collect::<Vec<_>>()}
                            </ul>
                        }.into_any()
                    }
                }}
            </div>
        </section>
    }
}
