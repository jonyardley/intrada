use std::sync::Arc;

use libsql::{Connection, Database};

use crate::auth::AuthConfig;
use crate::error::ApiError;

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

    /// Create a new database connection with PRAGMA foreign_keys = ON.
    ///
    /// SQLite disables foreign key enforcement by default on each new connection,
    /// so this must be called for every connection to ensure ON DELETE CASCADE works.
    pub async fn connect(&self) -> Result<Connection, ApiError> {
        let conn = self.db.connect()?;
        conn.execute("PRAGMA foreign_keys = ON", ()).await?;
        Ok(conn)
    }
}
