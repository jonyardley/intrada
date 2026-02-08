use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Tempo {
    pub marking: Option<String>,
    pub bpm: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum LibraryItem {
    Piece(super::piece::Piece),
    Exercise(super::exercise::Exercise),
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub enum ItemType {
    Piece,
    Exercise,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CreatePiece {
    pub title: String,
    pub composer: String,
    pub key: Option<String>,
    pub tempo: Option<Tempo>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct CreateExercise {
    pub title: String,
    pub composer: Option<String>,
    pub category: Option<String>,
    pub key: Option<String>,
    pub tempo: Option<Tempo>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct UpdatePiece {
    pub title: Option<String>,
    pub composer: Option<String>,
    pub key: Option<Option<String>>,
    pub tempo: Option<Option<Tempo>>,
    pub notes: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct UpdateExercise {
    pub title: Option<String>,
    pub composer: Option<Option<String>>,
    pub category: Option<Option<String>>,
    pub key: Option<Option<String>>,
    pub tempo: Option<Option<Tempo>>,
    pub notes: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ListQuery {
    pub search: Option<String>,
    pub item_type: Option<ItemType>,
    pub key: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
}
