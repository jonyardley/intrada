//! HTTP API client for communicating with the intrada-api server.
//!
//! This module replaces localStorage persistence with HTTP calls to the REST API.
//! All functions are async and return `Result<T, ApiError>`.

use gloo_net::http::Request;
use serde::Serialize;

use intrada_core::domain::goal::Goal;
use intrada_core::domain::types::CreateGoal;
use intrada_core::{CreateItem, Item, PracticeSession, Routine, UpdateItem};

use crate::clerk_bindings;

/// Compile-time API base URL with fallback to production.
const API_BASE_URL: &str = match option_env!("INTRADA_API_URL") {
    Some(url) => url,
    None => "https://intrada-api.fly.dev",
};

/// Build a full endpoint URL from a path (e.g., "/api/pieces").
fn endpoint(path: &str) -> String {
    format!("{}{}", API_BASE_URL, path)
}

/// Errors that can occur when communicating with the API.
#[derive(Debug)]
pub enum ApiError {
    /// Connection failed, timeout, or other network issue.
    Network(String),
    /// HTTP error response (status code + error message from server).
    Server(u16, String),
    /// JSON parsing/serialisation failed.
    Deserialize(String),
}

impl ApiError {
    /// Convert to a user-friendly error message string.
    pub fn to_user_message(&self) -> String {
        match self {
            ApiError::Network(_) => {
                "Unable to connect to the server. Please check your connection.".to_string()
            }
            ApiError::Server(status, msg) => match status {
                400 => msg.clone(),
                401 => "Your session has expired. Please sign in again.".to_string(),
                404 => "The requested item was not found.".to_string(),
                _ => "The server encountered an error. Please try again.".to_string(),
            },
            ApiError::Deserialize(_) => "Received unexpected data from the server.".to_string(),
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Network(msg) => write!(f, "Network error: {msg}"),
            ApiError::Server(status, msg) => write!(f, "Server error ({status}): {msg}"),
            ApiError::Deserialize(msg) => write!(f, "Deserialize error: {msg}"),
        }
    }
}

/// Represents the error body returned by the API server: `{ "error": "..." }`.
#[derive(serde::Deserialize)]
struct ErrorBody {
    error: String,
}

/// Parse an error response body, falling back to a generic message.
async fn parse_error_body(response: gloo_net::http::Response) -> String {
    let status = response.status();
    match response.json::<ErrorBody>().await {
        Ok(body) => body.error,
        Err(_) => format!("HTTP {status}"),
    }
}

/// Get the current auth token (if available) for request headers.
async fn auth_header_value() -> Option<String> {
    let token = clerk_bindings::get_auth_token().await?;
    Some(format!("Bearer {token}"))
}

// ---------------------------------------------------------------------------
// Library Operations
// ---------------------------------------------------------------------------

/// Fetch all items from the API.
pub async fn fetch_items() -> Result<Vec<Item>, ApiError> {
    get_json("/api/items").await
}

/// Create a new item on the server.
pub async fn create_item(item: &CreateItem) -> Result<Item, ApiError> {
    post_json("/api/items", item).await
}

/// Update an existing item on the server.
pub async fn update_item(id: &str, item: &UpdateItem) -> Result<Item, ApiError> {
    put_json(&format!("/api/items/{id}"), item).await
}

/// Delete an item from the server.
pub async fn delete_item(id: &str) -> Result<(), ApiError> {
    delete(&format!("/api/items/{id}")).await
}

// ---------------------------------------------------------------------------
// Session Operations
// ---------------------------------------------------------------------------

/// Fetch all completed practice sessions from the API.
pub async fn fetch_sessions() -> Result<Vec<PracticeSession>, ApiError> {
    get_json("/api/sessions").await
}

/// Save a completed practice session to the server.
pub async fn create_session(session: &PracticeSession) -> Result<PracticeSession, ApiError> {
    post_json("/api/sessions", session).await
}

/// Delete a practice session from the server.
pub async fn delete_session(id: &str) -> Result<(), ApiError> {
    delete(&format!("/api/sessions/{id}")).await
}

// ---------------------------------------------------------------------------
// Routine Operations
// ---------------------------------------------------------------------------

