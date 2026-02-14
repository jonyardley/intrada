use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LibraryError {
    #[error("{message}")]
    Validation { field: String, message: String },
    #[error("Item not found: {id}")]
    NotFound { id: String },
}
