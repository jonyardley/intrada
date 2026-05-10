use chrono::Utc;
use libsql::Connection;
use rand::RngCore;

use crate::db;
use crate::db::tokens::{CreatedTokenResponse, TokenListItem, TOKEN_PREFIX};
use crate::error::ApiError;

/// Length of the user-visible prefix shown in the list endpoint:
/// `intrada_pat_` + first 4 hex chars of the body. Long enough to identify
/// the token in account settings, short enough to not be guessable.
const PREFIX_DISPLAY_LEN: usize = TOKEN_PREFIX.len() + 4;

const NAME_MAX_LEN: usize = 100;

fn validate_name(name: &str) -> Result<&str, ApiError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(ApiError::Validation("Token name cannot be empty".into()));
    }
    if trimmed.chars().count() > NAME_MAX_LEN {
        return Err(ApiError::Validation(format!(
            "Token name cannot exceed {NAME_MAX_LEN} characters"
        )));
    }
    Ok(trimmed)
}

/// Generate a fresh PAT. Returns the full token (shown to the user once),
/// its hex SHA-256 hash (stored at rest), and the visible prefix.
fn generate_pat() -> (String, String, String) {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let body: String = bytes.iter().fold(String::with_capacity(64), |mut s, b| {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
        s
    });
    let token = format!("{TOKEN_PREFIX}{body}");
    let hash = db::tokens::hash_token(&token);
    let prefix = token[..PREFIX_DISPLAY_LEN].to_string();
    (token, hash, prefix)
}

pub async fn create_token(
    conn: &Connection,
    user_id: &str,
    name: &str,
) -> Result<CreatedTokenResponse, ApiError> {
    let trimmed = validate_name(name)?.to_string();
    let (token, hash, prefix) = generate_pat();
    let id = ulid::Ulid::new().to_string();
    let created_at = Utc::now();

    db::tokens::insert(conn, &id, user_id, &trimmed, &hash, &prefix, created_at).await?;

    Ok(CreatedTokenResponse {
        id,
        name: trimmed,
        token,
        prefix,
        created_at,
    })
}

pub async fn list_tokens(conn: &Connection, user_id: &str) -> Result<Vec<TokenListItem>, ApiError> {
    db::tokens::list(conn, user_id).await
}

pub async fn revoke_token(
    conn: &Connection,
    user_id: &str,
    token_id: &str,
) -> Result<(), ApiError> {
    let revoked = db::tokens::revoke(conn, user_id, token_id).await?;
    if !revoked {
        return Err(ApiError::NotFound(format!("Token not found: {token_id}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_pat_format_and_uniqueness() {
        // 32 bytes of entropy → 64 hex chars, plus the prefix.
        const TOKEN_BODY_LEN: usize = 64;

        let (t1, h1, p1) = generate_pat();
        let (t2, h2, _) = generate_pat();
        assert!(t1.starts_with(TOKEN_PREFIX));
        assert_eq!(t1.len(), TOKEN_PREFIX.len() + TOKEN_BODY_LEN);
        assert_eq!(h1.len(), 64); // hex SHA-256
        assert_eq!(p1.len(), PREFIX_DISPLAY_LEN);
        assert!(t1.starts_with(&p1));
        // Two consecutive tokens must not collide. Probabilistic but
        // 256-bit entropy → collision astronomically unlikely.
        assert_ne!(t1, t2);
        assert_ne!(h1, h2);
    }

    #[test]
    fn validate_name_rejects_empty() {
        assert!(validate_name("").is_err());
        assert!(validate_name("   ").is_err());
    }

    #[test]
    fn validate_name_trims() {
        assert_eq!(validate_name("  hi  ").unwrap(), "hi");
    }

    #[test]
    fn validate_name_rejects_too_long() {
        let long = "x".repeat(NAME_MAX_LEN + 1);
        assert!(validate_name(&long).is_err());
    }
}
