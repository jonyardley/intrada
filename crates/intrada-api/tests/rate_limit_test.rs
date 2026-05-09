//! Integration tests for the MCP rate limiter (Phase 6A of #477).
//!
//! Sibling unit tests live in `intrada-api/src/rate_limit.rs`; these
//! cover the wiring: middleware position, bypass logic, CORS-on-429,
//! end-to-end against `setup_test_app_with_rate_limit`.

mod common;

use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tower::ServiceExt;

/// Helper: build an MCP `tools/list` JSON-RPC request authed with the
/// given Bearer token (or no Authorization header when `token` is None).
fn mcp_request(token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder()
        .method("POST")
        .uri("/api/mcp")
        .header("content-type", "application/json")
        // `Origin` triggers the CORS layer to attach
        // `Access-Control-Allow-Origin` to the response — needed for the
        // 429-still-has-CORS-headers assertion.
        .header("origin", "https://claude.ai");
    if let Some(t) = token {
        builder = builder.header("authorization", format!("Bearer {t}"));
    }
    builder
        .body(Body::from(
            serde_json::to_string(&json!({
                "jsonrpc": "2.0",
                "method": "tools/list",
                "id": 1
            }))
            .unwrap(),
        ))
        .unwrap()
}

#[tokio::test]
async fn pat_rate_limit_returns_429_with_retry_after() {
    // limit=2 so we don't have to send 60 requests; window=60s so the
    // bucket doesn't reset mid-test under CI load.
    let app = common::setup_test_app_with_rate_limit(2, Duration::from_secs(60)).await;

    // Mint a PAT via the auth-disabled `/api/account/tokens` endpoint
    // (same pattern as `mcp_test::pat_authenticates_mcp_call`).
    let (_, body) = common::post_json(
        app.clone(),
        "/api/account/tokens",
        json!({"name": "rate-limit-test"}),
    )
    .await;
    let v: Value = common::json(&body);
    let token = v["token"].as_str().unwrap().to_string();

    // First two MCP calls should succeed.
    for i in 1..=2 {
        let resp = app
            .clone()
            .oneshot(mcp_request(Some(&token)))
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "request {i} should succeed within bucket"
        );
    }

    // Third call should be rate-limited.
    let resp = app.oneshot(mcp_request(Some(&token))).await.unwrap();
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

    let retry_after = resp
        .headers()
        .get("retry-after")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default();
    let retry_secs: u64 = retry_after.parse().expect("retry-after must be integer");
    assert!(
        (1..=60).contains(&retry_secs),
        "retry-after should be in (0, window]; got {retry_secs}"
    );

    // CORS headers must still be on the 429 — verifies the rate-limit
    // layer is wrapped by the CORS layer (rate-limit is innermost).
    assert_eq!(
        resp.headers()
            .get("access-control-allow-origin")
            .and_then(|v| v.to_str().ok()),
        Some("*"),
        "429 responses must carry CORS headers, otherwise browser MCP \
         clients see a CORS error instead of the actual rate-limit signal"
    );
}

#[tokio::test]
async fn auth_disabled_calls_bypass_rate_limit() {
    // No CLERK_ISSUER_URL → AuthSource::Disabled. These calls don't
    // attribute to a token_id and must bypass the limiter entirely.
    // (JWT bypass exercises the same code path — there's no PAT to
    // resolve, so middleware passes through.)
    let app = common::setup_test_app_with_rate_limit(2, Duration::from_secs(60)).await;

    for i in 1..=10 {
        let resp = app.clone().oneshot(mcp_request(None)).await.unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "unauthed-but-disabled-mode request {i} should pass through"
        );
    }
}

