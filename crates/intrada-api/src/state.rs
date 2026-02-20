use std::sync::Arc;

use libsql::Database;

use crate::auth::AuthConfig;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub allowed_origin: String,
    pub auth_config: Option<AuthConfig>,
}

impl AppState {
    pub fn new(db: Database, allowed_origin: String, auth_config: Option<AuthConfig>) -> Self {
        Self {
            db: Arc::new(db),
            allowed_origin,
            auth_config,
        }
    }
}
