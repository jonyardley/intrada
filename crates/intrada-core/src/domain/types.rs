use serde::{Deserialize, Serialize};

use super::item::{Item, ItemKind};

/// Top-level serialisation unit for library data.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct LibraryData {
    #[serde(default)]
    pub items: Vec<Item>,
}

/// Tempo representation with optional marking (e.g. "Allegro") and BPM.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Tempo {
    pub marking: Option<String>,
    pub bpm: Option<u16>,
}

impl Tempo {
    /// Build a Tempo from optional parts. Returns None if both are absent.
    #[must_use]
    pub fn from_parts(marking: Option<String>, bpm: Option<u16>) -> Option<Self> {
        if marking.is_some() || bpm.is_some() {
            Some(Self { marking, bpm })
        } else {
            None
        }
    }

    /// Format for display: "Allegro (132 BPM)", "Allegro", "132 BPM", or empty string.
    #[must_use]
    pub fn format_display(&self) -> String {
        match (&self.marking, self.bpm) {
            (Some(marking), Some(bpm)) => format!("{marking} ({bpm} BPM)"),
            (Some(marking), None) => marking.clone(),
            (None, Some(bpm)) => format!("{bpm} BPM"),
            (None, None) => String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct CreateItem {
    pub title: String,
    pub kind: ItemKind,
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
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct UpdateItem {
    pub title: Option<String>,
    pub composer: Option<Option<String>>,
    pub category: Option<Option<String>>,
    pub key: Option<Option<String>>,
    pub tempo: Option<Option<Tempo>>,
    pub notes: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
}

use super::goal::GoalKind;
use super::session::PracticeSession;

/// Top-level serialisation unit for `sessions.json` / `intrada:sessions`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SessionsData {
    #[serde(default)]
    pub sessions: Vec<PracticeSession>,
}

/// Input for creating a new goal.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct CreateGoal {
    pub title: String,
    pub kind: GoalKind,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
}

/// PATCH-style update for a goal. `Option<Option<T>>` fields use three-state
/// semantics: `None` = skip, `Some(None)` = clear, `Some(Some(v))` = set.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct UpdateGoal {
    pub title: Option<String>,
    pub deadline: Option<Option<chrono::DateTime<chrono::Utc>>>,
}

/// Filters for listing/searching library items.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ListQuery {
    pub text: Option<String>,
    pub item_type: Option<ItemKind>,
    pub key: Option<String>,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_parts_both_none_returns_none() {
        assert_eq!(Tempo::from_parts(None, None), None);
    }

    #[test]
    fn test_from_parts_marking_only() {
        let tempo = Tempo::from_parts(Some("Allegro".to_string()), None);
        assert_eq!(
            tempo,
            Some(Tempo {
                marking: Some("Allegro".to_string()),
                bpm: None,
            })
        );
    }

    #[test]
    fn test_from_parts_bpm_only() {
        let tempo = Tempo::from_parts(None, Some(120));
        assert_eq!(
            tempo,
            Some(Tempo {
                marking: None,
                bpm: Some(120),
            })
        );
    }

    #[test]
    fn test_from_parts_both_present() {
        let tempo = Tempo::from_parts(Some("Andante".to_string()), Some(72));
        assert_eq!(
            tempo,
            Some(Tempo {
                marking: Some("Andante".to_string()),
                bpm: Some(72),
            })
        );
    }
}
