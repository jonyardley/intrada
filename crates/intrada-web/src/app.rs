use std::cell::RefCell;
use std::rc::Rc;

use crux_core::Core;
use leptos::prelude::*;
use leptos_router::components::{Redirect, Route, Router, Routes};
use leptos_router::hooks::use_navigate;
use leptos_router::path;
use leptos_router::NavigateOptions;
use send_wrapper::SendWrapper;
use wasm_bindgen::prelude::*;

use intrada_core::{Event, Intrada, SessionEvent, ViewModel};

use crate::components::welcome_carousel::welcome_already_seen;
use crate::components::{
    provide_toast, AppFooter, AppHeader, BottomTabBar, ErrorBanner, ToastStack, WelcomeCarousel,
};
#[cfg(debug_assertions)]
use crate::views::DesignCatalogue;
use crate::views::{
    AccountDeleteView, AddLibraryItemForm, AnalyticsPage, DetailView, EditLibraryItemForm,
    LibraryListView, LoginView, McpTokensView, NotFoundView, SessionActiveView, SessionNewView,
    SessionSummaryView, SessionsAllView, SessionsListView, SetDetailView, SetEditView,
    SettingsView, WelcomeView,
};
use intrada_web::core_bridge::{init_core, load_session_in_progress, process_effects};
use intrada_web::js_bridge;
use intrada_web::session_lifecycle;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

/// App-level signal for focus mode — when true, navigation and non-essential UI are hidden.
///
/// Newtype wrapper so `provide_context` doesn't collide with other `RwSignal<bool>` contexts.
#[derive(Clone, Copy)]
pub struct FocusMode(pub RwSignal<bool>);

impl FocusMode {
    pub fn get(&self) -> bool {
        self.0.get()
    }
    pub fn set(&self, val: bool) {
        self.0.set(val);
    }
}

/// Auth state shared across the app. Provided as context at the App level so
/// any view (public or private) can read whether the user is signed in.
#[derive(Clone, Copy)]
pub struct AuthState {
    pub is_authenticated: RwSignal<bool>,
    pub auth_loading: RwSignal<bool>,
    pub auth_error: RwSignal<bool>,
}

