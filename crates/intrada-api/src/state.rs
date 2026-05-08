use std::sync::{Arc, RwLock};
use std::time::Duration;

use libsql::{Connection, Database};

use crate::auth::AuthConfig;
use crate::clerk::ClerkClient;
use crate::error::ApiError;
use crate::storage::R2Client;

/// Heartbeat interval for the background liveness probe.
///
/// Turso's hosted libsql sessions silently rot on idle (timeout exact
/// value undocumented; we've seen rot well under 5 minutes after
/// machine suspend). Pinging every 30s keeps the session warm so a
/// real user request rarely lands on a dead connection.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

/// Shared, self-healing database handle.
///
/// Turso's remote HTTP connections don't share replication state across
/// `db.connect()` calls, so we reuse a single `Connection` to guarantee
/// read-your-own-writes consistency. But that single HTTP session can rot
/// (idle timeout, machine suspend/resume, network blip) and never recovers
/// on its own. `Db` keeps the underlying `Database` so we can rebuild the
/// `Connection` on demand when a query fails.
#[derive(Clone)]
pub struct Db {
    db: Arc<Database>,
    conn: Arc<RwLock<Connection>>,
}

impl Db {
    pub fn new(db: Database, conn: Connection) -> Self {
        Self {
            db: Arc::new(db),
            conn: Arc::new(RwLock::new(conn)),
        }
    }

    /// Cheap clone of the current shared connection.
    pub fn conn(&self) -> Connection {
        self.conn
            .read()
            .expect("db connection lock poisoned")
            .clone()
    }

    /// Rebuild the shared connection from the underlying `Database`.
    /// Subsequent `conn()` calls observe the new connection.
    pub fn reconnect(&self) -> Result<Connection, libsql::Error> {
        let fresh = self.db.connect()?;
        *self.conn.write().expect("db connection lock poisoned") = fresh.clone();
        Ok(fresh)
    }

    /// Spawn a background task that pings the connection every
    /// `HEARTBEAT_INTERVAL` and reconnects on failure. Runs for the
    /// lifetime of the tokio runtime — no shutdown signal is needed
    /// today (the task dies with the runtime). Caller doesn't need to
    /// hold the returned `JoinHandle` unless they want explicit cancel.
    pub fn spawn_heartbeat(&self) -> tokio::task::JoinHandle<()> {
        let db = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(HEARTBEAT_INTERVAL);
            // Skip the first tick — `tokio::time::interval` fires
            // immediately by default, but startup already proved the
            // connection works.
            interval.tick().await;
            loop {
                interval.tick().await;
                if db.conn().query("SELECT 1", ()).await.is_ok() {
                    continue;
                }
                tracing::warn!("DB heartbeat failed; reconnecting");
                match db.reconnect() {
                    Ok(fresh) => match fresh.query("SELECT 1", ()).await {
                        Ok(_) => tracing::info!("DB heartbeat reconnect succeeded"),
                        // WARN not ERROR — the next interval will retry,
                        // and a real user request hitting this in the
                        // window will surface a proper error itself.
                        // Logging at ERROR here would double-fire Sentry
                        // for the same incident.
                        Err(err) => {
                            tracing::warn!(?err, "DB heartbeat: query failing after reconnect")
                        }
                    },
                    Err(err) => {
                        tracing::warn!(?err, "DB heartbeat: failed to rebuild connection")
                    }
                }
            }
        })
    }
}

#[derive(Clone)]
pub struct AppState {
    db: Db,
    pub allowed_origin: String,
    pub auth_config: Option<AuthConfig>,
    pub r2: Option<R2Client>,
    pub clerk: Option<ClerkClient>,
}

impl AppState {
    pub fn new(
        db: Db,
        allowed_origin: String,
        auth_config: Option<AuthConfig>,
        r2: Option<R2Client>,
        clerk: Option<ClerkClient>,
    ) -> Self {
        Self {
            db,
            allowed_origin,
            auth_config,
            r2,
            clerk,
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
        self.db.conn()
    }

    /// Rebuild the shared connection. Use after a query fails so subsequent
    /// requests get a working session instead of a permanently-broken one.
    pub fn reconnect(&self) -> Result<Connection, libsql::Error> {
        self.db.reconnect()
    }
}
