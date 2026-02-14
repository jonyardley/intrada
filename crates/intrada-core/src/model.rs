use serde::{Deserialize, Serialize};

use crate::domain::{Exercise, ListQuery, Piece};

/// Internal application state — not exposed to shells.
#[derive(Debug, Default)]
pub struct Model {
    pub pieces: Vec<Piece>,
    pub exercises: Vec<Exercise>,
    pub active_query: Option<ListQuery>,
    pub last_error: Option<String>,
}

/// Serializable view state sent to shells for rendering.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ViewModel {
    pub items: Vec<LibraryItemView>,
    pub error: Option<String>,
}

/// Flattened representation of a piece or exercise for display.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LibraryItemView {
    pub id: String,
    pub item_type: String,
    pub title: String,
    pub subtitle: String,
    pub category: Option<String>,
    pub key: Option<String>,
    pub tempo: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}
