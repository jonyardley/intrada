mod common;

use axum::http::StatusCode;
use intrada_api::auth::AuthConfig;
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header};
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::pkcs8::EncodePublicKey;
use rsa::RsaPrivateKey;
use serde::Serialize;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

const TEST_ISSUER: &str = "https://test-issuer.example.com";

/// Claims structure matching what the API expects.
#[derive(Serialize)]
struct TestClaims {
    sub: String,
    iss: String,
    exp: u64,
    iat: u64,
}

/// Generate an RSA key pair and return (encoding_key, auth_config).
fn test_keys() -> (EncodingKey, AuthConfig) {
    let mut rng = rand::thread_rng();
    let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("Failed to generate RSA key");

    let private_pem = private_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .expect("Failed to export private key PEM");
    let encoding_key =
        EncodingKey::from_rsa_pem(private_pem.as_bytes()).expect("Failed to create encoding key");

    let public_pem = private_key
        .to_public_key()
        .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
        .expect("Failed to export public key PEM");
    let decoding_key =
        DecodingKey::from_rsa_pem(public_pem.as_bytes()).expect("Failed to create decoding key");

    let auth_config = AuthConfig {
        issuer: TEST_ISSUER.to_string(),
        decoding_keys: Arc::new(RwLock::new(vec![decoding_key])),
    };

    (encoding_key, auth_config)
}

/// Create a signed JWT with the given subject and expiry.
fn make_token(encoding_key: &EncodingKey, sub: &str, issuer: &str, exp: u64) -> String {
    let now = chrono::Utc::now().timestamp() as u64;
    let claims = TestClaims {
        sub: sub.to_string(),
        iss: issuer.to_string(),
        exp,
        iat: now,
    };
    let header = Header::new(jsonwebtoken::Algorithm::RS256);
    encode(&header, &claims, encoding_key).expect("Failed to encode JWT")
}

fn future_exp() -> u64 {
    (chrono::Utc::now().timestamp() + 3600) as u64
}

fn past_exp() -> u64 {
    (chrono::Utc::now().timestamp() - 3600) as u64
}

// ---------------------------------------------------------------------------
// Test cases
// ---------------------------------------------------------------------------

#[tokio::test]
async fn valid_token_returns_200() {
    let (enc_key, auth_config) = test_keys();
    let app = common::setup_test_app_with_auth(auth_config).await;

    let token = make_token(&enc_key, "user_123", TEST_ISSUER, future_exp());
    let (status, _body) = common::get_with_auth(app, "/api/items", &token).await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn expired_token_returns_401() {
    let (enc_key, auth_config) = test_keys();
    let app = common::setup_test_app_with_auth(auth_config).await;

    let token = make_token(&enc_key, "user_123", TEST_ISSUER, past_exp());
    let (status, _body) = common::get_with_auth(app, "/api/items", &token).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn wrong_issuer_returns_401() {
    let (enc_key, auth_config) = test_keys();
    let app = common::setup_test_app_with_auth(auth_config).await;

    let token = make_token(
        &enc_key,
        "user_123",
        "https://wrong-issuer.example.com",
        future_exp(),
    );
    let (status, _body) = common::get_with_auth(app, "/api/items", &token).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn missing_auth_header_returns_401() {
    let (_enc_key, auth_config) = test_keys();
    let app = common::setup_test_app_with_auth(auth_config).await;

    // Use plain GET without auth header
    let (status, _body) = common::get(app, "/api/items").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn malformed_bearer_prefix_returns_401() {
    let (enc_key, auth_config) = test_keys();
    let app = common::setup_test_app_with_auth(auth_config).await;

    let token = make_token(&enc_key, "user_123", TEST_ISSUER, future_exp());

    // Send with wrong prefix "Token" instead of "Bearer"
    let request = axum::http::Request::builder()
        .method("GET")
        .uri("/api/items")
        .header("Authorization", format!("Token {token}"))
        .body(axum::body::Body::empty())
        .unwrap();

    let response = tower::ServiceExt::oneshot(app, request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn auth_disabled_returns_200_without_token() {
    // setup_test_app() uses auth_config: None
    let app = common::setup_test_app().await;

    let (status, _body) = common::get(app, "/api/items").await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn garbage_token_returns_401() {
    let (_enc_key, auth_config) = test_keys();
    let app = common::setup_test_app_with_auth(auth_config).await;

    let (status, _body) = common::get_with_auth(app, "/api/items", "not.a.jwt").await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn user_isolation_cannot_see_other_users_data() {
    let (enc_key, auth_config) = test_keys();
    let app = common::setup_test_app_with_auth(auth_config).await;

    let token_a = make_token(&enc_key, "user_A", TEST_ISSUER, future_exp());
    let token_b = make_token(&enc_key, "user_B", TEST_ISSUER, future_exp());

    // User A creates an item
    let (status, _body) = common::post_json_with_auth(
        app.clone(),
        "/api/items",
        json!({
            "title": "User A's Piece",
            "kind": "piece",
            "composer": "Composer",
            "tags": []
        }),
        &token_a,
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // User B should see an empty list
    let (status, body) = common::get_with_auth(app, "/api/items", &token_b).await;
    assert_eq!(status, StatusCode::OK);
    let items: Vec<serde_json::Value> = common::json(&body);
    assert!(items.is_empty(), "User B should not see User A's items");
}
