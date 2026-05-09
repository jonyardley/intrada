use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use intrada_core::{Event, OAuthEvent, OAuthFinalizeParams, ViewModel};
use intrada_web::core_bridge::process_effects;
use intrada_web::types::{IsLoading, IsSubmitting, SharedCore};

use crate::components::{Button, ButtonVariant, Card};

/// OAuth consent screen — the user-facing step of the OAuth 2.1 flow
/// for MCP clients (Phase 5 of #477).
///
/// Reached via redirect from `/oauth/authorize` on the API. Reads the
/// OAuth params from its URL query, shows a consent UI gated by the
/// existing Clerk auth flow, and on Allow dispatches a Crux event that
/// POSTs to `/oauth/finalize` with the user's JWT. On success, the
/// model's `oauth_redirect_url` field surfaces the URL that the
/// browser should navigate to (containing the auth code + state for
/// the OAuth client).
#[component]
pub fn OAuthConsentView() -> impl IntoView {
    let core = expect_context::<SharedCore>();
    let view_model = expect_context::<RwSignal<ViewModel>>();
    let is_loading = expect_context::<IsLoading>();
    let is_submitting = expect_context::<IsSubmitting>();

    // Reset any stale OAuth state from a previous consent attempt so a
    // pre-existing `oauth_redirect_url` doesn't trigger an immediate
    // navigate-on-mount.
    {
        let core = core.clone();
        Effect::new(move |_| {
            let core_ref = core.borrow();
            let effects = core_ref.process_event(Event::OAuth(OAuthEvent::ResetConsent));
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        });
    }

    let query = use_query_map();

    // Pull each param. URL-decoded by leptos_router. Missing required
    // params are treated as a malformed-redirect error.
    let params_signal = Signal::derive(move || {
        query.with(|q| {
            let response_type = q.get("response_type")?;
            let client_id = q.get("client_id")?;
            let redirect_uri = q.get("redirect_uri")?;
            let code_challenge = q.get("code_challenge")?;
            let code_challenge_method = q.get("code_challenge_method")?;
            // `state` and `scope` may be empty strings; we read those
            // back as `None` rather than `Some("")` to keep the
            // round-tripped URL clean.
            let state = q.get("state").filter(|s: &String| !s.is_empty());
            let scope = q.get("scope").filter(|s: &String| !s.is_empty());

            Some(OAuthFinalizeParams {
                response_type,
                client_id,
                redirect_uri,
                state,
                scope,
                code_challenge,
                code_challenge_method,
            })
        })
    });

    let in_flight = Signal::derive(move || view_model.get().oauth_in_flight);
    let redirect_url = Signal::derive(move || view_model.get().oauth_redirect_url);

    // When finalize succeeds the API returns a redirect URL pointing at
    // the OAuth client's redirect_uri (with `?code=…&state=…`). Navigate
    // the browser to it. Use `window.location.assign` so the user can't
    // back-button into the consent page mid-flow.
    Effect::new(move |_| {
        if let Some(url) = redirect_url.get() {
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href(&url);
            }
        }
    });

    // Cancel button: redirect to the OAuth client's redirect_uri with
    // `?error=access_denied&state=…` per RFC 6749 §4.1.2.1. Use `&` as
    // the separator if redirect_uri already contains a query string —
    // mirrors the same fix on the API side in /oauth/finalize.
    let on_cancel = {
        let params = params_signal;
        Callback::new(move |_: leptos::ev::MouseEvent| {
            let Some(p) = params.get() else { return };
            let separator = if p.redirect_uri.contains('?') {
                '&'
            } else {
                '?'
            };
            let mut url = format!("{}{separator}error=access_denied", p.redirect_uri);
            if let Some(s) = p.state {
                url.push_str(&format!("&state={}", urlencode(&s)));
            }
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href(&url);
            }
        })
    };

    let on_allow = {
        let core = core.clone();
        let params = params_signal;
        Callback::new(move |_: leptos::ev::MouseEvent| {
            let Some(p) = params.get() else { return };
            let core_ref = core.borrow();
            let effects = core_ref.process_event(Event::OAuth(OAuthEvent::FinalizeConsent(p)));
            process_effects(&core_ref, effects, &view_model, &is_loading, &is_submitting);
        })
    };

    view! {
        <div class="max-w-md mx-auto py-card-comfortable space-y-section pb-[env(safe-area-inset-bottom)]">
            <Show
                when=move || params_signal.get().is_some()
                fallback=|| {
                    view! {
                        <div class="space-y-card">
                            <h1 class="page-title">"Invalid request"</h1>
                            <p class="text-sm text-secondary">
                                "This page expects OAuth parameters from your AI client. "
                                "If you reached it directly, go back and start the connection from your client."
                            </p>
                        </div>
                    }
                }
            >
                {move || {
                    let p = params_signal.get().expect("guarded by Show.when");
                    let client_id = p.client_id.clone();
                    view! {
                        <div class="space-y-card">
                            <h1 class="page-title">"Connect your AI client"</h1>
                            <p class="text-sm text-secondary">
                                "An AI client wants to act on your Intrada account. "
                                "It will be able to read your library, sessions, and routines, and create or edit items on your behalf."
                            </p>
                        </div>

                        <Card>
                            <div class="space-y-card">
                                <div>
                                    <span class="field-label">"Client"</span>
                                    <code class="block w-full px-card-compact py-card-compact mt-1 rounded-md bg-surface-secondary border border-border-default font-mono text-xs text-primary break-all">
                                        {client_id}
                                    </code>
                                </div>
                                <div>
                                    <span class="field-label">"Will redirect to"</span>
                                    <code class="block w-full px-card-compact py-card-compact mt-1 rounded-md bg-surface-secondary border border-border-default font-mono text-xs text-primary break-all">
                                        {p.redirect_uri.clone()}
                                    </code>
                                </div>
                            </div>
                        </Card>

                        <p class="hint-text">
                            "Allowing creates a new MCP token attributed to this client. "
                            "You can revoke it at any time from MCP tokens in Settings."
                        </p>

                        <div class="flex flex-col gap-card-compact">
                            <Button
                                variant=ButtonVariant::Primary
                                full_width=true
                                on_click=on_allow
                                loading=in_flight
                            >
                                "Allow"
                            </Button>
                            <Button
                                variant=ButtonVariant::Secondary
                                full_width=true
                                on_click=on_cancel
                                disabled=in_flight
                            >
                                "Cancel"
                            </Button>
                        </div>
                    }
                }}
            </Show>
        </div>
    }
}

fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
