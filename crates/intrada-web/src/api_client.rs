//! Minimal API configuration for the intrada web shell.
//!
//! HTTP request construction now lives in `intrada-core/src/http.rs`.
//! This module only provides the API base URL and Clerk auth helper.

use crate::js_bridge;

/// Compile-time API base URL with fallback to production.
/// Treats an empty `INTRADA_API_URL` the same as unset (uses the default).
pub const API_BASE_URL: &str = match option_env!("INTRADA_API_URL") {
    Some(url) if !url.is_empty() => url,
    _ => "https://intrada-api.fly.dev",
};

/// Get the current Clerk auth token formatted as a Bearer header value.
pub async fn auth_header_value() -> Option<String> {
    let token = js_bridge::get_auth_token().await?;
    Some(format!("Bearer {token}"))
}
