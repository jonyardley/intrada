#![allow(dead_code)]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;
use tower::ServiceExt;

use intrada_api::auth::AuthConfig;
use intrada_api::migrations;
use intrada_api::rate_limit::{IpRateLimiter, McpRateLimiter};
use intrada_api::routes;
use intrada_api::state::{AppState, Db};
use std::sync::Arc;
use std::time::Duration;

/// Create a fresh Axum router backed by a temporary SQLite database file.
/// Each call returns an isolated database — tests don't share state.
pub async fn setup_test_app() -> Router {
    setup_test_app_inner(None, "http://localhost:3000").await
}

/// Create a test router with auth enabled using the given `AuthConfig`.
pub async fn setup_test_app_with_auth(auth_config: AuthConfig) -> Router {
    setup_test_app_inner(Some(auth_config), "http://localhost:3000").await
}

/// Create a test router with a custom allowed-origin string (supports
/// comma-separated values to exercise multi-origin CORS).
pub async fn setup_test_app_with_origin(allowed_origin: &str) -> Router {
    setup_test_app_inner(None, allowed_origin).await
}

/// Create a test router with a tightened per-token rate-limiter — used by
/// `tests/rate_limit_test.rs` to exercise 429 responses without
/// hammering the default 60-req/min bucket.
#[allow(dead_code)]
pub async fn setup_test_app_with_rate_limit(limit: u32, window: Duration) -> Router {
    let limiter = Arc::new(McpRateLimiter::new(limit, window));

    let tmp_dir = std::env::temp_dir();
    let db_path = tmp_dir.join(format!("intrada_test_{}.db", ulid::Ulid::new()));
    let db = libsql::Builder::new_local(&db_path)
        .build()
        .await
        .expect("Failed to build test database");
    let conn = db.connect().expect("Failed to connect to test database");
    migrations::run_migrations_direct(&conn)
        .await
        .expect("Failed to run migrations");

    let state = AppState::new(
        Db::new(db, conn),
        "http://localhost:3000".to_string(),
        None,
        None,
        None,
    )
    .with_rate_limiter(limiter);
    routes::api_router(state)
}

/// Create a test router with a tightened per-IP OAuth rate-limiter — used by
/// `tests/rate_limit_test.rs` to exercise 429 on `/oauth/register`.
#[allow(dead_code)]
pub async fn setup_test_app_with_oauth_ip_limit(limit: u32, window: Duration) -> Router {
    let limiter = Arc::new(IpRateLimiter::new(limit, window));

    let tmp_dir = std::env::temp_dir();
    let db_path = tmp_dir.join(format!("intrada_test_{}.db", ulid::Ulid::new()));
    let db = libsql::Builder::new_local(&db_path)
        .build()
        .await
        .expect("Failed to build test database");
    let conn = db.connect().expect("Failed to connect to test database");
    migrations::run_migrations_direct(&conn)
        .await
        .expect("Failed to run migrations");

    let state = AppState::new(
        Db::new(db, conn),
        "http://localhost:3000".to_string(),
        None,
        None,
        None,
    )
    .with_oauth_ip_limiter(limiter);
    routes::api_router(state)
}

/// Create a test router with a tightened per-IP MCP rate-limiter — used by
/// `tests/rate_limit_test.rs` to exercise bogus-PAT flood protection.
#[allow(dead_code)]
pub async fn setup_test_app_with_mcp_ip_limit(limit: u32, window: Duration) -> Router {
    let limiter = Arc::new(IpRateLimiter::new(limit, window));

    let tmp_dir = std::env::temp_dir();
    let db_path = tmp_dir.join(format!("intrada_test_{}.db", ulid::Ulid::new()));
    let db = libsql::Builder::new_local(&db_path)
        .build()
        .await
        .expect("Failed to build test database");
    let conn = db.connect().expect("Failed to connect to test database");
    migrations::run_migrations_direct(&conn)
        .await
        .expect("Failed to run migrations");

    let state = AppState::new(
        Db::new(db, conn),
        "http://localhost:3000".to_string(),
        None,
        None,
        None,
    )
    .with_mcp_ip_limiter(limiter);
    routes::api_router(state)
}

