mod common;

use axum::http::StatusCode;
use serde_json::{json, Value};

#[tokio::test]
async fn exchange_rejects_unauthenticated_request() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(app, "/api/auth/ios/exchange", json!({})).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let v: Value = common::json(&body);
    assert!(
        v["error"].as_str().unwrap().contains("Clerk JWT"),
        "should explain that a JWT is required, got: {}",
        v["error"]
    );
}

#[tokio::test]
async fn exchange_rejects_pat_auth() {
    let app = common::setup_test_app().await;

    // Create a valid PAT first.
    let (_, body) = common::post_json(
        app.clone(),
        "/api/account/tokens",
        json!({ "name": "test pat" }),
    )
    .await;
    let v: Value = common::json(&body);
    let token = v["token"].as_str().unwrap();

    // Try to use the PAT to call the exchange endpoint.
    let (status, body) =
        common::post_json_with_auth(app, "/api/auth/ios/exchange", json!({}), token).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let v: Value = common::json(&body);
    assert!(
        v["error"].as_str().unwrap().contains("Clerk JWT"),
        "should reject PAT auth with a clear message, got: {}",
        v["error"]
    );
}

/// Verify that creating a new "iOS App" token revokes prior ones with the
/// same name. We can't hit the exchange endpoint's happy path without a real
/// Clerk JWT, so we test the revoke-by-name behaviour through the token list.
#[tokio::test]
async fn revoke_by_name_revokes_prior_tokens() {
    let app = common::setup_test_app().await;

    // Create two tokens with the same name (simulating repeated iOS sign-ins).
    let (s1, _) = common::post_json(
        app.clone(),
        "/api/account/tokens",
        json!({ "name": "iOS App" }),
    )
    .await;
    assert_eq!(s1, StatusCode::CREATED);

    let (s2, _) = common::post_json(
        app.clone(),
        "/api/account/tokens",
        json!({ "name": "iOS App" }),
    )
    .await;
    assert_eq!(s2, StatusCode::CREATED);

    // Also create a differently-named token to verify it's untouched.
    let (s3, _) = common::post_json(
        app.clone(),
        "/api/account/tokens",
        json!({ "name": "MCP Client" }),
    )
    .await;
    assert_eq!(s3, StatusCode::CREATED);

    // Revoke all "iOS App" tokens by name via the service layer directly.
    // We use the DB connection from the test harness.
    let (app2, conn) = common::setup_test_app_with_conn(None, "http://localhost:3000").await;

    // Seed two "iOS App" tokens via the API.
    let (_, b1) = common::post_json(
        app2.clone(),
        "/api/account/tokens",
        json!({ "name": "iOS App" }),
    )
    .await;
    let id1: Value = common::json(&b1);
    let (_, b2) = common::post_json(
        app2.clone(),
        "/api/account/tokens",
        json!({ "name": "iOS App" }),
    )
    .await;
    let id2: Value = common::json(&b2);
    // One unrelated token.
    let (_, b3) = common::post_json(
        app2.clone(),
        "/api/account/tokens",
        json!({ "name": "Other" }),
    )
    .await;
    let _id3: Value = common::json(&b3);

    // Call revoke_by_name (the function used by the exchange endpoint).
    let revoked = intrada_api::db::tokens::revoke_by_name(
        &conn, "", /* auth-disabled user_id */
        "iOS App",
    )
    .await
    .expect("revoke_by_name should succeed");
    assert_eq!(revoked, 2, "should revoke exactly the 2 'iOS App' tokens");

    // List tokens via the API. "iOS App" tokens are internal and should
    // be hidden from the MCP tokens list — only the "Other" token appears.
    let (_, body) = common::get(app2, "/api/account/tokens").await;
    let tokens: Vec<Value> = common::json(&body);
    assert_eq!(
        tokens.len(),
        1,
        "only the non-iOS token should appear in the list"
    );
    assert_eq!(tokens[0]["name"].as_str().unwrap(), "Other");
    assert!(
        tokens[0]["revoked_at"].is_null(),
        "Non-iOS token should NOT be revoked"
    );

    // Calling revoke_by_name again should return 0 (already revoked).
    let revoked_again = intrada_api::db::tokens::revoke_by_name(&conn, "", "iOS App")
        .await
        .expect("second revoke_by_name should succeed");
    assert_eq!(
        revoked_again, 0,
        "already-revoked tokens should not be re-revoked"
    );

    // Suppress unused variable warnings.
    let _ = (id1, id2);
}
