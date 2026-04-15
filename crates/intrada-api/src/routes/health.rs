use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    // Try the cached connection first.
    if state.conn().query("SELECT 1", ()).await.is_ok() {
        return (
            StatusCode::OK,
            Json(serde_json::json!({ "status": "ok", "database": "ok" })),
        );
    }

    // The shared HTTP session may have rotted (idle timeout, machine
    // suspend/resume, etc.). Rebuild it once and retry — if this succeeds,
    // every subsequent request also recovers, since they all read from the
    // same slot.
    tracing::warn!("DB health probe failed on cached connection; reconnecting");
    match state.reconnect() {
        Ok(fresh) => match fresh.query("SELECT 1", ()).await {
            Ok(_) => {
                tracing::info!("DB reconnect succeeded");
                (
                    StatusCode::OK,
                    Json(serde_json::json!({
                        "status": "ok",
                        "database": "ok",
                        "reconnected": true,
                    })),
                )
            }
            Err(err) => {
                tracing::error!(?err, "DB query still failing after reconnect");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(serde_json::json!({ "status": "degraded", "database": "error" })),
                )
            }
        },
        Err(err) => {
            tracing::error!(?err, "Failed to rebuild DB connection");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "status": "degraded", "database": "error" })),
            )
        }
    }
}
