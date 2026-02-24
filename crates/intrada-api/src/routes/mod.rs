mod goals;
mod health;
mod items;
mod routines;
mod sessions;

use axum::http::{header, HeaderValue, Method};
use axum::Router;
use tower_http::cors::CorsLayer;

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

    Router::new()
        .nest("/api", api_routes())
        .layer(cors)
        .with_state(state)
}

fn api_routes() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .nest("/items", items::router())
        .nest("/sessions", sessions::router())
        .nest("/routines", routines::router())
        .nest("/goals", goals::router())
}
