//! Helpers for safe-to-log identifiers in tracing spans + Sentry.
//!
//! Phase 6B of #477. The MCP dispatcher and OAuth endpoints record
//! `user.id.hash` / `token.id.hash` on their spans so dashboards can
//! group by identity (e.g. "which token spiked errors?") without raw
//! user IDs or token IDs ever leaving the API process.

use sha2::{Digest, Sha256};

/// Stable, non-reversible hash for use as a span attribute.
///
/// Truncated to 16 hex chars (8 bytes / 64 bits) — enough to bucket
/// requests by identity in dashboards without surfacing raw identifiers
/// in Sentry. The truncation is a deliberate one-way reduction; we do
/// not need (and explicitly do not want) the ability to map a hash back
/// to its source.
///
/// Empty input returns empty output rather than the SHA-256 of the
/// empty string, so "no identity" is visually distinguishable from "the
/// empty-string user" in dashboards. (`AuthSource::Disabled` produces an
/// empty `user_id`; we don't want every dev-mode request to share a
/// non-empty hash bucket.)
pub fn hash_id(id: &str) -> String {
    if id.is_empty() {
        return String::new();
    }
    let digest = Sha256::digest(id.as_bytes());
    let mut out = String::with_capacity(16);
    for byte in digest.iter().take(8) {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_id_is_stable() {
        assert_eq!(hash_id("user_42"), hash_id("user_42"));
    }

    #[test]
    fn hash_id_is_16_hex_chars() {
        let h = hash_id("user_42");
        assert_eq!(h.len(), 16);
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hash_id_distinguishes_inputs() {
        assert_ne!(hash_id("user_a"), hash_id("user_b"));
    }

    #[test]
    fn empty_in_empty_out() {
        assert_eq!(hash_id(""), "");
    }
}
