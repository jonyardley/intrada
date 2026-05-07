mod common;

use axum::http::StatusCode;
use intrada_api::auth::AuthConfig;
use intrada_api::db::account::AccountPreferences;
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header};
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::pkcs8::EncodePublicKey;
use rsa::RsaPrivateKey;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

const TEST_ISSUER: &str = "https://test-issuer.example.com";

#[derive(Serialize)]
struct TestClaims {
    sub: String,
    iss: String,
    exp: u64,
    iat: u64,
}

fn test_keys() -> (EncodingKey, AuthConfig) {
    let mut rng = rand::thread_rng();
    let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate RSA key");
    let private_pem = private_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .unwrap();
    let encoding_key = EncodingKey::from_rsa_pem(private_pem.as_bytes()).unwrap();
    let public_pem = private_key
        .to_public_key()
        .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
        .unwrap();
    let decoding_key = DecodingKey::from_rsa_pem(public_pem.as_bytes()).unwrap();

    let auth_config = AuthConfig {
        issuer: TEST_ISSUER.to_string(),
        decoding_keys: Arc::new(RwLock::new(vec![decoding_key])),
    };
    (encoding_key, auth_config)
}

fn make_token(encoding_key: &EncodingKey, sub: &str) -> String {
    let now = chrono::Utc::now().timestamp() as u64;
    let claims = TestClaims {
        sub: sub.to_string(),
        iss: TEST_ISSUER.to_string(),
        exp: now + 3600,
        iat: now,
    };
    let header = Header::new(jsonwebtoken::Algorithm::RS256);
    encode(&header, &claims, encoding_key).unwrap()
}

#[tokio::test]
async fn get_preferences_returns_defaults_when_missing() {
    let app = common::setup_test_app().await;
    let (status, body) = common::get(app, "/api/account/preferences").await;
    assert_eq!(status, StatusCode::OK);
    let prefs: AccountPreferences = common::json(&body);
    assert_eq!(prefs.default_focus_minutes, 15);
    assert_eq!(prefs.default_rep_count, 10);
}

#[tokio::test]
async fn put_preferences_creates_then_updates() {
    let app = common::setup_test_app().await;

    let (status, body) = common::put_json(
        app.clone(),
        "/api/account/preferences",
        json!({ "default_focus_minutes": 20, "default_rep_count": 8 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let prefs: AccountPreferences = common::json(&body);
    assert_eq!(prefs.default_focus_minutes, 20);
    assert_eq!(prefs.default_rep_count, 8);

    // Re-read returns the saved values.
    let (_, body) = common::get(app.clone(), "/api/account/preferences").await;
    let prefs: AccountPreferences = common::json(&body);
    assert_eq!(prefs.default_focus_minutes, 20);

    // Update existing row.
    let (status, body) = common::put_json(
        app.clone(),
        "/api/account/preferences",
        json!({ "default_focus_minutes": 30, "default_rep_count": 12 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let prefs: AccountPreferences = common::json(&body);
    assert_eq!(prefs.default_focus_minutes, 30);
    assert_eq!(prefs.default_rep_count, 12);
}

#[tokio::test]
async fn put_preferences_rejects_zero_values() {
    let app = common::setup_test_app().await;
    let (status, _) = common::put_json(
        app,
        "/api/account/preferences",
        json!({ "default_focus_minutes": 0, "default_rep_count": 5 }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_account_removes_only_calling_users_data() {
    let (enc_key, auth_config) = test_keys();
    let app = common::setup_test_app_with_auth(auth_config).await;

    let token_a = make_token(&enc_key, "user_A");
    let token_b = make_token(&enc_key, "user_B");

    // Seed: each user creates an item.
    let (status, _) = common::post_json_with_auth(
        app.clone(),
        "/api/items",
        json!({ "title": "A's piece", "kind": "piece", "composer": "X", "tags": [] }),
        &token_a,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let (status, _) = common::post_json_with_auth(
        app.clone(),
        "/api/items",
        json!({ "title": "B's piece", "kind": "piece", "composer": "Y", "tags": [] }),
        &token_b,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Seed: user_A also saves preferences.
    let (status, _) = common::put_json_with_auth(
        app.clone(),
        "/api/account/preferences",
        json!({ "default_focus_minutes": 25, "default_rep_count": 6 }),
        &token_a,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // user_A deletes their account.
    let (status, _) = common::delete_with_auth(app.clone(), "/api/account", &token_a).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // user_A's items are gone.
    let (status, body) = common::get_with_auth(app.clone(), "/api/items", &token_a).await;
    assert_eq!(status, StatusCode::OK);
    let items: Vec<serde_json::Value> = common::json(&body);
    assert!(items.is_empty(), "user_A items should be deleted");

    // user_A's preferences are reset to defaults.
    let (_, body) = common::get_with_auth(app.clone(), "/api/account/preferences", &token_a).await;
    let prefs: AccountPreferences = common::json(&body);
    assert_eq!(prefs.default_focus_minutes, 15);
    assert_eq!(prefs.default_rep_count, 10);

    // user_B's items are intact.
    let (status, body) = common::get_with_auth(app.clone(), "/api/items", &token_b).await;
    assert_eq!(status, StatusCode::OK);
    let items: Vec<serde_json::Value> = common::json(&body);
    assert_eq!(items.len(), 1, "user_B items should be untouched");
}

#[tokio::test]
async fn delete_account_is_idempotent() {
    let (enc_key, auth_config) = test_keys();
    let app = common::setup_test_app_with_auth(auth_config).await;
    let token = make_token(&enc_key, "user_X");

    // Delete with no data — should succeed.
    let (status, _) = common::delete_with_auth(app.clone(), "/api/account", &token).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // And again — still succeeds.
    let (status, _) = common::delete_with_auth(app, "/api/account", &token).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn delete_account_requires_auth_when_enabled() {
    let (_, auth_config) = test_keys();
    let app = common::setup_test_app_with_auth(auth_config).await;

    let (status, _) = common::delete(app, "/api/account").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn delete_account_refuses_empty_user_id_in_auth_disabled_mode() {
    // Auth disabled (no AuthConfig) → AuthUser yields "". The handler
    // must refuse to proceed; otherwise destructive cleanup (R2 prefix
    // list, Clerk delete) could fan out across all users.
    let app = common::setup_test_app().await;
    let (status, _) = common::delete(app, "/api/account").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
