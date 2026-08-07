//! Self-directed practice — the built session, play-through altitudes and
//! qualitative capture (#1256, `specs/built-session.md`). Entities only in
//! Phase A; the compose/resolution flows arrive with their screens.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::engine::Circle;

/// Decision 19b: a countable target with no authored node. The dictated
/// sentence *is* the gate; the parsed parameters are read back, never asked
/// for. Low-band prior, cold-testable, own mastery — enforced where evidence
/// lands (Phase B), not stored here.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct UserDrill {
    pub id: String,
    pub name: String,
    /// The criterion sentence, verbatim — editable in place (A4).
    pub criterion: String,
    pub tempo_bpm: Option<u16>,
    pub keys: Vec<String>,
    pub passes_to_open: u8,
    pub serves: Option<Serves>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Where a user drill's evidence shows in the ability picture (decision 19):
/// a fluency circle or a named node (an authored node or a piece's).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Serves {
    Circle(Circle),
    Node(String),
}

/// Decision 19c: the genuinely unmeasurable target — time and notes on the
/// judgement track, no gate, no mastery.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct JournalItem {
    pub id: String,
    pub name: String,
    pub notes: Option<String>,
    /// The piece it is kept with (A5), where there is one.
    pub linked_item_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Decision 19: today's composed session — the strongest form of a steer,
/// never a mode. Blocks are ordered; each keeps its own gate semantics via
/// its target kind.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct BuiltSession {
    pub id: String,
    /// Provenance, in the user's words — "From Friday's lesson" (A6).
    pub source: Option<String>,
    pub blocks: Vec<BuiltBlock>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct BuiltBlock {
    pub id: String,
    pub target: BuiltTarget,
    pub minutes: Option<u16>,
}

/// The three-way resolution's outcome, plus the tune pipeline entry a piece
/// gets by nature (decision 19 / the brief's journey A).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum BuiltTarget {
    /// (a) matched an authored node — evidence at full weight.
    Node { node: String },
    /// (b) user drill — own node, own gate.
    UserDrill { drill_id: String },
    /// (c) journal — judgement track only.
    Journal { journal_id: String },
    /// A piece headed for the tune pipeline.
    Piece { item_id: String },
}

/// One gated run-through (Journey B's run-through altitude): section-level
/// verdicts only, never note-level. Off-piste and unmonitored play are
/// already the engine's `WanderRecord` and `unmonitored_seconds`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PlayThroughRecord {
    pub id: String,
    pub item_id: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    /// False when the user chose "Don't count this run" (B1).
    pub counted: bool,
    pub sections: Vec<SectionVerdict>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// "Did it hold?" — one tap per section (B1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SectionVerdict {
    pub section: String,
    pub held: bool,
    pub at: DateTime<Utc>,
}

/// Voice-first qualitative capture (decision 17): audio is kept,
/// transcription is opportunistic, and none of it ever feeds mastery.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Reflection {
    pub id: String,
    pub kind: ReflectionKind,
    /// The session or wander it was said in, where there was one.
    pub session_ref: Option<String>,
    pub transcript: Option<String>,
    /// Shell-relative path to the kept audio; the core never reads the bytes.
    pub audio_path: Option<String>,
    pub duration_s: Option<u32>,
    pub at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum ReflectionKind {
    /// Off-piste's "Found something? Say it" (B2).
    VoiceNote,
    /// The session-close reflection (C2).
    SessionClose,
}

/// At most one per block, and only where feel is the point (C1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct FeelEntry {
    pub id: String,
    pub block_id: String,
    pub feel: Feel,
    pub at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum Feel {
    FoughtIt,
    GettingThere,
    ItSang,
}

// ── Events ───────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct CreateUserDrill {
    pub name: String,
    pub criterion: String,
    pub tempo_bpm: Option<u16>,
    pub keys: Vec<String>,
    pub passes_to_open: u8,
    pub serves: Option<Serves>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct CreateJournalItem {
    pub name: String,
    pub notes: Option<String>,
    pub linked_item_id: Option<String>,
}

