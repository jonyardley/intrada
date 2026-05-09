use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::app::AuthState;

/// Landing route for the Clerk OAuth redirect — `/sso-callback`.
///
/// The actual callback is processed by the inline JS bridge in `index.html`
/// (it calls `clerk.handleRedirectCallback` once the SDK finishes loading
/// from the CDN, then navigates to `/`). On slow networks that download +
/// callback round-trip can take several seconds, during which Leptos would
/// otherwise render `NotFoundView` because no route matched. This view
/// stands in with a friendly "Signing you in…" state until either the
/// auth listener flips `is_authenticated` true (defence-in-depth: we then
/// push to `/library` ourselves, in case Clerk's own redirect doesn't
/// fire), the inline script reports an init failure, or 15s pass with no
/// progress.
#[component]
pub fn SsoCallbackView() -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let timed_out = RwSignal::new(false);
    // No callback params at all → the user reached `/sso-callback` outside
    // the OAuth flow (manual URL entry, deep link, refresh after success).
    // Skip the 15s wait and surface a clearer message immediately.
    let no_callback_params = web_sys::window()
        .and_then(|w| w.location().search().ok())
        .map(|s| s.is_empty())
        .unwrap_or(false);

    // Effect runs on every signal tick; guard so we only fire `navigate`
    // once even if `is_authenticated` flickers (Clerk's listener can fire
    // multiple times around session establishment).
    let navigated = RwSignal::new(false);
    Effect::new(move |_| {
        if auth.is_authenticated.get() && !navigated.get_untracked() {
            navigated.set(true);
            let navigate = use_navigate();
            navigate(
                "/library",
                NavigateOptions {
                    replace: true,
                    ..Default::default()
                },
            );
        }
    });

    // Timeout handle stored in a signal so on_cleanup can cancel it. Without
    // this the closure runs after the view is disposed and writes to a dead
    // `timed_out` signal.
    let timeout_handle: RwSignal<Option<i32>> = RwSignal::new(None);
    on_cleanup(move || {
        if let Some(handle) = timeout_handle.get_untracked() {
            if let Some(window) = web_sys::window() {
                window.clear_timeout_with_handle(handle);
            }
        }
    });

    if !no_callback_params {
        if let Some(window) = web_sys::window() {
            let cb: Closure<dyn FnMut()> = Closure::new(move || {
                if !auth.is_authenticated.get_untracked() {
                    timed_out.set(true);
                }
            });
            if let Ok(id) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                15_000,
            ) {
                timeout_handle.set(Some(id));
            }
            cb.forget();
        }
    }

    view! {
        <div class="relative z-0 min-h-screen text-primary flex items-center justify-center px-4 pt-safe pb-safe">
            <div class="text-center max-w-sm" aria-live="polite">
                <div class="flex items-center justify-center gap-2.5 mb-4">
                    <svg class="w-7 h-7 text-accent" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M9 9l10.5-3m0 6.553v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 11-.99-3.467l2.31-.66a2.25 2.25 0 001.632-2.163zm0 0V2.25L9 5.25v10.303m0 0v3.75a2.25 2.25 0 01-1.632 2.163l-1.32.377a1.803 1.803 0 01-.99-3.467l2.31-.66A2.25 2.25 0 009 15.553z" />
                    </svg>
                    <h1 class="page-title">"Intrada"</h1>
                </div>
                <Show
                    when=move || !auth.auth_error.get() && !timed_out.get() && !no_callback_params
                    fallback=move || view! {
                        <p class="text-secondary mb-6" role="alert">
                            {move || if no_callback_params {
                                "This page is part of signing in. Take me back to sign in."
                            } else {
                                "We couldn't finish signing you in. Please try again."
                            }}
                        </p>
                        <A
                            href="/login"
                            attr:class="inline-flex items-center gap-2 text-accent-text hover:text-accent-hover font-medium"
                        >
                            "Back to sign in"
                        </A>
                    }
                >
                    <div class="flex items-center justify-center gap-3" role="status">
                        <span class="animate-spin rounded-full h-5 w-5 border-2 border-accent-focus border-t-transparent" aria-hidden="true"></span>
                        <p class="text-muted">"Signing you in…"</p>
                    </div>
                </Show>
            </div>
        </div>
    }
}
