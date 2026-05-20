mod common;

use axum::http::StatusCode;
use intrada_core::domain::goal::{Goal, GoalStatus};
use serde_json::json;

// ── List ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_goals_empty() {
    let app = common::setup_test_app().await;
    let (status, body) = common::get(app, "/api/goals").await;

    assert_eq!(status, StatusCode::OK);
    let goals: Vec<Goal> = common::json(&body);
    assert!(goals.is_empty());
}

// ── Create ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_goal_minimal() {
    let app = common::setup_test_app().await;
    let (status, body) =
        common::post_json(app, "/api/goals", json!({ "date": "2026-05-15" })).await;

    assert_eq!(status, StatusCode::CREATED);
    let goal: Goal = common::json(&body);
    assert!(!goal.id.is_empty());
    assert_eq!(goal.date, "2026-05-15");
    assert_eq!(goal.status, GoalStatus::Active);
    assert!(goal.title.is_none());
    assert!(goal.notes.is_none());
    assert!(goal.deadline.is_none());
    assert!(goal.completed_at.is_none());
    assert!(goal.items.is_empty());
    assert!(goal.photos.is_empty());
}

#[tokio::test]
async fn create_goal_full() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/goals",
        json!({
            "date": "2026-05-15",
            "title": "Learn Bach Prelude",
            "notes": "Focus on bars 12-24",
            "deadline": "2026-06-01"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let goal: Goal = common::json(&body);
    assert!(!goal.id.is_empty());
    assert_eq!(goal.date, "2026-05-15");
    assert_eq!(goal.title.as_deref(), Some("Learn Bach Prelude"));
    assert_eq!(goal.notes.as_deref(), Some("Focus on bars 12-24"));
    assert_eq!(goal.deadline.as_deref(), Some("2026-06-01"));
    assert_eq!(goal.status, GoalStatus::Active);
}

#[tokio::test]
async fn create_goal_invalid_date_returns_400() {
    let app = common::setup_test_app().await;
    let (status, _body) =
        common::post_json(app, "/api/goals", json!({ "date": "not-a-date" })).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// `validate_goal_date` previously rejected future dates, but that produced
// false rejections for users east of UTC near midnight (the client form
// fills `date` from local time and the server compares against UTC). The
// restriction was dropped; we now accept any valid YYYY-MM-DD.
#[tokio::test]
async fn create_goal_far_future_date_accepted() {
    let app = common::setup_test_app().await;
    let (status, _body) =
        common::post_json(app, "/api/goals", json!({ "date": "2099-01-01" })).await;

    assert_eq!(status, StatusCode::CREATED);
}

// ── Get by ID ──────────────────────────────────────────────────────────

#[tokio::test]
async fn get_goal_not_found() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::get(app, "/api/goals/nonexistent").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_goal_by_id() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/goals",
        json!({ "date": "2026-05-15", "title": "My Goal" }),
    )
    .await;
    let created: Goal = common::json(&body);

    let (status, body) = common::get(app, &format!("/api/goals/{}", created.id)).await;
    assert_eq!(status, StatusCode::OK);
    let fetched: Goal = common::json(&body);
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.title.as_deref(), Some("My Goal"));
    assert_eq!(fetched.date, "2026-05-15");
}

