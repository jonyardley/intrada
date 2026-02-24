use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use chrono::Utc;
use serde::Deserialize;

use intrada_core::domain::goal::{Goal, GoalStatus};
use intrada_core::domain::types::CreateGoal;
use intrada_core::validation;

use crate::auth::AuthUser;
use crate::db;
use crate::error::ApiError;
use crate::state::AppState;

/// API-specific update DTO that includes status changes.
///
/// The core `UpdateGoal` doesn't include status (it uses separate GoalEvents),
/// but the REST API handles status transitions via PUT for simplicity.
#[derive(Deserialize, Debug)]
pub struct ApiUpdateGoal {
    pub title: Option<String>,
    pub status: Option<String>,
    pub deadline: Option<Option<chrono::DateTime<chrono::Utc>>>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_goals).post(create_goal))
        .route("/{id}", get(get_goal).put(update_goal).delete(delete_goal))
}

async fn list_goals(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
) -> Result<Json<Vec<Goal>>, ApiError> {
    let conn = state.db.connect()?;
    let goals = db::goals::list_goals(&conn, &user_id).await?;
    Ok(Json(goals))
}

async fn get_goal(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<Goal>, ApiError> {
    let conn = state.db.connect()?;
    let goal = db::goals::get_goal(&conn, &id, &user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Goal not found: {id}")))?;
    Ok(Json(goal))
}

async fn create_goal(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Json(input): Json<CreateGoal>,
) -> Result<(StatusCode, Json<Goal>), ApiError> {
    validation::validate_create_goal(&input)?;
    let conn = state.db.connect()?;
    let goal = db::goals::insert_goal(&conn, &user_id, &input).await?;
    Ok((StatusCode::CREATED, Json(goal)))
}

async fn update_goal(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
    Json(input): Json<ApiUpdateGoal>,
) -> Result<Json<Goal>, ApiError> {
    // Validate title if provided
    if let Some(ref title) = input.title {
        if title.trim().is_empty() {
            return Err(ApiError::Validation("Title cannot be empty".to_string()));
        }
        if title.len() > intrada_core::validation::MAX_GOAL_TITLE {
            return Err(ApiError::Validation(format!(
                "Title must be {} characters or fewer",
                intrada_core::validation::MAX_GOAL_TITLE
            )));
        }
    }

    let conn = state.db.connect()?;
    let mut goal = db::goals::get_goal(&conn, &id, &user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Goal not found: {id}")))?;

    // Apply title update
    if let Some(title) = input.title {
        goal.title = title;
    }

    // Apply deadline update (three-state: None = skip, Some(None) = clear, Some(Some(d)) = set)
    if let Some(deadline) = input.deadline {
        goal.deadline = deadline;
    }

    // Apply status transition with enforcement
    if let Some(ref status_str) = input.status {
        let new_status = match status_str.as_str() {
            "active" => GoalStatus::Active,
            "completed" => GoalStatus::Completed,
            "archived" => GoalStatus::Archived,
            other => {
                return Err(ApiError::Validation(format!(
                    "Invalid status: {other}. Must be active, completed, or archived"
                )));
            }
        };

        // Enforce status transitions
        match (&goal.status, &new_status) {
            // Active → Completed (final achievement)
            (GoalStatus::Active, GoalStatus::Completed) => {
                goal.status = GoalStatus::Completed;
                goal.completed_at = Some(Utc::now());
            }
            // Active → Archived (soft removal)
            (GoalStatus::Active, GoalStatus::Archived) => {
                goal.status = GoalStatus::Archived;
            }
            // Archived → Active (reactivate)
            (GoalStatus::Archived, GoalStatus::Active) => {
                goal.status = GoalStatus::Active;
                goal.completed_at = None;
            }
            // No-op: same status
            (current, new) if current == new => {}
            // All other transitions are rejected
            (current, new) => {
                return Err(ApiError::Validation(format!(
                    "Cannot transition from {current:?} to {new:?}"
                )));
            }
        }
    }

    goal.updated_at = Utc::now();
    db::goals::update_goal(&conn, &user_id, &goal).await?;

    Ok(Json(goal))
}

async fn delete_goal(
    State(state): State<AppState>,
    AuthUser(user_id): AuthUser,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let conn = state.db.connect()?;
    let deleted = db::goals::delete_goal(&conn, &id, &user_id).await?;
    if deleted {
        Ok(Json(serde_json::json!({ "message": "Goal deleted" })))
    } else {
        Err(ApiError::NotFound(format!("Goal not found: {id}")))
    }
}
