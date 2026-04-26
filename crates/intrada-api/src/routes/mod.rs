mod health;
mod items;
mod lessons;
mod routines;
mod sessions;

use axum::http::{header, HeaderValue, Method};
use axum::Router;
use tower_http::cors::{AllowOrigin, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::state::AppState;

pub fn api_router(state: AppState) -> Router {
    // ALLOWED_ORIGIN supports comma-separated values so a single API server
    // can serve multiple frontends. In particular, the Tauri iOS WebView
    // page origin is `tauri://localhost` (not the devUrl), so local dev
    // needs both `http://localhost:8080` (web) and `tauri://localhost` (iOS).
    let origins: Vec<HeaderValue> = state
        .allowed_origin
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            s.parse::<HeaderValue>()
                .unwrap_or_else(|_| panic!("Invalid ALLOWED_ORIGIN value: {s}"))
        })
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION]);

    let trace = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO));

    Router::new()
        .nest("/api", api_routes())
        .layer(cors)
        .layer(trace)
        .with_state(state)
}

fn api_routes() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .nest("/items", items::router())
        .nest("/sessions", sessions::router())
        .nest("/routines", routines::router())
        .nest("/lessons", lessons::router())
}