/// Local-first only (invariant 6: new engine/domain code targets local-first
/// exclusively) — no handler here may emit `Http`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum BuiltSessionEvent {
    CreateUserDrill(CreateUserDrill),
    UpdateUserDrill {
        drill: UserDrill,
    },
    DeleteUserDrill {
        id: String,
    },
    CreateJournalItem(CreateJournalItem),
    UpdateJournalItem {
        journal: JournalItem,
    },
    DeleteJournalItem {
        id: String,
    },
    /// Upsert by id — the compose flow (Phase B) edits in place.
    SaveBuiltSession {
        session: BuiltSession,
    },
    RecordPlayThrough {
        record: PlayThroughRecord,
    },
    RecordReflection {
        kind: ReflectionKind,
        session_ref: Option<String>,
        transcript: Option<String>,
        audio_path: Option<String>,
        duration_s: Option<u32>,
    },
    RecordFeel {
        block_id: String,
        feel: Feel,
    },
}

use crate::app::{Effect, Event};
use crate::model::Model;
use crate::persistence;
use crux_core::command::Command;

fn mint_id() -> String {
    ulid::Ulid::generate().to_string()
}

pub fn handle_built_session_event(
    event: BuiltSessionEvent,
    model: &mut Model,
) -> Command<Effect, Event> {
    let now = chrono::Utc::now();
    match event {
        BuiltSessionEvent::CreateUserDrill(input) => {
            let drill = UserDrill {
                id: mint_id(),
                name: input.name,
                criterion: input.criterion,
                tempo_bpm: input.tempo_bpm,
                keys: input.keys,
                passes_to_open: input.passes_to_open,
                serves: input.serves,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };
            model.user_drills.push(drill.clone());
            save_and_render(persistence::save_user_drill(drill))
        }
        BuiltSessionEvent::UpdateUserDrill { mut drill } => {
            let Some(stored) = model.user_drills.iter_mut().find(|d| d.id == drill.id) else {
                model.surface_error("That drill no longer exists.");
                return crux_core::render::render();
            };
            drill.updated_at = now;
            *stored = drill.clone();
            save_and_render(persistence::save_user_drill(drill))
        }
        BuiltSessionEvent::DeleteUserDrill { id } => {
            let Some(index) = model.user_drills.iter().position(|d| d.id == id) else {
                model.surface_error("That drill no longer exists.");
                return crux_core::render::render();
            };
            let mut tombstone = model.user_drills.remove(index);
            tombstone.deleted_at = Some(now);
            tombstone.updated_at = now;
            save_and_render(persistence::save_user_drill(tombstone))
        }
        BuiltSessionEvent::CreateJournalItem(input) => {
            let journal = JournalItem {
                id: mint_id(),
                name: input.name,
                notes: input.notes,
                linked_item_id: input.linked_item_id,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };
            model.journal_items.push(journal.clone());
            save_and_render(persistence::save_journal_item(journal))
        }
        BuiltSessionEvent::UpdateJournalItem { mut journal } => {
            let Some(stored) = model.journal_items.iter_mut().find(|j| j.id == journal.id) else {
                model.surface_error("That journal item no longer exists.");
                return crux_core::render::render();
            };
            journal.updated_at = now;
            *stored = journal.clone();
            save_and_render(persistence::save_journal_item(journal))
        }
        BuiltSessionEvent::DeleteJournalItem { id } => {
            let Some(index) = model.journal_items.iter().position(|j| j.id == id) else {
                model.surface_error("That journal item no longer exists.");
                return crux_core::render::render();
            };
            let mut tombstone = model.journal_items.remove(index);
            tombstone.deleted_at = Some(now);
            tombstone.updated_at = now;
            save_and_render(persistence::save_journal_item(tombstone))
        }
        BuiltSessionEvent::SaveBuiltSession { mut session } => {
            session.updated_at = now;
            match model.built_sessions.iter_mut().find(|s| s.id == session.id) {
                Some(stored) => *stored = session.clone(),
                None => model.built_sessions.push(session.clone()),
            }
            save_and_render(persistence::save_built_session(session))
        }
        BuiltSessionEvent::RecordPlayThrough { mut record } => {
            record.updated_at = now;
            model.play_throughs.push(record.clone());
            save_and_render(persistence::save_play_through(record))
        }
        BuiltSessionEvent::RecordReflection {
            kind,
            session_ref,
            transcript,
            audio_path,
            duration_s,
        } => {
            let reflection = Reflection {
                id: mint_id(),
                kind,
                session_ref,
                transcript,
                audio_path,
                duration_s,
                at: now,
                updated_at: now,
                deleted_at: None,
            };
            model.reflections.push(reflection.clone());
            save_and_render(persistence::save_reflection(reflection))
        }
        BuiltSessionEvent::RecordFeel { block_id, feel } => {
            let entry = FeelEntry {
                id: mint_id(),
                block_id,
                feel,
                at: now,
                updated_at: now,
                deleted_at: None,
            };
            model.feel_entries.push(entry.clone());
            save_and_render(persistence::save_feel_entry(entry))
        }
    }
}

