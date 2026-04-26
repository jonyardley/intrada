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
