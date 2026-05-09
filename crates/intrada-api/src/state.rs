use std::sync::{Arc, RwLock};
use std::time::Duration;

use libsql::{Connection, Database};

use crate::auth::AuthConfig;
use crate::clerk::ClerkClient;
use crate::db::is_transient_db_error;
use crate::error::ApiError;
use crate::rate_limit::{IpRateLimiter, McpRateLimiter};
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

    /// Run a DB closure with retry-on-transient. Reconnects between
    /// attempts so a half-broken HTTP session is replaced before the
    /// next try. Total max wall time ≈ 600 ms (50 ms + 500 ms backoff).
    ///
    /// Sized for Turso's typical Hrana stream-not-found / connection-
    /// closed signatures (INTRADA-API-36): the heartbeat (above) keeps
    /// the session warm, but a request can still land in the narrow
    /// window between a stream dropping and the heartbeat noticing.
    /// Per-request retry covers that gap.
    ///
    /// Only `ApiError::Internal` whose string matches a known transient
    /// substring triggers retry — terminal errors (NotFound, Validation,
    /// Unauthorized, or Internal with non-transient message) surface
    /// immediately so real bugs aren't masked.
    pub async fn with_transient_retry<F, Fut, T>(&self, mut op: F) -> Result<T, ApiError>
    where
        F: FnMut(Connection) -> Fut,
        Fut: std::future::Future<Output = Result<T, ApiError>>,
    {
        const BACKOFF_MS: &[u64] = &[0, 50, 500];
        for (attempt, backoff) in BACKOFF_MS.iter().enumerate() {
            if *backoff > 0 {
                tokio::time::sleep(Duration::from_millis(*backoff)).await;
                let _ = self.reconnect();
            }
            let conn = self.conn();
            match op(conn).await {
                Ok(value) => return Ok(value),
                Err(ApiError::Internal(msg))
                    if attempt + 1 < BACKOFF_MS.len() && is_transient_db_error(&msg) =>
                {
                    tracing::warn!(
                        attempt = attempt + 1,
                        error = %msg,
                        "Transient DB error; reconnecting and retrying",
                    );
                    continue;
                }
                Err(other) => return Err(other),
            }
        }
        // Loop always returns Ok or Err inside its body — last iteration
        // either succeeds, or hits the non-transient branch and returns.
        unreachable!("with_transient_retry loop must return")
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
    /// Per-token rate limiter for `/api/mcp/*`. Defaults to production
    /// limits (60 req/min/token); tests can swap in a tighter bucket
    /// via [`AppState::with_rate_limiter`].
    pub rate_limiter: Arc<McpRateLimiter>,
    /// Per-IP rate limiter for `/api/mcp/*`. Guards against bogus-PAT
    /// floods that would otherwise burn `lookup_by_hash` DB calls
    /// (300 req/min/IP in production).
    pub mcp_ip_limiter: Arc<IpRateLimiter>,
    /// Per-IP rate limiter for OAuth endpoints. Guards the unauthenticated
    /// `/oauth/register` DCR endpoint against DB-flooding abuse
    /// (20 req/min/IP in production).
    pub oauth_ip_limiter: Arc<IpRateLimiter>,
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
            rate_limiter: Arc::new(McpRateLimiter::production()),
            mcp_ip_limiter: Arc::new(IpRateLimiter::production_mcp()),
            oauth_ip_limiter: Arc::new(IpRateLimiter::production_oauth()),
        }
    }

    /// Replace the per-token MCP rate limiter — used by integration tests
    /// to inject a tighter bucket without breaking the existing `new()` arity.
    pub fn with_rate_limiter(mut self, rate_limiter: Arc<McpRateLimiter>) -> Self {
        self.rate_limiter = rate_limiter;
        self
    }

    /// Replace the per-IP MCP rate limiter — used by integration tests.
    pub fn with_mcp_ip_limiter(mut self, limiter: Arc<IpRateLimiter>) -> Self {
        self.mcp_ip_limiter = limiter;
        self
    }

    /// Replace the per-IP OAuth rate limiter — used by integration tests.
    pub fn with_oauth_ip_limiter(mut self, limiter: Arc<IpRateLimiter>) -> Self {
        self.oauth_ip_limiter = limiter;
        self
    }

    /// Get the R2 client, or return an error if not configured.
    pub fn r2(&self) -> Result<&R2Client, ApiError> {
        self.r2
            .as_ref()
            .ok_or_else(|| ApiError::Internal("Photo storage (R2) is not configured".into()))
    }

    /// The web app's base URL — used by the OAuth `/authorize` redirect
    /// to land the user on the consent page. Sourced from the
    /// `ALLOWED_ORIGIN` allowlist's first entry (which IS the production
    /// web origin); this avoids introducing yet another env var to keep
    /// in sync. Falls back to `myintrada.com` if nothing's configured.
    pub fn web_base_url(&self) -> String {
        self.allowed_origin
            .split(',')
            .map(|s| s.trim())
            .find(|s| !s.is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "https://myintrada.com".to_string())
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

    /// Run a DB closure with retry-on-transient. Passthrough to
    /// [`Db::with_transient_retry`] so handlers can write
    /// `state.with_transient_retry(|conn| async move { … })` without
    /// reaching into `state.db` directly.
    pub async fn with_transient_retry<F, Fut, T>(&self, op: F) -> Result<T, ApiError>
    where
        F: FnMut(Connection) -> Fut,
        Fut: std::future::Future<Output = Result<T, ApiError>>,
    {
        self.db.with_transient_retry(op).await
    }
}