fn save_and_render(save: Command<Effect, Event>) -> Command<Effect, Event> {
    Command::all([save, crux_core::render::render()])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::assert_round_trips;
    use crate::engine::Circle;
    use chrono::{TimeZone, Utc};

    fn at() -> chrono::DateTime<chrono::Utc> {
        Utc.with_ymd_and_hms(2020, 1, 1, 10, 0, 0).unwrap()
    }

    fn sample_drill() -> UserDrill {
        UserDrill {
            id: "01J0000000000000000000DRIL".into(),
            name: "Descending run into the bridge".into(),
            criterion: "Three clean passes at 72, left hand alone".into(),
            tempo_bpm: Some(72),
            keys: vec!["F".into()],
            passes_to_open: 3,
            serves: Some(Serves::Node("alice-bridge".into())),
            created_at: at(),
            updated_at: at(),
            deleted_at: None,
        }
    }

    #[test]
    fn user_drill_round_trips_on_the_wire() {
        assert_round_trips(sample_drill());
    }

    #[test]
    fn user_drill_with_no_optionals_round_trips() {
        assert_round_trips(UserDrill {
            tempo_bpm: None,
            keys: vec![],
            serves: None,
            deleted_at: Some(at()),
            ..sample_drill()
        });
    }

    #[test]
    fn serves_circle_round_trips() {
        assert_round_trips(Serves::Circle(Circle::Hands));
    }

    #[test]
    fn journal_item_round_trips_on_the_wire() {
        assert_round_trips(JournalItem {
            id: "01J0000000000000000000JRNL".into(),
            name: "Rubato feel in the intro".into(),
            notes: Some("Time and notes, kept with the piece".into()),
            linked_item_id: Some("01J0000000000000000000PIEC".into()),
            created_at: at(),
            updated_at: at(),
            deleted_at: None,
        });
    }

    #[test]
    fn built_session_round_trips_with_every_target_kind() {
        assert_round_trips(BuiltSession {
            id: "01J000000000000000000BSESH".into(),
            source: Some("From Friday's lesson".into()),
            blocks: vec![
                BuiltBlock {
                    id: "b1".into(),
                    target: BuiltTarget::Node {
                        node: "shell-voicings".into(),
                    },
                    minutes: Some(5),
                },
                BuiltBlock {
                    id: "b2".into(),
                    target: BuiltTarget::UserDrill {
                        drill_id: "01J0000000000000000000DRIL".into(),
                    },
                    minutes: None,
                },
                BuiltBlock {
                    id: "b3".into(),
                    target: BuiltTarget::Journal {
                        journal_id: "01J0000000000000000000JRNL".into(),
                    },
                    minutes: Some(8),
                },
                BuiltBlock {
                    id: "b4".into(),
                    target: BuiltTarget::Piece {
                        item_id: "01J0000000000000000000PIEC".into(),
                    },
                    minutes: Some(10),
                },
            ],
            created_at: at(),
            updated_at: at(),
            deleted_at: None,
        });
    }

    #[test]
    fn play_through_record_round_trips_on_the_wire() {
        assert_round_trips(PlayThroughRecord {
            id: "01J0000000000000000000PLAY".into(),
            item_id: "01J0000000000000000000PIEC".into(),
            started_at: at(),
            ended_at: at(),
            counted: true,
            sections: vec![
                SectionVerdict {
                    section: "The bridge, from memory".into(),
                    held: true,
                    at: at(),
                },
                SectionVerdict {
                    section: "Out head".into(),
                    held: false,
                    at: at(),
                },
            ],
            updated_at: at(),
            deleted_at: None,
        });
    }

    #[test]
    fn reflection_round_trips_on_the_wire() {
        for kind in [ReflectionKind::VoiceNote, ReflectionKind::SessionClose] {
            assert_round_trips(Reflection {
                id: "01J0000000000000000000REFL".into(),
                kind,
                session_ref: Some("01J000000000000000000BSESH".into()),
                transcript: Some("The bridge still rushes from memory".into()),
                audio_path: Some("reflections/01J.m4a".into()),
                duration_s: Some(24),
                at: at(),
                updated_at: at(),
                deleted_at: None,
            });
        }
    }

    #[test]
    fn feel_entry_round_trips_for_every_feel() {
        for feel in [Feel::FoughtIt, Feel::GettingThere, Feel::ItSang] {
            assert_round_trips(FeelEntry {
                id: "01J0000000000000000000FEEL".into(),
                block_id: "b1".into(),
                feel,
                at: at(),
                updated_at: at(),
                deleted_at: None,
            });
        }
    }

    // ── Events, model and persistence (Phase A handlers) ──────────────

    use crate::app::{Effect, Event, Intrada};
    use crate::model::Model;
    use crate::persistence::{BuiltSessionData, PersistenceOperation, PersistenceOutput};
    use crux_core::App;

    fn update(model: &mut Model, event: BuiltSessionEvent) -> Vec<Effect> {
        let app = Intrada;
        let mut cmd = app.update(Event::BuiltSession(event), model);
        cmd.effects().collect()
    }

    fn persistence_ops(effects: &[Effect]) -> Vec<PersistenceOperation> {
        effects
            .iter()
            .filter_map(|e| match e {
                Effect::Persistence(req) => Some(req.operation.clone()),
                _ => None,
            })
            .collect()
    }

    fn assert_no_http(effects: &[Effect]) {
        assert!(
            !effects.iter().any(|e| matches!(e, Effect::Http(_))),
            "local-first path must never emit Http (invariant 1)"
        );
    }

    fn sample_create_drill() -> CreateUserDrill {
        CreateUserDrill {
            name: "Descending run".into(),
            criterion: "Three clean passes at 72".into(),
            tempo_bpm: Some(72),
            keys: vec![],
            passes_to_open: 3,
            serves: None,
        }
    }

    #[test]
    fn create_user_drill_mints_a_ulid_and_persists() {
        let mut model = Model::test_default();
        let effects = update(
            &mut model,
            BuiltSessionEvent::CreateUserDrill(sample_create_drill()),
        );
        assert_eq!(model.user_drills.len(), 1);
        let drill = &model.user_drills[0];
        assert_eq!(drill.id.len(), 26, "client-minted ulid (invariant 3)");
        assert_eq!(drill.created_at, drill.updated_at);
        assert_eq!(
            persistence_ops(&effects),
            vec![PersistenceOperation::SaveUserDrill(drill.clone())]
        );
        assert_no_http(&effects);
    }

    #[test]
    fn update_user_drill_stamps_updated_at_and_persists() {
        let mut model = Model::test_default();
        update(
            &mut model,
            BuiltSessionEvent::CreateUserDrill(sample_create_drill()),
        );
        let mut edited = model.user_drills[0].clone();
        edited.criterion = "Five clean passes".into();
        edited.updated_at = at();
        let effects = update(
            &mut model,
            BuiltSessionEvent::UpdateUserDrill {
                drill: edited.clone(),
            },
        );
        let stored = &model.user_drills[0];
        assert_eq!(stored.criterion, "Five clean passes");
        assert!(
            stored.updated_at > at(),
            "core stamps updated_at, never trusts the caller's"
        );
        assert_eq!(
            persistence_ops(&effects),
            vec![PersistenceOperation::SaveUserDrill(stored.clone())]
        );
        assert_no_http(&effects);
    }

    #[test]
    fn delete_user_drill_tombstones_never_hard_deletes() {
        let mut model = Model::test_default();
        update(
            &mut model,
            BuiltSessionEvent::CreateUserDrill(sample_create_drill()),
        );
        let id = model.user_drills[0].id.clone();
        let effects = update(&mut model, BuiltSessionEvent::DeleteUserDrill { id });
        assert!(model.user_drills.is_empty(), "model holds live rows only");
        match persistence_ops(&effects).as_slice() {
            [PersistenceOperation::SaveUserDrill(saved)] => {
                assert!(saved.deleted_at.is_some())
            }
            other => panic!("expected one tombstone save, got {other:?}"),
        }
        assert_no_http(&effects);
    }

    #[test]
    fn unknown_user_drill_update_surfaces_an_error() {
        let mut model = Model::test_default();
        let effects = update(
            &mut model,
            BuiltSessionEvent::UpdateUserDrill {
                drill: sample_drill(),
            },
        );
        assert!(model.last_error.is_some());
        assert!(persistence_ops(&effects).is_empty(), "nothing to persist");
    }

    #[test]
    fn create_journal_item_mints_and_persists() {
        let mut model = Model::test_default();
        let effects = update(
            &mut model,
            BuiltSessionEvent::CreateJournalItem(CreateJournalItem {
                name: "Rubato feel".into(),
                notes: None,
                linked_item_id: None,
            }),
        );
        assert_eq!(model.journal_items.len(), 1);
        let journal = &model.journal_items[0];
        assert_eq!(journal.id.len(), 26);
        assert_eq!(
            persistence_ops(&effects),
            vec![PersistenceOperation::SaveJournalItem(journal.clone())]
        );
        assert_no_http(&effects);
    }

    #[test]
    fn delete_journal_item_tombstones() {
        let mut model = Model::test_default();
        update(
            &mut model,
            BuiltSessionEvent::CreateJournalItem(CreateJournalItem {
                name: "Rubato feel".into(),
                notes: None,
                linked_item_id: None,
            }),
        );
        let id = model.journal_items[0].id.clone();
        let effects = update(&mut model, BuiltSessionEvent::DeleteJournalItem { id });
        assert!(model.journal_items.is_empty(), "model holds live rows only");
        match persistence_ops(&effects).as_slice() {
            [PersistenceOperation::SaveJournalItem(saved)] => {
                assert!(saved.deleted_at.is_some())
            }
            other => panic!("expected one tombstone save, got {other:?}"),
        }
    }

    fn sample_built_session() -> BuiltSession {
        BuiltSession {
            id: "01J000000000000000000BSESH".into(),
            source: None,
            blocks: vec![],
            created_at: at(),
            updated_at: at(),
            deleted_at: None,
        }
    }

    #[test]
    fn save_built_session_upserts_and_persists() {
        let mut model = Model::test_default();
        let effects = update(
            &mut model,
            BuiltSessionEvent::SaveBuiltSession {
                session: sample_built_session(),
            },
        );
        assert_eq!(model.built_sessions.len(), 1);
        assert!(model.built_sessions[0].updated_at > at());
        assert_eq!(
            persistence_ops(&effects),
            vec![PersistenceOperation::SaveBuiltSession(
                model.built_sessions[0].clone()
            )]
        );
        assert_no_http(&effects);

        // Saving the same id again replaces, never duplicates.
        update(
            &mut model,
            BuiltSessionEvent::SaveBuiltSession {
                session: sample_built_session(),
            },
        );
        assert_eq!(model.built_sessions.len(), 1);
    }

    #[test]
    fn record_play_through_persists() {
        let mut model = Model::test_default();
        let record = PlayThroughRecord {
            id: "01J0000000000000000000PLAY".into(),
            item_id: "piece".into(),
            started_at: at(),
            ended_at: at(),
            counted: true,
            sections: vec![],
            updated_at: at(),
            deleted_at: None,
        };
        let effects = update(
            &mut model,
            BuiltSessionEvent::RecordPlayThrough {
                record: record.clone(),
            },
        );
        assert_eq!(model.play_throughs.len(), 1);
        assert_eq!(
            persistence_ops(&effects),
            vec![PersistenceOperation::SavePlayThrough(
                model.play_throughs[0].clone()
            )]
        );
        assert_no_http(&effects);
    }

    #[test]
    fn record_reflection_mints_and_persists() {
        let mut model = Model::test_default();
        let effects = update(
            &mut model,
            BuiltSessionEvent::RecordReflection {
                kind: ReflectionKind::SessionClose,
                session_ref: None,
                transcript: Some("The bridge still rushes".into()),
                audio_path: Some("reflections/x.m4a".into()),
                duration_s: Some(24),
            },
        );
        assert_eq!(model.reflections.len(), 1);
        assert_eq!(model.reflections[0].id.len(), 26);
        assert_eq!(
            persistence_ops(&effects),
            vec![PersistenceOperation::SaveReflection(
                model.reflections[0].clone()
            )]
        );
        assert_no_http(&effects);
    }

    #[test]
    fn record_feel_mints_and_persists() {
        let mut model = Model::test_default();
        let effects = update(
            &mut model,
            BuiltSessionEvent::RecordFeel {
                block_id: "b1".into(),
                feel: Feel::ItSang,
            },
        );
        assert_eq!(model.feel_entries.len(), 1);
        assert_eq!(model.feel_entries[0].id.len(), 26);
        assert_eq!(model.feel_entries[0].feel, Feel::ItSang);
        assert_eq!(
            persistence_ops(&effects),
            vec![PersistenceOperation::SaveFeelEntry(
                model.feel_entries[0].clone()
            )]
        );
        assert_no_http(&effects);
    }

    // ── Hydration and write results ────────────────────────────────────

    fn app_update(model: &mut Model, event: Event) -> Vec<Effect> {
        let app = Intrada;
        let mut cmd = app.update(event, model);
        cmd.effects().collect()
    }

    #[test]
    fn local_first_start_loads_built_session_data() {
        let mut model = Model::test_default();
        let effects = app_update(
            &mut model,
            Event::StartApp {
                api_base_url: "http://localhost".into(),
                local_first: true,
            },
        );
        assert!(persistence_ops(&effects).contains(&PersistenceOperation::LoadBuiltSessionData));
    }

    #[test]
    fn online_start_never_touches_the_built_session_store() {
        let mut model = Model::test_default();
        let effects = app_update(
            &mut model,
            Event::StartApp {
                api_base_url: "http://localhost".into(),
                local_first: false,
            },
        );
        assert!(!persistence_ops(&effects).contains(&PersistenceOperation::LoadBuiltSessionData));
    }

    #[test]
    fn built_store_loaded_populates_the_model() {
        let mut model = Model::test_default();
        let data = BuiltSessionData {
            user_drills: vec![sample_drill()],
            journal_items: vec![],
            built_sessions: vec![sample_built_session()],
            play_throughs: vec![],
            reflections: vec![],
            feel_entries: vec![],
        };
        app_update(
            &mut model,
            Event::BuiltStoreLoaded(PersistenceOutput::BuiltSessionData(data)),
        );
        assert_eq!(model.user_drills.len(), 1);
        assert_eq!(model.built_sessions.len(), 1);
    }

    #[test]
    fn built_store_loaded_failed_surfaces_without_reloading() {
        let mut model = Model::test_default();
        let effects = app_update(
            &mut model,
            Event::BuiltStoreLoaded(PersistenceOutput::Failed),
        );
        assert!(model.last_error.is_some());
        assert!(persistence_ops(&effects).is_empty(), "no reload loop");
    }

    #[test]
    fn built_store_written_failed_surfaces_and_rehydrates() {
        let mut model = Model::test_default();
        let effects = app_update(
            &mut model,
            Event::BuiltStoreWritten(PersistenceOutput::Failed),
        );
        assert!(model.last_error.is_some(), "invariant 5: never silent");
        assert_eq!(
            persistence_ops(&effects),
            vec![PersistenceOperation::LoadBuiltSessionData]
        );
    }

    #[test]
    fn built_store_written_ack_is_a_noop() {
        let mut model = Model::test_default();
        let effects = app_update(&mut model, Event::BuiltStoreWritten(PersistenceOutput::Ack));
        assert!(model.last_error.is_none());
        assert!(persistence_ops(&effects).is_empty());
    }

    // ── Bridge payloads (#846) ──────────────────────────────────────────

    #[test]
    fn built_session_events_round_trip_on_the_wire() {
        assert_round_trips(Event::BuiltSession(BuiltSessionEvent::CreateUserDrill(
            sample_create_drill(),
        )));
        assert_round_trips(Event::BuiltSession(BuiltSessionEvent::RecordFeel {
            block_id: "b1".into(),
            feel: Feel::GettingThere,
        }));
        assert_round_trips(Event::BuiltSession(BuiltSessionEvent::SaveBuiltSession {
            session: sample_built_session(),
        }));
    }

    #[test]
    fn built_session_persistence_payloads_round_trip_on_the_wire() {
        assert_round_trips(PersistenceOperation::SaveUserDrill(sample_drill()));
        assert_round_trips(PersistenceOperation::LoadBuiltSessionData);
        assert_round_trips(PersistenceOutput::BuiltSessionData(BuiltSessionData {
            user_drills: vec![sample_drill()],
            journal_items: vec![],
            built_sessions: vec![sample_built_session()],
            play_throughs: vec![],
            reflections: vec![],
            feel_entries: vec![],
        }));
    }
}
