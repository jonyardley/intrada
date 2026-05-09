//! CORS tests for the API router.
//!
//! Verifies that `ALLOWED_ORIGIN` correctly handles single and multi-origin
//! values. Particularly important for the Tauri iOS WebView whose page
//! origin is `tauri://localhost` (not the configured devUrl).

mod common;

use axum::body::Body;
use axum::http::{Method, Request};
use tower::ServiceExt;

use common::setup_test_app_with_origin;

/// Send a request with a given Origin header and return the value of the
/// `Access-Control-Allow-Origin` response header (if any).
async fn cors_origin_for(allowed_origin: &str, request_origin: &str) -> Option<String> {
    let app = setup_test_app_with_origin(allowed_origin).await;
    let request = Request::builder()
        .method("GET")
        .uri("/api/items")
        .header("Origin", request_origin)
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    response
        .headers()
        .get("access-control-allow-origin")
        .map(|v| v.to_str().unwrap().to_string())
}

#[tokio::test]
async fn single_origin_value_is_allowed() {
    let allow = cors_origin_for("http://localhost:8080", "http://localhost:8080").await;
    assert_eq!(allow.as_deref(), Some("http://localhost:8080"));
}

#[tokio::test]
async fn multiple_origins_both_allowed() {
    let allowed = "http://localhost:8080,tauri://localhost";

    let web = cors_origin_for(allowed, "http://localhost:8080").await;
    assert_eq!(web.as_deref(), Some("http://localhost:8080"));

    let ios = cors_origin_for(allowed, "tauri://localhost").await;
    assert_eq!(ios.as_deref(), Some("tauri://localhost"));
}

#[tokio::test]
async fn disallowed_origin_gets_no_cors_header() {
    let allow = cors_origin_for("http://localhost:8080", "https://evil.example.com").await;
    assert!(
        allow.is_none(),
        "Expected no Access-Control-Allow-Origin for disallowed origin, got {allow:?}"
    );
}

#[tokio::test]
async fn whitespace_around_commas_is_trimmed() {
    let allowed = "  http://localhost:8080 , tauri://localhost  ";

    let web = cors_origin_for(allowed, "http://localhost:8080").await;
    assert_eq!(web.as_deref(), Some("http://localhost:8080"));

    let ios = cors_origin_for(allowed, "tauri://localhost").await;
    assert_eq!(ios.as_deref(), Some("tauri://localhost"));
}

#[tokio::test]
async fn empty_segments_are_ignored() {
    // Trailing/leading commas and double commas should not produce empty
    // entries that error out.
    let allowed = ",,http://localhost:8080,,tauri://localhost,,";

    let web = cors_origin_for(allowed, "http://localhost:8080").await;
    assert_eq!(web.as_deref(), Some("http://localhost:8080"));
}

/// Locks in the contract that the permissive `oauth_cors` and `mcp_cors`
/// allow Sentry's W3C `baggage` + `sentry-trace` headers in preflight.
/// Without this, the web app's Sentry browser SDK auto-injects those
/// headers on cross-origin fetches to `intrada-api.fly.dev` (it matches
/// `tracePropagationTargets`), the preflight rejects them, and the
/// browser surfaces "TypeError: Failed to fetch" before the actual
/// request leaves — silently breaking OAuth `/oauth/finalize` and any
/// browser-based MCP client. The strict `/api/*` CORS already had this;
/// this test stops the same regression on the OAuth + MCP surfaces.
async fn assert_preflight_allows_sentry_headers(uri: &str, method: &str) {
    let app = common::setup_test_app().await;
    let request = Request::builder()
        .method(Method::OPTIONS)
        .uri(uri)
        .header("Origin", "https://myintrada.com")
        .header("Access-Control-Request-Method", method)
        .header(
            "Access-Control-Request-Headers",
            "authorization,content-type,baggage,sentry-trace",
        )
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let allow_headers = response
        .headers()
        .get("access-control-allow-headers")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_ascii_lowercase();
    assert!(
        allow_headers.contains("baggage"),
        "preflight for {uri} must allow `baggage` (Sentry distributed tracing); got: {allow_headers}"
    );
    assert!(
        allow_headers.contains("sentry-trace"),
        "preflight for {uri} must allow `sentry-trace` (Sentry distributed tracing); got: {allow_headers}"
    );
}

#[tokio::test]
async fn oauth_preflight_allows_sentry_propagation_headers() {
    // /oauth/finalize is what the web app POSTs from the consent screen.
    // /oauth/token is what claude.ai POSTs (cross-origin from claude.ai).
    assert_preflight_allows_sentry_headers("/oauth/finalize", "POST").await;
    assert_preflight_allows_sentry_headers("/oauth/token", "POST").await;
    assert_preflight_allows_sentry_headers("/oauth/register", "POST").await;
}

#[tokio::test]
async fn mcp_preflight_allows_sentry_propagation_headers() {
    assert_preflight_allows_sentry_headers("/api/mcp", "POST").await;
}

#[tokio::test]
async fn preflight_request_returns_cors_headers_for_allowed_origin() {
    let app = setup_test_app_with_origin("tauri://localhost").await;
    let request = Request::builder()
        .method(Method::OPTIONS)
        .uri("/api/items")
        .header("Origin", "tauri://localhost")
        .header("Access-Control-Request-Method", "POST")
        .header(
            "Access-Control-Request-Headers",
            "content-type,authorization",
        )
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let headers = response.headers();

    assert_eq!(
        headers
            .get("access-control-allow-origin")
            .and_then(|v| v.to_str().ok()),
        Some("tauri://localhost"),
    );
    let allow_methods = headers
        .get("access-control-allow-methods")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(allow_methods.contains("POST"));
}