#[component]
pub fn App() -> impl IntoView {
    // Auth state signals — drive the auth gate
    let auth = AuthState {
        is_authenticated: RwSignal::new(false),
        auth_loading: RwSignal::new(true),
        auth_error: RwSignal::new(false),
    };
    let AuthState {
        is_authenticated,
        auth_loading,
        auth_error,
    } = auth;

    // Initialize Clerk
    js_bridge::init_clerk();

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
                if js_bridge::is_signed_in() {
                    if let Some(id) = js_bridge::get_user_id() {
                        js_bridge::sentry_set_user(&id);
                    }
                    // No breadcrumb here — Clerk's `addListener` fires
                    // immediately on subscribe with the current state, so the
                    // listener path below catches both fresh and warm sign-ins
                    // and emits the breadcrumb there. Emitting here too would
                    // double-fire on every load.
                    is_authenticated.set(true);
                    auth_loading.set(false);
                    return;
                }
                if js_bridge::init_failed() {
                    // Clerk failed to init (bad key, wrong domain, etc.)
                    auth_error.set(true);
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
            let signed_in = js_bridge::is_signed_in();
            if signed_in {
                if let Some(id) = js_bridge::get_user_id() {
                    js_bridge::sentry_set_user(&id);
                }
                js_bridge::sentry_breadcrumb("auth", "signed-in", "info");
            } else {
                js_bridge::sentry_clear_user();
                js_bridge::sentry_breadcrumb("auth", "signed-out", "info");
            }
            is_authenticated.set(signed_in);
            auth_loading.set(false);
        });
        js_bridge::add_auth_listener(&closure);
        closure.forget(); // leak intentionally — lives for app lifetime
    }

    // ─── Crux core + app-level reactive state ─────────────────────────
    // Mounted once at the App level (was previously inside the
    // `AuthenticatedApp` wrapper). Keeping it here means navigating between
    // public (`/`, `/login`) and private routes doesn't re-init the core or
    // drop the in-memory view_model. Data fetches still gate on auth via
    // the Effect below.
    let core: SharedCore = SendWrapper::new(Rc::new(RefCell::new(Core::<Intrada>::new())));
    let view_model = RwSignal::new(ViewModel::default());
    let is_loading = IsLoading::new(true);
    let is_submitting = IsSubmitting::new(false);
    let focus_mode = FocusMode(RwSignal::new(false));

    provide_context(auth);
    provide_context(core.clone());
    provide_context(view_model);
    provide_context(is_loading);
    provide_context(is_submitting);
    provide_context(focus_mode);
    provide_toast();

    // Session-lifecycle Effect (#309 Phase D + #474 Phase B). Drives
    // both the background-audio plugin (lock-screen Now Playing /
    // AVAudioSession) and the live-activity plugin (Lock Screen +
    // Dynamic Island) from the same vm.active_session transitions.
    // Mounted at the app level so Some → None fires the end calls even
    // when the user navigates away from /sessions/active before
    // finishing — e.g. backing to home, switching tabs, or hitting
    // "Discard Session" from /sessions/new. Mounting inside
    // <SessionTimer> would leak both plugins.
    session_lifecycle::mount_session_lifecycle(view_model);

    // Initialise core data when (and only when) the user is authenticated.
    // Public routes don't trigger this. The `initialized` flag prevents the
    // Effect from re-running on subsequent reactive ticks. On sign-out the
    // flag resets so a subsequent sign-in (same browser session) re-fetches
    // — without this, signing back in after a logout would render stale
    // data from the previous session.
    let initialized = RwSignal::new(false);
    let core_for_init = core.clone();
    Effect::new(move |_| {
        let authed = auth.is_authenticated.get();
        if authed && !initialized.get_untracked() {
            init_core(&view_model, &is_loading, &is_submitting);

            // Recover any in-progress session from localStorage (FR-008).
            if let Some(session) = load_session_in_progress() {
                let core_ref = core_for_init.borrow();
                let effects = core_ref
                    .process_event(Event::Session(SessionEvent::RecoverSession { session }));
                process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
            }

            initialized.set(true);
        } else if !authed && initialized.get_untracked() {
            initialized.set(false);
        }
    });

    view! {
        <Router>
            // Fixed gradient background — stays behind all content, does not scroll
            <div class="fixed inset-0 -z-10 bg-linear-to-b from-[var(--color-bg-gradient-top)] to-[var(--color-bg-gradient-bottom)]"></div>

            <Show
                when=move || auth_loading.get()
                fallback=move || view! { <AppRoutes /> }
            >
                <AuthLoadingScreen />
            </Show>
        </Router>
    }
}

/// Routes are split into public (`/`, `/login`) and private (everything
/// else, wrapped in `AuthenticatedShell`). The shell renders the chrome
/// (header, footer, tab bar, welcome carousel) and redirects to `/login`
/// if the user isn't authenticated.
#[component]
fn AppRoutes() -> impl IntoView {
    view! {
        <Routes transition=true fallback=|| view! { <NotFoundView /> }>
            // ─── Public routes ────────────────────────────────────────
            <Route path=path!("/") view=|| view! { <WelcomeView /> } />
            <Route path=path!("/login") view=|| view! { <LoginView /> } />

            // ─── Private routes ───────────────────────────────────────
            // /library/new MUST come before /library/:id to avoid "new"
            // matching :id.
            <Route path=path!("/library") view=|| view! {
                <AuthenticatedShell><LibraryListView /></AuthenticatedShell>
            } />
            <Route path=path!("/library/new") view=|| view! {
                <AuthenticatedShell><AddLibraryItemForm /></AuthenticatedShell>
            } />
            // /library/sets/:id — Set Detail. Literal "sets" segment so it
            // doesn't collide with /library/:id (piece/exercise detail).
            <Route path=path!("/library/sets/:id") view=|| view! {
                <AuthenticatedShell><SetDetailView /></AuthenticatedShell>
            } />
            <Route path=path!("/library/:id") view=|| view! {
                <AuthenticatedShell><DetailView /></AuthenticatedShell>
            } />
            <Route path=path!("/library/:id/edit") view=|| view! {
                <AuthenticatedShell><EditLibraryItemForm /></AuthenticatedShell>
            } />
            <Route path=path!("/sessions") view=|| view! {
                <AuthenticatedShell><SessionsListView /></AuthenticatedShell>
            } />
            <Route path=path!("/sessions/all") view=|| view! {
                <AuthenticatedShell><SessionsAllView /></AuthenticatedShell>
            } />
            <Route path=path!("/sessions/new") view=|| view! {
                <AuthenticatedShell><SessionNewView /></AuthenticatedShell>
            } />
            <Route path=path!("/sessions/active") view=|| view! {
                <AuthenticatedShell><SessionActiveView /></AuthenticatedShell>
            } />
            <Route path=path!("/sessions/summary") view=|| view! {
                <AuthenticatedShell><SessionSummaryView /></AuthenticatedShell>
            } />
            // /routines folded into Library (Sets type-tab). Legacy URL
            // redirects to the right tab.
            <Route path=path!("/routines") view=|| view! {
                <Redirect
                    path="/library?type=set"
                    options=NavigateOptions { replace: true, ..Default::default() }
                />
            } />
            <Route path=path!("/routines/:id/edit") view=|| view! {
                <AuthenticatedShell><SetEditView /></AuthenticatedShell>
            } />
            <Route path=path!("/analytics") view=|| view! {
                <AuthenticatedShell><AnalyticsPage /></AuthenticatedShell>
            } />
            <Route path=path!("/design") view=|| view! {
                <AuthenticatedShell><DesignRouteView /></AuthenticatedShell>
            } />
            <Route path=path!("/settings") view=|| view! {
                <AuthenticatedShell><SettingsView /></AuthenticatedShell>
            } />
            <Route path=path!("/settings/delete-account") view=|| view! {
                <AuthenticatedShell><AccountDeleteView /></AuthenticatedShell>
            } />
            <Route path=path!("/settings/mcp-tokens") view=|| view! {
                <AuthenticatedShell><McpTokensView /></AuthenticatedShell>
            } />
        </Routes>
    }
}