/// Request body for creating a routine via the API.
#[derive(serde::Serialize)]
pub struct CreateRoutineApiRequest {
    pub name: String,
    pub entries: Vec<CreateRoutineEntryApiRequest>,
}

/// Entry within a create/update routine API request.
#[derive(serde::Serialize)]
pub struct CreateRoutineEntryApiRequest {
    pub item_id: String,
    pub item_title: String,
    pub item_type: String,
}

/// Request body for updating a routine via the API.
#[derive(serde::Serialize)]
pub struct UpdateRoutineApiRequest {
    pub name: String,
    pub entries: Vec<CreateRoutineEntryApiRequest>,
}

/// Fetch all routines from the API.
pub async fn fetch_routines() -> Result<Vec<Routine>, ApiError> {
    get_json("/api/routines").await
}

/// Create a new routine on the server.
pub async fn create_routine(routine: &CreateRoutineApiRequest) -> Result<Routine, ApiError> {
    post_json("/api/routines", routine).await
}

/// Update an existing routine on the server.
pub async fn update_routine(
    id: &str,
    routine: &UpdateRoutineApiRequest,
) -> Result<Routine, ApiError> {
    put_json(&format!("/api/routines/{id}"), routine).await
}

/// Delete a routine from the server.
pub async fn delete_routine(id: &str) -> Result<(), ApiError> {
    delete(&format!("/api/routines/{id}")).await
}

// ---------------------------------------------------------------------------
// Goal Operations
// ---------------------------------------------------------------------------

/// Request body for updating a goal via the API.
///
/// Mirrors the `ApiUpdateGoal` DTO on the server side. The core uses separate
/// GoalEvents for status changes, but the REST API handles everything via PUT.
#[derive(serde::Serialize)]
pub struct UpdateGoalApiRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<Option<chrono::DateTime<chrono::Utc>>>,
}

/// Fetch all goals from the API.
pub async fn fetch_goals() -> Result<Vec<Goal>, ApiError> {
    get_json("/api/goals").await
}

/// Create a new goal on the server.
pub async fn create_goal(goal: &CreateGoal) -> Result<Goal, ApiError> {
    post_json("/api/goals", goal).await
}

/// Update an existing goal on the server.
pub async fn update_goal(id: &str, update: &UpdateGoalApiRequest) -> Result<Goal, ApiError> {
    put_json(&format!("/api/goals/{id}"), update).await
}

/// Delete a goal from the server.
pub async fn delete_goal(id: &str) -> Result<(), ApiError> {
    delete(&format!("/api/goals/{id}")).await
}

// ---------------------------------------------------------------------------
// Generic HTTP helpers (with 401 retry)
// ---------------------------------------------------------------------------

/// GET an endpoint and deserialise the JSON response.
/// Retries once with a fresh auth token on 401.
async fn get_json<R: serde::de::DeserializeOwned>(path: &str) -> Result<R, ApiError> {
    let url = endpoint(path);
    let auth = auth_header_value().await;

    let mut req = Request::get(&url);
    if let Some(ref auth) = auth {
        req = req.header("Authorization", auth);
    }
    let response = req
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    // Retry once with fresh token on 401
    if response.status() == 401 {
        if let Some(fresh_auth) = auth_header_value().await {
            let retry_response = Request::get(&url)
                .header("Authorization", &fresh_auth)
                .send()
                .await
                .map_err(|e| ApiError::Network(e.to_string()))?;

            if !retry_response.ok() {
                let status = retry_response.status();
                let msg = parse_error_body(retry_response).await;
                return Err(ApiError::Server(status, msg));
            }

            return retry_response
                .json::<R>()
                .await
                .map_err(|e| ApiError::Deserialize(e.to_string()));
        }
        let msg = parse_error_body(response).await;
        return Err(ApiError::Server(401, msg));
    }

    if !response.ok() {
        let status = response.status();
        let msg = parse_error_body(response).await;
        return Err(ApiError::Server(status, msg));
    }

    response
        .json::<R>()
        .await
        .map_err(|e| ApiError::Deserialize(e.to_string()))
}

