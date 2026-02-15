mod exercises;
mod health;
mod pieces;

use axum::Router;

use crate::state::AppState;

pub fn api_router(state: AppState) -> Router {
    Router::new().nest("/api", api_routes()).with_state(state)
}

fn api_routes() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .nest("/pieces", pieces::router())
        .nest("/exercises", exercises::router())
}
