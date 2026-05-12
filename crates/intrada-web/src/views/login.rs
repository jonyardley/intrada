use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use leptos_router::NavigateOptions;

use crate::components::{BrandMark, BrandMarkSize, Button, ButtonSize, ButtonVariant};
use intrada_web::js_bridge;

use crate::app::AuthState;

/// Dedicated sign-in route at `/login`.
///
/// Used by returning users, "sign in" links from the marketing homepage,
/// session-expiry redirects, and the iOS app on launch (which skips the
/// marketing homepage entirely — see `WelcomeView`).
///
/// Renders a centred whisper-soft card on the gradient background with a
/// single primary-indigo "Sign in with Google" CTA. Matches Pencil node
/// `Me9nU`.
///
/// On mount, if the user is already authenticated, redirects to /library.
#[component]
pub fn LoginView() -> impl IntoView {
    let auth = expect_context::<AuthState>();
    let signing_in = RwSignal::new(false);

    // Redirect signed-in users away — they shouldn't see the login screen.
    Effect::new(move |_| {
        if auth.is_authenticated.get() {
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

    let sign_in_error = RwSignal::new(None::<String>);

    let on_sign_in = Callback::new(move |_| {
        signing_in.set(true);
        sign_in_error.set(None);
        leptos::task::spawn_local(async move {
            js_bridge::sign_in_with_google().await;
            // If we reach here, the redirect didn't happen — something failed.
            // On success the page navigates away and this code never runs.
            signing_in.set(false);
            if let Some(err) = js_bridge::init_error() {
                sign_in_error.set(Some(err));
            } else {
                sign_in_error.set(Some("Sign-in failed. Please try again.".into()));
            }
        });
    });

    let signing_in_signal = signing_in.read_only();
    let auth_error_signal = auth.auth_error;
    let disabled_signal = Signal::derive(move || signing_in.get() || auth_error_signal.get());

    view! {
        <div class="relative z-0 min-h-screen text-primary flex flex-col items-center justify-center px-4 pt-safe pb-safe">
            // Brand wordmark — small, sits above the card to anchor the page
            <A href="/" attr:class="mb-10 no-underline">
                <BrandMark size=BrandMarkSize::Md />
            </A>

            <div class="card p-8 sm:p-10 max-w-sm w-full text-center">
                <h1 class="page-title mb-2">"Welcome"</h1>
                <p class="text-muted mb-8">"Sign in to continue your practice."</p>

                <Show when=move || auth_error_signal.get()>
                    <p class="text-danger-text text-sm mb-4">
                        "Sign-in is temporarily unavailable. Please try again later."
                    </p>
                    {move || auth.auth_error_detail.get().map(|detail| view! {
                        <p class="text-muted text-xs mb-4 font-mono break-all">{detail}</p>
                    })}
                </Show>

                {move || sign_in_error.get().map(|err| view! {
                    <p class="text-danger-text text-sm mb-4 font-mono break-all">{err}</p>
                })}

                <Button
                    variant=ButtonVariant::Primary
                    size=ButtonSize::Hero
                    full_width=true
                    on_click=on_sign_in
                    disabled=disabled_signal
                    loading=signing_in_signal
                >
                    // Hide the Google glyph during the loading window so the
                    // Button's spinner doesn't compete with it side-by-side.
                    <Show when=move || !signing_in.get()>
                        <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                            <path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z" fill="#4285F4"/>
                            <path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z" fill="#34A853"/>
                            <path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z" fill="#FBBC05"/>
                            <path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z" fill="#EA4335"/>
                        </svg>
                    </Show>
                    {move || if signing_in.get() { "Signing in..." } else { "Sign in with Google" }}
                </Button>

                <p class="text-faint text-xs mt-6 leading-relaxed">
                    "By continuing you agree to our Terms & Privacy Policy."
                </p>
            </div>
        </div>
    }
}