/// POST JSON body to an endpoint and deserialise the 201 response.
/// Retries once with a fresh auth token on 401.
async fn post_json<B: Serialize, R: serde::de::DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<R, ApiError> {
    let url = endpoint(path);
    let auth = auth_header_value().await;

    let mut req = Request::post(&url).header("Content-Type", "application/json");
    if let Some(ref auth) = auth {
        req = req.header("Authorization", auth);
    }
    let response = req
        .json(body)
        .map_err(|e| ApiError::Deserialize(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    // Retry once with fresh token on 401
    if response.status() == 401 {
        if let Some(fresh_auth) = auth_header_value().await {
            let retry_response = Request::post(&url)
                .header("Content-Type", "application/json")
                .header("Authorization", &fresh_auth)
                .json(body)
                .map_err(|e| ApiError::Deserialize(e.to_string()))?
                .send()
                .await
                .map_err(|e| ApiError::Network(e.to_string()))?;

            if !retry_response.ok() {
                let status = retry_response.status();
                let msg = parse_error_body(retry_response).await;
                return Err(ApiError::Server(status, msg));
            }

            return retry_response
                .json::<R>()
                .await
                .map_err(|e| ApiError::Deserialize(e.to_string()));
        }
        let msg = parse_error_body(response).await;
        return Err(ApiError::Server(401, msg));
    }

    if !response.ok() {
        let status = response.status();
        let msg = parse_error_body(response).await;
        return Err(ApiError::Server(status, msg));
    }

    response
        .json::<R>()
        .await
        .map_err(|e| ApiError::Deserialize(e.to_string()))
}

/// PUT JSON body to an endpoint and deserialise the 200 response.
/// Retries once with a fresh auth token on 401.
async fn put_json<B: Serialize, R: serde::de::DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<R, ApiError> {
    let url = endpoint(path);
    let auth = auth_header_value().await;

    let mut req = Request::put(&url).header("Content-Type", "application/json");
    if let Some(ref auth) = auth {
        req = req.header("Authorization", auth);
    }
    let response = req
        .json(body)
        .map_err(|e| ApiError::Deserialize(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    // Retry once with fresh token on 401
    if response.status() == 401 {
        if let Some(fresh_auth) = auth_header_value().await {
            let retry_response = Request::put(&url)
                .header("Content-Type", "application/json")
                .header("Authorization", &fresh_auth)
                .json(body)
                .map_err(|e| ApiError::Deserialize(e.to_string()))?
                .send()
                .await
                .map_err(|e| ApiError::Network(e.to_string()))?;

            if !retry_response.ok() {
                let status = retry_response.status();
                let msg = parse_error_body(retry_response).await;
                return Err(ApiError::Server(status, msg));
            }

            return retry_response
                .json::<R>()
                .await
                .map_err(|e| ApiError::Deserialize(e.to_string()));
        }
        let msg = parse_error_body(response).await;
        return Err(ApiError::Server(401, msg));
    }

    if !response.ok() {
        let status = response.status();
        let msg = parse_error_body(response).await;
        return Err(ApiError::Server(status, msg));
    }

    response
        .json::<R>()
        .await
        .map_err(|e| ApiError::Deserialize(e.to_string()))
}

/// DELETE an endpoint. Expects 200 OK with no body needed.
/// Retries once with a fresh auth token on 401.
async fn delete(path: &str) -> Result<(), ApiError> {
    let url = endpoint(path);
    let auth = auth_header_value().await;

    let mut req = Request::delete(&url);
    if let Some(ref auth) = auth {
        req = req.header("Authorization", auth);
    }
    let response = req
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    // Retry once with fresh token on 401
    if response.status() == 401 {
        if let Some(fresh_auth) = auth_header_value().await {
            let retry_response = Request::delete(&url)
                .header("Authorization", &fresh_auth)
                .send()
                .await
                .map_err(|e| ApiError::Network(e.to_string()))?;

            if !retry_response.ok() {
                let status = retry_response.status();
                let msg = parse_error_body(retry_response).await;
                return Err(ApiError::Server(status, msg));
            }

            return Ok(());
        }
        let msg = parse_error_body(response).await;
        return Err(ApiError::Server(401, msg));
    }

    if !response.ok() {
        let status = response.status();
        let msg = parse_error_body(response).await;
        return Err(ApiError::Server(status, msg));
    }

    Ok(())
}
