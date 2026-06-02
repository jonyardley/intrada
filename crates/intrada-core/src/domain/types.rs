use serde::{Deserialize, Deserializer, Serialize};

use super::item::{Item, ItemKind, Modality};

/// Deserialize a three-state `Option<Option<T>>` PATCH field, format-aware so it
/// round-trips on BOTH wire formats the same type crosses:
/// - **Self-describing (JSON, the API):** a present field is a single `Option<T>`
///   — absent → `None` (skip, via `#[serde(default)]`), `null` → `Some(None)`
///   (clear), `v` → `Some(Some(v))` (set). Classic `double_option`.
/// - **Non-self-describing (bincode, the iOS FFI bridge):** positional — the
///   Swift serializer always writes *both* option levels, so read them directly.
///   Applying the JSON path here reads one level too few, misaligning the byte
///   stream so the whole Update event fails to decode (edits silently don't
///   save — #846).
fn double_option<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    if deserializer.is_human_readable() {
        Deserialize::deserialize(deserializer).map(Some)
    } else {
        Deserialize::deserialize(deserializer)
    }
}

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
    pub key: Option<String>,
    #[serde(default)]
    pub modality: Option<Modality>,
    pub tempo: Option<Tempo>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
}

/// PATCH-style update. `Option<Option<T>>` fields are three-state:
/// `None` = skip, `Some(None)` = clear, `Some(Some(v))` = set — deserialized via
/// the format-aware `double_option` so the type round-trips on both JSON (API)
/// and bincode (iOS bridge); see that fn's docs.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct UpdateItem {
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<ItemKind>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option"
    )]
    pub composer: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option"
    )]
    pub key: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option"
    )]
    pub modality: Option<Option<Modality>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option"
    )]
    pub tempo: Option<Option<Tempo>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "double_option"
    )]
    pub notes: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub priority: Option<bool>,
}

use super::session::PracticeSession;

/// Top-level serialisation unit for `sessions.json` / `intrada:sessions`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SessionsData {
    #[serde(default)]
    pub sessions: Vec<PracticeSession>,
}

// ── API request DTOs ─────────────────────────────────────────────────

/// Request body for creating a set via the REST API.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct CreateSetRequest {
    pub name: String,
    pub entries: Vec<CreateSetEntryRequest>,
}

/// Entry within a create/update set API request.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct CreateSetEntryRequest {
    pub item_id: String,
    pub item_title: String,
    pub item_type: ItemKind,
}

/// Request body for updating a set via the REST API.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct UpdateSetRequest {
    pub name: String,
    pub entries: Vec<CreateSetEntryRequest>,
}

// ── Conversion helpers ──────────────────────────────────────────────

impl CreateSetRequest {
    /// Build from a domain `Set`.
    pub fn from_set(set: &super::set::Set) -> Self {
        Self {
            name: set.name.clone(),
            entries: set
                .entries
                .iter()
                .map(|e| CreateSetEntryRequest {
                    item_id: e.item_id.clone(),
                    item_title: e.item_title.clone(),
                    item_type: e.item_type.clone(),
                })
                .collect(),
        }
    }
}

impl UpdateSetRequest {
    /// Build from a domain `Set`.
    pub fn from_set(set: &super::set::Set) -> Self {
        Self {
            name: set.name.clone(),
            entries: set
                .entries
                .iter()
                .map(|e| CreateSetEntryRequest {
                    item_id: e.item_id.clone(),
                    item_title: e.item_title.clone(),
                    item_type: e.item_type.clone(),
                })
                .collect(),
        }
    }
}

/// Filters for listing/searching library items.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ListQuery {
    pub text: Option<String>,
    pub item_type: Option<ItemKind>,
    pub key: Option<String>,
    /// Empty vec means "no filter". Avoids `Option<Vec<T>>` which
    /// serde-reflection (used by Crux typegen) cannot handle.
    pub tags: Vec<String>,
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

    // ── FFI bincode round-trip (#846) ────────────────────────────────────
    // The native iOS shell ships these write payloads as positional bincode
    // (crux's BincodeFfiFormat). A serde attr that assumes a self-describing
    // format (see `double_option` above) misaligns that wire and the event
    // silently fails to decode. These guard against that whole class.

    /// Round-trip through crux's actual FFI format (`BincodeFfiFormat`) — the
    /// exact wire the iOS bridge uses, so the test can't drift from the real
    /// serializer and we don't take a direct bincode dependency.
    fn assert_round_trips<T>(value: T)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + std::fmt::Debug + PartialEq,
    {
        use crux_core::bridge::{BincodeFfiFormat, FfiFormat};
        let mut bytes = Vec::new();
        BincodeFfiFormat::serialize(&mut bytes, &value).expect("serialize");
        let back: T =
            BincodeFfiFormat::deserialize(&bytes).expect("must decode on the FFI wire (#846)");
        assert_eq!(value, back, "round-trip changed the value");
    }

    #[test]
    fn update_item_round_trips_on_ffi_bincode_wire() {
        // Every field outer-`Some` (mirrors the Swift serializer, which writes
        // all fields regardless of `skip_serializing_if`). Covers both
        // three-state branches: `Some(Some)` = set, `Some(None)` = clear — the
        // exact shapes that failed to decode pre-fix.
        assert_round_trips(UpdateItem {
            title: Some("Renamed".to_string()),
            kind: Some(ItemKind::Exercise),
            composer: Some(Some("Bach".to_string())),
            key: Some(None),
            modality: Some(Some(Modality::Minor)),
            tempo: Some(Some(Tempo {
                marking: Some("Allegro".to_string()),
                bpm: Some(120),
            })),
            notes: Some(None),
            tags: Some(vec!["etude".to_string()]),
            priority: Some(true),
        });
    }

    #[test]
    fn create_item_round_trips_on_ffi_bincode_wire() {
        assert_round_trips(CreateItem {
            title: "Clair de Lune".to_string(),
            kind: ItemKind::Piece,
            composer: Some("Debussy".to_string()),
            key: Some("Db".to_string()),
            modality: Some(Modality::Major),
            tempo: Some(Tempo {
                marking: None,
                bpm: Some(72),
            }),
            notes: None,
            tags: vec!["impressionist".to_string()],
        });
    }
}
