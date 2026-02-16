use intrada_api::migrations;
use intrada_api::routes;
use intrada_api::state::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let database_url = std::env::var("TURSO_DATABASE_URL").expect("TURSO_DATABASE_URL must be set");
    let auth_token = std::env::var("TURSO_AUTH_TOKEN").expect("TURSO_AUTH_TOKEN must be set");
    let allowed_origin =
        std::env::var("ALLOWED_ORIGIN").unwrap_or_else(|_| "http://localhost:8080".to_string());

    tracing::info!("Connecting to database...");
    let db = libsql::Builder::new_remote(database_url, auth_token)
        .build()
        .await
        .expect("Failed to connect to database");

    let conn = db.connect().expect("Failed to create database connection");

    tracing::info!("Running migrations...");
    migrations::run_migrations(&conn)
        .await
        .expect("Failed to run migrations");

    let state = AppState::new(db, allowed_origin);
    let router = routes::api_router(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("Failed to bind to port 8080");

    tracing::info!("Server listening on 0.0.0.0:8080");
    axum::serve(listener, router).await.expect("Server failed");
}
