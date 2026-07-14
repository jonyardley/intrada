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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct LibraryData {
    #[serde(default)]
    pub items: Vec<Item>,
}

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct CreateSetRequest {
    pub name: String,
    pub entries: Vec<CreateSetEntryRequest>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct CreateSetEntryRequest {
    pub item_id: String,
    pub item_title: String,
    pub item_type: ItemKind,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct UpdateSetRequest {
    pub name: String,
    pub entries: Vec<CreateSetEntryRequest>,
}

// ── Conversion helpers ──────────────────────────────────────────────

impl CreateSetRequest {
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

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum SortField {
    #[default]
    DateAdded,
    LastPracticed,
    Title,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum SortDirection {
    #[default]
    Descending,
    Ascending,
}

/// Default = Date Added, newest first.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct LibrarySort {
    pub field: SortField,
    pub direction: SortDirection,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_sort_defaults_to_date_added_descending() {
        let sort = LibrarySort::default();
        assert_eq!(sort.field, SortField::DateAdded);
        assert_eq!(sort.direction, SortDirection::Descending);
    }

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
    fn update_item_absent_field_does_not_round_trip_on_rust_bincode_serialize() {
        // LANDMINE GUARD (#908): `skip_serializing_if` omits an outer-`None`
        // field on the bincode serialize path while deserialize reads it
        // positionally — asymmetric on the FFI wire (#846 silent-decode class),
        // safe today only because Swift serializes and Rust only deserializes.
        // Pins the asymmetry: making bincode serialize field-complete flips this.
        use crux_core::bridge::{BincodeFfiFormat, FfiFormat};

        let with_absent_field = UpdateItem {
            title: Some("Renamed".to_string()),
            kind: None,
            composer: Some(Some("Bach".to_string())),
            key: Some(None),
            modality: Some(Some(Modality::Minor)),
            tempo: Some(None),
            notes: Some(Some("phrasing".to_string())),
            tags: Some(vec!["etude".to_string()]),
            priority: Some(true),
        };

        let mut bytes = Vec::new();
        BincodeFfiFormat::serialize(&mut bytes, &with_absent_field).expect("serialize");
        let decoded: Result<UpdateItem, _> = BincodeFfiFormat::deserialize(&bytes);

        if let Ok(back) = decoded {
            assert_ne!(
                back, with_absent_field,
                "bincode serialize became field-complete — if you fixed the \
                 skip_serializing_if asymmetry (#908), update this guard to \
                 assert a clean round-trip"
            );
        }
    }

    #[test]
    fn update_event_round_trips_on_ffi_bincode_wire() {
        // Wraps the DTO in the event that actually crosses the bridge, so
        // enum/struct framing is covered too (#777). See the bare-DTO test above.
        use crate::domain::item::ItemEvent;
        assert_round_trips(ItemEvent::Update {
            id: "01HX0000000000000000000000".to_string(),
            input: UpdateItem {
                title: Some("Renamed".to_string()),
                kind: Some(ItemKind::Exercise),
                composer: Some(Some("Bach".to_string())),
                key: Some(None),
                modality: Some(Some(Modality::Minor)),
                tempo: Some(None),
                notes: Some(Some("phrasing".to_string())),
                tags: Some(vec!["etude".to_string()]),
                priority: Some(true),
            },
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

    // Remaining bridge-crossing write payloads — guard against a #846-class break.

    #[test]
    fn item_tag_events_round_trip_on_ffi_bincode_wire() {
        use crate::domain::item::ItemEvent;
        assert_round_trips(ItemEvent::AddTags {
            id: "p1".to_string(),
            tags: vec!["etude".to_string(), "warmup".to_string()],
        });
        assert_round_trips(ItemEvent::RemoveTags {
            id: "p1".to_string(),
            tags: vec!["etude".to_string()],
        });
    }

    #[test]
    fn item_link_events_round_trip_on_ffi_bincode_wire() {
        use crate::domain::item::ItemEvent;
        assert_round_trips(ItemEvent::LinkExercise {
            piece_id: "p1".to_string(),
            exercise_id: "e1".to_string(),
        });
        assert_round_trips(ItemEvent::UnlinkExercise {
            piece_id: "p1".to_string(),
            exercise_id: "e1".to_string(),
        });
        assert_round_trips(ItemEvent::ReorderLinkedExercises {
            piece_id: "p1".to_string(),
            ordered_ids: vec!["e1".to_string(), "e2".to_string()],
        });
    }

    #[test]
    fn set_requests_round_trip_on_ffi_bincode_wire() {
        let entries = vec![CreateSetEntryRequest {
            item_id: "p1".to_string(),
            item_title: "Clair de Lune".to_string(),
            item_type: ItemKind::Piece,
        }];
        assert_round_trips(CreateSetRequest {
            name: "Warm-ups".to_string(),
            entries: entries.clone(),
        });
        assert_round_trips(UpdateSetRequest {
            name: "Warm-ups (revised)".to_string(),
            entries,
        });
    }

    #[test]
    fn save_session_persistence_op_round_trips_on_ffi_bincode_wire() {
        // PracticeSession crosses the bridge as a SaveSession persistence Effect;
        // its optional-heavy SetlistEntry + rep_history is exactly the #846 risk.
        use crate::domain::session::{
            CompletionStatus, EntryStatus, PracticeSession, RepAction, SetlistEntry,
        };
        use crate::persistence::PersistenceOperation;
        let now = chrono::Utc::now();
        let entry = SetlistEntry {
            id: "e1".to_string(),
            item_id: "p1".to_string(),
            item_title: "Clair de Lune".to_string(),
            item_type: ItemKind::Piece,
            position: 0,
            duration_secs: 300,
            status: EntryStatus::Completed,
            notes: Some("phrasing".to_string()),
            score: Some(4),
            intention: Some("evenness".to_string()),
            rep_target: Some(5),
            rep_count: Some(5),
            rep_target_reached: Some(true),
            rep_history: Some(vec![RepAction::Success, RepAction::Missed]),
            planned_duration_secs: Some(300),
            achieved_tempo: Some(120),
            group_id: None,
        };
        assert_round_trips(PersistenceOperation::SaveSession(PracticeSession {
            id: "s1".to_string(),
            entries: vec![entry],
            session_notes: Some("solid".to_string()),
            session_intention: Some("warm up".to_string()),
            started_at: now,
            completed_at: now,
            total_duration_secs: 300,
            completion_status: CompletionStatus::Completed,
            session_score: Some(8),
            reflection_improved: None,
            reflection_still_rough: None,
            reflection_next_target: None,
        }));
    }

    #[test]
    fn block_grouping_session_events_round_trip_on_ffi_bincode_wire() {
        use crate::domain::session::SessionEvent;
        assert_round_trips(SessionEvent::ReorderBlock {
            group_id: "g1".to_string(),
            new_position: 2,
        });
        assert_round_trips(SessionEvent::KeepOnlyPiece {
            group_id: "g1".to_string(),
        });
        assert_round_trips(SessionEvent::UngroupBlock {
            group_id: "g1".to_string(),
        });
        assert_round_trips(SessionEvent::UngroupAllBlocks);
        assert_round_trips(SessionEvent::RemoveBlock {
            group_id: "g1".to_string(),
        });
        assert_round_trips(SessionEvent::AddExerciseToBlock {
            group_id: "g1".to_string(),
            item_id: "ex-D".to_string(),
        });
    }

    #[test]
    fn per_entry_config_session_events_round_trip_on_ffi_bincode_wire() {
        // Newly wired from Swift (session builder per-entry settings) — the
        // `Option`-heavy shapes are exactly the #846 risk, both the `Some` and
        // `None` (clear) sides.
        use crate::domain::session::SessionEvent;
        assert_round_trips(SessionEvent::SetEntryIntention {
            entry_id: "e1".to_string(),
            intention: Some("evenness".to_string()),
        });
        assert_round_trips(SessionEvent::SetEntryIntention {
            entry_id: "e1".to_string(),
            intention: None,
        });
        assert_round_trips(SessionEvent::SetRepTarget {
            entry_id: "e1".to_string(),
            target: Some(7),
        });
        assert_round_trips(SessionEvent::SetRepTarget {
            entry_id: "e1".to_string(),
            target: None,
        });
        assert_round_trips(SessionEvent::SetEntryDuration {
            entry_id: "e1".to_string(),
            duration_secs: Some(600),
        });
        assert_round_trips(SessionEvent::SetEntryDuration {
            entry_id: "e1".to_string(),
            duration_secs: None,
        });
    }

    #[test]
    fn setlist_entry_group_id_round_trips_on_ffi_bincode_wire() {
        use crate::domain::session::{EntryStatus, SetlistEntry};
        assert_round_trips(SetlistEntry {
            id: "e1".to_string(),
            item_id: "ex1".to_string(),
            item_title: "Scales".to_string(),
            item_type: ItemKind::Exercise,
            position: 0,
            duration_secs: 0,
            status: EntryStatus::NotAttempted,
            notes: None,
            score: None,
            intention: None,
            rep_target: None,
            rep_count: None,
            rep_target_reached: None,
            rep_history: None,
            planned_duration_secs: None,
            achieved_tempo: None,
            group_id: Some("block-1".to_string()),
        });
    }

    #[test]
    fn session_reflection_event_round_trips_on_ffi_bincode_wire() {
        use crate::domain::session::{ReflectionField, SessionEvent};
        for field in [
            ReflectionField::Improved,
            ReflectionField::StillRough,
            ReflectionField::NextTarget,
        ] {
            assert_round_trips(SessionEvent::UpdateSessionReflection {
                field,
                text: Some("bars 12-14 at 80 first".to_string()),
            });
            assert_round_trips(SessionEvent::UpdateSessionReflection { field, text: None });
        }
    }

    #[test]
    fn recover_session_event_round_trips_on_ffi_bincode_wire() {
        use crate::domain::session::{ActiveSession, SessionEvent};
        use chrono::TimeZone;
        let anchor = chrono::Utc.timestamp_opt(1_700_000_000, 0).unwrap();
        assert_round_trips(SessionEvent::RecoverSession {
            session: ActiveSession {
                id: "s1".to_string(),
                entries: vec![],
                current_index: 0,
                current_item_started_at: anchor,
                session_started_at: anchor,
                session_intention: Some("even RH at 96".to_string()),
            },
            now: anchor,
        });
    }

    #[test]
    fn practice_session_with_reflections_round_trips_on_ffi_bincode_wire() {
        use crate::domain::session::{CompletionStatus, PracticeSession};
        use chrono::TimeZone;
        assert_round_trips(PracticeSession {
            id: "s1".to_string(),
            entries: vec![],
            session_notes: Some("note".to_string()),
            session_intention: Some("even RH at 96".to_string()),
            started_at: chrono::Utc.timestamp_opt(1_700_000_000, 0).unwrap(),
            completed_at: chrono::Utc.timestamp_opt(1_700_003_600, 0).unwrap(),
            total_duration_secs: 3600,
            completion_status: CompletionStatus::Completed,
            session_score: Some(7),
            reflection_improved: Some("thumb-unders even".to_string()),
            reflection_still_rough: None,
            reflection_next_target: Some("bridge at 80".to_string()),
        });
    }
}
