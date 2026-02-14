use std::cell::RefCell;
use std::rc::Rc;

use crux_core::Core;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use send_wrapper::SendWrapper;

use intrada_core::{Event, Intrada, ViewModel};

use crate::components::{AppFooter, AppHeader};
use crate::core_bridge::process_effects;
use crate::data::create_stub_data;
use crate::types::SharedCore;
use crate::views::{
    AddExerciseForm, AddPieceForm, DetailView, EditExerciseForm, EditPieceForm, LibraryListView,
    NotFoundView,
};

#[component]
pub fn App() -> impl IntoView {
    let core: SharedCore = SendWrapper::new(Rc::new(RefCell::new(Core::<Intrada>::new())));
    let view_model = RwSignal::new(ViewModel::default());
    let sample_counter = RwSignal::new(0_usize);

    // Initialize: load stub data on mount
    {
        let core_ref = core.borrow();
        let (pieces, exercises) = create_stub_data();
        let effects = core_ref.process_event(Event::DataLoaded { pieces, exercises });
        process_effects(&core_ref, effects, &view_model);
    }

    let core_clone = core.clone();
    let core_clone2 = core.clone();
    let core_clone3 = core.clone();
    let core_clone4 = core.clone();
    let core_clone5 = core.clone();
    let core_clone6 = core.clone();

    view! {
        <Router>
            <div class="min-h-screen bg-gradient-to-b from-slate-50 to-slate-100 text-slate-800">
                // Header
                <AppHeader />

                // Main content — routed by URL
                <main class="max-w-4xl mx-auto px-6 py-10" role="main">
                    <Routes fallback=|| view! { <NotFoundView /> }>
                        <Route path=path!("/") view=move || view! {
                            <LibraryListView
                                view_model=view_model
                                core=core_clone.clone()
                                sample_counter=sample_counter
                            />
                        } />
                        <Route path=path!("/library/:id") view=move || view! {
                            <DetailView
                                view_model=view_model
                                core=core_clone2.clone()
                            />
                        } />
                        <Route path=path!("/pieces/new") view=move || view! {
                            <AddPieceForm
                                view_model=view_model
                                core=core_clone3.clone()
                            />
                        } />
                        <Route path=path!("/exercises/new") view=move || view! {
                            <AddExerciseForm
                                view_model=view_model
                                core=core_clone4.clone()
                            />
                        } />
                        <Route path=path!("/pieces/:id/edit") view=move || view! {
                            <EditPieceForm
                                view_model=view_model
                                core=core_clone5.clone()
                            />
                        } />
                        <Route path=path!("/exercises/:id/edit") view=move || view! {
                            <EditExerciseForm
                                view_model=view_model
                                core=core_clone6.clone()
                            />
                        } />
                    </Routes>
                </main>

                // Footer
                <AppFooter />
            </div>
        </Router>
    }
}
