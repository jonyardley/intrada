use serde::{Deserialize, Serialize};

/// Tempo representation with optional marking (e.g. "Allegro") and BPM.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Tempo {
    pub marking: Option<String>,
    pub bpm: Option<u16>,
}

impl Tempo {
    /// Build a Tempo from optional parts. Returns None if both are absent.
    pub fn from_parts(marking: Option<String>, bpm: Option<u16>) -> Option<Self> {
        if marking.is_some() || bpm.is_some() {
            Some(Self { marking, bpm })
        } else {
            None
        }
    }

    /// Format for display: "Allegro (132 BPM)", "Allegro", "132 BPM", or None.
    pub fn format_display(&self) -> Option<String> {
        match (&self.marking, self.bpm) {
            (Some(marking), Some(bpm)) => Some(format!("{marking} ({bpm} BPM)")),
            (Some(marking), None) => Some(marking.clone()),
            (None, Some(bpm)) => Some(format!("{bpm} BPM")),
            (None, None) => None,
        }
    }
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

/// PATCH-style update. `Option<Option<T>>` fields use three-state semantics:
/// - `None` = field not being updated (skip)
/// - `Some(None)` = clear the field
/// - `Some(Some(v))` = set to new value
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

/// Filters for listing/searching library items.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ListQuery {
    pub text: Option<String>,
    pub item_type: Option<String>,
    pub key: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}
