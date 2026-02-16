mod common;

use axum::http::StatusCode;
use intrada_core::domain::exercise::Exercise;
use serde_json::json;

#[tokio::test]
async fn list_exercises_empty() {
    let app = common::setup_test_app().await;
    let (status, body) = common::get(app, "/api/exercises").await;

    assert_eq!(status, StatusCode::OK);
    let exercises: Vec<Exercise> = common::json(&body);
    assert!(exercises.is_empty());
}

#[tokio::test]
async fn create_exercise_valid() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/exercises",
        json!({
            "title": "Hanon No. 1",
            "tags": []
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let exercise: Exercise = common::json(&body);
    assert_eq!(exercise.title, "Hanon No. 1");
    assert!(!exercise.id.is_empty());
    assert!(exercise.composer.is_none()); // composer is optional
}

#[tokio::test]
async fn create_exercise_with_all_fields() {
    let app = common::setup_test_app().await;
    let (status, body) = common::post_json(
        app,
        "/api/exercises",
        json!({
            "title": "Hanon No. 1",
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
    let exercise: Exercise = common::json(&body);
    assert_eq!(exercise.title, "Hanon No. 1");
    assert_eq!(exercise.composer.as_deref(), Some("Charles-Louis Hanon"));
    assert_eq!(exercise.category.as_deref(), Some("Technique"));
    assert_eq!(exercise.key.as_deref(), Some("C Major"));
    assert_eq!(exercise.tags, vec!["technique", "warm-up"]);
}

#[tokio::test]
async fn create_exercise_empty_title_returns_400() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::post_json(
        app,
        "/api/exercises",
        json!({
            "title": "",
            "tags": []
        }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_exercise_existing() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/exercises",
        json!({ "title": "Hanon No. 1", "tags": [] }),
    )
    .await;
    let created: Exercise = common::json(&body);

    let (status, body) = common::get(app, &format!("/api/exercises/{}", created.id)).await;
    assert_eq!(status, StatusCode::OK);
    let fetched: Exercise = common::json(&body);
    assert_eq!(fetched.id, created.id);
    assert_eq!(fetched.title, "Hanon No. 1");
}

#[tokio::test]
async fn get_exercise_not_found() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::get(app, "/api/exercises/nonexistent-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_exercise_existing() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/exercises",
        json!({
            "title": "Hanon No. 1",
            "composer": "Charles-Louis Hanon",
            "tags": []
        }),
    )
    .await;
    let created: Exercise = common::json(&body);

    let (status, body) = common::put_json(
        app,
        &format!("/api/exercises/{}", created.id),
        json!({ "title": "Hanon No. 2" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let updated: Exercise = common::json(&body);
    assert_eq!(updated.title, "Hanon No. 2");
    assert_eq!(updated.composer.as_deref(), Some("Charles-Louis Hanon")); // unchanged
    assert!(updated.updated_at > created.updated_at);
}

#[tokio::test]
async fn update_exercise_not_found() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::put_json(
        app,
        "/api/exercises/nonexistent-id",
        json!({ "title": "Hanon No. 2" }),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_exercise_existing() {
    let app = common::setup_test_app().await;

    let (_, body) = common::post_json(
        app.clone(),
        "/api/exercises",
        json!({ "title": "Hanon No. 1", "tags": [] }),
    )
    .await;
    let created: Exercise = common::json(&body);

    let (status, _body) =
        common::delete(app.clone(), &format!("/api/exercises/{}", created.id)).await;
    assert_eq!(status, StatusCode::OK);

    // Verify gone
    let (status, _body) = common::get(app, &format!("/api/exercises/{}", created.id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_exercise_not_found() {
    let app = common::setup_test_app().await;
    let (status, _body) = common::delete(app, "/api/exercises/nonexistent-id").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_exercises_returns_created_items() {
    let app = common::setup_test_app().await;

    common::post_json(
        app.clone(),
        "/api/exercises",
        json!({ "title": "Exercise A", "tags": [] }),
    )
    .await;
    common::post_json(
        app.clone(),
        "/api/exercises",
        json!({ "title": "Exercise B", "tags": [] }),
    )
    .await;

    let (status, body) = common::get(app, "/api/exercises").await;
    assert_eq!(status, StatusCode::OK);
    let exercises: Vec<Exercise> = common::json(&body);
    assert_eq!(exercises.len(), 2);
}

#[tokio::test]
async fn exercise_optional_composer() {
    let app = common::setup_test_app().await;

    // Create without composer
    let (status, body) = common::post_json(
        app.clone(),
        "/api/exercises",
        json!({ "title": "Scales", "tags": [] }),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let exercise: Exercise = common::json(&body);
    assert!(exercise.composer.is_none());

    // Update to add composer
    let (status, body) = common::put_json(
        app,
        &format!("/api/exercises/{}", exercise.id),
        json!({ "composer": "Teacher" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let updated: Exercise = common::json(&body);
    assert_eq!(updated.composer.as_deref(), Some("Teacher"));
}
