//! Clerk Backend API client.
//!
//! Used for user deletion (GDPR Art. 17) and iOS token exchange (fetching
//! user profile after the iOS app authenticates via ASWebAuthenticationSession
//! and exchanges the Clerk JWT for a long-lived PAT). The frontend talks to
//! Clerk directly for sign-in/out via `@clerk/clerk-js` on web; this
//! server-side client uses the Backend API key for privileged operations.

use serde::Deserialize;

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

    /// Fetch a Clerk user's profile. Used by the iOS token exchange endpoint
    /// to return the user's email alongside the minted PAT.
    pub async fn get_user(&self, user_id: &str) -> Result<ClerkUser, ApiError> {
        let url = format!("{CLERK_API_BASE}/users/{user_id}");
        let response = self
            .http
            .get(&url)
            .bearer_auth(&self.secret_key)
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("Clerk get_user failed: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::Internal(format!(
                "Clerk get_user returned {status}: {body}"
            )));
        }

        response
            .json::<ClerkUser>()
            .await
            .map_err(|e| ApiError::Internal(format!("Clerk get_user parse failed: {e}")))
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

#[derive(Debug, Deserialize)]
pub struct ClerkUser {
    pub id: String,
    pub primary_email_address_id: Option<String>,
    pub email_addresses: Vec<ClerkEmailAddress>,
}

#[derive(Debug, Deserialize)]
pub struct ClerkEmailAddress {
    pub id: String,
    pub email_address: String,
}

impl ClerkUser {
    pub fn primary_email(&self) -> Option<&str> {
        let primary_id = self.primary_email_address_id.as_deref()?;
        self.email_addresses
            .iter()
            .find(|e| e.id == primary_id)
            .map(|e| e.email_address.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(primary_id: Option<&str>, emails: &[(&str, &str)]) -> ClerkUser {
        ClerkUser {
            id: "user_123".into(),
            primary_email_address_id: primary_id.map(Into::into),
            email_addresses: emails
                .iter()
                .map(|(id, addr)| ClerkEmailAddress {
                    id: (*id).into(),
                    email_address: (*addr).into(),
                })
                .collect(),
        }
    }

    #[test]
    fn primary_email_matches_by_id() {
        let u = user(Some("ea_2"), &[("ea_1", "a@x.com"), ("ea_2", "b@x.com")]);
        assert_eq!(u.primary_email(), Some("b@x.com"));
    }

    #[test]
    fn primary_email_none_when_no_primary_id() {
        let u = user(None, &[("ea_1", "a@x.com")]);
        assert_eq!(u.primary_email(), None);
    }

    #[test]
    fn primary_email_none_when_id_not_found() {
        let u = user(Some("ea_missing"), &[("ea_1", "a@x.com")]);
        assert_eq!(u.primary_email(), None);
    }

    #[test]
    fn primary_email_none_when_no_addresses() {
        let u = user(Some("ea_1"), &[]);
        assert_eq!(u.primary_email(), None);
    }
}
