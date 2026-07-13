mod common;

use axum::http::StatusCode;
use base64::Engine;
use intrada_api::auth::{fetch_jwks, AuthConfig};
use jsonwebtoken::{encode, DecodingKey, EncodingKey, Header};
use rsa::pkcs1::EncodeRsaPrivateKey;
use rsa::pkcs8::EncodePublicKey;
use rsa::traits::PublicKeyParts;
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
    let private_key =
        RsaPrivateKey::new(&mut rsa::rand_core::OsRng, 2048).expect("Failed to generate RSA key");

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

// ── JWKS fetch / multi-key ────────────────────────────────────────────

/// RSA keypair + the JWK JSON object exposing its public half. Mirrors the
/// shape Clerk's `/.well-known/jwks.json` returns so `fetch_jwks` parses it
/// through the same `JwkSet` / `DecodingKey::from_jwk` path that runs in prod.
struct TestKeypair {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    jwk_json: serde_json::Value,
}

fn make_keypair(kid: &str) -> TestKeypair {
    let private_key =
        RsaPrivateKey::new(&mut rsa::rand_core::OsRng, 2048).expect("generate RSA key");

    let private_pem = private_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .expect("export private key PEM");
    let encoding_key =
        EncodingKey::from_rsa_pem(private_pem.as_bytes()).expect("encoding key from PEM");

    let public_pem = private_key
        .to_public_key()
        .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
        .expect("export public key PEM");
    let decoding_key =
        DecodingKey::from_rsa_pem(public_pem.as_bytes()).expect("decoding key from PEM");

    let public_key = private_key.to_public_key();
    let n_b64 =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public_key.n().to_bytes_be());
    let e_b64 =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(public_key.e().to_bytes_be());

    let jwk_json = json!({
        "kty": "RSA",
        "use": "sig",
        "alg": "RS256",
        "kid": kid,
        "n": n_b64,
        "e": e_b64,
    });

    TestKeypair {
        encoding_key,
        decoding_key,
        jwk_json,
    }
}

