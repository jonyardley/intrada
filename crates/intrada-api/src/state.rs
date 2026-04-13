use libsql::Connection;

use crate::auth::AuthConfig;
use crate::error::ApiError;
use crate::storage::R2Client;

#[derive(Clone)]
pub struct AppState {
    /// Single shared database connection. Turso's remote HTTP connections
    /// don't share replication state across `db.connect()` calls, so a new
    /// connection can fail to see rows written by a previous one. Reusing
    /// one connection guarantees read-your-own-writes consistency.
    conn: Connection,
    pub allowed_origin: String,
    pub auth_config: Option<AuthConfig>,
    pub r2: Option<R2Client>,
}

impl AppState {
    pub fn new(
        conn: Connection,
        allowed_origin: String,
        auth_config: Option<AuthConfig>,
        r2: Option<R2Client>,
    ) -> Self {
        Self {
            conn,
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

    /// Return the shared database connection.
    ///
    /// `Connection` is `Clone` (wraps an `Arc`), so this is cheap. All
    /// handlers share the same underlying HTTP session to Turso, which
    /// ensures read-your-own-writes consistency across requests.
    pub fn conn(&self) -> Connection {
        self.conn.clone()
    }
}
