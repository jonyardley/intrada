use std::sync::Arc;

use libsql::{Connection, Database};

use crate::auth::AuthConfig;
use crate::error::ApiError;
use crate::storage::R2Client;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Database>,
    pub allowed_origin: String,
    pub auth_config: Option<AuthConfig>,
    pub r2: Option<R2Client>,
}

impl AppState {
    pub fn new(
        db: Database,
        allowed_origin: String,
        auth_config: Option<AuthConfig>,
        r2: Option<R2Client>,
    ) -> Self {
        Self {
            db: Arc::new(db),
            allowed_origin,
            auth_config,
            r2,
        }
    }

    /// Get the R2 client, or return an error if not configured.
    pub fn r2(&self) -> Result<&R2Client, ApiError> {
        self.r2
            .as_ref()
            .ok_or_else(|| ApiError::Internal("Photo storage (R2) is not configured".into()))
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
