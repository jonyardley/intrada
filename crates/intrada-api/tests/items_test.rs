mod common;

use axum::http::StatusCode;
use intrada_core::domain::item::Item;
use serde_json::json;

#[tokio::test]
async fn list_items_empty() {
    let app = common::setup_test_app().await;
    let (status, body) = common::get(app, "/api/items").await;

    assert_eq!(status, StatusCode::OK);
    let items: Vec<Item> = common::json(&body);
    assert!(items.is_empty());
}

#[tokio::test]
async fn create_piece_valid() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/items",
        json!({
            "title": "Clair de Lune",
            "kind": "piece",
            "composer": "Claude Debussy",
            "tags": []
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let item: Item = common::json(&body);
    assert_eq!(item.title, "Clair de Lune");
    assert_eq!(item.composer.as_deref(), Some("Claude Debussy"));
    assert_eq!(item.kind.to_string(), "piece");
    assert!(!item.id.is_empty());
}

#[tokio::test]
async fn create_exercise_valid() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/items",
        json!({
            "title": "Hanon No. 1",
            "kind": "exercise",
            "tags": []
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let item: Item = common::json(&body);
    assert_eq!(item.title, "Hanon No. 1");
    assert_eq!(item.kind.to_string(), "exercise");
    assert!(!item.id.is_empty());
    assert!(item.composer.is_none());
}

#[tokio::test]
async fn create_piece_empty_title_returns_400() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::post_json(
        app,
        "/api/items",
        json!({
            "title": "",
            "kind": "piece",
            "composer": "Debussy",
            "tags": []
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_piece_empty_composer_returns_400() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::post_json(
        app,
        "/api/items",
        json!({
            "title": "Clair de Lune",
            "kind": "piece",
            "composer": "",
            "tags": []
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_item_existing() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/items",
        json!({
            "title": "Clair de Lune",
            "kind": "piece",
            "composer": "Claude Debussy",
            "tags": []
        }),
    )
    .await;
    let created: Item = common::json(&body);

    let (status, body) = common::get(app, &format!("/api/items/{}", created.id)).await;
    assert_eq!(status, StatusCode::OK);
    let fetched: Item = common::json(&body);
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.title, "Clair de Lune");
}

#[tokio::test]
async fn get_item_not_found() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::get(app, "/api/items/nonexistent-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_item_existing() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/items",
        json!({
            "title": "Clair de Lune",
            "kind": "piece",
            "composer": "Claude Debussy",
            "tags": []
        }),
    )
    .await;
    let created: Item = common::json(&body);

    let (status, body) = common::put_json(
        app,
        &format!("/api/items/{}", created.id),
        json!({ "title": "Reverie" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let updated: Item = common::json(&body);
    assert_eq!(updated.title, "Reverie");
    assert_eq!(updated.composer.as_deref(), Some("Claude Debussy")); // unchanged
    assert!(updated.updated_at > created.updated_at);
}

#[tokio::test]
async fn update_item_not_found() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::put_json(
        app,
        "/api/items/nonexistent-id",
        json!({ "title": "Reverie" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_item_existing() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/items",
        json!({
            "title": "Clair de Lune",
            "kind": "piece",
            "composer": "Claude Debussy",
            "tags": []
        }),
    )
    .await;
    let created: Item = common::json(&body);

    let (status, _body) = common::delete(app.clone(), &format!("/api/items/{}", created.id)).await;
    assert_eq!(status, StatusCode::OK);

    // Verify gone
    let (status, _body) = common::get(app, &format!("/api/items/{}", created.id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_item_not_found() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::delete(app, "/api/items/nonexistent-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_piece_with_tags_roundtrip() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/items",
        json!({
            "title": "Clair de Lune",
            "kind": "piece",
            "composer": "Claude Debussy",
            "tags": ["impressionist", "piano"]
        }),
    )
    .await;
    let created: Item = common::json(&body);
    assert_eq!(created.tags, vec!["impressionist", "piano"]);

    let (_, body) = common::get(app, &format!("/api/items/{}", created.id)).await;
    let fetched: Item = common::json(&body);
    assert_eq!(fetched.tags, vec!["impressionist", "piano"]);
}

#[tokio::test]
async fn create_piece_with_tempo() {
    let app = common::setup_test_app().await;

    let (status, body) = common::post_json(
        app,
        "/api/items",
        json!({
            "title": "Pathetique",
            "kind": "piece",
            "composer": "Beethoven",
            "tempo": { "marking": "Grave", "bpm": 50 },
            "tags": []
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let item: Item = common::json(&body);
    let tempo = item.tempo.unwrap();
    assert_eq!(tempo.marking.as_deref(), Some("Grave"));
    assert_eq!(tempo.bpm, Some(50));
}

#[tokio::test]
async fn create_piece_with_all_fields() {
    let app = common::setup_test_app().await;

    let (status, body) = common::post_json(
        app,
        "/api/items",
        json!({
            "title": "Clair de Lune",
            "kind": "piece",
            "composer": "Claude Debussy",
            "key": "Db Major",
            "tempo": { "marking": "Andante", "bpm": 66 },
            "notes": "Third movement of Suite bergamasque",
            "tags": ["impressionist", "piano"]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let item: Item = common::json(&body);
    assert_eq!(item.key.as_deref(), Some("Db Major"));
    assert_eq!(
        item.notes.as_deref(),
        Some("Third movement of Suite bergamasque")
    );
    assert_eq!(
        item.tempo.as_ref().and_then(|t| t.marking.as_deref()),
        Some("Andante")
    );
    assert_eq!(item.tempo.as_ref().and_then(|t| t.bpm), Some(66));
}

#[tokio::test]
async fn list_items_returns_created_items() {
    let app = common::setup_test_app().await;

    common::post_json(
        app.clone(),
        "/api/items",
        json!({ "title": "Piece A", "kind": "piece", "composer": "Composer A", "tags": [] }),
    )
    .await;
    common::post_json(
        app.clone(),
        "/api/items",
        json!({ "title": "Exercise B", "kind": "exercise", "tags": [] }),
    )
    .await;

    let (status, body) = common::get(app, "/api/items").await;
    assert_eq!(status, StatusCode::OK);
    let items: Vec<Item> = common::json(&body);
    assert_eq!(items.len(), 2);
}

#[tokio::test]
async fn update_item_partial_preserves_other_fields() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/items",
        json!({
            "title": "Clair de Lune",
            "kind": "piece",
            "composer": "Claude Debussy",
            "key": "Db Major",
            "tempo": { "marking": "Andante", "bpm": 66 },
            "notes": "Third movement",
            "tags": ["piano"]
        }),
    )
    .await;
    let created: Item = common::json(&body);

    let (_, body) = common::put_json(
        app,
        &format!("/api/items/{}", created.id),
        json!({ "title": "Reverie" }),
    )
    .await;
    let updated: Item = common::json(&body);

    assert_eq!(updated.title, "Reverie");
    assert_eq!(updated.composer.as_deref(), Some("Claude Debussy"));
    assert_eq!(updated.key.as_deref(), Some("Db Major"));
    assert_eq!(updated.notes.as_deref(), Some("Third movement"));
    assert_eq!(updated.tags, vec!["piano"]);
    assert_eq!(
        updated.tempo.as_ref().and_then(|t| t.marking.as_deref()),
        Some("Andante")
    );
}

#[tokio::test]
async fn create_exercise_with_all_fields() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/items",
        json!({
            "title": "Hanon No. 1",
            "kind": "exercise",
            "composer": "Charles-Louis Hanon",
            "category": "Technique",
            "key": "C Major",
            "tempo": { "marking": "Allegro", "bpm": 120 },
            "notes": "Focus on even finger strength",
            "tags": ["technique", "warm-up"]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let item: Item = common::json(&body);
    assert_eq!(item.title, "Hanon No. 1");
    assert_eq!(item.kind.to_string(), "exercise");
    assert_eq!(item.composer.as_deref(), Some("Charles-Louis Hanon"));
    assert_eq!(item.category.as_deref(), Some("Technique"));
    assert_eq!(item.key.as_deref(), Some("C Major"));
    assert_eq!(item.tags, vec!["technique", "warm-up"]);
}

#[tokio::test]
async fn exercise_optional_composer() {
    let app = common::setup_test_app().await;

    // Create without composer
    let (status, body) = common::post_json(
        app.clone(),
        "/api/items",
        json!({ "title": "Scales", "kind": "exercise", "tags": [] }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let item: Item = common::json(&body);
    assert!(item.composer.is_none());

    // Update to add composer
    let (status, body) = common::put_json(
        app,
        &format!("/api/items/{}", item.id),
        json!({ "composer": "Teacher" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let updated: Item = common::json(&body);
    assert_eq!(updated.composer.as_deref(), Some("Teacher"));
}