/// Wraps a private route's view with auth-gate + chrome (header, footer,
/// bottom tab bar, welcome carousel). Mounts per route navigation; the
/// underlying Crux core / view_model contexts are provided at App level
/// so this remount is cheap.
#[component]
fn AuthenticatedShell(children: Children) -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let focus_mode = expect_context::<FocusMode>();

    // Welcome carousel — show for first-time users (localStorage gate).
    // Re-evaluates on each shell mount; once dismissed, the localStorage flag
    // is set so future mounts see false.
    let show_welcome = RwSignal::new(!welcome_already_seen());

    // Auth gate: redirect to /login if not authed.
    Effect::new(move |_| {
        if !auth.is_authenticated.get() {
            let navigate = use_navigate();
            navigate(
                "/login",
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
        }
    });

    // Skip rendering the private chrome + children when unauthed at mount
    // time. The Effect above handles the actual redirect; rendering an
    // empty placeholder while it fires keeps deep-linked unauthed users
    // from briefly seeing AppHeader / data-fetching private views.
    // `get_untracked` so we don't subscribe — sign-in/out flows trigger a
    // route change which unmounts this shell anyway.
    if !auth.is_authenticated.get_untracked() {
        return view! { <div></div> }.into_any();
    }

    view! {
        <div class="relative z-0 min-h-screen text-primary">
            // Welcome carousel overlay — shown once for first-time users.
            <Show when=move || show_welcome.get()>
                <WelcomeCarousel show=show_welcome />
            </Show>

            // Header — hidden in focus mode
            <Show when=move || !focus_mode.get()>
                <AppHeader />
            </Show>

            // Main content
            <main
                class=move || if focus_mode.get() {
                    "focus-mode-container"
                } else {
                    "max-w-4xl mx-auto px-4 sm:px-6 py-6 sm:py-10 pb-20 sm:pb-10"
                }
                role="main"
            >
                <ErrorBanner />
                <ToastStack />
                {children()}
            </main>

            // Footer — hidden in focus mode
            <Show when=move || !focus_mode.get()>
                <AppFooter />
            </Show>

            // Mobile bottom tab bar (hidden on sm: and wider) — hidden in focus mode
            <Show when=move || !focus_mode.get()>
                <BottomTabBar />
            </Show>
        </div>
    }
    .into_any()
}

/// Loading screen shown while Clerk initializes.
#[component]
fn AuthLoadingScreen() -> impl IntoView {
    view! {
        <div class="relative z-0 min-h-screen text-primary flex items-center justify-center">
            <div class="text-center">
                <h1 class="page-title mb-2">"Intrada"</h1>
                <p class="text-muted">"Loading..."</p>
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
