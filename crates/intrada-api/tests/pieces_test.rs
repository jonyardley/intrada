mod common;

use axum::http::StatusCode;
use intrada_core::domain::piece::Piece;
use serde_json::json;

#[tokio::test]
async fn list_pieces_empty() {
    let app = common::setup_test_app().await;
    let (status, body) = common::get(app, "/api/pieces").await;

    assert_eq!(status, StatusCode::OK);
    let pieces: Vec<Piece> = common::json(&body);
    assert!(pieces.is_empty());
}

#[tokio::test]
async fn create_piece_valid() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/pieces",
        json!({
            "title": "Clair de Lune",
            "composer": "Claude Debussy",
            "tags": []
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let piece: Piece = common::json(&body);
    assert_eq!(piece.title, "Clair de Lune");
    assert_eq!(piece.composer, "Claude Debussy");
    assert!(!piece.id.is_empty());
}

#[tokio::test]
async fn create_piece_empty_title_returns_400() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::post_json(
        app,
        "/api/pieces",
        json!({
            "title": "",
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
        "/api/pieces",
        json!({
            "title": "Clair de Lune",
            "composer": "",
            "tags": []
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_piece_existing() {
    let app = common::setup_test_app().await;

    // Create a piece
    let (_, body) = common::post_json(
        app.clone(),
        "/api/pieces",
        json!({
            "title": "Clair de Lune",
            "composer": "Claude Debussy",
            "tags": []
        }),
    )
    .await;
    let created: Piece = common::json(&body);

    // Get it back
    let (status, body) = common::get(app, &format!("/api/pieces/{}", created.id)).await;
    assert_eq!(status, StatusCode::OK);
    let fetched: Piece = common::json(&body);
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.title, "Clair de Lune");
}

#[tokio::test]
async fn get_piece_not_found() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::get(app, "/api/pieces/nonexistent-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_piece_existing() {
    let app = common::setup_test_app().await;

    // Create
    let (_, body) = common::post_json(
        app.clone(),
        "/api/pieces",
        json!({
            "title": "Clair de Lune",
            "composer": "Claude Debussy",
            "tags": []
        }),
    )
    .await;
    let created: Piece = common::json(&body);

    // Update title only
    let (status, body) = common::put_json(
        app,
        &format!("/api/pieces/{}", created.id),
        json!({ "title": "Reverie" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let updated: Piece = common::json(&body);
    assert_eq!(updated.title, "Reverie");
    assert_eq!(updated.composer, "Claude Debussy"); // unchanged
    assert!(updated.updated_at > created.updated_at);
}

#[tokio::test]
async fn update_piece_not_found() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::put_json(
        app,
        "/api/pieces/nonexistent-id",
        json!({ "title": "Reverie" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_piece_existing() {
    let app = common::setup_test_app().await;

    // Create
    let (_, body) = common::post_json(
        app.clone(),
        "/api/pieces",
        json!({
            "title": "Clair de Lune",
            "composer": "Claude Debussy",
            "tags": []
        }),
    )
    .await;
    let created: Piece = common::json(&body);

    // Delete
    let (status, _body) = common::delete(app.clone(), &format!("/api/pieces/{}", created.id)).await;
    assert_eq!(status, StatusCode::OK);

    // Verify gone
    let (status, _body) = common::get(app, &format!("/api/pieces/{}", created.id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_piece_not_found() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::delete(app, "/api/pieces/nonexistent-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_piece_with_tags_roundtrip() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/pieces",
        json!({
            "title": "Clair de Lune",
            "composer": "Claude Debussy",
            "tags": ["impressionist", "piano"]
        }),
    )
    .await;
    let created: Piece = common::json(&body);
    assert_eq!(created.tags, vec!["impressionist", "piano"]);

    // Fetch and verify tags persisted
    let (_, body) = common::get(app, &format!("/api/pieces/{}", created.id)).await;
    let fetched: Piece = common::json(&body);
    assert_eq!(fetched.tags, vec!["impressionist", "piano"]);
}

#[tokio::test]
async fn create_piece_with_tempo() {
    let app = common::setup_test_app().await;

    // Tempo with both marking and BPM
    let (status, body) = common::post_json(
        app.clone(),
        "/api/pieces",
        json!({
            "title": "Pathetique",
            "composer": "Beethoven",
            "tempo": { "marking": "Grave", "bpm": 50 },
            "tags": []
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let piece: Piece = common::json(&body);
    let tempo = piece.tempo.unwrap();
    assert_eq!(tempo.marking.as_deref(), Some("Grave"));
    assert_eq!(tempo.bpm, Some(50));
}

#[tokio::test]
async fn create_piece_with_all_fields() {
    let app = common::setup_test_app().await;

    let (status, body) = common::post_json(
        app,
        "/api/pieces",
        json!({
            "title": "Clair de Lune",
            "composer": "Claude Debussy",
            "key": "Db Major",
            "tempo": { "marking": "Andante", "bpm": 66 },
            "notes": "Third movement of Suite bergamasque",
            "tags": ["impressionist", "piano"]
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let piece: Piece = common::json(&body);
    assert_eq!(piece.key.as_deref(), Some("Db Major"));
    assert_eq!(
        piece.notes.as_deref(),
        Some("Third movement of Suite bergamasque")
    );
    assert_eq!(
        piece.tempo.as_ref().and_then(|t| t.marking.as_deref()),
        Some("Andante")
    );
    assert_eq!(piece.tempo.as_ref().and_then(|t| t.bpm), Some(66));
}

#[tokio::test]
async fn list_pieces_returns_created_items() {
    let app = common::setup_test_app().await;

    // Create two pieces
    common::post_json(
        app.clone(),
        "/api/pieces",
        json!({ "title": "Piece A", "composer": "Composer A", "tags": [] }),
    )
    .await;
    common::post_json(
        app.clone(),
        "/api/pieces",
        json!({ "title": "Piece B", "composer": "Composer B", "tags": [] }),
    )
    .await;

    let (status, body) = common::get(app, "/api/pieces").await;
    assert_eq!(status, StatusCode::OK);
    let pieces: Vec<Piece> = common::json(&body);
    assert_eq!(pieces.len(), 2);
}

#[tokio::test]
async fn update_piece_partial_preserves_other_fields() {
    let app = common::setup_test_app().await;

    // Create with all fields
    let (_, body) = common::post_json(
        app.clone(),
        "/api/pieces",
        json!({
            "title": "Clair de Lune",
            "composer": "Claude Debussy",
            "key": "Db Major",
            "tempo": { "marking": "Andante", "bpm": 66 },
            "notes": "Third movement",
            "tags": ["piano"]
        }),
    )
    .await;
    let created: Piece = common::json(&body);

    // Update only the title
    let (_, body) = common::put_json(
        app,
        &format!("/api/pieces/{}", created.id),
        json!({ "title": "Reverie" }),
    )
    .await;
    let updated: Piece = common::json(&body);

    assert_eq!(updated.title, "Reverie");
    assert_eq!(updated.composer, "Claude Debussy");
    assert_eq!(updated.key.as_deref(), Some("Db Major"));
    assert_eq!(updated.notes.as_deref(), Some("Third movement"));
    assert_eq!(updated.tags, vec!["piano"]);
    assert_eq!(
        updated.tempo.as_ref().and_then(|t| t.marking.as_deref()),
        Some("Andante")
    );
}
