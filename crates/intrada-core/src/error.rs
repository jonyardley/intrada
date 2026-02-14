use thiserror::Error;

/// Domain errors returned by validation and event processing.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum LibraryError {
    #[error("{message}")]
    Validation { field: String, message: String },
    #[error("Item not found: {id}")]
    NotFound { id: String },
}
