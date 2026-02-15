use std::cell::RefCell;
use std::rc::Rc;

use crux_core::Core;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use send_wrapper::SendWrapper;

use intrada_core::{Event, Intrada, ViewModel};

use crate::components::{AppFooter, AppHeader};
use crate::core_bridge::{load_library_data, load_sessions_data, process_effects};
use crate::types::SharedCore;
use crate::views::{
    AddLibraryItemForm, DetailView, EditLibraryItemForm, LibraryListView, NotFoundView,
    SessionsListView,
};

#[component]
pub fn App() -> impl IntoView {
    let core: SharedCore = SendWrapper::new(Rc::new(RefCell::new(Core::<Intrada>::new())));
    let view_model = RwSignal::new(ViewModel::default());

    // Initialize: load from localStorage (or seed stub data on first run)
    {
        let core_ref = core.borrow();
        let (pieces, exercises) = load_library_data();
        let effects = core_ref.process_event(Event::DataLoaded { pieces, exercises });
        process_effects(&core_ref, effects, &view_model);

        let sessions = load_sessions_data();
        let effects = core_ref.process_event(Event::SessionsLoaded { sessions });
        process_effects(&core_ref, effects, &view_model);
    }

    // Provide core and view_model via Leptos context so child components
    // can access them with use_context() instead of prop drilling.
    provide_context(core);
    provide_context(view_model);

    view! {
        <Router>
            <div class="min-h-screen bg-gradient-to-b from-slate-50 to-slate-100 text-slate-800">
                // Header
                <AppHeader />

                // Main content — routed by URL
                <main class="max-w-4xl mx-auto px-6 py-10" role="main">
                    <Routes fallback=|| view! { <NotFoundView /> }>
                        <Route path=path!("/") view=move || view! {
                            <LibraryListView />
                        } />
                        // /library/new MUST come before /library/:id to avoid "new" matching :id
                        <Route path=path!("/library/new") view=move || view! {
                            <AddLibraryItemForm />
                        } />
                        <Route path=path!("/library/:id") view=move || view! {
                            <DetailView />
                        } />
                        <Route path=path!("/library/:id/edit") view=move || view! {
                            <EditLibraryItemForm />
                        } />
                        <Route path=path!("/sessions") view=move || view! {
                            <SessionsListView />
                        } />
                    </Routes>
                </main>

                // Footer
                <AppFooter />
            </div>
        </Router>
    }
}
