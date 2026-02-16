mod common;

use axum::http::StatusCode;

#[tokio::test]
async fn health_check_returns_ok() {
    let app = common::setup_test_app().await;
    let (status, body) = common::get(app, "/api/health").await;

    assert_eq!(status, StatusCode::OK);
    let json: serde_json::Value = common::json(&body);
    assert_eq!(json["status"], "ok");
    assert_eq!(json["database"], "ok");
}
