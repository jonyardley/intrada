use std::cell::RefCell;
use std::rc::Rc;

use crux_core::Core;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use send_wrapper::SendWrapper;

use intrada_core::{Event, Intrada, SessionEvent, ViewModel};

use crate::components::{AppFooter, AppHeader, BottomTabBar, ErrorBanner};
use crate::views::{
    AddLibraryItemForm, AnalyticsPage, DetailView, EditLibraryItemForm, LibraryListView,
    NotFoundView, SessionActiveView, SessionNewView, SessionSummaryView, SessionsListView,
};
use intrada_web::core_bridge::{fetch_initial_data, load_session_in_progress, process_effects};
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

#[component]
pub fn App() -> impl IntoView {
    let core: SharedCore = SendWrapper::new(Rc::new(RefCell::new(Core::<Intrada>::new())));
    let view_model = RwSignal::new(ViewModel::default());
    let is_loading: IsLoading = RwSignal::new(false);
    let is_submitting: IsSubmitting = RwSignal::new(false);

    // Provide context BEFORE init so process_effects can use expect_context
    provide_context(core.clone());
    provide_context(view_model);
    provide_context(is_loading);
    provide_context(is_submitting);

    // Initialize: fetch data from API and recover any in-progress session
    {
        // Spawn async HTTP fetches for library data and sessions
        fetch_initial_data(&view_model, &is_loading, &is_submitting);

        // Recover any in-progress session from localStorage (crash recovery — FR-008)
        if let Some(session) = load_session_in_progress() {
            let core_ref = core.borrow();
            let effects =
                core_ref.process_event(Event::Session(SessionEvent::RecoverSession { session }));
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        }
    }

    view! {
        <Router>
            // Fixed gradient background — stays behind all content, does not scroll
            <div class="fixed inset-0 -z-10 bg-linear-to-br from-gray-950 via-indigo-950 to-purple-950"></div>

            <div class="relative z-0 min-h-screen text-white">
                // Header
                <AppHeader />

                // Main content — routed by URL
                <main class="max-w-4xl mx-auto px-4 sm:px-6 py-6 sm:py-10 pb-20 sm:pb-10" role="main">
                    // Global error banner
                    <ErrorBanner />

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
                        <Route path=path!("/analytics") view=move || view! {
                            <AnalyticsPage />
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
