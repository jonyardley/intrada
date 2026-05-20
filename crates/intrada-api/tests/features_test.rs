mod common;

use axum::http::StatusCode;
use intrada_core::domain::features::FeatureFlags;
use serde_json::Value;

#[tokio::test]
async fn features_endpoint_returns_flags_struct() {
    let app = common::setup_test_app().await;
    let (status, body) = common::get(app, "/api/features").await;
    assert_eq!(status, StatusCode::OK);
    let flags: FeatureFlags = common::json(&body);
    assert!(
        flags.goals,
        "auth-disabled dev mode should enable all flags"
    );
}

#[tokio::test]
async fn features_endpoint_response_is_object_with_goals_field() {
    let app = common::setup_test_app().await;
    let (status, body) = common::get(app, "/api/features").await;
    assert_eq!(status, StatusCode::OK);
    let value: Value = common::json(&body);
    assert!(value.is_object(), "expected JSON object, got {value}");
    assert!(value.get("goals").is_some(), "missing `goals` field");
}
