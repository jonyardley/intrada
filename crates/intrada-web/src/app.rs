use std::cell::RefCell;
use std::rc::Rc;

use crux_core::Core;
use leptos::prelude::*;
use send_wrapper::SendWrapper;

use intrada_core::{Event, Intrada, ViewModel};

use crate::core_bridge::process_effects;
use crate::data::create_stub_data;
use crate::types::{SharedCore, ViewState};
use crate::views::{
    AddExerciseForm, AddPieceForm, DetailView, EditExerciseForm, EditPieceForm, LibraryListView,
};

#[component]
pub fn App() -> impl IntoView {
    let core: SharedCore = SendWrapper::new(Rc::new(RefCell::new(Core::<Intrada>::new())));
    let view_model = RwSignal::new(ViewModel::default());
    let view_state = RwSignal::new(ViewState::List);
    let sample_counter = RwSignal::new(0_usize);

    // Initialize: load stub data on mount
    {
        let core_ref = core.borrow();
        let (pieces, exercises) = create_stub_data();
        let effects = core_ref.process_event(Event::DataLoaded { pieces, exercises });
        process_effects(&core_ref, effects, &view_model);
    }

    let core_for_view = core.clone();

    view! {
        <div class="min-h-screen bg-gradient-to-b from-slate-50 to-slate-100 text-slate-800">
            // Header
            <header class="bg-white shadow-sm border-b border-slate-200" role="banner">
                <div class="max-w-4xl mx-auto px-6 py-5 flex items-center justify-between">
                    <div>
                        <h1 class="text-3xl font-bold tracking-tight text-slate-900">"Intrada"</h1>
                        <p class="text-sm text-slate-500 mt-0.5">"Your music practice companion"</p>
                    </div>
                    <span
                        class="inline-flex items-center rounded-full bg-amber-100 px-3 py-1 text-xs font-medium text-amber-800"
                        aria-label="Application version"
                    >
                        "v0.1.0"
                    </span>
                </div>
            </header>

            // Main content — routed by ViewState
            <main class="max-w-4xl mx-auto px-6 py-10" role="main">
                {move || {
                    let vs = view_state.get();
                    let core = core_for_view.clone();
                    match vs {
                        ViewState::List => {
                            view! {
                                <LibraryListView
                                    view_model=view_model
                                    view_state=view_state
                                    core=core.clone()
                                    sample_counter=sample_counter
                                />
                            }.into_any()
                        }
                        ViewState::Detail(id) => {
                            view! {
                                <DetailView
                                    id=id.clone()
                                    view_model=view_model
                                    view_state=view_state
                                    core=core.clone()
                                />
                            }.into_any()
                        }
                        ViewState::AddPiece => {
                            view! {
                                <AddPieceForm
                                    view_model=view_model
                                    view_state=view_state
                                    core=core.clone()
                                />
                            }.into_any()
                        }
                        ViewState::AddExercise => {
                            view! {
                                <AddExerciseForm
                                    view_model=view_model
                                    view_state=view_state
                                    core=core.clone()
                                />
                            }.into_any()
                        }
                        ViewState::EditPiece(id) => {
                            view! {
                                <EditPieceForm
                                    id=id.clone()
                                    view_model=view_model
                                    view_state=view_state
                                    core=core.clone()
                                />
                            }.into_any()
                        }
                        ViewState::EditExercise(id) => {
                            view! {
                                <EditExerciseForm
                                    id=id.clone()
                                    view_model=view_model
                                    view_state=view_state
                                    core=core.clone()
                                />
                            }.into_any()
                        }
                    }
                }}
            </main>

            // Footer
            <footer class="max-w-4xl mx-auto px-6 py-6 border-t border-slate-200" role="contentinfo">
                <p class="text-xs text-slate-400 text-center">
                    "Built with Rust, Leptos & Crux \u{2014} Page reload resets to stub data"
                </p>
            </footer>
        </div>
    }
}
