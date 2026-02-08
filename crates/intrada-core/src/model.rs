use serde::{Deserialize, Serialize};

use crate::domain::{Exercise, Piece};

#[derive(Default, Debug)]
pub struct Model {
    pub pieces: Vec<Piece>,
    pub exercises: Vec<Exercise>,
    pub last_error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ViewModel {
    pub items: Vec<LibraryItemView>,
    pub item_count: usize,
    pub error: Option<String>,
    pub status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LibraryItemView {
    pub id: String,
    pub item_type: String,
    pub title: String,
    pub subtitle: String,
    pub key: Option<String>,
    pub tempo: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}
