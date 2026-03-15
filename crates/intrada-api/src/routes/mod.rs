mod health;
mod items;
mod routines;
mod sessions;

use axum::http::{header, HeaderValue, Method};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::state::AppState;

pub fn api_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            state
                .allowed_origin
                .parse::<HeaderValue>()
                .expect("Invalid ALLOWED_ORIGIN value"),
        )
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
}
