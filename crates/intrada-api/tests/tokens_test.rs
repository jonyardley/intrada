mod common;

use axum::http::StatusCode;
use serde_json::{json, Value};

#[tokio::test]
async fn create_token_returns_full_pat_once() {
    let app = common::setup_test_app().await;
    let (status, body) =
        common::post_json(app, "/api/account/tokens", json!({ "name": "My laptop" })).await;

    assert_eq!(status, StatusCode::CREATED);
    let v: Value = common::json(&body);
    let token = v["token"].as_str().expect("token field present");
    assert!(token.starts_with("intrada_pat_"), "got: {token}");
    assert!(
        token.len() > 60,
        "token should be ~76 chars (12 prefix + 64 hex), got {}",
        token.len()
    );
    assert_eq!(v["name"], "My laptop");
    assert!(v["prefix"].as_str().unwrap().starts_with("intrada_pat_"));
    assert!(!v["id"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn create_token_trims_name_and_rejects_empty() {
    let app = common::setup_test_app().await;
    let (status, body) =
        common::post_json(app, "/api/account/tokens", json!({ "name": "  " })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let v: Value = common::json(&body);
    assert!(v["error"]
        .as_str()
        .unwrap()
        .contains("Token name cannot be empty"));
}

#[tokio::test]
async fn list_tokens_excludes_full_token_and_includes_prefix() {
    let app = common::setup_test_app().await;
    let (status, _) = common::post_json(
        app.clone(),
        "/api/account/tokens",
        json!({ "name": "Token A" }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = common::get(app, "/api/account/tokens").await;
    assert_eq!(status, StatusCode::OK);
    let tokens: Vec<Value> = common::json(&body);
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0]["name"], "Token A");
    assert!(tokens[0]["prefix"]
        .as_str()
        .unwrap()
        .starts_with("intrada_pat_"));
    // Critical: the full token must never be exposed by the list endpoint.
    assert!(tokens[0].get("token").is_none());
    // Hash must not leak either.
    assert!(tokens[0].get("hash").is_none());
}

#[tokio::test]
async fn revoke_token_marks_revoked_at() {
    let app = common::setup_test_app().await;
    let (_, body) = common::post_json(
        app.clone(),
        "/api/account/tokens",
        json!({ "name": "Revoke me" }),
    )
    .await;
    let v: Value = common::json(&body);
    let id = v["id"].as_str().unwrap().to_string();

    let (status, _) = common::delete(app.clone(), &format!("/api/account/tokens/{id}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, body) = common::get(app, "/api/account/tokens").await;
    let tokens: Vec<Value> = common::json(&body);
    assert_eq!(tokens.len(), 1);
    assert!(
        tokens[0]["revoked_at"].is_string(),
        "revoked_at should be set after DELETE; got {:?}",
        tokens[0]["revoked_at"]
    );
}

#[tokio::test]
async fn revoke_unknown_token_returns_404() {
    let app = common::setup_test_app().await;
    let (status, _) = common::delete(app, "/api/account/tokens/01HXYZUNKNOWN0000000000000").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn revoke_already_revoked_token_returns_404() {
    let app = common::setup_test_app().await;
    let (_, body) = common::post_json(
        app.clone(),
        "/api/account/tokens",
        json!({ "name": "double-revoke" }),
    )
    .await;
    let v: Value = common::json(&body);
    let id = v["id"].as_str().unwrap().to_string();

    let url = format!("/api/account/tokens/{id}");
    let (s1, _) = common::delete(app.clone(), &url).await;
    assert_eq!(s1, StatusCode::NO_CONTENT);
    let (s2, _) = common::delete(app, &url).await;
    assert_eq!(s2, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn pat_authenticates_subsequent_requests() {
    let app = common::setup_test_app().await;
    let (_, body) = common::post_json(
        app.clone(),
        "/api/account/tokens",
        json!({ "name": "auth me" }),
    )
    .await;
    let v: Value = common::json(&body);
    let token = v["token"].as_str().unwrap().to_string();

    // Use the PAT to hit a protected endpoint.
    let (status, _) = common::get_with_auth(app, "/api/items", &token).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn revoked_pat_is_rejected() {
    let app = common::setup_test_app().await;
    let (_, body) = common::post_json(
        app.clone(),
        "/api/account/tokens",
        json!({ "name": "revoke-then-use" }),
    )
    .await;
    let v: Value = common::json(&body);
    let id = v["id"].as_str().unwrap().to_string();
    let token = v["token"].as_str().unwrap().to_string();

    let (status, _) = common::delete(app.clone(), &format!("/api/account/tokens/{id}")).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Revoked PATs must be rejected even when the API runs in
    // auth-disabled mode (no CLERK_ISSUER_URL). The PAT path takes
    // precedence over the disabled-auth fallback.
    let (status, _) = common::get_with_auth(app, "/api/items", &token).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unknown_pat_is_rejected() {
    let app = common::setup_test_app().await;
    // Random PAT-looking string, not in the DB.
    let bogus = "intrada_pat_deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";
    let (status, _) = common::get_with_auth(app, "/api/items", bogus).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
