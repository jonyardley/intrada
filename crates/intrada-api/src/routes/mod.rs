mod account;
mod health;
mod items;
mod lessons;
mod oauth;
mod sessions;
mod sets;
mod tokens;

use axum::http::{header, HeaderName, HeaderValue, Method, Request};
use axum::Router;
use sentry::integrations::tower::{NewSentryLayer, SentryHttpLayer};
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::{DefaultOnFailure, DefaultOnResponse, TraceLayer};
use tracing::{info_span, Level, Span};

use crate::state::AppState;

pub fn api_router(state: AppState) -> Router {
    // ALLOWED_ORIGIN supports comma-separated values so a single API server
    // can serve multiple frontends. In particular, the Tauri iOS WebView
    // page origin is `tauri://localhost` (not the devUrl), so local dev
    // needs both `http://localhost:8080` (web) and `tauri://localhost` (iOS).
    let origins: Vec<HeaderValue> = state
        .allowed_origin
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<HeaderValue>()
                .unwrap_or_else(|_| panic!("Invalid ALLOWED_ORIGIN value: {s}"))
        })
        .collect();

    // Strict CORS for the user-facing API surface. Browser-based MCP
    // clients use the separate `/api/mcp` route below with its own
    // permissive CORS.
    let strict_cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            // Sentry's browser SDK auto-instruments `fetch` to attach W3C
            // Baggage and Sentry trace headers for distributed tracing.
            // Without these in the allow-list, cross-origin requests fail
            // CORS preflight with "Request header field baggage is not
            // allowed by Access-Control-Allow-Headers." This bites the iOS
            // shell specifically — its WebView origin is `tauri://localhost`
            // (cross-origin to the API), so every request preflights. The
            // web app is same-origin via Trunk's proxy and never preflights,
            // hiding the issue until you ship to iOS.
            HeaderName::from_static("baggage"),
            HeaderName::from_static("sentry-trace"),
        ]);

    // Custom span includes the Origin header so CORS preflight failures
    // are debuggable from logs alone — without it, you see "OPTIONS 200"
    // with no clue what origin the browser sent.
    let make_span = |req: &Request<_>| -> Span {
        let origin = req
            .headers()
            .get("origin")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("-");
        info_span!(
            "request",
            method = %req.method(),
            uri = %req.uri(),
            version = ?req.version(),
            origin = %origin,
        )
    };

    // Permissive CORS for `/api/mcp/*` only. Per #481 — MCP auth is
    // bearer-token-only (PAT), no cookies are involved, so CORS adds zero
    // security on this route while removing a real product capability
    // (browser-based MCP agents). The PAT IS the auth gate.
    let mcp_cors = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods([Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    // Permissive CORS for OAuth endpoints. Same rationale as MCP —
    // cross-origin claude.ai-style flows are the whole point. The
    // discovery doc and DCR/token endpoints don't accept cookies; PKCE
    // provides the security on the public-client OAuth flow.
    let oauth_cors = CorsLayer::new()
        .allow_origin(AllowOrigin::any())
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let trace = TraceLayer::new_for_http()
        .make_span_with(make_span)
        .on_response(DefaultOnResponse::new().level(Level::INFO))
        // Lower the on_failure event (default ERROR → WARN). Without this,
        // every 5xx generates a generic `tracing::error!("response failed")`
        // alongside the actual handler error from `error.rs`, which Sentry
        // captures as a separate, content-free issue. WARN keeps the log
        // line for local visibility but doesn't trip Sentry's default
        // ERROR-only event capture.
        .on_failure(DefaultOnFailure::new().level(Level::WARN));

    // Layer order matters: rate-limit applied first (innermost), CORS
    // wraps it. That way a 429 from the limiter still passes through
    // the CORS layer on its way out and gets `Access-Control-Allow-Origin: *`
    // headers — verified in `tests/rate_limit_test.rs`.
    let mcp_routes = crate::mcp::router()
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::rate_limit::mcp_rate_limit,
        ))
        .layer(mcp_cors);
    let oauth_routes = oauth::router().layer(oauth_cors);

    Router::new()
        // Sibling nests: axum routes by path specificity. The OAuth
        // surface (`.well-known/*` and `/oauth/*`) is rooted because RFC
        // 8414 requires `.well-known/*` at the host root, and the rest
        // of the OAuth endpoints traditionally live there too. Each
        // subtree carries its own CorsLayer.
        .merge(oauth_routes)
        .nest("/api/mcp", mcp_routes)
        .nest("/api", api_routes().layer(strict_cors))
        .layer(trace)
        // Sentry layers: per-request hub + HTTP transaction (route-aware via
        // the `tower-axum-matched-path` feature). NewSentryLayer must wrap
        // SentryHttpLayer so each request gets an isolated hub.
        .layer(SentryHttpLayer::new().enable_transaction())
        .layer(NewSentryLayer::new_from_top())
        .with_state(state)
}

fn api_routes() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .nest("/account", account::router())
        .nest("/items", items::router())
        .nest("/sessions", sessions::router())
        .nest("/sets", sets::router())
        .nest("/lessons", lessons::router())
}
