mod common;

use axum::http::StatusCode;
use intrada_core::domain::item::ItemKind;
use intrada_core::domain::routine::Routine;
use serde_json::json;

fn sample_routine_body() -> serde_json::Value {
    json!({
        "name": "Morning Warm-Up",
        "entries": [
            {
                "item_id": "piece-001",
                "item_title": "Clair de Lune",
                "item_type": "piece"
            },
            {
                "item_id": "exercise-001",
                "item_title": "Hanon No. 1",
                "item_type": "exercise"
            }
        ]
    })
}

// ── Create ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_routine_returns_201() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(app, "/api/routines", sample_routine_body()).await;

    assert_eq!(status, StatusCode::CREATED);
    let routine: Routine = common::json(&body);
    assert!(!routine.id.is_empty());
    assert_eq!(routine.name, "Morning Warm-Up");
    assert_eq!(routine.entries.len(), 2);
}

#[tokio::test]
async fn create_routine_with_entries() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(app, "/api/routines", sample_routine_body()).await;

    assert_eq!(status, StatusCode::CREATED);
    let routine: Routine = common::json(&body);

    assert_eq!(routine.entries.len(), 2);
    assert_eq!(routine.entries[0].item_id, "piece-001");
    assert_eq!(routine.entries[0].item_title, "Clair de Lune");
    assert_eq!(routine.entries[0].item_type, ItemKind::Piece);
    assert_eq!(routine.entries[0].position, 0);
    assert_eq!(routine.entries[1].item_id, "exercise-001");
    assert_eq!(routine.entries[1].item_title, "Hanon No. 1");
    assert_eq!(routine.entries[1].item_type, ItemKind::Exercise);
    assert_eq!(routine.entries[1].position, 1);
}

#[tokio::test]
async fn create_routine_empty_name_returns_400() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::post_json(
        app,
        "/api/routines",
        json!({
            "name": "",
            "entries": [
                {
                    "item_id": "piece-001",
                    "item_title": "Clair de Lune",
                    "item_type": "piece"
                }
            ]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_routine_no_entries_returns_400() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::post_json(
        app,
        "/api/routines",
        json!({
            "name": "Empty Routine",
            "entries": []
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ── List ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_routines() {
    let app = common::setup_test_app().await;

    common::post_json(app.clone(), "/api/routines", sample_routine_body()).await;
    common::post_json(
        app.clone(),
        "/api/routines",
        json!({
            "name": "Evening Practice",
            "entries": [
                {
                    "item_id": "piece-002",
                    "item_title": "Moonlight Sonata",
                    "item_type": "piece"
                }
            ]
        }),
    )
    .await;

    let (status, body) = common::get(app, "/api/routines").await;
    assert_eq!(status, StatusCode::OK);
    let routines: Vec<Routine> = common::json(&body);
    assert_eq!(routines.len(), 2);

    // Verify entries are included in list response
    assert_eq!(routines[0].entries.len(), 2);
    assert_eq!(routines[1].entries.len(), 1);
}

#[tokio::test]
async fn list_routines_empty() {
    let app = common::setup_test_app().await;
    let (status, body) = common::get(app, "/api/routines").await;

    assert_eq!(status, StatusCode::OK);
    let routines: Vec<Routine> = common::json(&body);
    assert!(routines.is_empty());
}

// ── Get by ID ──────────────────────────────────────────────────────────

#[tokio::test]
async fn get_routine_by_id() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(app.clone(), "/api/routines", sample_routine_body()).await;
    let created: Routine = common::json(&body);

    let (status, body) = common::get(app, &format!("/api/routines/{}", created.id)).await;
    assert_eq!(status, StatusCode::OK);
    let fetched: Routine = common::json(&body);
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "Morning Warm-Up");
    assert_eq!(fetched.entries.len(), 2);
}

#[tokio::test]
async fn get_routine_not_found_returns_404() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::get(app, "/api/routines/nonexistent-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Update ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn update_routine_name() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(app.clone(), "/api/routines", sample_routine_body()).await;
    let created: Routine = common::json(&body);

    let (status, body) = common::put_json(
        app,
        &format!("/api/routines/{}", created.id),
        json!({
            "name": "Afternoon Warm-Up",
            "entries": [
                {
                    "item_id": "piece-001",
                    "item_title": "Clair de Lune",
                    "item_type": "piece"
                },
                {
                    "item_id": "exercise-001",
                    "item_title": "Hanon No. 1",
                    "item_type": "exercise"
                }
            ]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let updated: Routine = common::json(&body);
    assert_eq!(updated.name, "Afternoon Warm-Up");
    assert_eq!(updated.entries.len(), 2);
    assert!(updated.updated_at > created.updated_at);
}

#[tokio::test]
async fn update_routine_entries() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(app.clone(), "/api/routines", sample_routine_body()).await;
    let created: Routine = common::json(&body);
    assert_eq!(created.entries.len(), 2);

    // Replace entries with a single different entry
    let (status, body) = common::put_json(
        app,
        &format!("/api/routines/{}", created.id),
        json!({
            "name": "Morning Warm-Up",
            "entries": [
                {
                    "item_id": "piece-003",
                    "item_title": "Gymnopedie No. 1",
                    "item_type": "piece"
                }
            ]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let updated: Routine = common::json(&body);
    assert_eq!(updated.entries.len(), 1);
    assert_eq!(updated.entries[0].item_id, "piece-003");
    assert_eq!(updated.entries[0].item_title, "Gymnopedie No. 1");
    assert_eq!(updated.entries[0].position, 0);
}

#[tokio::test]
async fn update_routine_not_found_returns_404() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::put_json(
        app,
        "/api/routines/nonexistent-id",
        json!({
            "name": "Ghost Routine",
            "entries": [
                {
                    "item_id": "piece-001",
                    "item_title": "Clair de Lune",
                    "item_type": "piece"
                }
            ]
        }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Delete ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_routine() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(app.clone(), "/api/routines", sample_routine_body()).await;
    let created: Routine = common::json(&body);

    let (status, _body) =
        common::delete(app.clone(), &format!("/api/routines/{}", created.id)).await;
    assert_eq!(status, StatusCode::OK);

    // Verify gone
    let (status, _body) = common::get(app, &format!("/api/routines/{}", created.id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_routine_not_found_returns_404() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::delete(app, "/api/routines/nonexistent-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_routine_cascades_entries() {
    let app = common::setup_test_app().await;

    // Create two routines
    let (_, body) = common::post_json(app.clone(), "/api/routines", sample_routine_body()).await;
    let routine_a: Routine = common::json(&body);

    let (_, body) = common::post_json(
        app.clone(),
        "/api/routines",
        json!({
            "name": "Other Routine",
            "entries": [
                {
                    "item_id": "piece-999",
                    "item_title": "Separate Piece",
                    "item_type": "piece"
                }
            ]
        }),
    )
    .await;
    let routine_b: Routine = common::json(&body);

    // Delete routine A
    let (status, _body) =
        common::delete(app.clone(), &format!("/api/routines/{}", routine_a.id)).await;
    assert_eq!(status, StatusCode::OK);

    // Routine A is gone
    let (status, _body) =
        common::get(app.clone(), &format!("/api/routines/{}", routine_a.id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Routine B is unaffected and still has its entries
    let (status, body) = common::get(app, &format!("/api/routines/{}", routine_b.id)).await;
    assert_eq!(status, StatusCode::OK);
    let fetched_b: Routine = common::json(&body);
    assert_eq!(fetched_b.entries.len(), 1);
    assert_eq!(fetched_b.entries[0].item_title, "Separate Piece");
}
