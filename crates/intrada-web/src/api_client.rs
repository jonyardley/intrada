//! HTTP API client for communicating with the intrada-api server.
//!
//! This module replaces localStorage persistence with HTTP calls to the REST API.
//! All functions are async and return `Result<T, ApiError>`.

use gloo_net::http::Request;
use serde::Serialize;

use intrada_core::{
    CreateExercise, CreatePiece, Exercise, Piece, PracticeSession, UpdateExercise, UpdatePiece,
};

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

// ---------------------------------------------------------------------------
// Library Operations
// ---------------------------------------------------------------------------

/// Fetch all pieces from the API.
pub async fn fetch_pieces() -> Result<Vec<Piece>, ApiError> {
    let response = Request::get(&endpoint("/api/pieces"))
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !response.ok() {
        let msg = parse_error_body(response).await;
        return Err(ApiError::Server(0, msg));
    }

    response
        .json::<Vec<Piece>>()
        .await
        .map_err(|e| ApiError::Deserialize(e.to_string()))
}

/// Fetch all exercises from the API.
pub async fn fetch_exercises() -> Result<Vec<Exercise>, ApiError> {
    let response = Request::get(&endpoint("/api/exercises"))
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !response.ok() {
        let msg = parse_error_body(response).await;
        return Err(ApiError::Server(0, msg));
    }

    response
        .json::<Vec<Exercise>>()
        .await
        .map_err(|e| ApiError::Deserialize(e.to_string()))
}

/// Create a new piece on the server.
pub async fn create_piece(piece: &CreatePiece) -> Result<Piece, ApiError> {
    post_json("/api/pieces", piece).await
}

/// Update an existing piece on the server.
pub async fn update_piece(id: &str, piece: &UpdatePiece) -> Result<Piece, ApiError> {
    put_json(&format!("/api/pieces/{id}"), piece).await
}

/// Delete a piece from the server.
pub async fn delete_piece(id: &str) -> Result<(), ApiError> {
    delete(&format!("/api/pieces/{id}")).await
}

/// Create a new exercise on the server.
pub async fn create_exercise(exercise: &CreateExercise) -> Result<Exercise, ApiError> {
    post_json("/api/exercises", exercise).await
}

/// Update an existing exercise on the server.
pub async fn update_exercise(id: &str, exercise: &UpdateExercise) -> Result<Exercise, ApiError> {
    put_json(&format!("/api/exercises/{id}"), exercise).await
}

/// Delete an exercise from the server.
pub async fn delete_exercise(id: &str) -> Result<(), ApiError> {
    delete(&format!("/api/exercises/{id}")).await
}

// ---------------------------------------------------------------------------
// Session Operations
// ---------------------------------------------------------------------------

/// Fetch all completed practice sessions from the API.
pub async fn fetch_sessions() -> Result<Vec<PracticeSession>, ApiError> {
    let response = Request::get(&endpoint("/api/sessions"))
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !response.ok() {
        let msg = parse_error_body(response).await;
        return Err(ApiError::Server(0, msg));
    }

    response
        .json::<Vec<PracticeSession>>()
        .await
        .map_err(|e| ApiError::Deserialize(e.to_string()))
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
// Generic HTTP helpers
// ---------------------------------------------------------------------------

/// POST JSON body to an endpoint and deserialise the 201 response.
async fn post_json<B: Serialize, R: serde::de::DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<R, ApiError> {
    let response = Request::post(&endpoint(path))
        .header("Content-Type", "application/json")
        .json(body)
        .map_err(|e| ApiError::Deserialize(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

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
async fn put_json<B: Serialize, R: serde::de::DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<R, ApiError> {
    let response = Request::put(&endpoint(path))
        .header("Content-Type", "application/json")
        .json(body)
        .map_err(|e| ApiError::Deserialize(e.to_string()))?
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

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
async fn delete(path: &str) -> Result<(), ApiError> {
    let response = Request::delete(&endpoint(path))
        .send()
        .await
        .map_err(|e| ApiError::Network(e.to_string()))?;

    if !response.ok() {
        let status = response.status();
        let msg = parse_error_body(response).await;
        return Err(ApiError::Server(status, msg));
    }

    Ok(())
}
