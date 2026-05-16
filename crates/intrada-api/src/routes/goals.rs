use axum::extract::{DefaultBodyLimit, Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use tower_http::limit::RequestBodyLimitLayer;

use intrada_core::domain::goal::Goal;
use intrada_core::domain::types::{CreateGoal, LinkGoalItem, UpdateGoal};

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::services;
use crate::services::goals::MAX_PHOTO_SIZE;
use crate::state::AppState;

/// HTTP body limit for photo uploads: 5 MB photo + multipart overhead.
const PHOTO_BODY_LIMIT: usize = 6 * 1024 * 1024;

#[derive(serde::Deserialize)]
struct ListGoalsQuery {
    status: Option<String>,
}

pub fn router() -> Router<AppState> {
    // Photo upload gets its own sub-router so the 6 MB body limit is
    // scoped to just that route — JSON endpoints keep axum's default.
    let photo_upload = Router::new()
        .route("/{id}/photos", post(upload_photo))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(PHOTO_BODY_LIMIT));

    Router::new()
        .route("/", get(list_goals).post(create_goal))
        .route("/{id}", get(get_goal).put(update_goal).delete(delete_goal))
        .route("/{id}/photos/{photo_id}", delete(delete_photo))
        .route("/{id}/items", post(link_item))
        .route("/{id}/items/{item_id}", delete(unlink_item))
        .merge(photo_upload)
}

async fn list_goals(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Query(query): Query<ListGoalsQuery>,
) -> Result<Json<Vec<Goal>>, ApiError> {
    let r2 = state.r2.clone();
    let goals = state
        .with_transient_retry(|conn| {
            let user_id = user_id.clone();
            let status = query.status.clone();
            let r2 = r2.clone();
            async move {
                services::goals::list_goals(&conn, r2.as_ref(), &user_id, status.as_deref()).await
            }
        })
        .await?;
    Ok(Json(goals))
}

async fn get_goal(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Goal>, ApiError> {
    let r2 = state.r2.clone();
    let goal = state
        .with_transient_retry(|conn| {
            let id = id.clone();
            let user_id = user_id.clone();
            let r2 = r2.clone();
            async move { services::goals::get_goal(&conn, r2.as_ref(), &id, &user_id).await }
        })
        .await?;
    Ok(Json(goal))
}

async fn create_goal(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Json(input): Json<CreateGoal>,
) -> Result<(StatusCode, Json<Goal>), ApiError> {
    let conn = state.conn();
    let goal = services::goals::create_goal(&conn, state.r2.as_ref(), &user_id, &input).await?;
    Ok((StatusCode::CREATED, Json(goal)))
}

async fn update_goal(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path(id): Path<String>,
    Json(input): Json<UpdateGoal>,
) -> Result<Json<Goal>, ApiError> {
    let conn = state.conn();
    let goal =
        services::goals::update_goal(&conn, state.r2.as_ref(), &id, &user_id, &input).await?;
    Ok(Json(goal))
}

async fn delete_goal(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let conn = state.conn();
    services::goals::delete_goal(&conn, state.r2.as_ref(), &id, &user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Photo endpoints ────────────────────────────────────────────────────

async fn upload_photo(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path(id): Path<String>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let r2 = state.r2()?;
    let conn = state.conn();

    // Extract the photo field from multipart
    let field = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::Validation(format!("Invalid multipart data: {e}")))?
        .ok_or_else(|| ApiError::Validation("No photo field in request".into()))?;

    let data = field
        .bytes()
        .await
        .map_err(|e| ApiError::Validation(format!("Failed to read photo data: {e}")))?;

    // Early body-size guard before handing off to the service. The service
    // checks again (so non-HTTP callers like MCP get the same behaviour),
    // but rejecting here avoids a wasted clone of the bytes into the service.
    if data.len() > MAX_PHOTO_SIZE {
        return Err(ApiError::Validation(format!(
            "Photo exceeds maximum size of {} MB",
            MAX_PHOTO_SIZE / (1024 * 1024)
        )));
    }

    let photo = services::goals::upload_goal_photo(&conn, r2, &user_id, &id, &data).await?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": photo.id,
            "url": photo.url,
            "created_at": photo.created_at.to_rfc3339(),
        })),
    ))
}

async fn delete_photo(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path((id, photo_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let conn = state.conn();
    services::goals::delete_goal_photo(&conn, state.r2.as_ref(), &user_id, &id, &photo_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ── Goal item endpoints ────────────────────────────────────────────────

async fn link_item(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path(id): Path<String>,
    Json(input): Json<LinkGoalItem>,
) -> Result<StatusCode, ApiError> {
    let conn = state.conn();
    services::goals::link_item(&conn, &id, &user_id, &input).await?;
    Ok(StatusCode::CREATED)
}

async fn unlink_item(
    State(state): State<AppState>,
    AuthUser { user_id, .. }: AuthUser,
    Path((id, item_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let conn = state.conn();
    services::goals::unlink_item(&conn, &id, &item_id, &user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
