pub mod goals;
pub mod items;
pub mod routines;
pub mod sessions;

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
