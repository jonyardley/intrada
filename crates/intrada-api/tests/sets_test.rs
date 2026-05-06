mod common;

use axum::http::StatusCode;
use intrada_core::domain::item::ItemKind;
use intrada_core::domain::set::Set;
use serde_json::json;

fn sample_set_body() -> serde_json::Value {
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
async fn create_set_returns_201() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(app, "/api/sets", sample_set_body()).await;

    assert_eq!(status, StatusCode::CREATED);
    let set: Set = common::json(&body);
    assert!(!set.id.is_empty());
    assert_eq!(set.name, "Morning Warm-Up");
    assert_eq!(set.entries.len(), 2);
}

#[tokio::test]
async fn create_set_with_entries() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(app, "/api/sets", sample_set_body()).await;

    assert_eq!(status, StatusCode::CREATED);
    let set: Set = common::json(&body);

    assert_eq!(set.entries.len(), 2);
    assert_eq!(set.entries[0].item_id, "piece-001");
    assert_eq!(set.entries[0].item_title, "Clair de Lune");
    assert_eq!(set.entries[0].item_type, ItemKind::Piece);
    assert_eq!(set.entries[0].position, 0);
    assert_eq!(set.entries[1].item_id, "exercise-001");
    assert_eq!(set.entries[1].item_title, "Hanon No. 1");
    assert_eq!(set.entries[1].item_type, ItemKind::Exercise);
    assert_eq!(set.entries[1].position, 1);
}

#[tokio::test]
async fn create_set_empty_name_returns_400() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::post_json(
        app,
        "/api/sets",
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
async fn create_set_no_entries_returns_400() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::post_json(
        app,
        "/api/sets",
        json!({
            "name": "Empty Set",
            "entries": []
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

// ── List ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn list_sets() {
    let app = common::setup_test_app().await;

    common::post_json(app.clone(), "/api/sets", sample_set_body()).await;
    common::post_json(
        app.clone(),
        "/api/sets",
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

    let (status, body) = common::get(app, "/api/sets").await;
    assert_eq!(status, StatusCode::OK);
    let sets: Vec<Set> = common::json(&body);
    assert_eq!(sets.len(), 2);

    // Verify entries are included in list response
    assert_eq!(sets[0].entries.len(), 2);
    assert_eq!(sets[1].entries.len(), 1);
}

#[tokio::test]
async fn list_sets_empty() {
    let app = common::setup_test_app().await;
    let (status, body) = common::get(app, "/api/sets").await;

    assert_eq!(status, StatusCode::OK);
    let sets: Vec<Set> = common::json(&body);
    assert!(sets.is_empty());
}

// ── Get by ID ──────────────────────────────────────────────────────────

#[tokio::test]
async fn get_set_by_id() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(app.clone(), "/api/sets", sample_set_body()).await;
    let created: Set = common::json(&body);

    let (status, body) = common::get(app, &format!("/api/sets/{}", created.id)).await;
    assert_eq!(status, StatusCode::OK);
    let fetched: Set = common::json(&body);
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.name, "Morning Warm-Up");
    assert_eq!(fetched.entries.len(), 2);
}

#[tokio::test]
async fn get_set_not_found_returns_404() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::get(app, "/api/sets/nonexistent-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

// ── Update ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn update_set_name() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(app.clone(), "/api/sets", sample_set_body()).await;
    let created: Set = common::json(&body);

    let (status, body) = common::put_json(
        app,
        &format!("/api/sets/{}", created.id),
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
    let updated: Set = common::json(&body);
    assert_eq!(updated.name, "Afternoon Warm-Up");
    assert_eq!(updated.entries.len(), 2);
    assert!(updated.updated_at > created.updated_at);
}

#[tokio::test]
async fn update_set_entries() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(app.clone(), "/api/sets", sample_set_body()).await;
    let created: Set = common::json(&body);
    assert_eq!(created.entries.len(), 2);

    // Replace entries with a single different entry
    let (status, body) = common::put_json(
        app,
        &format!("/api/sets/{}", created.id),
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
    let updated: Set = common::json(&body);
    assert_eq!(updated.entries.len(), 1);
    assert_eq!(updated.entries[0].item_id, "piece-003");
    assert_eq!(updated.entries[0].item_title, "Gymnopedie No. 1");
    assert_eq!(updated.entries[0].position, 0);
}

#[tokio::test]
async fn update_set_not_found_returns_404() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::put_json(
        app,
        "/api/sets/nonexistent-id",
        json!({
            "name": "Ghost Set",
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
async fn delete_set() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(app.clone(), "/api/sets", sample_set_body()).await;
    let created: Set = common::json(&body);

    let (status, _body) = common::delete(app.clone(), &format!("/api/sets/{}", created.id)).await;
    assert_eq!(status, StatusCode::OK);

    // Verify gone
    let (status, _body) = common::get(app, &format!("/api/sets/{}", created.id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_set_not_found_returns_404() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::delete(app, "/api/sets/nonexistent-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_set_cascades_entries() {
    let app = common::setup_test_app().await;

    // Create two sets
    let (_, body) = common::post_json(app.clone(), "/api/sets", sample_set_body()).await;
    let set_a: Set = common::json(&body);

    let (_, body) = common::post_json(
        app.clone(),
        "/api/sets",
        json!({
            "name": "Other Set",
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
    let set_b: Set = common::json(&body);

    // Delete set A
    let (status, _body) = common::delete(app.clone(), &format!("/api/sets/{}", set_a.id)).await;
    assert_eq!(status, StatusCode::OK);

    // Set A is gone
    let (status, _body) = common::get(app.clone(), &format!("/api/sets/{}", set_a.id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Set B is unaffected and still has its entries
    let (status, body) = common::get(app, &format!("/api/sets/{}", set_b.id)).await;
    assert_eq!(status, StatusCode::OK);
    let fetched_b: Set = common::json(&body);
    assert_eq!(fetched_b.entries.len(), 1);
    assert_eq!(fetched_b.entries[0].item_title, "Separate Piece");
}