#[tokio::test]
async fn invalid_pat_does_not_consume_bucket_for_valid_one() {
    // A flood of malformed/unknown PATs from one client must not exhaust
    // the bucket of a legitimate token — the bucket is keyed on
    // resolved `token_id`, so unresolved PATs simply pass through to
    // the handler-level extractor (which 401s). Verifies that the
    // first 100 garbage requests don't impact the legitimate token's
    // budget.
    let app = common::setup_test_app_with_rate_limit(2, Duration::from_secs(60)).await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/account/tokens",
        json!({"name": "real-token"}),
    )
    .await;
    let v: Value = common::json(&body);
    let valid_token = v["token"].as_str().unwrap().to_string();

    let bogus = "intrada_pat_0000000000000000000000000000000000000000000000000000000000000000";
    for _ in 0..100 {
        let _ = app.clone().oneshot(mcp_request(Some(bogus))).await.unwrap();
    }

    // Real token's bucket should still be intact — 2 OKs.
    for i in 1..=2 {
        let resp = app
            .clone()
            .oneshot(mcp_request(Some(&valid_token)))
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "valid token's budget {i} should not be consumed by bogus-token flood"
        );
    }
}

#[tokio::test]
async fn revoked_pat_does_not_consume_bucket() {
    // Locks in the contract that the middleware swallows
    // `resolve_pat` errors (revoked token → Err(Unauthorized)) and
    // passes the request through untouched. If a future change ever
    // started charging unresolved-PAT requests against a bucket, the
    // legitimate token below would lose its budget — this test catches
    // that regression. The 401 itself is produced by the handler-level
    // `AuthUser` extractor, not the middleware.
    let app = common::setup_test_app_with_rate_limit(2, Duration::from_secs(60)).await;

    // Mint a token, then revoke it.
    let (_, body) = common::post_json(
        app.clone(),
        "/api/account/tokens",
        json!({"name": "to-be-revoked"}),
    )
    .await;
    let v: Value = common::json(&body);
    let revoked_id = v["id"].as_str().unwrap().to_string();
    let revoked_token = v["token"].as_str().unwrap().to_string();
    let (status, _) =
        common::delete(app.clone(), &format!("/api/account/tokens/{revoked_id}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Mint a separate token that should keep its full budget despite
    // the revoked-token flood below.
    let (_, body) = common::post_json(
        app.clone(),
        "/api/account/tokens",
        json!({"name": "still-valid"}),
    )
    .await;
    let v: Value = common::json(&body);
    let valid_token = v["token"].as_str().unwrap().to_string();

    // 100 requests on the revoked token — all 401, none counted.
    for _ in 0..100 {
        let resp = app
            .clone()
            .oneshot(mcp_request(Some(&revoked_token)))
            .await
            .unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // Valid token still has its full budget.
    for i in 1..=2 {
        let resp = app
            .clone()
            .oneshot(mcp_request(Some(&valid_token)))
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "valid token's budget {i} should not be consumed by revoked-token traffic"
        );
    }
}

#[tokio::test]
async fn options_preflight_does_not_consume_bucket() {
    // Browser MCP clients preflight every request. CORS preflights
    // must short-circuit before the bucket check, otherwise 60 preflights
    // would silently exhaust a token's minute budget.
    let app = common::setup_test_app_with_rate_limit(1, Duration::from_secs(60)).await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/account/tokens",
        json!({"name": "preflight-test"}),
    )
    .await;
    let v: Value = common::json(&body);
    let token = v["token"].as_str().unwrap().to_string();

    // Send 5 OPTIONS preflights. None should count.
    for i in 1..=5 {
        let preflight = Request::builder()
            .method("OPTIONS")
            .uri("/api/mcp")
            .header("origin", "https://claude.ai")
            .header("access-control-request-method", "POST")
            .header(
                "access-control-request-headers",
                "authorization,content-type",
            )
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(preflight).await.unwrap();
        // CorsLayer responds 200 (with allow-origin) for preflights.
        assert!(
            resp.status().is_success() || resp.status() == StatusCode::NO_CONTENT,
            "preflight {i} should return success; got {}",
            resp.status()
        );
    }

    // The real call should still succeed — bucket wasn't burnt by preflights.
    let resp = app.oneshot(mcp_request(Some(&token))).await.unwrap();
    let status = resp.status();
    let body = resp
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    assert_eq!(
        status,
        StatusCode::OK,
        "limit=1 request after 5 preflights should still succeed; body: {}",
        String::from_utf8_lossy(&body)
    );
}
