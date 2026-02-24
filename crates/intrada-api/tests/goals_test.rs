mod common;

use axum::http::StatusCode;
use intrada_core::domain::goal::Goal;
use serde_json::json;

#[tokio::test]
async fn list_goals_empty() {
    let app = common::setup_test_app().await;
    let (status, body) = common::get(app, "/api/goals").await;

    assert_eq!(status, StatusCode::OK);
    let goals: Vec<Goal> = common::json(&body);
    assert!(goals.is_empty());
}

#[tokio::test]
async fn create_frequency_goal() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/goals",
        json!({
            "title": "Practise 5 days per week",
            "kind": { "type": "session_frequency", "target_days_per_week": 5 }
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let goal: Goal = common::json(&body);
    assert_eq!(goal.title, "Practise 5 days per week");
    assert!(!goal.id.is_empty());
}

#[tokio::test]
async fn create_practice_time_goal() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/goals",
        json!({
            "title": "Practise 120 minutes per week",
            "kind": { "type": "practice_time", "target_minutes_per_week": 120 }
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let goal: Goal = common::json(&body);
    assert_eq!(goal.title, "Practise 120 minutes per week");
}

#[tokio::test]
async fn create_item_mastery_goal() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/goals",
        json!({
            "title": "Master Clair de Lune",
            "kind": { "type": "item_mastery", "item_id": "item-123", "target_score": 4 }
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let goal: Goal = common::json(&body);
    assert_eq!(goal.title, "Master Clair de Lune");
}

#[tokio::test]
async fn create_milestone_goal() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/goals",
        json!({
            "title": "Memorise first movement",
            "kind": { "type": "milestone", "description": "Complete memorisation of the first movement" }
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let goal: Goal = common::json(&body);
    assert_eq!(goal.title, "Memorise first movement");
}

#[tokio::test]
async fn list_goals_returns_created_goals() {
    let app = common::setup_test_app().await;

    common::post_json(
        app.clone(),
        "/api/goals",
        json!({
            "title": "Goal A",
            "kind": { "type": "session_frequency", "target_days_per_week": 3 }
        }),
    )
    .await;
    common::post_json(
        app.clone(),
        "/api/goals",
        json!({
            "title": "Goal B",
            "kind": { "type": "practice_time", "target_minutes_per_week": 60 }
        }),
    )
    .await;

    let (status, body) = common::get(app, "/api/goals").await;
    assert_eq!(status, StatusCode::OK);
    let goals: Vec<Goal> = common::json(&body);
    assert_eq!(goals.len(), 2);
}

#[tokio::test]
async fn get_goal_existing() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/goals",
        json!({
            "title": "My Goal",
            "kind": { "type": "session_frequency", "target_days_per_week": 5 }
        }),
    )
    .await;
    let created: Goal = common::json(&body);

    let (status, body) = common::get(app, &format!("/api/goals/{}", created.id)).await;
    assert_eq!(status, StatusCode::OK);
    let fetched: Goal = common::json(&body);
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.title, "My Goal");
}

#[tokio::test]
async fn get_goal_not_found() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::get(app, "/api/goals/nonexistent-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_goal_title() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/goals",
        json!({
            "title": "Original Title",
            "kind": { "type": "session_frequency", "target_days_per_week": 5 }
        }),
    )
    .await;
    let created: Goal = common::json(&body);

    let (status, body) = common::put_json(
        app,
        &format!("/api/goals/{}", created.id),
        json!({ "title": "Updated Title" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let updated: Goal = common::json(&body);
    assert_eq!(updated.title, "Updated Title");
    assert!(updated.updated_at > created.updated_at);
}

#[tokio::test]
async fn update_goal_status_to_completed() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/goals",
        json!({
            "title": "Finish Goal",
            "kind": { "type": "milestone", "description": "Test milestone" }
        }),
    )
    .await;
    let created: Goal = common::json(&body);
    assert!(created.completed_at.is_none());

    let (status, body) = common::put_json(
        app,
        &format!("/api/goals/{}", created.id),
        json!({ "status": "completed" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let updated: Goal = common::json(&body);
    assert_eq!(
        serde_json::to_value(&updated.status).unwrap(),
        json!("completed")
    );
    assert!(updated.completed_at.is_some());
}

#[tokio::test]
async fn update_goal_status_to_archived() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/goals",
        json!({
            "title": "Archive Me",
            "kind": { "type": "session_frequency", "target_days_per_week": 3 }
        }),
    )
    .await;
    let created: Goal = common::json(&body);

    let (status, body) = common::put_json(
        app,
        &format!("/api/goals/{}", created.id),
        json!({ "status": "archived" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let updated: Goal = common::json(&body);
    assert_eq!(
        serde_json::to_value(&updated.status).unwrap(),
        json!("archived")
    );
}

#[tokio::test]
async fn reactivate_archived_goal() {
    let app = common::setup_test_app().await;

    // Create and archive
    let (_, body) = common::post_json(
        app.clone(),
        "/api/goals",
        json!({
            "title": "Reactivate Me",
            "kind": { "type": "session_frequency", "target_days_per_week": 3 }
        }),
    )
    .await;
    let created: Goal = common::json(&body);

    let (_, _) = common::put_json(
        app.clone(),
        &format!("/api/goals/{}", created.id),
        json!({ "status": "archived" }),
    )
    .await;

    // Reactivate
    let (status, body) = common::put_json(
        app,
        &format!("/api/goals/{}", created.id),
        json!({ "status": "active" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let updated: Goal = common::json(&body);
    assert_eq!(
        serde_json::to_value(&updated.status).unwrap(),
        json!("active")
    );
}

#[tokio::test]
async fn reject_completed_to_active_transition() {
    let app = common::setup_test_app().await;

    // Create and complete
    let (_, body) = common::post_json(
        app.clone(),
        "/api/goals",
        json!({
            "title": "Completed Goal",
            "kind": { "type": "milestone", "description": "Done" }
        }),
    )
    .await;
    let created: Goal = common::json(&body);

    let (_, _) = common::put_json(
        app.clone(),
        &format!("/api/goals/{}", created.id),
        json!({ "status": "completed" }),
    )
    .await;

    // Try to reactivate — should fail
    let (status, _body) = common::put_json(
        app,
        &format!("/api/goals/{}", created.id),
        json!({ "status": "active" }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn delete_goal_existing() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/goals",
        json!({
            "title": "Delete Me",
            "kind": { "type": "session_frequency", "target_days_per_week": 1 }
        }),
    )
    .await;
    let created: Goal = common::json(&body);

    let (status, _body) = common::delete(app.clone(), &format!("/api/goals/{}", created.id)).await;
    assert_eq!(status, StatusCode::OK);

    // Verify gone
    let (status, _body) = common::get(app, &format!("/api/goals/{}", created.id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_goal_not_found() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::delete(app, "/api/goals/nonexistent-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_goal_empty_title_returns_400() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::post_json(
        app,
        "/api/goals",
        json!({
            "title": "",
            "kind": { "type": "session_frequency", "target_days_per_week": 5 }
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_goal_out_of_range_target_returns_400() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::post_json(
        app,
        "/api/goals",
        json!({
            "title": "Too many days",
            "kind": { "type": "session_frequency", "target_days_per_week": 10 }
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_goal_with_deadline() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/goals",
        json!({
            "title": "Goal with deadline",
            "kind": { "type": "session_frequency", "target_days_per_week": 5 },
            "deadline": "2026-12-31T23:59:59Z"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let goal: Goal = common::json(&body);
    assert!(goal.deadline.is_some());
}

#[tokio::test]
async fn update_goal_not_found() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::put_json(
        app,
        "/api/goals/nonexistent-id",
        json!({ "title": "New Title" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
