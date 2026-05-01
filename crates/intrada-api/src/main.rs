use intrada_api::auth;
use intrada_api::migrations;
use intrada_api::routes;
use intrada_api::state::{AppState, Db};
use intrada_api::storage::R2Client;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() {
    // Sentry first, so panics + tracing events from startup are captured.
    // No-op when SENTRY_DSN is unset (local dev without Sentry).
    let _sentry_guard = std::env::var("SENTRY_DSN").ok().map(|dsn| {
        sentry::init((
            dsn,
            sentry::ClientOptions {
                release: option_env!("GIT_SHA").map(Into::into),
                environment: Some(
                    if cfg!(debug_assertions) {
                        "development"
                    } else {
                        "production"
                    }
                    .into(),
                ),
                traces_sample_rate: 0.1,
                send_default_pii: false,
                ..Default::default()
            },
        ))
    });

    let env_filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .with(sentry::integrations::tracing::layer())
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

    // FK enforcement is intentionally OFF. Turso's remote HTTP protocol
    // checks FK constraints via a parent-table read, which suffers the same
    // cross-connection consistency issue that caused photo upload 404s.
    // All cascade deletes are handled explicitly in application code
    // (delete_session, delete_routine, delete_lesson).

    tracing::info!("Running migrations...");
    migrations::run_migrations(&conn)
        .await
        .expect("Failed to run migrations");

    let auth_config = match std::env::var("CLERK_ISSUER_URL") {
        Ok(issuer_url) => {
            tracing::info!("Fetching JWKS from {issuer_url}...");
            let keys = auth::fetch_jwks(&issuer_url)
                .await
                .expect("Failed to fetch JWKS");
            tracing::info!("Loaded {} JWKS key(s)", keys.len());
            Some(auth::AuthConfig {
                issuer: issuer_url,
                decoding_keys: std::sync::Arc::new(tokio::sync::RwLock::new(keys)),
            })
        }
        Err(_) => {
            tracing::warn!("CLERK_ISSUER_URL not set — auth disabled (all requests pass through)");
            None
        }
    };

    // Spawn background JWKS refresh task (every 60 minutes)
    if let Some(ref config) = auth_config {
        let config = config.clone();
        tokio::spawn(async move {
            let interval = std::time::Duration::from_secs(60 * 60);
            loop {
                tokio::time::sleep(interval).await;
                config.refresh_jwks().await;
            }
        });
    }

    let r2 = match R2Client::from_env() {
        Ok(client) => {
            tracing::info!("R2 photo storage configured");
            Some(client)
        }
        Err(msg) => {
            tracing::warn!("R2 not configured — photo upload disabled ({msg})");
            None
        }
    };

    let state = AppState::new(Db::new(db, conn), allowed_origin, auth_config, r2);
    let router = routes::api_router(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3001".to_string());
    let addr = format!("0.0.0.0:{port}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {addr}"));

    tracing::info!("Server listening on {addr}");
    axum::serve(listener, router).await.expect("Server failed");
}
