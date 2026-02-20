use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;
use tower::ServiceExt;

use intrada_api::migrations;
use intrada_api::routes;
use intrada_api::state::AppState;

/// Create a fresh Axum router backed by a temporary SQLite database file.
/// Each call returns an isolated database — tests don't share state.
pub async fn setup_test_app() -> Router {
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

    let state = AppState::new(db, "http://localhost:3000".to_string(), None);
    routes::api_router(state)
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

/// Deserialize a JSON response body.
pub fn json<T: DeserializeOwned>(body: &[u8]) -> T {
    serde_json::from_slice(body).expect("Failed to deserialize response body")
}
