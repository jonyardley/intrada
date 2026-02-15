use serde::{Deserialize, Serialize};

use crate::domain::session::Session;
use crate::domain::{Exercise, ListQuery, Piece};

/// Internal application state — not exposed to shells.
#[derive(Debug, Default)]
pub struct Model {
    pub pieces: Vec<Piece>,
    pub exercises: Vec<Exercise>,
    pub sessions: Vec<Session>,
    pub active_query: Option<ListQuery>,
    pub last_error: Option<String>,
}

/// Serializable view state sent to shells for rendering.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ViewModel {
    pub items: Vec<LibraryItemView>,
    pub sessions: Vec<SessionView>,
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
    pub practice: Option<ItemPracticeSummary>,
}

/// Practice summary for a library item.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ItemPracticeSummary {
    pub session_count: usize,
    pub total_minutes: u32,
}

/// Flattened representation of a session for display.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SessionView {
    pub id: String,
    pub item_id: String,
    pub item_title: String,
    pub item_type: String,
    pub duration_minutes: u32,
    pub started_at: String,
    pub logged_at: String,
    pub notes: Option<String>,
}
