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
    AddLibraryItemForm, DetailView, EditLibraryItemForm, LibraryListView, NotFoundView,
};

#[component]
pub fn App() -> impl IntoView {
    let core: SharedCore = SendWrapper::new(Rc::new(RefCell::new(Core::<Intrada>::new())));
    let view_model = RwSignal::new(ViewModel::default());

    // Initialize: load stub data on mount
    {
        let core_ref = core.borrow();
        let (pieces, exercises) = create_stub_data();
        let effects = core_ref.process_event(Event::DataLoaded { pieces, exercises });
        process_effects(&core_ref, effects, &view_model);
    }

    let core_clone2 = core.clone();
    let core_clone3 = core.clone();
    let core_clone4 = core.clone();

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
                            />
                        } />
                        // /library/new MUST come before /library/:id to avoid "new" matching :id
                        <Route path=path!("/library/new") view=move || view! {
                            <AddLibraryItemForm
                                view_model=view_model
                                core=core_clone2.clone()
                            />
                        } />
                        <Route path=path!("/library/:id") view=move || view! {
                            <DetailView
                                view_model=view_model
                                core=core_clone3.clone()
                            />
                        } />
                        <Route path=path!("/library/:id/edit") view=move || view! {
                            <EditLibraryItemForm
                                view_model=view_model
                                core=core_clone4.clone()
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