// ── Update ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn update_goal_title() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/goals",
        json!({ "date": "2026-05-15", "title": "Old Title" }),
    )
    .await;
    let created: Goal = common::json(&body);

    let (status, body) = common::put_json(
        app,
        &format!("/api/goals/{}", created.id),
        json!({ "title": "New Title" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let updated: Goal = common::json(&body);
    assert_eq!(updated.title.as_deref(), Some("New Title"));
    assert!(updated.updated_at > created.updated_at);
}

#[tokio::test]
async fn complete_goal() {
    let app = common::setup_test_app().await;

    let (_, body) =
        common::post_json(app.clone(), "/api/goals", json!({ "date": "2026-05-15" })).await;
    let created: Goal = common::json(&body);
    assert_eq!(created.status, GoalStatus::Active);
    assert!(created.completed_at.is_none());

    let (status, body) = common::put_json(
        app,
        &format!("/api/goals/{}", created.id),
        json!({ "status": "completed" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let updated: Goal = common::json(&body);
    assert_eq!(updated.status, GoalStatus::Completed);
    assert!(updated.completed_at.is_some());
}

// ── Delete ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_goal() {
    let app = common::setup_test_app().await;

    let (_, body) =
        common::post_json(app.clone(), "/api/goals", json!({ "date": "2026-05-15" })).await;
    let created: Goal = common::json(&body);

    let (status, _body) = common::delete(app.clone(), &format!("/api/goals/{}", created.id)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify the goal is gone
    let (status, _body) = common::get(app, &format!("/api/goals/{}", created.id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Link / Unlink items ───────────────────────────────────────────────

#[tokio::test]
async fn link_and_unlink_item() {
    let app = common::setup_test_app().await;

    // Create a goal
    let (_, body) =
        common::post_json(app.clone(), "/api/goals", json!({ "date": "2026-05-15" })).await;
    let created: Goal = common::json(&body);
    assert!(created.items.is_empty());

    // Link an item
    let (status, _body) = common::post_json(
        app.clone(),
        &format!("/api/goals/{}/items", created.id),
        json!({
            "item_id": "piece-001",
            "item_title": "Bach Prelude",
            "item_type": "piece"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Verify the item is present on the goal
    let (status, body) = common::get(app.clone(), &format!("/api/goals/{}", created.id)).await;
    assert_eq!(status, StatusCode::OK);
    let fetched: Goal = common::json(&body);
    assert_eq!(fetched.items.len(), 1);
    assert_eq!(fetched.items[0].item_id, "piece-001");
    assert_eq!(fetched.items[0].item_title, "Bach Prelude");

    // Unlink the item
    let (status, _body) = common::delete(
        app.clone(),
        &format!("/api/goals/{}/items/piece-001", created.id),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Verify the item is gone
    let (status, body) = common::get(app, &format!("/api/goals/{}", created.id)).await;
    assert_eq!(status, StatusCode::OK);
    let fetched: Goal = common::json(&body);
    assert!(fetched.items.is_empty());
}

// ── Filter by status ──────────────────────────────────────────────────

#[tokio::test]
async fn filter_goals_by_status() {
    let app = common::setup_test_app().await;

    // Create an active goal
    let (_, body) = common::post_json(
        app.clone(),
        "/api/goals",
        json!({ "date": "2026-05-14", "title": "Active Goal" }),
    )
    .await;
    let active_goal: Goal = common::json(&body);

    // Create another goal and mark it completed
    let (_, body) = common::post_json(
        app.clone(),
        "/api/goals",
        json!({ "date": "2026-05-15", "title": "Completed Goal" }),
    )
    .await;
    let to_complete: Goal = common::json(&body);

    let (status, _body) = common::put_json(
        app.clone(),
        &format!("/api/goals/{}", to_complete.id),
        json!({ "status": "completed" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Default (no filter) — should return only active goals
    let (status, body) = common::get(app.clone(), "/api/goals").await;
    assert_eq!(status, StatusCode::OK);
    let goals: Vec<Goal> = common::json(&body);
    assert_eq!(goals.len(), 1);
    assert_eq!(goals[0].id, active_goal.id);

    // Explicit ?status=active
    let (status, body) = common::get(app.clone(), "/api/goals?status=active").await;
    assert_eq!(status, StatusCode::OK);
    let goals: Vec<Goal> = common::json(&body);
    assert_eq!(goals.len(), 1);
    assert_eq!(goals[0].status, GoalStatus::Active);

    // ?status=completed
    let (status, body) = common::get(app.clone(), "/api/goals?status=completed").await;
    assert_eq!(status, StatusCode::OK);
    let goals: Vec<Goal> = common::json(&body);
    assert_eq!(goals.len(), 1);
    assert_eq!(goals[0].status, GoalStatus::Completed);

    // ?status=all
    let (status, body) = common::get(app, "/api/goals?status=all").await;
    assert_eq!(status, StatusCode::OK);
    let goals: Vec<Goal> = common::json(&body);
    assert_eq!(goals.len(), 2);
}

#[tokio::test]
async fn reopen_goal_clears_completed_at() {
    let app = common::setup_test_app().await;
    let (_, body) =
        common::post_json(app.clone(), "/api/goals", json!({ "date": "2026-05-15" })).await;
    let created: Goal = common::json(&body);

    // Mark as completed
    let (_, body) = common::put_json(
        app.clone(),
        &format!("/api/goals/{}", created.id),
        json!({ "status": "completed" }),
    )
    .await;
    let completed: Goal = common::json(&body);
    assert_eq!(completed.status, GoalStatus::Completed);
    assert!(completed.completed_at.is_some());

    // Reopen by flipping status back to active
    let (status, body) = common::put_json(
        app,
        &format!("/api/goals/{}", created.id),
        json!({ "status": "active" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let reopened: Goal = common::json(&body);
    assert_eq!(reopened.status, GoalStatus::Active);
    assert!(
        reopened.completed_at.is_none(),
        "completed_at should be cleared on reopen, was {:?}",
        reopened.completed_at
    );
}

// ── Targets (Phase 1) ─────────────────────────────────────────────────

#[tokio::test]
async fn create_goal_with_target_confidence() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/goals",
        json!({ "date": "2026-05-15", "target_confidence": 4 }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let goal: Goal = common::json(&body);
    assert_eq!(goal.target_confidence, Some(4));
}

#[tokio::test]
async fn create_goal_target_confidence_out_of_range_returns_400() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::post_json(
        app,
        "/api/goals",
        json!({ "date": "2026-05-15", "target_confidence": 9 }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_goal_clears_target_confidence() {
    let app = common::setup_test_app().await;
    let (_, body) = common::post_json(
        app.clone(),
        "/api/goals",
        json!({ "date": "2026-05-15", "target_confidence": 3 }),
    )
    .await;
    let created: Goal = common::json(&body);
    assert_eq!(created.target_confidence, Some(3));

    let (status, body) = common::put_json(
        app,
        &format!("/api/goals/{}", created.id),
        json!({ "target_confidence": null }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let updated: Goal = common::json(&body);
    assert!(updated.target_confidence.is_none());
}

#[tokio::test]
async fn link_item_with_targets() {
    let app = common::setup_test_app().await;
    let (_, body) =
        common::post_json(app.clone(), "/api/goals", json!({ "date": "2026-05-15" })).await;
    let goal: Goal = common::json(&body);

    let (status, _body) = common::post_json(
        app.clone(),
        &format!("/api/goals/{}/items", goal.id),
        json!({
            "item_id": "piece-001",
            "item_title": "Bach Prelude",
            "item_type": "piece",
            "target_date": "2026-06-01",
            "target_confidence": 4
        }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    let (status, body) = common::get(app, &format!("/api/goals/{}", goal.id)).await;
    assert_eq!(status, StatusCode::OK);
    let fetched: Goal = common::json(&body);
    assert_eq!(fetched.items.len(), 1);
    assert_eq!(fetched.items[0].target_date.as_deref(), Some("2026-06-01"));
    assert_eq!(fetched.items[0].target_confidence, Some(4));
}

#[tokio::test]
async fn patch_goal_item_targets() {
    let app = common::setup_test_app().await;
    let (_, body) =
        common::post_json(app.clone(), "/api/goals", json!({ "date": "2026-05-15" })).await;
    let goal: Goal = common::json(&body);

    common::post_json(
        app.clone(),
        &format!("/api/goals/{}/items", goal.id),
        json!({
            "item_id": "piece-001",
            "item_title": "Bach",
            "item_type": "piece"
        }),
    )
    .await;

    let (status, _body) = common::patch_json(
        app.clone(),
        &format!("/api/goals/{}/items/piece-001", goal.id),
        json!({ "target_date": "2026-07-01", "target_confidence": 5 }),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, body) = common::get(app, &format!("/api/goals/{}", goal.id)).await;
    let fetched: Goal = common::json(&body);
    assert_eq!(fetched.items[0].target_date.as_deref(), Some("2026-07-01"));
    assert_eq!(fetched.items[0].target_confidence, Some(5));
}

#[tokio::test]
async fn patch_goal_item_partial_update_preserves_other_fields() {
    let app = common::setup_test_app().await;
    let (_, body) =
        common::post_json(app.clone(), "/api/goals", json!({ "date": "2026-05-15" })).await;
    let goal: Goal = common::json(&body);

    common::post_json(
        app.clone(),
        &format!("/api/goals/{}/items", goal.id),
        json!({
            "item_id": "piece-001",
            "item_title": "Bach",
            "item_type": "piece",
            "target_date": "2026-06-01",
            "target_confidence": 3
        }),
    )
    .await;

    // PATCH only confidence — date should be preserved
    let (status, _body) = common::patch_json(
        app.clone(),
        &format!("/api/goals/{}/items/piece-001", goal.id),
        json!({ "target_confidence": 5 }),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, body) = common::get(app, &format!("/api/goals/{}", goal.id)).await;
    let fetched: Goal = common::json(&body);
    assert_eq!(fetched.items[0].target_date.as_deref(), Some("2026-06-01"));
    assert_eq!(fetched.items[0].target_confidence, Some(5));
}

#[tokio::test]
async fn patch_goal_item_clears_targets_with_null() {
    let app = common::setup_test_app().await;
    let (_, body) =
        common::post_json(app.clone(), "/api/goals", json!({ "date": "2026-05-15" })).await;
    let goal: Goal = common::json(&body);

    common::post_json(
        app.clone(),
        &format!("/api/goals/{}/items", goal.id),
        json!({
            "item_id": "piece-001",
            "item_title": "Bach",
            "item_type": "piece",
            "target_date": "2026-06-01",
            "target_confidence": 3
        }),
    )
    .await;

    let (status, _body) = common::patch_json(
        app.clone(),
        &format!("/api/goals/{}/items/piece-001", goal.id),
        json!({ "target_date": null, "target_confidence": null }),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (_, body) = common::get(app, &format!("/api/goals/{}", goal.id)).await;
    let fetched: Goal = common::json(&body);
    assert!(fetched.items[0].target_date.is_none());
    assert!(fetched.items[0].target_confidence.is_none());
}

#[tokio::test]
async fn patch_goal_item_target_confidence_out_of_range_returns_400() {
    let app = common::setup_test_app().await;
    let (_, body) =
        common::post_json(app.clone(), "/api/goals", json!({ "date": "2026-05-15" })).await;
    let goal: Goal = common::json(&body);

    common::post_json(
        app.clone(),
        &format!("/api/goals/{}/items", goal.id),
        json!({
            "item_id": "piece-001",
            "item_title": "Bach",
            "item_type": "piece"
        }),
    )
    .await;

    let (status, _body) = common::patch_json(
        app,
        &format!("/api/goals/{}/items/piece-001", goal.id),
        json!({ "target_confidence": 9 }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn patch_goal_item_not_found_returns_404() {
    let app = common::setup_test_app().await;
    let (_, body) =
        common::post_json(app.clone(), "/api/goals", json!({ "date": "2026-05-15" })).await;
    let goal: Goal = common::json(&body);

    let (status, _body) = common::patch_json(
        app,
        &format!("/api/goals/{}/items/nonexistent", goal.id),
        json!({ "target_confidence": 4 }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}
