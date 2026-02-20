use std::cell::RefCell;
use std::rc::Rc;

use crux_core::Core;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use send_wrapper::SendWrapper;
use wasm_bindgen::prelude::*;

use intrada_core::{Event, Intrada, SessionEvent, ViewModel};

use crate::components::{AppFooter, AppHeader, BottomTabBar, ErrorBanner};
#[cfg(debug_assertions)]
use crate::views::DesignCatalogue;
use crate::views::{
    AddLibraryItemForm, AnalyticsPage, DetailView, EditLibraryItemForm, LibraryListView,
    NotFoundView, RoutineEditView, RoutinesListView, SessionActiveView, SessionNewView,
    SessionSummaryView, SessionsListView,
};
use intrada_web::clerk_bindings;
use intrada_web::core_bridge::{fetch_initial_data, load_session_in_progress, process_effects};
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

#[component]
pub fn App() -> impl IntoView {
    // Auth state signal — drives the auth gate
    let is_authenticated = RwSignal::new(false);
    let auth_loading = RwSignal::new(true);

    // Initialize Clerk
    clerk_bindings::init_clerk();

    // Poll for Clerk readiness, then set auth state
    {
        leptos::task::spawn_local(async move {
            // Give Clerk a moment to initialize, then check status
            // We poll a few times since Clerk loads async from CDN
            let has_key = option_env!("CLERK_PUBLISHABLE_KEY")
                .map(|k| !k.is_empty())
                .unwrap_or(false);
            if !has_key {
                // No Clerk key — skip auth gate entirely (dev mode)
                is_authenticated.set(true);
                auth_loading.set(false);
                return;
            }

            for _ in 0..50 {
                gloo_timers::future::TimeoutFuture::new(100).await;
                if clerk_bindings::is_signed_in() {
                    is_authenticated.set(true);
                    auth_loading.set(false);
                    return;
                }
                if clerk_bindings::init_failed() {
                    // Clerk failed to init (bad key, wrong domain, etc.)
                    // Bypass auth so the app is still usable.
                    is_authenticated.set(true);
                    auth_loading.set(false);
                    return;
                }
            }
            // After 5 seconds, Clerk has loaded but user is not signed in
            auth_loading.set(false);
        });
    }

    // Listen for auth state changes after initial load
    {
        let closure = Closure::new(move || {
            let signed_in = clerk_bindings::is_signed_in();
            is_authenticated.set(signed_in);
            auth_loading.set(false);
        });
        clerk_bindings::add_auth_listener(&closure);
        closure.forget(); // leak intentionally — lives for app lifetime
    }

    view! {
        <Router>
            // Fixed gradient background — stays behind all content, does not scroll
            <div class="fixed inset-0 -z-10 bg-linear-to-br from-gray-950 via-indigo-950 to-purple-950"></div>

            <Show
                when=move || auth_loading.get()
                fallback=move || {
                    view! {
                        <Show
                            when=move || is_authenticated.get()
                            fallback=move || view! { <SignInScreen /> }
                        >
                            <AuthenticatedApp />
                        </Show>
                    }
                }
            >
                <AuthLoadingScreen />
            </Show>
        </Router>
    }
}

/// The main authenticated application — only rendered when signed in.
#[component]
fn AuthenticatedApp() -> impl IntoView {
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
                    <Route path=path!("/routines") view=move || view! {
                        <RoutinesListView />
                    } />
                    <Route path=path!("/routines/:id/edit") view=move || view! {
                        <RoutineEditView />
                    } />
                    <Route path=path!("/analytics") view=move || view! {
                        <AnalyticsPage />
                    } />
                    <Route path=path!("/design") view=move || view! {
                        <DesignRouteView />
                    } />
                </Routes>
            </main>

            // Footer
            <AppFooter />

            // Mobile bottom tab bar (hidden on sm: and wider)
            <BottomTabBar />
        </div>
    }
}

/// Loading screen shown while Clerk initializes.
#[component]
fn AuthLoadingScreen() -> impl IntoView {
    view! {
        <div class="relative z-0 min-h-screen text-white flex items-center justify-center">
            <div class="text-center">
                <h1 class="text-3xl font-bold tracking-tight mb-2">"Intrada"</h1>
                <p class="text-gray-400">"Loading..."</p>
            </div>
        </div>
    }
}

/// Sign-in screen shown when user is not authenticated.
#[component]
fn SignInScreen() -> impl IntoView {
    let signing_in = RwSignal::new(false);

    let on_sign_in = move |_| {
        signing_in.set(true);
        leptos::task::spawn_local(async move {
            clerk_bindings::sign_in_with_google().await;
            // Redirect will happen — no need to update state
        });
    };

    view! {
        <div class="relative z-0 min-h-screen text-white flex items-center justify-center px-4">
            <div class="glass-chrome rounded-2xl p-8 sm:p-12 max-w-sm w-full text-center">
                <h1 class="text-3xl sm:text-4xl font-bold tracking-tight mb-2">"Intrada"</h1>
                <p class="text-gray-400 mb-8">"Your music practice companion"</p>

                <button
                    on:click=on_sign_in
                    disabled=move || signing_in.get()
                    class="w-full flex items-center justify-center gap-3 px-6 py-3 rounded-xl
                           bg-white/10 hover:bg-white/15 border border-white/20
                           text-white font-medium transition-all duration-200
                           disabled:opacity-50 disabled:cursor-not-allowed"
                    aria-label="Sign in with Google"
                >
                    <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z" fill="#4285F4"/>
                        <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/>
                        <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/>
                        <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/>
                    </svg>
                    {move || if signing_in.get() { "Signing in..." } else { "Sign in with Google" }}
                </button>
            </div>
        </div>
    }
}

/// Design catalogue route — shows the component catalogue in debug builds,
/// falls back to 404 in release builds.
#[component]
fn DesignRouteView() -> impl IntoView {
    #[cfg(debug_assertions)]
    {
        view! { <DesignCatalogue /> }.into_any()
    }
    #[cfg(not(debug_assertions))]
    {
        view! { <NotFoundView /> }.into_any()
    }
}
