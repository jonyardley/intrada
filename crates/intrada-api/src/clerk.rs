//! Clerk Backend API client.
//!
//! Used today only for user deletion (GDPR Art. 17). The frontend talks to
//! Clerk directly for sign-in/out via `@clerk/clerk-js`; this server-side
//! client uses the Backend API key to perform privileged operations the
//! frontend can't.

use crate::error::ApiError;

const CLERK_API_BASE: &str = "https://api.clerk.com/v1";

#[derive(Clone)]
pub struct ClerkClient {
    secret_key: String,
    http: reqwest::Client,
}

impl ClerkClient {
    /// Build a client from `CLERK_SECRET_KEY`. Returns `None` if unset
    /// (matches the R2 / auth-config pattern — local dev runs without it).
    pub fn from_env() -> Option<Self> {
        let secret_key = std::env::var("CLERK_SECRET_KEY").ok()?;
        if secret_key.trim().is_empty() {
            return None;
        }
        Some(Self {
            secret_key,
            http: reqwest::Client::new(),
        })
    }

    /// Delete a Clerk user. Idempotent: 404 is treated as success so a
    /// retry after a partial deletion succeeds.
    pub async fn delete_user(&self, user_id: &str) -> Result<(), ApiError> {
        let url = format!("{CLERK_API_BASE}/users/{user_id}");
        let response = self
            .http
            .delete(&url)
            .bearer_auth(&self.secret_key)
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("Clerk delete failed: {e}")))?;

        let status = response.status();
        if status.is_success() || status == reqwest::StatusCode::NOT_FOUND {
            return Ok(());
        }

        let body = response.text().await.unwrap_or_default();
        Err(ApiError::Internal(format!(
            "Clerk delete returned {status}: {body}"
        )))
    }
}
