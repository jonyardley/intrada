use std::cell::RefCell;
use std::rc::Rc;

use crux_core::Core;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use send_wrapper::SendWrapper;

use intrada_core::{Event, Intrada, SessionEvent, ViewModel};

use crate::components::{AppFooter, AppHeader, BottomTabBar};
use crate::views::{
    AddLibraryItemForm, DetailView, EditLibraryItemForm, LibraryListView, NotFoundView,
    SessionActiveView, SessionNewView, SessionSummaryView, SessionsListView,
};
use intrada_web::core_bridge::{
    load_library_data, load_session_in_progress, load_sessions_data, process_effects,
};
use intrada_web::types::SharedCore;

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

        // Recover any in-progress session from localStorage (crash recovery)
        if let Some(session) = load_session_in_progress() {
            let effects =
                core_ref.process_event(Event::Session(SessionEvent::RecoverSession { session }));
            process_effects(&core_ref, effects, &view_model);
        }
    }

    // Provide core and view_model via Leptos context so child components
    // can access them with use_context() instead of prop drilling.
    provide_context(core);
    provide_context(view_model);

    view! {
        <Router>
            // Fixed gradient background — stays behind all content, does not scroll
            <div class="fixed inset-0 -z-10 bg-linear-to-br from-gray-950 via-indigo-950 to-purple-950"></div>

            <div class="relative z-0 min-h-screen text-white">
                // Header
                <AppHeader />

                // Main content — routed by URL
                <main class="max-w-4xl mx-auto px-4 sm:px-6 py-6 sm:py-10 pb-20 sm:pb-10" role="main">
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
                        <Route path=path!("/sessions/new") view=move || view! {
                            <SessionNewView />
                        } />
                        <Route path=path!("/sessions/active") view=move || view! {
                            <SessionActiveView />
                        } />
                        <Route path=path!("/sessions/summary") view=move || view! {
                            <SessionSummaryView />
                        } />
                    </Routes>
                </main>

                // Footer
                <AppFooter />

                // Mobile bottom tab bar (hidden on sm: and wider)
                <BottomTabBar />
            </div>
        </Router>
    }
}