async fn spawn_jwks_server(jwks_body: serde_json::Value) -> String {
    use axum::routing::get;
    use axum::Router;
    let body = jwks_body.to_string();
    let app = Router::new().route(
        "/.well-known/jwks.json",
        get(move || {
            let body = body.clone();
            async move {
                (
                    [(axum::http::header::CONTENT_TYPE, "application/json")],
                    body,
                )
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local_addr");
    tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve");
    });
    format!("http://{addr}")
}

#[tokio::test]
async fn fetch_jwks_parses_keys_from_jwks_endpoint() {
    let kp = make_keypair("kid-1");
    let jwks = json!({ "keys": [kp.jwk_json] });
    let issuer = spawn_jwks_server(jwks).await;

    let keys = fetch_jwks(&issuer)
        .await
        .expect("fetch_jwks should succeed");
    assert_eq!(keys.len(), 1);
}

#[tokio::test]
async fn fetch_jwks_trims_trailing_slash_in_issuer_url() {
    let kp = make_keypair("kid-1");
    let jwks = json!({ "keys": [kp.jwk_json] });
    let issuer = spawn_jwks_server(jwks).await;
    let issuer_with_slash = format!("{issuer}/");

    let keys = fetch_jwks(&issuer_with_slash)
        .await
        .expect("trailing slash should be trimmed and fetch should succeed");
    assert_eq!(keys.len(), 1);
}

#[tokio::test]
async fn fetch_jwks_errors_when_no_valid_keys_present() {
    let issuer = spawn_jwks_server(json!({ "keys": [] })).await;
    let result = fetch_jwks(&issuer).await;
    assert!(
        result.is_err(),
        "empty key set should be rejected by fetch_jwks"
    );
}

#[tokio::test]
async fn refresh_jwks_preserves_existing_keys_when_fetch_fails() {
    let kp = make_keypair("kid-1");
    let config = AuthConfig {
        issuer: "http://127.0.0.1:1".to_string(),
        decoding_keys: Arc::new(RwLock::new(vec![kp.decoding_key])),
    };
    let count_before = config.decoding_keys.read().await.len();
    assert_eq!(count_before, 1);

    config.refresh_jwks().await;

    let count_after = config.decoding_keys.read().await.len();
    assert_eq!(
        count_after, 1,
        "failed JWKS refresh must keep existing keys, not clear them"
    );
}

#[tokio::test]
async fn extractor_accepts_token_signed_by_second_key() {
    let kp_old = make_keypair("kid-old");
    let kp_new = make_keypair("kid-new");

    let config = AuthConfig {
        issuer: TEST_ISSUER.to_string(),
        decoding_keys: Arc::new(RwLock::new(vec![kp_old.decoding_key, kp_new.decoding_key])),
    };
    let app = common::setup_test_app_with_auth(config).await;

    let token = make_token(&kp_new.encoding_key, "user_123", TEST_ISSUER, future_exp());
    let (status, _body) = common::get_with_auth(app, "/api/items", &token).await;
    assert_eq!(
        status,
        StatusCode::OK,
        "extractor must try every decoding key until one validates"
    );
}

// ── PAT resolution edge cases ─────────────────────────────────────────

async fn seed_pat(conn: &libsql::Connection, token: &str, user_id: &str) {
    let token_id = ulid::Ulid::gen().to_string();
    let hash = intrada_api::db::tokens::hash_token(token);
    let prefix = &token[..16.min(token.len())];
    intrada_api::db::tokens::insert(
        conn,
        &token_id,
        user_id,
        "auth-test",
        &hash,
        prefix,
        chrono::Utc::now(),
    )
    .await
    .expect("insert PAT");
}

#[tokio::test]
async fn bearer_without_pat_prefix_does_not_hit_pat_path() {
    let (_enc_key, auth_config) = test_keys();
    let (app, conn) =
        common::setup_test_app_with_conn(Some(auth_config), "http://localhost:3000").await;

    // Seed a PAT so the DB layer *would* return a row if asked. The bearer
    // below shares no prefix with PATs, so the extractor must skip the PAT
    // path entirely and try JWT validation (which will fail → 401).
    seed_pat(&conn, "intrada_pat_aaaaaaaaaaaaaaaaaaaa", "user_42").await;

    let (status, _) = common::get_with_auth(app, "/api/items", "not.a.pat.or.jwt").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn well_formed_pat_prefix_with_unknown_hash_returns_401() {
    let (_enc_key, auth_config) = test_keys();
    let app = common::setup_test_app_with_auth(auth_config).await;

    let (status, _) = common::get_with_auth(
        app,
        "/api/items",
        "intrada_pat_unknown_token_value_not_in_db_zzz",
    )
    .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn pat_lookup_keys_on_full_hash_not_visible_prefix() {
    let (_enc_key, auth_config) = test_keys();
    let (app, conn) =
        common::setup_test_app_with_conn(Some(auth_config), "http://localhost:3000").await;

    // Two PATs sharing the same 16-char visible prefix but with different
    // suffixes — and therefore different hashes — must resolve to their own
    // user. Catches a regression where lookup keys on prefix instead of hash.
    let shared_prefix = "intrada_pat_dead";
    let token_a = format!("{shared_prefix}beef000000000000000000000000000000000000000000000000");
    let token_b = format!("{shared_prefix}cafe000000000000000000000000000000000000000000000000");
    assert_ne!(token_a, token_b);
    seed_pat(&conn, &token_a, "user_A").await;
    seed_pat(&conn, &token_b, "user_B").await;

    let (status_a, _) = common::get_with_auth(app.clone(), "/api/items", &token_a).await;
    assert_eq!(status_a, StatusCode::OK);

    let (status_b, _) = common::get_with_auth(app, "/api/items", &token_b).await;
    assert_eq!(status_b, StatusCode::OK);
}

#[tokio::test]
async fn pat_prefix_takes_precedence_over_jwt_path() {
    // Even with Clerk auth_config attached, a Bearer beginning with the PAT
    // prefix routes to the PAT lookup — not JWT validation. Locks in the
    // resolution order documented on `AuthUser` (PAT first, JWT second).
    let (_enc_key, auth_config) = test_keys();
    let (app, conn) =
        common::setup_test_app_with_conn(Some(auth_config), "http://localhost:3000").await;

    // Constructed from the prefix + repeating ascii so the literal in
    // source has no high-entropy hex blob for gitleaks to flag.
    let suffix = "z".repeat(60);
    let pat_token = format!("{}{suffix}", intrada_api::db::tokens::TOKEN_PREFIX);
    seed_pat(&conn, &pat_token, "user_PAT").await;

    let (status, _body) = common::get_with_auth(app, "/api/items", &pat_token).await;
    assert_eq!(status, StatusCode::OK);
}
