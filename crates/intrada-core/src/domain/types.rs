use serde::{Deserialize, Serialize};

use super::item::{Item, ItemKind};
use super::key::Key;

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
}

/// A tempo as typed on a form: the BPM arrives as the raw text, so the core
/// refuses what it cannot read instead of the shell dropping it (#2224).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct TempoInput {
    pub marking: Option<String>,
    pub bpm: Option<String>,
}

#[cfg(test)]
impl From<Tempo> for TempoInput {
    fn from(tempo: Tempo) -> Self {
        Self {
            marking: tempo.marking,
            bpm: tempo.bpm.map(|b| b.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct CreateItem {
    pub title: String,
    pub kind: ItemKind,
    pub composer: Option<String>,
    pub key: Option<Key>,
    pub tempo: Option<TempoInput>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    /// The page the fields were read off (#1436). Appended last, because the
    /// bincode wire is positional. Keeping it on the create is what stops the
    /// user photographing the same page twice: the one they scanned to fill
    /// the form is the one they practise from.
    #[serde(default)]
    pub photo_id: Option<String>,
    /// Variations chosen while creating the item, as labels: a live library
    /// row with the label is reused, any other is minted (#2246). Only
    /// `ItemEvent::Add` honours this: a `CreateItem` reaching
    /// `SetPieceLinks` or `AddPieceInFull` never carries any (#1436's
    /// `photo_id` sets the precedent for a shared field meaning one thing by
    /// event).
    #[serde(default)]
    pub variation_labels: Vec<String>,
}

/// PATCH-style update. `Option<Option<T>>` fields are three-state:
/// `None` = skip, `Some(None)` = clear, `Some(Some(v))` = set. `tempo` is
/// two-state: `None` = skip, and a `TempoInput` with both parts blank clears.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct UpdateItem {
    pub title: Option<String>,
    pub kind: Option<ItemKind>,
    pub composer: Option<Option<String>>,
    pub key: Option<Option<Key>>,
    pub tempo: Option<TempoInput>,
    pub notes: Option<Option<String>>,
    pub tags: Option<Vec<String>>,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ListQuery {
    pub text: Option<String>,
    pub item_type: Option<ItemKind>,
    pub key: Option<Key>,
    /// Empty vec means "no filter". Avoids `Option<Vec<T>>` which
    /// serde-reflection (used by Crux typegen) cannot handle.
    pub tags: Vec<String>,
    pub priority_only: bool,
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

impl LibrarySort {
    /// The blob is positional bincode; the shell names its storage key by
    /// this number, so a shape change bumps it here and nowhere else (#2089).
    pub const BLOB_VERSION: u32 = 1;
}

/// Round-trip through crux's actual FFI format (`BincodeFfiFormat`) — the
/// exact wire the iOS bridge uses, so tests can't drift from the real
/// serializer and we don't take a direct bincode dependency. Shared by every
/// bridge-crossing type's round-trip guard (#846 class).
#[cfg(test)]
pub(crate) fn assert_round_trips<T>(value: T)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_sort_defaults_to_date_added_descending() {
        let sort = LibrarySort::default();
        assert_eq!(sort.field, SortField::DateAdded);
        assert_eq!(sort.direction, SortDirection::Descending);
    }

    // ── Wire ──

    const PINNED_SORT_HEX: &str = "0100000000000000";
    /// The last variant of each enum, so a reorder at either tail moves a pinned byte.
    const PINNED_TAIL_SORT_HEX: &str = "0200000001000000";

    const PINNED_SORT_BLOB_VERSION: u32 = 1;

    /// Positional bincode, saved by the shell under a key built from
    /// `BLOB_VERSION` (#2089); the failure message is the protocol.
    #[test]
    fn library_sort_blob_wire_is_pinned() {
        use crux_core::bridge::{BincodeFfiFormat, FfiFormat};
        for (sort, pinned) in [
            (
                LibrarySort {
                    field: SortField::LastPracticed,
                    direction: SortDirection::Descending,
                },
                PINNED_SORT_HEX,
            ),
            (
                LibrarySort {
                    field: SortField::Title,
                    direction: SortDirection::Ascending,
                },
                PINNED_TAIL_SORT_HEX,
            ),
        ] {
            let mut bytes = Vec::new();
            BincodeFfiFormat::serialize(&mut bytes, &sort).expect("serialize");
            let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
            assert_eq!(
                (LibrarySort::BLOB_VERSION, hex.as_str()),
                (PINNED_SORT_BLOB_VERSION, pinned),
                "the library sort blob changed shape: bump LibrarySort::BLOB_VERSION and PINNED_SORT_BLOB_VERSION, then re-pin the hex. Never only the hex."
            );
            let back: LibrarySort =
                BincodeFfiFormat::deserialize(&bytes).expect("must decode on the FFI wire (#846)");
            assert_eq!(back, sort);
        }
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
    // format misaligns that wire and the event silently fails to decode. These
    // guard against that whole class, via the module-level `assert_round_trips`
    // shared with the other domain modules.

    #[test]
    fn update_item_round_trips_on_ffi_bincode_wire() {
        // Both three-state branches, `Some(Some)` = set and `Some(None)` =
        // clear: the shapes that failed to decode before #846.
        assert_round_trips(UpdateItem {
            title: Some("Renamed".to_string()),
            kind: Some(ItemKind::Exercise),
            composer: Some(Some("Bach".to_string())),
            key: Some(None),
            tempo: Some(TempoInput {
                marking: Some("Allegro".to_string()),
                bpm: Some("120".to_string()),
            }),
            notes: Some(None),
            tags: Some(vec!["etude".to_string()]),
            priority: Some(true),
        });
    }

    #[test]
    fn update_item_skipped_fields_round_trip_on_ffi_bincode_wire() {
        assert_round_trips(UpdateItem {
            title: Some("Renamed".to_string()),
            composer: Some(None),
            ..UpdateItem::default()
        });
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
                tempo: Some(TempoInput::default()),
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
            key: crate::domain::key::Key::parse("Db major"),
            tempo: Some(TempoInput {
                marking: None,
                bpm: Some("72".to_string()),
            }),
            notes: None,
            tags: vec!["impressionist".to_string()],
            photo_id: None,
            variation_labels: Vec::new(),
        });
    }

    // Remaining bridge-crossing write payloads — guard against a #846-class break.

    #[test]
    fn save_session_persistence_op_round_trips_on_ffi_bincode_wire() {
        // PracticeSession crosses the bridge as a SaveSession persistence Effect;
        // its optional-heavy SetlistEntry + rep_history is exactly the #846 risk.
        use crate::domain::session::{
            CompletionStatus, EntryStatus, Play, PracticeSession, RepAction, RepEvent, SetlistEntry,
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
            intention: Some("evenness".to_string()),
            planned_duration_secs: Some(300),
            group_id: None,
            planned_rep_target: Some(5),
            plays: vec![Play {
                id: "play-1".to_string(),
                section_id: None,
                key: crate::domain::key::Key::parse("Bb minor"),
                variation_ids: vec!["v-1".to_string()],
                tempo_changes: vec![],
                started_at: now,
                seconds: 300,
                rep_target: Some(5),
                rep_count: Some(5),
                rep_history: Some(vec![
                    RepEvent {
                        action: RepAction::Success,
                        at: now,
                        click_sounding: None,
                        tempo: None,
                    },
                    RepEvent {
                        action: RepAction::Missed,
                        at: now,
                        click_sounding: None,
                        tempo: None,
                    },
                ]),
                achieved_tempo: Some(120),
                click_pattern: None,
                score: Some(4),
                away: Vec::new(),
            }],
            segments: Vec::new(),
            focus: None,
            intention_met: None,
            felt: None,
            got_in_the_way: Vec::new(),
            note_points: Vec::new(),
            planned_variation_ids: vec![],
            planned_key: crate::domain::key::Key::parse("E flat major"),
        };
        assert_round_trips(PersistenceOperation::SaveSession(PracticeSession {
            id: "s1".to_string(),
            entries: vec![entry],
            session_notes: Some("solid".to_string()),
            started_at: now,
            completed_at: now,
            total_duration_secs: 300,
            completion_status: CompletionStatus::Completed,
            session_score: Some(8),
            capture_version: None,
        }));
    }

    #[test]
    fn block_grouping_session_events_round_trip_on_ffi_bincode_wire() {
        use crate::domain::session::SessionEvent;
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
        assert_round_trips(SessionEvent::SetEntryPlan {
            entry_id: "e1".to_string(),
            section_ids: vec!["s-1".to_string()],
            variation_ids: vec!["v-1".to_string()],
        });
    }

    #[test]
    fn save_item_and_variation_ops_round_trip_on_ffi_bincode_wire() {
        // Keys, variation ids, a tombstoned section and links ride SaveItem and
        // SaveItems to the store (#846 class).
        use crate::persistence::PersistenceOperation;
        use chrono::TimeZone;
        let at = chrono::Utc.timestamp_opt(1_700_000_000, 0).unwrap();
        let item = Item {
            id: "ex-1".to_string(),
            title: "Shells".to_string(),
            kind: ItemKind::Exercise,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            exercise_links: vec![
                crate::domain::link::ExerciseLink {
                    id: "l-1".to_string(),
                    exercise_id: "ex-2".to_string(),
                    section_id: Some("s-1".to_string()),
                    position: 0,
                    updated_at: at,
                    deleted_at: None,
                },
                crate::domain::link::ExerciseLink {
                    id: "l-2".to_string(),
                    exercise_id: "ex-3".to_string(),
                    section_id: None,
                    position: 1,
                    updated_at: at,
                    deleted_at: Some(at),
                },
            ],
            created_at: at,
            updated_at: at,
            priority: false,
            chord_chart: None,
            photo_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            metre: None,
            sections: vec![
                crate::domain::section::ItemSection {
                    id: "s-1".to_string(),
                    name: "A1".to_string(),
                    bars: Some(crate::domain::section::BarRange { first: 1, last: 16 }),
                    kind: crate::domain::section::SectionKind::Form,
                    target_bpm: Some(96),
                    position: 0,
                    updated_at: at,
                    deleted_at: None,
                },
                crate::domain::section::ItemSection {
                    id: "s-2".to_string(),
                    name: String::new(),
                    bars: Some(crate::domain::section::BarRange {
                        first: 12,
                        last: 14,
                    }),
                    kind: crate::domain::section::SectionKind::TroubleSpot,
                    target_bpm: None,
                    position: 1,
                    updated_at: at,
                    deleted_at: Some(at),
                },
            ],
            keys: vec![crate::domain::key::Key::C_MAJOR],
            variation_ids: vec!["v-1".to_string()],
        };
        assert_round_trips(PersistenceOperation::SaveItem(item.clone()));
        assert_round_trips(PersistenceOperation::SaveItems(vec![item]));
        let row = crate::domain::variation::Variation {
            id: "v-1".to_string(),
            label: "Hands separately".to_string(),
            updated_at: at,
            deleted_at: Some(at),
        };
        assert_round_trips(PersistenceOperation::SaveVariations(vec![row.clone()]));
        assert_round_trips(PersistenceOperation::LoadVariations);
        assert_round_trips(crate::persistence::PersistenceOutput::Variations(vec![row]));
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
            intention: None,
            planned_duration_secs: None,
            group_id: Some("block-1".to_string()),
            planned_rep_target: None,
            plays: Vec::new(),
            segments: Vec::new(),
            focus: None,
            intention_met: None,
            felt: None,
            got_in_the_way: Vec::new(),
            note_points: Vec::new(),
            planned_variation_ids: vec![],
            planned_key: None,
        });
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
                reflection: None,
                segment: None,
            },
            now: anchor,
        });
    }

    #[test]
    fn practice_session_round_trips_on_ffi_bincode_wire() {
        use crate::domain::session::{CompletionStatus, PracticeSession};
        use chrono::TimeZone;
        assert_round_trips(PracticeSession {
            id: "s1".to_string(),
            entries: vec![],
            session_notes: Some("note".to_string()),
            started_at: chrono::Utc.timestamp_opt(1_700_000_000, 0).unwrap(),
            completed_at: chrono::Utc.timestamp_opt(1_700_003_600, 0).unwrap(),
            total_duration_secs: 3600,
            completion_status: CompletionStatus::Completed,
            session_score: Some(7),
            capture_version: None,
        });
    }
}