async fn setup_test_app_inner(auth_config: Option<AuthConfig>, allowed_origin: &str) -> Router {
    let (router, _conn) = setup_test_app_with_conn(auth_config, allowed_origin).await;
    router
}

/// Like `setup_test_app_inner`, but also returns a clone of the underlying
/// `libsql::Connection` so tests can inject rows directly (e.g. cross-user
/// isolation tests that need to seed a row attributable to a different
/// `user_id` than the one the Router will resolve from auth).
#[allow(dead_code)]
pub async fn setup_test_app_with_conn(
    auth_config: Option<AuthConfig>,
    allowed_origin: &str,
) -> (Router, libsql::Connection) {
    let tmp_dir = std::env::temp_dir();
    let db_path = tmp_dir.join(format!("intrada_test_{}.db", ulid::Ulid::new()));

    let db = libsql::Builder::new_local(&db_path)
        .build()
        .await
        .expect("Failed to build test database");

    let conn = db.connect().expect("Failed to connect to test database");

    // FK enforcement OFF to match prod (Turso compatibility).
    // All cascade deletes are handled explicitly in application code.

    migrations::run_migrations_direct(&conn)
        .await
        .expect("Failed to run migrations");

    let state = AppState::new(
        Db::new(db, conn.clone()),
        allowed_origin.to_string(),
        auth_config,
        None,
        None,
    );
    (routes::api_router(state), conn)
}

/// Send a GET request and return the response.
pub async fn get(router: Router, uri: &str) -> (StatusCode, Vec<u8>) {
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, body)
}

/// Send a POST request with a JSON body and return the response.
pub async fn post_json(
    router: Router,
    uri: &str,
    body: impl serde::Serialize,
) -> (StatusCode, Vec<u8>) {
    let json = serde_json::to_string(&body).unwrap();
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, body)
}

/// Send a PUT request with a JSON body and return the response.
pub async fn put_json(
    router: Router,
    uri: &str,
    body: impl serde::Serialize,
) -> (StatusCode, Vec<u8>) {
    let json = serde_json::to_string(&body).unwrap();
    let request = Request::builder()
        .method("PUT")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, body)
}

/// Send a PATCH request with a JSON body and return the response.
pub async fn patch_json(
    router: Router,
    uri: &str,
    body: impl serde::Serialize,
) -> (StatusCode, Vec<u8>) {
    let json = serde_json::to_string(&body).unwrap();
    let request = Request::builder()
        .method("PATCH")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(json))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, body)
}

/// Send a DELETE request and return the response.
pub async fn delete(router: Router, uri: &str) -> (StatusCode, Vec<u8>) {
    let request = Request::builder()
        .method("DELETE")
        .uri(uri)
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, body)
}

/// Send a GET request with an Authorization header.
pub async fn get_with_auth(router: Router, uri: &str, token: &str) -> (StatusCode, Vec<u8>) {
    let request = Request::builder()
        .method("GET")
        .uri(uri)
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, body)
}

/// Send a POST request with JSON body and Authorization header.
pub async fn post_json_with_auth(
    router: Router,
    uri: &str,
    body: impl serde::Serialize,
    token: &str,
) -> (StatusCode, Vec<u8>) {
    let json = serde_json::to_string(&body).unwrap();
    let request = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::from(json))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, body)
}

/// Send a PUT request with JSON body and Authorization header.
#[allow(dead_code)]
pub async fn put_json_with_auth(
    router: Router,
    uri: &str,
    body: impl serde::Serialize,
    token: &str,
) -> (StatusCode, Vec<u8>) {
    let json = serde_json::to_string(&body).unwrap();
    let request = Request::builder()
        .method("PUT")
        .uri(uri)
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::from(json))
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, body)
}

/// Send a DELETE request with an Authorization header.
#[allow(dead_code)]
pub async fn delete_with_auth(router: Router, uri: &str, token: &str) -> (StatusCode, Vec<u8>) {
    let request = Request::builder()
        .method("DELETE")
        .uri(uri)
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap();

    let response = router.oneshot(request).await.unwrap();
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .unwrap()
        .to_bytes()
        .to_vec();
    (status, body)
}

/// Deserialize a JSON response body.
pub fn json<T: DeserializeOwned>(body: &[u8]) -> T {
    serde_json::from_slice(body).expect("Failed to deserialize response body")
}
