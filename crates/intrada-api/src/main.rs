mod db;
mod error;
mod routes;
mod state;

use sqlx::PgPool;
use state::AppState;

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    let state = AppState::new(pool);
    let router = routes::api_router(state);

    Ok(router.into())
}
