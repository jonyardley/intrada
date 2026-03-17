pub mod items;
pub mod routines;
pub mod sessions;

use intrada_core::domain::item::ItemKind;

use crate::error::ApiError;

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
