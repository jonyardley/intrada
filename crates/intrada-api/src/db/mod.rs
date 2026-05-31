pub mod account;
pub mod audit;
pub mod items;
pub mod oauth;
pub mod sessions;
pub mod sets;
pub mod tokens;

use intrada_core::domain::item::ItemKind;

use crate::error::ApiError;

/// Substrings that indicate the request didn't reach Turso (transport-
/// level), as opposed to "Turso said no" (SQL/constraint). On these we
/// retry; on anything else we fail immediately so a real bug isn't
/// hidden by silent retries.
///
/// Shared between the migration runner (startup) and the per-request
/// retry helper (runtime, `state::Db::with_transient_retry`). Adding a
/// new signature here gates retries everywhere.
pub const TRANSIENT_ERROR_SUBSTRINGS: &[&str] = &[
    "connection closed before message completed",
    "connection reset",
    "connection refused",
    "broken pipe",
    "timeout",
    "timed out",
    "stream not found",
    "unexpected end of file",
];

/// True if the given error message contains any [`TRANSIENT_ERROR_SUBSTRINGS`]
/// signature (case-insensitive). Use to gate retry decisions.
pub fn is_transient_db_error(err: &str) -> bool {
    let lower = err.to_ascii_lowercase();
    TRANSIENT_ERROR_SUBSTRINGS
        .iter()
        .any(|needle| lower.contains(needle))
}

/// Extract a column value from a libsql row, converting errors to `ApiError`.
///
/// Replaces the repetitive `row.get(N).map_err(|e| ApiError::Internal(e.to_string()))`
/// pattern used across all row-parsing functions.
///
/// Usage: `let name: String = col!(row, 2)?;`
macro_rules! col {
    ($row:expr, $idx:expr) => {
        $row.get($idx)
            .map_err(|e| $crate::error::ApiError::Internal(e.to_string()))
    };
}

pub(crate) use col;

/// Convert a database string to an `ItemKind` enum value.
pub(crate) fn item_kind_from_str(s: &str) -> Result<ItemKind, ApiError> {
    match s {
        "piece" => Ok(ItemKind::Piece),
        "exercise" => Ok(ItemKind::Exercise),
        other => Err(ApiError::Internal(format!("Invalid item_type: {other}"))),
    }
}

/// Convert an `ItemKind` enum value to its database string representation.
pub(crate) fn item_kind_to_str(kind: &ItemKind) -> &'static str {
    match kind {
        ItemKind::Piece => "piece",
        ItemKind::Exercise => "exercise",
    }
}
