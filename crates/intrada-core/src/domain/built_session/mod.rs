//! Self-directed practice — the built session, play-through altitudes and
//! qualitative capture (#1256, `specs/built-session.md`).
//!
//! - [`compose`] is the steer sheet and decision 19's three-way resolution.
//! - [`criterion`] reads a dictated sentence back as gate parameters.
//! - [`blocks`] turns a composed session into a plan the drill loop runs.
//! - [`capture`] is Journey C's measurement budget: when feel is asked, and
//!   when the close may ask for a reflection.
//! - [`steer`] is last night's reflection, back as one morning proposal.

pub mod blocks;
pub mod capture;
pub mod compose;
pub mod criterion;
pub mod playthrough;
pub mod steer;
#[cfg(test)]
pub(crate) mod tests_support;

use chrono::{DateTime, Utc};
use crux_core::command::Command;
use serde::{Deserialize, Serialize};

use crate::app::{Effect, Event};
use crate::engine::{Altitude, Circle, CoachEvent, SteerPlacement};
use crate::model::Model;
use crate::persistence;
use crate::validation;

use compose::{ComposeDraft, ComposeEntry, Question, Resolution, ResolutionKind};
use criterion::parse_criterion;

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
/// already the engine's `WanderRecord` and `UnmonitoredRecord`.
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
    /// C3: what became of the morning proposal this reflection earned. The
    /// whole memory of the offer, so a declined one leaves no trace beyond the
    /// fact that it will not be asked again, and an accepted one rebuilds its
    /// block after a relaunch rather than losing it with the plan.
    pub steer: SteerState,
    /// When it was answered. Bounds how long an accepted steer stays today's.
    pub steer_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum SteerState {
    #[default]
    Unoffered,
    Accepted,
    Declined,
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

/// One feel, asked at most once per block and only where feel is the point
/// (C1) — the budget is the asking surface's, enforced in Phase D, not here.
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
    RecordReflection {
        kind: ReflectionKind,
        session_ref: Option<String>,
        transcript: Option<String>,
        audio_path: Option<String>,
        duration_s: Option<u32>,
    },
    /// Transcription is opportunistic and may land long after the audio was
    /// kept (C2), so the transcript arrives as its own event rather than
    /// holding the save open until the shell has one.
    AttachTranscript {
        reflection_id: String,
        transcript: String,
    },
    RecordFeel {
        block_id: String,
        feel: Feel,
    },

    // ── Qualitative capture (Journey C) ──────────────────────────────
    /// C1's first-class exit. Nothing is written: a skipped feel is not a
    /// fourth feel, it is the absence of one.
    SkipFeel,
    /// C2's "Not tonight", respected without a follow-up nudge.
    DismissReflection,
    /// C3's "Add it to today": takes the card down and puts the block in
    /// today's plan. Carries only the reflection, because the target is
    /// re-derived from it rather than taken from the card.
    AcceptProposedSteer {
        reflection_id: String,
    },
    /// C3's "Not today" — no block, no trace, and never proposed again.
    DeclineProposedSteer {
        reflection_id: String,
    },

    // ── The steer sheet (Journey A) ──────────────────────────────────
    /// Open the sheet (A2). Idempotent: reopening keeps the list, because a
    /// steer the user is halfway through writing is not a mistake.
    OpenCompose,
    /// Dismiss it. "Declining costs nothing and the hero is untouched."
    CancelCompose,
    /// One typed or dictated name. `picked_item_id` is set when the user tapped
    /// a library suggestion rather than typing free text — the shell reports
    /// which row was tapped and decides nothing.
    AddComposeEntry {
        text: String,
        picked_item_id: Option<String>,
    },
    RemoveComposeEntry {
        entry_id: String,
    },
    /// A3's "Yes — same drill".
    ConfirmNodeMatch {
        entry_id: String,
    },
    /// A3's "No, it's different" and A4's two exits: one surface, two ways out,
    /// so a misjudged kind costs one tap either way.
    ChooseResolutionKind {
        entry_id: String,
        kind: ResolutionKind,
    },
    /// A4 — the criterion sentence is the gate.
    ResolveAsUserDrill {
        entry_id: String,
        criterion: String,
        serves: Option<Serves>,
    },
    /// A5 — the judgement track.
    ResolveAsJournal {
        entry_id: String,
        notes: Option<String>,
        linked_item_id: Option<String>,
    },
    /// Compose it (A6). Refused while any question is open: the price was
    /// stated, so it has to have been paid.
    BuildSession {
        source: Option<String>,
    },
    /// A6's declinable advice, accepted. Declining is simply not sending it.
    UseSuggestedShape,
    /// Start the composed session in the canonical drill loop (A6 → A7).
    StartBuiltSession {
        session_id: String,
        now: DateTime<Utc>,
    },

    // ── The altitudes (Journey B) ────────────────────────────────────
    /// Open B0 for a piece. Which altitudes it can take is
    /// [`playthrough::altitude_offer`]'s call, so a shell reading `chord_chart`
    /// itself would be a second answer to the same question.
    OpenPlayThrough {
        item_id: String,
    },
    ClosePlayThrough,
    /// B0's answer to "Play it through — how should it count?". The piece is
    /// resolved here, where the library lives; the engine is handed the named
    /// sections and never does a lookup of its own.
    StartPlayThrough {
        item_id: String,
        altitude: Altitude,
        now: DateTime<Utc>,
    },
}

// ── Handlers ─────────────────────────────────────────────────────────

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
            let input = validation::normalize_create_user_drill(input);
            if let Err(e) = validation::validate_create_user_drill(&input) {
                model.surface_error(e.to_string());
                return crux_core::render::render();
            }
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
            // Provenance and tombstone are the core's to set: an update must
            // not be able to resurrect a deleted drill or delete a live one.
            drill.created_at = stored.created_at;
            drill.deleted_at = stored.deleted_at;
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
            let input = validation::normalize_create_journal_item(input);
            if let Err(e) = validation::validate_create_journal_item(&input) {
                model.surface_error(e.to_string());
                return crux_core::render::render();
            }
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
            journal.created_at = stored.created_at;
            journal.deleted_at = stored.deleted_at;
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
                steer: SteerState::Unoffered,
                steer_at: None,
                updated_at: now,
                deleted_at: None,
            };
            // The close asked once and has its answer, whichever way it went.
            model.reflection_prompt = false;
            model.reflections.push(reflection.clone());
            save_and_render(persistence::save_reflection(reflection))
        }
        BuiltSessionEvent::AttachTranscript {
            reflection_id,
            transcript,
        } => {
            // A tombstoned row is one the user deleted while the transcription
            // was still running: writing to it would resurrect their words.
            let Some(reflection) = model.reflections.iter_mut().find(|reflection| {
                reflection.id == reflection_id && reflection.deleted_at.is_none()
            }) else {
                return crux_core::render::render();
            };
            reflection.transcript = Some(transcript);
            reflection.updated_at = now;
            let reflection = reflection.clone();
            save_and_render(persistence::save_reflection(reflection))
        }
        BuiltSessionEvent::RecordFeel { block_id, feel } => {
            // One per block, ever — the budget is the core's, so a shell that
            // asked twice still only writes one (decision 17).
            if model
                .feel_entries
                .iter()
                .any(|entry| entry.block_id == block_id)
            {
                model.feel_prompt = None;
                return crux_core::render::render();
            }
            let entry = FeelEntry {
                id: mint_id(),
                block_id,
                feel,
                at: now,
                updated_at: now,
                deleted_at: None,
            };
            model.feel_prompt = None;
            model.feel_entries.push(entry.clone());
            save_and_render(persistence::save_feel_entry(entry))
        }

        // ── Qualitative capture ──────────────────────────────────────
        BuiltSessionEvent::SkipFeel => {
            model.feel_prompt = None;
            crux_core::render::render()
        }
        BuiltSessionEvent::DismissReflection => {
            model.reflection_prompt = false;
            crux_core::render::render()
        }
        BuiltSessionEvent::AcceptProposedSteer { reflection_id } => {
            answer_steer(model, &reflection_id, SteerState::Accepted, now)
        }
        BuiltSessionEvent::DeclineProposedSteer { reflection_id } => {
            answer_steer(model, &reflection_id, SteerState::Declined, now)
        }

        // ── The steer sheet ──────────────────────────────────────────
        BuiltSessionEvent::OpenCompose => {
            model.compose.get_or_insert_with(ComposeDraft::default);
            crux_core::render::render()
        }
        BuiltSessionEvent::CancelCompose => {
            model.compose = None;
            crux_core::render::render()
        }
        BuiltSessionEvent::AddComposeEntry {
            text,
            picked_item_id,
        } => {
            let name = text.trim().to_string();
            if name.is_empty() {
                model.surface_error("Give it a name and it can go in today's list.");
                return crux_core::render::render();
            }
            let resolution = compose::resolve(
                &name,
                picked_item_id.as_deref(),
                &model.resolution_context(now),
            );
            let draft = model.compose.get_or_insert_with(ComposeDraft::default);
            // Adding the same thing twice is a slip, not a second block.
            if draft.entries.iter().any(|entry| {
                entry.resolution == resolution && matches!(resolution, Resolution::Settled(_))
            }) {
                return crux_core::render::render();
            }
            draft.entries.push(ComposeEntry {
                id: mint_id(),
                name,
                resolution,
            });
            crux_core::render::render()
        }
        BuiltSessionEvent::RemoveComposeEntry { entry_id } => {
            if let Some(draft) = model.compose.as_mut() {
                draft.entries.retain(|entry| entry.id != entry_id);
            }
            crux_core::render::render()
        }
        BuiltSessionEvent::ConfirmNodeMatch { entry_id } => {
            let Some(draft) = model.compose.as_mut() else {
                return crux_core::render::render();
            };
            if let Some(entry) = draft.entry_mut(&entry_id) {
                if let Some(Question::NodeMatch { node, .. }) = entry.question() {
                    entry.resolution =
                        Resolution::Settled(BuiltTarget::Node { node: node.clone() });
                }
            }
            crux_core::render::render()
        }
        BuiltSessionEvent::ChooseResolutionKind { entry_id, kind } => {
            let Some(draft) = model.compose.as_mut() else {
                return crux_core::render::render();
            };
            if let Some(entry) = draft.entry_mut(&entry_id) {
                // Only an open question changes kind: a settled entry is a
                // block, and re-asking it would un-pay a paid resolution.
                if entry.question().is_some() {
                    entry.resolution = Resolution::Asking(match kind {
                        ResolutionKind::UserDrill => Question::UserDrill,
                        ResolutionKind::Journal => Question::Journal,
                    });
                }
            }
            crux_core::render::render()
        }
        BuiltSessionEvent::ResolveAsUserDrill {
            entry_id,
            criterion,
            serves,
        } => {
            let Some(name) = model.compose_entry_name(&entry_id) else {
                return crux_core::render::render();
            };
            let parsed = parse_criterion(&criterion);
            let input = validation::normalize_create_user_drill(CreateUserDrill {
                name,
                criterion,
                tempo_bpm: parsed.tempo_bpm,
                keys: parsed.keys,
                passes_to_open: parsed.passes_to_open,
                serves,
            });
            if let Err(e) = validation::validate_create_user_drill(&input) {
                model.surface_error(e.to_string());
                return crux_core::render::render();
            }
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
            model.settle_compose_entry(
                &entry_id,
                BuiltTarget::UserDrill {
                    drill_id: drill.id.clone(),
                },
            );
            model.user_drills.push(drill.clone());
            save_and_render(persistence::save_user_drill(drill))
        }
        BuiltSessionEvent::ResolveAsJournal {
            entry_id,
            notes,
            linked_item_id,
        } => {
            let Some(name) = model.compose_entry_name(&entry_id) else {
                return crux_core::render::render();
            };
            let input = validation::normalize_create_journal_item(CreateJournalItem {
                name,
                notes,
                linked_item_id,
            });
            if let Err(e) = validation::validate_create_journal_item(&input) {
                model.surface_error(e.to_string());
                return crux_core::render::render();
            }
            let journal = JournalItem {
                id: mint_id(),
                name: input.name,
                notes: input.notes,
                linked_item_id: input.linked_item_id,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };
            model.settle_compose_entry(
                &entry_id,
                BuiltTarget::Journal {
                    journal_id: journal.id.clone(),
                },
            );
            model.journal_items.push(journal.clone());
            save_and_render(persistence::save_journal_item(journal))
        }
        BuiltSessionEvent::BuildSession { source } => {
            let Some(draft) = model.compose.as_ref() else {
                return crux_core::render::render();
            };
            if draft.entries.is_empty() {
                model.surface_error("Add something to practise first.");
                return crux_core::render::render();
            }
            if draft.open_questions() > 0 {
                model.surface_error("A couple of questions left before this can be built.");
                return crux_core::render::render();
            }
            let blocks = draft
                .entries
                .iter()
                .filter_map(|entry| match &entry.resolution {
                    Resolution::Settled(target) => Some(BuiltBlock {
                        id: mint_id(),
                        target: target.clone(),
                        minutes: None,
                    }),
                    Resolution::Asking(_) => None,
                })
                .collect();
            let session = BuiltSession {
                id: mint_id(),
                source: source
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty()),
                blocks,
                created_at: now,
                updated_at: now,
                deleted_at: None,
            };
            model.compose = None;
            model.built_session_today = Some(session.id.clone());
            model.built_sessions.push(session.clone());
            save_and_render(persistence::save_built_session(session))
        }
        BuiltSessionEvent::UseSuggestedShape => {
            let Some(session) = model.built_session_today_mut() else {
                return crux_core::render::render();
            };
            session.blocks = blocks::suggested_shape(&session.blocks);
            session.updated_at = now;
            let session = session.clone();
            save_and_render(persistence::save_built_session(session))
        }
        BuiltSessionEvent::StartBuiltSession { session_id, now } => {
            let Some(session) = model
                .built_sessions
                .iter()
                .find(|session| session.id == session_id)
                .cloned()
            else {
                model.surface_error("That session is no longer there.");
                return crux_core::render::render();
            };
            let plan = blocks::plan_from_built(&session, &model.build_context());
            if plan.blocks.is_empty() {
                model.surface_error("Nothing in this session can run today.");
                return crux_core::render::render();
            }
            model.built_session_today = Some(session_id);
            let writes = model.coach.adopt_plan(plan, now);
            let mut commands = crate::app::coach_write_commands(model, writes);
            commands.push(crux_core::render::render());
            Command::all(commands)
        }

        BuiltSessionEvent::OpenPlayThrough { item_id } => {
            let Some(offer) = model
                .items
                .iter()
                .find(|item| item.id == item_id)
                .and_then(playthrough::altitude_offer)
            else {
                model.surface_error("That piece is no longer there.");
                return crux_core::render::render();
            };
            model.play_through_item = Some(offer.item_id);
            crux_core::render::render()
        }
        BuiltSessionEvent::ClosePlayThrough => {
            model.play_through_item = None;
            crux_core::render::render()
        }

        BuiltSessionEvent::StartPlayThrough {
            item_id,
            altitude,
            now,
        } => {
            let offer = model
                .items
                .iter()
                .find(|item| item.id == item_id)
                .and_then(playthrough::altitude_offer);
            let Some(offer) = offer else {
                model.surface_error("That piece is no longer there.");
                return crux_core::render::render();
            };
            if !offer.allows(altitude) {
                model.surface_error("Add some section labels to the chart first.");
                return crux_core::render::render();
            }
            let event = match altitude {
                Altitude::RunThrough => CoachEvent::StartRunThrough {
                    item_id: offer.item_id.clone(),
                    title: offer.title.clone(),
                    sections: offer.sections,
                    now,
                },
                Altitude::OffPiste => CoachEvent::GoOffPiste {
                    item_id: Some(offer.item_id.clone()),
                    now,
                },
                // The piece is dropped on purpose: this altitude captures
                // nothing, so there is no record to tag and nothing that could
                // later say what was played. See `SessionState::Unmonitored`.
                Altitude::Unmonitored => CoachEvent::GoUnmonitored { now },
            };
            let writes = model.coach.apply(&event);
            // Every altitude guards on `accepts_something_new`, so one already
            // running swallows this; silently, it would close the sheet on a
            // success haptic having started nothing (#846). Asked against the
            // altitude requested, or off-piste answers for a run-through.
            if model.coach.view().altitude != Some(altitude) {
                model.surface_error("Finish what you're playing first.");
                return crux_core::render::render();
            }
            model.play_through_item = None;
            let mut commands = crate::app::coach_write_commands(model, writes);
            commands.push(crux_core::render::render());
            Command::all(commands)
        }
    }
}

fn save_and_render(save: Command<Effect, Event>) -> Command<Effect, Event> {
    Command::all([save, crux_core::render::render()])
}

/// Both answers write the same column: the offer is spent either way, which is
/// what stops it coming back tomorrow. Accepting also places the block *now*,
/// because the Practice screen asks for a plan only when it has none — a steer
/// that waited would leave the card up over an unchanged plan.
///
/// The placement is settled **before** the answer is written (#1317): a confirm
/// that placed nothing must not spend the reflection and report success.
fn answer_steer(
    model: &mut Model,
    reflection_id: &str,
    answer: SteerState,
    now: DateTime<Utc>,
) -> Command<Effect, Event> {
    let Some(index) = model
        .reflections
        .iter()
        .position(|reflection| reflection.id == reflection_id && reflection.deleted_at.is_none())
    else {
        model.surface_action_error("That note is no longer there.");
        return crux_core::render::render();
    };
    // Answering twice is a double tap, not a change of mind: the second would
    // move `steer_at` and quietly extend how long the block rides today's plan.
    if model.reflections[index].steer != SteerState::Unoffered {
        return crux_core::render::render();
    }

    if answer == SteerState::Accepted {
        // Re-derived, not the card's: the library may have changed under it.
        let block = steer::target_of(&model.reflections[index], &model.steer_context())
            .and_then(|target| blocks::steer_block(&target, &model.build_context()));
        let Some(block) = block else {
            // True of the ambiguous sentence as well as the deleted target.
            model.surface_action_error("That doesn't match anything in your library.");
            return crux_core::render::render();
        };
        match model.coach.place_steer(block) {
            SteerPlacement::Placed | SteerPlacement::Deferred => {}
            SteerPlacement::AlreadyPlanned => {
                model.surface_action_error(
                    "That's already in today's session, so nothing was added.",
                );
            }
            // The offer outlives the answer: there will be a plan it can join.
            SteerPlacement::SessionInFlight => {
                model.surface_action_error(
                    "Finish what you're playing and it can go in the next session.",
                );
                return crux_core::render::render();
            }
        }
    }

    let reflection = &mut model.reflections[index];
    reflection.steer = answer;
    reflection.steer_at = Some(now);
    reflection.updated_at = now;
    let reflection = reflection.clone();
    // The card is answered whichever way it went, so it comes down either way.
    model.proposed_steer = None;
    save_and_render(persistence::save_reflection(reflection))
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
                steer: SteerState::Unoffered,
                steer_at: None,
                updated_at: at(),
                deleted_at: None,
            });
        }
    }

    #[test]
    fn every_steer_state_round_trips_on_the_wire() {
        for steer in [
            SteerState::Unoffered,
            SteerState::Accepted,
            SteerState::Declined,
        ] {
            assert_round_trips(Reflection {
                id: "01J0000000000000000000REFL".into(),
                kind: ReflectionKind::SessionClose,
                session_ref: None,
                transcript: None,
                audio_path: None,
                duration_s: None,
                at: at(),
                steer,
                steer_at: Some(at()),
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
    fn a_whitespace_only_criterion_is_refused_not_stored() {
        let mut model = Model::test_default();
        let effects = update(
            &mut model,
            BuiltSessionEvent::CreateUserDrill(CreateUserDrill {
                criterion: "   ".into(),
                ..sample_create_drill()
            }),
        );
        assert!(model.user_drills.is_empty(), "nothing persisted");
        assert!(persistence_ops(&effects).is_empty());
        assert!(model.last_error.is_some(), "the refusal reaches the user");
    }

    #[test]
    fn an_ungateable_drill_is_refused() {
        let mut model = Model::test_default();
        update(
            &mut model,
            BuiltSessionEvent::CreateUserDrill(CreateUserDrill {
                passes_to_open: 0,
                ..sample_create_drill()
            }),
        );
        assert!(
            model.user_drills.is_empty(),
            "zero passes is a gate nothing can open"
        );
    }

    #[test]
    fn drill_free_text_is_trimmed_before_it_is_stored() {
        let mut model = Model::test_default();
        update(
            &mut model,
            BuiltSessionEvent::CreateUserDrill(CreateUserDrill {
                name: "  Descending run  ".into(),
                ..sample_create_drill()
            }),
        );
        assert_eq!(model.user_drills[0].name, "Descending run");
    }

    #[test]
    fn a_nameless_journal_item_is_refused() {
        let mut model = Model::test_default();
        let effects = update(
            &mut model,
            BuiltSessionEvent::CreateJournalItem(CreateJournalItem {
                name: "  ".into(),
                notes: None,
                linked_item_id: None,
            }),
        );
        assert!(model.journal_items.is_empty());
        assert!(persistence_ops(&effects).is_empty());
        assert!(model.last_error.is_some());
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
    fn an_update_cannot_resurrect_a_tombstoned_drill() {
        let mut model = Model::test_default();
        update(
            &mut model,
            BuiltSessionEvent::CreateUserDrill(sample_create_drill()),
        );
        let mut revived = model.user_drills[0].clone();
        revived.deleted_at = Some(at());
        update(
            &mut model,
            BuiltSessionEvent::UpdateUserDrill { drill: revived },
        );
        assert_eq!(
            model.user_drills[0].deleted_at, None,
            "only Delete* sets a tombstone"
        );
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

    // ── The steer sheet, end to end (Journey A) ─────────────────────────

    use crate::domain::built_session::compose::{AskView, ComposeKind};
    use crate::domain::built_session::tests_support::sample_item;
    use crate::domain::item::ItemKind;
    use crate::engine::{BlockOrigin, CoachEvent};

    fn view(model: &Model) -> crate::model::ViewModel {
        Intrada.view(model)
    }

    /// Start the open block and play one pass of it, ending in a clean tap. At
    /// l0 the tap alone bounds the attempt (decision 20); at a clicked rung the
    /// phrase boundary opens the verdict window first.
    fn play_one_pass(model: &mut Model) {
        app_update(model, Event::Coach(CoachEvent::StartBlock { now: at() }));
        let block = model.coach.session.block().expect("a block");
        if !block.level.is_untimed() {
            let phrase = block.body_beats();
            for beat_index in [0, phrase] {
                app_update(model, Event::Coach(CoachEvent::Beat { beat_index }));
            }
        }
        app_update(
            model,
            Event::Coach(CoachEvent::Tap {
                clean: true,
                now: at(),
            }),
        );
    }

    fn add(model: &mut Model, text: &str) -> String {
        update(
            model,
            BuiltSessionEvent::AddComposeEntry {
                text: text.into(),
                picked_item_id: None,
            },
        );
        model
            .compose
            .as_ref()
            .unwrap()
            .entries
            .last()
            .unwrap()
            .id
            .clone()
    }

    /// The lesson's own list: a piece, a drill the user must describe, and
    /// something nothing can count.
    fn a_composed_day(model: &mut Model) -> String {
        model
            .items
            .push(sample_item("p1", "Alice in Wonderland", ItemKind::Piece));
        update(model, BuiltSessionEvent::OpenCompose);
        add(model, "Alice in Wonderland");
        let drill_entry = add(model, "Zzz stride pattern, bars 1 to 8");
        let journal_entry = add(model, "Zzz freer rubato in the intro");
        update(
            model,
            BuiltSessionEvent::ResolveAsUserDrill {
                entry_id: drill_entry,
                criterion: "Both hands together, three clean passes at 72".into(),
                serves: Some(Serves::Node("p1".into())),
            },
        );
        update(
            model,
            BuiltSessionEvent::ChooseResolutionKind {
                entry_id: journal_entry.clone(),
                kind: ResolutionKind::Journal,
            },
        );
        update(
            model,
            BuiltSessionEvent::ResolveAsJournal {
                entry_id: journal_entry,
                notes: None,
                linked_item_id: Some("p1".into()),
            },
        );
        update(
            model,
            BuiltSessionEvent::BuildSession {
                source: Some("From Friday's lesson".into()),
            },
        );
        model.built_session_today.clone().expect("a built session")
    }

    #[test]
    fn the_sheet_states_its_price_and_a_piece_costs_nothing() {
        let mut model = Model::test_default();
        model
            .items
            .push(sample_item("p1", "Alice in Wonderland", ItemKind::Piece));
        update(&mut model, BuiltSessionEvent::OpenCompose);
        add(&mut model, "Alice in Wonderland");
        add(&mut model, "Zzz stride pattern");

        let compose = view(&model).built.compose.expect("the sheet is open");
        assert_eq!(compose.entries[0].kind, ComposeKind::Piece);
        assert_eq!(compose.entries[1].kind, ComposeKind::Unresolved);
        assert_eq!(compose.questions.len(), 1, "the piece asks nothing");
        assert_eq!(compose.build_label, "Continue — one quick question");
        assert!(
            !compose.can_build,
            "the price was stated, so it must be paid"
        );
    }

    #[test]
    fn resolution_is_paid_once_per_item_ever() {
        let mut model = Model::test_default();
        a_composed_day(&mut model);

        // The second visit: the same three names, no questions at all (A2r).
        update(&mut model, BuiltSessionEvent::OpenCompose);
        add(&mut model, "Zzz stride pattern, bars 1 to 8");
        add(&mut model, "Zzz freer rubato in the intro");
        let compose = view(&model).built.compose.expect("the sheet is open");
        assert!(compose.questions.is_empty(), "add, add, add, build");
        assert_eq!(compose.build_label, "Build session — no questions today");
        assert!(compose.can_build);
    }

    #[test]
    fn the_user_drill_form_reads_the_sentence_back_rather_than_asking_again() {
        let mut model = Model::test_default();
        update(&mut model, BuiltSessionEvent::OpenCompose);
        add(
            &mut model,
            "Zzz stride pattern, three clean passes at 72, in F and G",
        );
        let compose = view(&model).built.compose.expect("the sheet is open");
        match &compose.questions[0].ask {
            AskView::UserDrill {
                tempo_bpm,
                keys,
                passes_to_open,
                ..
            } => {
                assert_eq!(*tempo_bpm, Some(72));
                assert_eq!(keys, &["F", "G"]);
                assert_eq!(*passes_to_open, 3);
            }
            other => panic!("expected the criterion form, got {other:?}"),
        }
    }

    #[test]
    fn a_created_drill_carries_the_name_the_user_gave_it() {
        let mut model = Model::test_default();
        update(&mut model, BuiltSessionEvent::OpenCompose);
        let entry = add(&mut model, "Zzz stride pattern");
        update(
            &mut model,
            BuiltSessionEvent::ResolveAsUserDrill {
                entry_id: entry,
                criterion: "Five clean passes at 72".into(),
                serves: None,
            },
        );
        let drill = &model.user_drills[0];
        assert_eq!(
            drill.name, "Zzz stride pattern",
            "the name is what the next visit recognises"
        );
        assert_eq!(
            drill.passes_to_open, 5,
            "parsed from the sentence, not asked"
        );
        assert_eq!(drill.tempo_bpm, Some(72));
    }

    #[test]
    fn a_refused_criterion_leaves_the_question_open_rather_than_half_creating() {
        let mut model = Model::test_default();
        update(&mut model, BuiltSessionEvent::OpenCompose);
        let entry = add(&mut model, "Zzz stride pattern");
        let effects = update(
            &mut model,
            BuiltSessionEvent::ResolveAsUserDrill {
                entry_id: entry,
                criterion: "   ".into(),
                serves: None,
            },
        );
        assert!(model.user_drills.is_empty());
        assert!(persistence_ops(&effects).is_empty());
        assert!(model.last_error.is_some());
        assert_eq!(
            view(&model).built.compose.unwrap().questions.len(),
            1,
            "the entry still owes its answer"
        );
    }

    #[test]
    fn a_kind_can_be_changed_but_a_paid_resolution_cannot_be_unpaid() {
        let mut model = Model::test_default();
        update(&mut model, BuiltSessionEvent::OpenCompose);
        let entry = add(&mut model, "Zzz stride pattern");
        update(
            &mut model,
            BuiltSessionEvent::ChooseResolutionKind {
                entry_id: entry.clone(),
                kind: ResolutionKind::Journal,
            },
        );
        assert!(matches!(
            view(&model).built.compose.unwrap().questions[0].ask,
            AskView::Journal
        ));
        update(
            &mut model,
            BuiltSessionEvent::ResolveAsJournal {
                entry_id: entry.clone(),
                notes: None,
                linked_item_id: None,
            },
        );
        update(
            &mut model,
            BuiltSessionEvent::ChooseResolutionKind {
                entry_id: entry,
                kind: ResolutionKind::UserDrill,
            },
        );
        assert!(
            view(&model).built.compose.unwrap().questions.is_empty(),
            "re-asking a settled entry would un-pay a paid resolution"
        );
    }

    #[test]
    fn building_with_a_question_open_is_refused_and_says_so() {
        let mut model = Model::test_default();
        update(&mut model, BuiltSessionEvent::OpenCompose);
        add(&mut model, "Zzz stride pattern");
        let effects = update(&mut model, BuiltSessionEvent::BuildSession { source: None });
        assert!(model.built_sessions.is_empty());
        assert!(persistence_ops(&effects).is_empty());
        assert!(model.last_error.is_some());
    }

    #[test]
    fn building_clears_the_sheet_and_persists_the_session() {
        let mut model = Model::test_default();
        let id = a_composed_day(&mut model);
        assert!(model.compose.is_none(), "the sheet is done with");
        let session = model.today_built_session().expect("today's steer");
        assert_eq!(session.id, id);
        assert_eq!(session.blocks.len(), 3);
        assert_eq!(session.source.as_deref(), Some("From Friday's lesson"));
    }

    #[test]
    fn the_composed_session_names_its_source_and_offers_a_shape() {
        let mut model = Model::test_default();
        a_composed_day(&mut model);
        let composed = view(&model).built.session.expect("A6");
        assert_eq!(composed.source.as_deref(), Some("From Friday's lesson"));
        assert_eq!(
            composed.blocks.iter().map(|b| b.kind).collect::<Vec<_>>(),
            vec![
                ComposeKind::Piece,
                ComposeKind::Exercise,
                ComposeKind::Journal
            ],
            "the user's own order, untouched"
        );
        assert!(
            composed.shape_advice.is_some(),
            "a piece first is exactly what the shape has something to say about"
        );
        assert!(composed.total_minutes > 0);
    }

    #[test]
    fn the_shape_is_advice_and_applying_it_is_the_users_choice() {
        let mut model = Model::test_default();
        a_composed_day(&mut model);
        let effects = update(&mut model, BuiltSessionEvent::UseSuggestedShape);
        let composed = view(&model).built.session.expect("A6");
        assert_eq!(
            composed.blocks.iter().map(|b| b.kind).collect::<Vec<_>>(),
            vec![
                ComposeKind::Exercise,
                ComposeKind::Journal,
                ComposeKind::Piece
            ]
        );
        assert_eq!(
            composed.shape_advice, None,
            "advice that repeats the order on screen is noise"
        );
        assert_eq!(
            persistence_ops(&effects).len(),
            1,
            "the reorder is persisted"
        );
        assert_no_http(&effects);
    }

    #[test]
    fn starting_a_built_session_enters_the_canonical_drill_loop() {
        let mut model = Model::test_default();
        let id = a_composed_day(&mut model);
        update(
            &mut model,
            BuiltSessionEvent::StartBuiltSession {
                session_id: id,
                now: at(),
            },
        );
        let drill = view(&model).coach.drill.expect("a block is open");
        assert_eq!(drill.drill_title, "Alice in Wonderland");
        assert_eq!(drill.origin, BlockOrigin::Judgement);
        assert_eq!(drill.gate_summary, "your call");
        assert_eq!(drill.gate_question, "Done for now?");
    }

    #[test]
    fn a_user_drill_block_shows_where_its_evidence_lands() {
        let mut model = Model::test_default();
        let id = a_composed_day(&mut model);
        update(&mut model, BuiltSessionEvent::UseSuggestedShape);
        update(
            &mut model,
            BuiltSessionEvent::StartBuiltSession {
                session_id: id,
                now: at(),
            },
        );
        let drill = view(&model).coach.drill.expect("a block is open");
        assert_eq!(drill.origin, BlockOrigin::UserDrill);
        assert_eq!(
            drill.serves.as_deref(),
            Some("Adds to Alice in Wonderland"),
            "A7's serves line, written by the core"
        );
        assert_eq!(drill.gate_target, 3, "the sentence's three clean passes");
    }

    #[test]
    fn a_user_drills_taps_land_on_its_own_node_at_full_weight() {
        let mut model = Model::test_default();
        let id = a_composed_day(&mut model);
        update(&mut model, BuiltSessionEvent::UseSuggestedShape);
        update(
            &mut model,
            BuiltSessionEvent::StartBuiltSession {
                session_id: id,
                now: at(),
            },
        );
        let node = model.coach.session.spec().expect("a spec").node.clone();
        let level = model.coach.session.block().expect("a block").level;
        play_one_pass(&mut model);
        assert!(
            model.coach.mastery.reading(&node, level, at()).evidence > 0.0,
            "a drill the user wrote still counts, on its own track"
        );
    }

    #[test]
    fn a_judgement_blocks_taps_never_reach_the_mastery_track() {
        let mut model = Model::test_default();
        let id = a_composed_day(&mut model);
        update(
            &mut model,
            BuiltSessionEvent::StartBuiltSession {
                session_id: id,
                now: at(),
            },
        );
        let node = model.coach.session.spec().expect("a spec").node.clone();
        let level = model.coach.session.block().expect("a block").level;
        play_one_pass(&mut model);
        assert_eq!(
            model.coach.mastery.reading(&node, level, at()).evidence,
            0.0,
            "decision 17: qualitative data never feeds mastery"
        );
        assert_eq!(
            model.coach.session.block().expect("a block").attempts.len(),
            1,
            "the time and the tap are still logged — it is the inference that stops"
        );
    }

    #[test]
    fn a_running_session_is_not_something_a_steer_may_replace() {
        let mut model = Model::test_default();
        let id = a_composed_day(&mut model);
        update(
            &mut model,
            BuiltSessionEvent::StartBuiltSession {
                session_id: id.clone(),
                now: at(),
            },
        );
        let block_id = model.coach.session.block().expect("a block").id.clone();
        update(
            &mut model,
            BuiltSessionEvent::StartBuiltSession {
                session_id: id,
                now: at(),
            },
        );
        assert_eq!(
            model.coach.session.block().expect("a block").id,
            block_id,
            "the blocks in flight have evidence riding on them"
        );
    }

    #[test]
    fn starting_a_session_whose_targets_have_all_gone_says_so() {
        let mut model = Model::test_default();
        let id = a_composed_day(&mut model);
        model.user_drills.clear();
        model.journal_items.clear();
        model.items.clear();
        update(
            &mut model,
            BuiltSessionEvent::StartBuiltSession {
                session_id: id,
                now: at(),
            },
        );
        assert!(view(&model).coach.drill.is_none());
        assert!(model.last_error.is_some(), "never a silent nothing-happens");
    }

    /// Without this the composed hero never leaves, so Start re-adopts a
    /// finished session and credits the same drill's mastery twice.
    #[test]
    fn a_session_that_has_been_practised_stops_being_todays_steer() {
        let mut model = Model::test_default();
        let id = a_composed_day(&mut model);
        update(
            &mut model,
            BuiltSessionEvent::StartBuiltSession {
                session_id: id,
                now: at(),
            },
        );
        assert!(
            view(&model).built.session.is_some(),
            "it is still today's steer while the loop is running over it"
        );

        app_update(
            &mut model,
            Event::Coach(CoachEvent::CloseSession { now: at() }),
        );
        assert_eq!(model.built_session_today, None);
        assert!(
            view(&model).built.session.is_none(),
            "the prescribed hero comes back once the session is over"
        );
    }

    /// The launch prompt's Discard and a session ending both clear the blob, so
    /// sharing one `SnapshotAction` would retire a composition the user has not
    /// practised — they declined a crash from before it was made (#1193).
    #[test]
    fn declining_a_crashed_session_leaves_todays_composition_alone() {
        let mut model = Model::test_default();
        let id = a_composed_day(&mut model);
        let mut crashed = crate::engine::CoachState::default();
        crashed.apply(&CoachEvent::GoUnmonitored { now: at() });

        app_update(
            &mut model,
            Event::Coach(CoachEvent::OfferRecovery {
                session: crashed.session,
            }),
        );
        app_update(&mut model, Event::Coach(CoachEvent::DeclineRecovery));

        assert_eq!(model.built_session_today, Some(id));
        assert!(
            view(&model).built.session.is_some(),
            "the composed hero is still today's steer — nothing was practised"
        );
    }

    #[test]
    fn cancelling_the_sheet_leaves_the_prescribed_day_untouched() {
        let mut model = Model::test_default();
        update(&mut model, BuiltSessionEvent::OpenCompose);
        add(&mut model, "Zzz stride pattern");
        update(&mut model, BuiltSessionEvent::CancelCompose);
        assert!(view(&model).built.compose.is_none());
        assert!(model.built_sessions.is_empty());
    }

    #[test]
    fn the_same_thing_added_twice_is_one_block() {
        let mut model = Model::test_default();
        model
            .items
            .push(sample_item("p1", "Alice in Wonderland", ItemKind::Piece));
        update(&mut model, BuiltSessionEvent::OpenCompose);
        add(&mut model, "Alice in Wonderland");
        add(&mut model, "alice in wonderland");
        assert_eq!(model.compose.as_ref().unwrap().entries.len(), 1);
    }

    #[test]
    fn the_whole_journey_never_touches_the_network() {
        let mut model = Model::test_default();
        model
            .items
            .push(sample_item("p1", "Alice in Wonderland", ItemKind::Piece));
        update(&mut model, BuiltSessionEvent::OpenCompose);
        let entry = add(&mut model, "Zzz stride pattern");
        let mut effects = update(
            &mut model,
            BuiltSessionEvent::ResolveAsUserDrill {
                entry_id: entry,
                criterion: "Three clean passes at 72".into(),
                serves: None,
            },
        );
        effects.extend(update(
            &mut model,
            BuiltSessionEvent::BuildSession { source: None },
        ));
        let id = model.built_session_today.clone().expect("a built session");
        effects.extend(update(
            &mut model,
            BuiltSessionEvent::StartBuiltSession {
                session_id: id,
                now: at(),
            },
        ));
        assert_no_http(&effects);
    }

    // ── The altitudes, end to end (Journey B) ───────────────────────────

    fn charted_piece(id: &str) -> crate::domain::item::Item {
        let mut item = sample_item(id, "Alice in Wonderland", ItemKind::Piece);
        item.chord_chart = Some(
            crate::domain::chart::parse_chart(
                "[A]\n| Cmaj7 | Am7 |\n[Bridge]\n| Dm7 | G7 |",
                "C",
                crate::domain::item::Modality::Major,
            )
            .expect("a valid chart"),
        );
        item
    }

    fn start_altitude(model: &mut Model, item_id: &str, altitude: Altitude) -> Vec<Effect> {
        update(
            model,
            BuiltSessionEvent::StartPlayThrough {
                item_id: item_id.into(),
                altitude,
                now: at(),
            },
        )
    }

    #[test]
    fn b0_asks_the_core_what_the_piece_may_be_played_at() {
        let mut model = Model::test_default();
        model.items.push(charted_piece("p1"));
        update(
            &mut model,
            BuiltSessionEvent::OpenPlayThrough {
                item_id: "p1".into(),
            },
        );

        let offer = view(&model).built.play_through.expect("the sheet is open");
        assert_eq!(offer.title, "Alice in Wonderland");
        assert!(offer.run_through_available);
        assert_eq!(offer.sections, vec!["A", "Bridge"]);
    }

    /// The sheet holds an id, not a snapshot: editing the chart behind it must
    /// not leave B0 offering a run-through the piece can no longer support.
    #[test]
    fn the_b0_offer_follows_the_chart_rather_than_the_moment_it_opened() {
        let mut model = Model::test_default();
        model
            .items
            .push(sample_item("p1", "Alice in Wonderland", ItemKind::Piece));
        update(
            &mut model,
            BuiltSessionEvent::OpenPlayThrough {
                item_id: "p1".into(),
            },
        );
        assert!(
            !view(&model)
                .built
                .play_through
                .expect("open")
                .run_through_available,
            "nothing to gate on yet"
        );

        model.items[0].chord_chart = charted_piece("p1").chord_chart;

        assert!(
            view(&model)
                .built
                .play_through
                .expect("still open")
                .run_through_available,
            "the offer is re-derived, so the chart the user just wrote counts"
        );
    }

    /// The sheet has no offer to draw either way, so a refusal that said
    /// nothing would leave the button dead under the finger (#846).
    #[test]
    fn b0_refuses_to_open_on_something_that_cannot_be_played_through() {
        let mut model = Model::test_default();
        model.items.push(charted_piece("p1"));
        model
            .items
            .push(sample_item("e1", "Two-fives", ItemKind::Exercise));

        for id in ["gone", "e1"] {
            model.last_error = None;
            update(
                &mut model,
                BuiltSessionEvent::OpenPlayThrough { item_id: id.into() },
            );

            assert!(view(&model).built.play_through.is_none(), "{id}");
            assert!(
                model.last_error.is_some(),
                "{id}: an exercise is drilled, not played through — and a deleted piece is gone"
            );
        }
    }

    #[test]
    fn starting_a_run_closes_the_sheet_that_asked_for_it() {
        let mut model = Model::test_default();
        model.items.push(charted_piece("p1"));
        update(
            &mut model,
            BuiltSessionEvent::OpenPlayThrough {
                item_id: "p1".into(),
            },
        );
        start_altitude(&mut model, "p1", Altitude::RunThrough);

        assert!(
            view(&model).built.play_through.is_none(),
            "a sheet left open over the run it started would cover the first section"
        );
    }

    #[test]
    fn closing_b0_leaves_nothing_running() {
        let mut model = Model::test_default();
        model.items.push(charted_piece("p1"));
        update(
            &mut model,
            BuiltSessionEvent::OpenPlayThrough {
                item_id: "p1".into(),
            },
        );
        update(&mut model, BuiltSessionEvent::ClosePlayThrough);

        assert!(view(&model).built.play_through.is_none());
    }

    #[test]
    fn b0_refused_mid_session_says_so_rather_than_closing_on_nothing() {
        let mut model = Model::test_default();
        model.items.push(charted_piece("p1"));
        start_altitude(&mut model, "p1", Altitude::OffPiste);
        model.last_error = None;

        update(
            &mut model,
            BuiltSessionEvent::OpenPlayThrough {
                item_id: "p1".into(),
            },
        );
        start_altitude(&mut model, "p1", Altitude::RunThrough);

        assert_eq!(
            view(&model).coach.altitude,
            Some(Altitude::OffPiste),
            "the session already in flight is not replaced"
        );
        assert!(model.last_error.is_some(), "the refusal has a surface");
        assert!(
            view(&model).built.play_through.is_some(),
            "the sheet stays up, because the user still has a choice to make"
        );
    }

    /// Decision 7 at the view layer: off-piste has a record to tag, and so a
    /// piece to name; unmonitored has neither.
    #[test]
    fn off_piste_names_the_piece_and_unmonitored_never_does() {
        let mut model = Model::test_default();
        model.items.push(charted_piece("p1"));
        start_altitude(&mut model, "p1", Altitude::OffPiste);

        let open = view(&model).coach.open_play.expect("off-piste is drawable");
        assert_eq!(open.altitude, Altitude::OffPiste);
        assert_eq!(open.title.as_deref(), Some("Alice in Wonderland"));
        assert_eq!(open.started_at, at(), "the clock the screen counts from");

        let mut model = Model::test_default();
        model.items.push(charted_piece("p1"));
        start_altitude(&mut model, "p1", Altitude::Unmonitored);

        let open = view(&model)
            .coach
            .open_play
            .expect("unmonitored is drawable");
        assert_eq!(open.altitude, Altitude::Unmonitored);
        assert_eq!(open.item_id, None);
        assert_eq!(
            open.title, None,
            "minutes only: the screen must not be able to say what was played"
        );
    }

    #[test]
    fn a_gated_run_draws_no_open_play_clock() {
        let mut model = Model::test_default();
        model.items.push(charted_piece("p1"));
        start_altitude(&mut model, "p1", Altitude::RunThrough);

        assert!(
            view(&model).coach.open_play.is_none(),
            "the run-through has its own view; two clocks would be two answers"
        );
    }

    #[test]
    fn b0_runs_a_charted_piece_through_its_own_sections() {
        let mut model = Model::test_default();
        model.items.push(charted_piece("p1"));
        let effects = start_altitude(&mut model, "p1", Altitude::RunThrough);

        let run = view(&model)
            .coach
            .run_through
            .expect("the run-through screen has something to draw");
        assert_eq!(run.sections, vec!["A", "Bridge"]);
        assert_eq!(run.current_section.as_deref(), Some("A"));
        assert_eq!(run.title, "Alice in Wonderland");
        assert_eq!(view(&model).coach.altitude, Some(Altitude::RunThrough));
        assert_no_http(&effects);
    }

    #[test]
    fn a_piece_with_no_named_sections_is_refused_rather_than_run_on_nothing() {
        let mut model = Model::test_default();
        model
            .items
            .push(sample_item("p1", "Untitled", ItemKind::Piece));
        start_altitude(&mut model, "p1", Altitude::RunThrough);

        assert!(view(&model).coach.run_through.is_none());
        assert!(
            model.last_error.is_some(),
            "a refusal with no surface is the silent-no-op bug (#846)"
        );
    }

    /// #1285: the minutes were computed on close and dropped, because nothing
    /// in the batch carried them and `recorded_at` therefore stayed `None`.
    #[test]
    fn unmonitored_minutes_reach_the_store() {
        let mut model = Model::test_default();
        model
            .items
            .push(sample_item("p1", "Untitled", ItemKind::Piece));
        start_altitude(&mut model, "p1", Altitude::Unmonitored);

        let closed_at = at() + chrono::TimeDelta::minutes(10);
        let effects = app_update(
            &mut model,
            Event::Coach(CoachEvent::CloseSession { now: closed_at }),
        );

        let (unmonitored, updated_at) = persistence_ops(&effects)
            .into_iter()
            .find_map(|op| match op {
                PersistenceOperation::SaveCoachRecords {
                    unmonitored,
                    updated_at,
                    ..
                } => Some((unmonitored, updated_at)),
                _ => None,
            })
            .expect("a SaveCoachRecords op carrying the minutes");
        assert_eq!(unmonitored.len(), 1);
        assert_eq!(unmonitored[0].started_at, at());
        assert_eq!(unmonitored[0].ended_at, closed_at);
        assert_eq!(updated_at, closed_at, "core-stamped, not shell-formatted");
        assert_no_http(&effects);
    }

    #[test]
    fn the_lower_altitudes_need_nothing_authored() {
        let mut model = Model::test_default();
        model
            .items
            .push(sample_item("p1", "Untitled", ItemKind::Piece));

        start_altitude(&mut model, "p1", Altitude::OffPiste);
        assert_eq!(view(&model).coach.altitude, Some(Altitude::OffPiste));
        assert!(model.last_error.is_none());
    }

    #[test]
    fn a_finished_run_lands_its_verdicts_and_its_record() {
        let mut model = Model::test_default();
        model.items.push(charted_piece("p1"));
        start_altitude(&mut model, "p1", Altitude::RunThrough);

        for held in [true, false] {
            app_update(
                &mut model,
                Event::Coach(CoachEvent::JudgeSection { held, now: at() }),
            );
        }
        let effects = app_update(
            &mut model,
            Event::Coach(CoachEvent::CloseSession { now: at() }),
        );

        assert_eq!(model.play_throughs.len(), 1, "the record is the user's");
        assert_eq!(model.play_throughs[0].sections.len(), 2);
        assert!(
            matches!(
                persistence_ops(&effects).first(),
                Some(PersistenceOperation::SaveCoachRecords { play_throughs, .. })
                    if play_throughs.len() == 1
            ),
            "a run rides the coach batch, so a failed write is offered again"
        );
        assert_no_http(&effects);

        let held = model
            .coach
            .mastery
            .get(
                "piece:p1#A",
                crate::engine::ParameterLevel {
                    tempo_bpm: 0,
                    click_level: crate::engine::ClickLevel::NoClick,
                },
            )
            .expect("the section that held has evidence");
        assert!(held.evidence() > 0.0);
    }

    #[test]
    fn the_launch_replay_rebuilds_a_run_throughs_evidence_too() {
        let mut model = Model::test_default();
        model.items.push(charted_piece("p1"));
        start_altitude(&mut model, "p1", Altitude::RunThrough);
        for held in [true, true] {
            app_update(
                &mut model,
                Event::Coach(CoachEvent::JudgeSection { held, now: at() }),
            );
        }
        app_update(
            &mut model,
            Event::Coach(CoachEvent::CloseSession { now: at() }),
        );
        let live = model.coach.mastery.clone();

        // Relaunch: the two loads race, and the run-throughs are on the second.
        let mut relaunched = Model::test_default();
        app_update(
            &mut relaunched,
            Event::CoachRecordsLoaded(PersistenceOutput::CoachRecords(vec![])),
        );
        app_update(
            &mut relaunched,
            Event::BuiltStoreLoaded(PersistenceOutput::BuiltSessionData(BuiltSessionData {
                user_drills: vec![],
                journal_items: vec![],
                built_sessions: vec![],
                play_throughs: model.play_throughs.clone(),
                reflections: vec![],
                feel_entries: vec![],
            })),
        );

        assert_eq!(
            relaunched.coach.mastery, live,
            "a rule the live path enforces and the replay does not is not a rule (#1214)"
        );
    }

    #[test]
    fn an_uncounted_run_rebuilds_to_nothing() {
        let mut model = Model::test_default();
        model.items.push(charted_piece("p1"));
        start_altitude(&mut model, "p1", Altitude::RunThrough);
        app_update(
            &mut model,
            Event::Coach(CoachEvent::JudgeSection {
                held: true,
                now: at(),
            }),
        );
        app_update(
            &mut model,
            Event::Coach(CoachEvent::DiscardRunThrough { now: at() }),
        );

        let mut relaunched = Model::test_default();
        app_update(
            &mut relaunched,
            Event::BuiltStoreLoaded(PersistenceOutput::BuiltSessionData(BuiltSessionData {
                user_drills: vec![],
                journal_items: vec![],
                built_sessions: vec![],
                play_throughs: model.play_throughs.clone(),
                reflections: vec![],
                feel_entries: vec![],
            })),
        );
        assert_eq!(
            relaunched.coach.mastery,
            Model::test_default().coach.mastery,
            "\"don't count this run\" has to mean it at launch as well as live"
        );
    }

    #[test]
    fn a_reload_mid_session_never_undoes_what_has_already_been_practised() {
        // Both legs of the rebuild in one session — a closed block and a closed
        // run-through — because a failed built-session write reissues the load,
        // so `BuiltStoreLoaded` is reachable mid-session, not only at launch.
        let mut model = Model::test_default();
        let id = a_composed_day(&mut model);
        // The shape puts the user drill first, which is the block whose taps
        // are evidence — a judgement block's would prove nothing here.
        update(&mut model, BuiltSessionEvent::UseSuggestedShape);
        update(
            &mut model,
            BuiltSessionEvent::StartBuiltSession {
                session_id: id,
                now: at(),
            },
        );
        let node = model.coach.session.spec().expect("a spec").node.clone();
        let level = model.coach.session.block().expect("a block").level;
        for _ in 0..3 {
            play_one_pass(&mut model);
        }
        // The gate opens on the third pass; the hold is what closes the block.
        app_update(
            &mut model,
            Event::Coach(CoachEvent::Tick {
                now: at() + chrono::TimeDelta::seconds(600),
            }),
        );
        assert!(
            !model.coach_blocks.is_empty(),
            "the gate opened and the block closed, so there is a record to lose"
        );
        let practised = model.coach.mastery.reading(&node, level, at()).evidence;
        assert!(practised > 0.0, "it is in the live track before the reload");

        let reloaded = BuiltSessionData {
            user_drills: model.user_drills.clone(),
            journal_items: model.journal_items.clone(),
            built_sessions: model.built_sessions.clone(),
            play_throughs: model.play_throughs.clone(),
            reflections: vec![],
            feel_entries: vec![],
        };
        app_update(
            &mut model,
            Event::BuiltStoreLoaded(PersistenceOutput::BuiltSessionData(reloaded)),
        );
        assert_eq!(
            model.coach.mastery.reading(&node, level, at()).evidence,
            practised,
            "a rebuild replays everything the live track was fed, not the launch snapshot"
        );
    }

    #[test]
    fn starting_an_altitude_on_a_piece_that_has_gone_says_so() {
        let mut model = Model::test_default();
        start_altitude(&mut model, "gone", Altitude::RunThrough);
        assert!(model.last_error.is_some());
        assert!(view(&model).coach.altitude.is_none());
    }

    // ── Bridge payloads (#846) ──────────────────────────────────────────

    fn sample_journal_item() -> JournalItem {
        JournalItem {
            id: "01J000000000000000000JOUR".into(),
            name: "Rubato feel".into(),
            notes: Some("Time and notes".into()),
            linked_item_id: Some("piece".into()),
            created_at: at(),
            updated_at: at(),
            deleted_at: None,
        }
    }

    fn sample_play_through() -> PlayThroughRecord {
        PlayThroughRecord {
            id: "01J0000000000000000000PLAY".into(),
            item_id: "piece".into(),
            started_at: at(),
            ended_at: at(),
            counted: true,
            sections: vec![SectionVerdict {
                section: "The bridge".into(),
                held: true,
                at: at(),
            }],
            updated_at: at(),
            deleted_at: None,
        }
    }

    fn sample_reflection() -> Reflection {
        Reflection {
            id: "01J000000000000000000REFL".into(),
            kind: ReflectionKind::SessionClose,
            session_ref: Some("built".into()),
            transcript: Some("The bridge still rushes".into()),
            audio_path: Some("reflections/r1.m4a".into()),
            duration_s: Some(24),
            at: at(),
            steer: SteerState::Unoffered,
            steer_at: None,
            updated_at: at(),
            deleted_at: None,
        }
    }

    fn sample_feel_entry() -> FeelEntry {
        FeelEntry {
            id: "01J000000000000000000FEEL".into(),
            block_id: "b1".into(),
            feel: Feel::ItSang,
            at: at(),
            updated_at: at(),
            deleted_at: None,
        }
    }

    /// Every variant, not a sample: a positional bincode wire has no "absent",
    /// so an untested variant is an untested field order (#846).
    #[test]
    fn every_built_session_event_round_trips_on_the_wire() {
        for event in [
            BuiltSessionEvent::CreateUserDrill(sample_create_drill()),
            BuiltSessionEvent::UpdateUserDrill {
                drill: sample_drill(),
            },
            BuiltSessionEvent::DeleteUserDrill { id: "d1".into() },
            BuiltSessionEvent::CreateJournalItem(CreateJournalItem {
                name: "Rubato feel".into(),
                notes: Some("Time and notes".into()),
                linked_item_id: Some("piece".into()),
            }),
            BuiltSessionEvent::UpdateJournalItem {
                journal: sample_journal_item(),
            },
            BuiltSessionEvent::DeleteJournalItem { id: "j1".into() },
            BuiltSessionEvent::SaveBuiltSession {
                session: sample_built_session(),
            },
            BuiltSessionEvent::RecordReflection {
                kind: ReflectionKind::VoiceNote,
                session_ref: Some("built".into()),
                transcript: Some("The bridge still rushes".into()),
                audio_path: Some("reflections/r1.m4a".into()),
                duration_s: Some(24),
            },
            BuiltSessionEvent::RecordFeel {
                block_id: "b1".into(),
                feel: Feel::GettingThere,
            },
            BuiltSessionEvent::OpenCompose,
            BuiltSessionEvent::CancelCompose,
            BuiltSessionEvent::AddComposeEntry {
                text: "Stride pattern".into(),
                picked_item_id: Some("p1".into()),
            },
            BuiltSessionEvent::RemoveComposeEntry {
                entry_id: "e1".into(),
            },
            BuiltSessionEvent::ConfirmNodeMatch {
                entry_id: "e1".into(),
            },
            BuiltSessionEvent::ChooseResolutionKind {
                entry_id: "e1".into(),
                kind: ResolutionKind::Journal,
            },
            BuiltSessionEvent::ResolveAsUserDrill {
                entry_id: "e1".into(),
                criterion: "Three clean passes at 72".into(),
                serves: Some(Serves::Circle(Circle::Bridge)),
            },
            BuiltSessionEvent::ResolveAsJournal {
                entry_id: "e1".into(),
                notes: Some("Time and notes".into()),
                linked_item_id: Some("p1".into()),
            },
            BuiltSessionEvent::BuildSession {
                source: Some("From Friday's lesson".into()),
            },
            BuiltSessionEvent::UseSuggestedShape,
            BuiltSessionEvent::StartBuiltSession {
                session_id: "01J000000000000000000BSESH".into(),
                now: at(),
            },
            BuiltSessionEvent::StartPlayThrough {
                item_id: "01J000000000000000000PIECE".into(),
                altitude: Altitude::RunThrough,
                now: at(),
            },
            BuiltSessionEvent::OpenPlayThrough {
                item_id: "01J000000000000000000PIECE".into(),
            },
            BuiltSessionEvent::ClosePlayThrough,
        ] {
            assert_round_trips(Event::BuiltSession(event));
        }
    }

    /// A stub bridge cannot catch a bincode break here (#846).
    #[test]
    fn journey_bs_view_payloads_cross_the_wire() {
        assert_round_trips(playthrough::AltitudeOffer {
            item_id: "01J000000000000000000PIECE".into(),
            title: "Alice in Wonderland".into(),
            run_through_available: true,
            sections: vec!["A".into(), "Bridge".into()],
        });
        for (altitude, item_id, title) in [
            (
                Altitude::OffPiste,
                Some("01J000000000000000000PIECE".to_string()),
                Some("Alice in Wonderland".to_string()),
            ),
            (Altitude::Unmonitored, None, None),
        ] {
            assert_round_trips(crate::engine::OpenPlayView {
                altitude,
                item_id,
                title,
                started_at: at(),
            });
        }
    }

    #[test]
    fn every_altitude_crosses_the_wire() {
        for altitude in [
            Altitude::RunThrough,
            Altitude::OffPiste,
            Altitude::Unmonitored,
        ] {
            assert_round_trips(altitude);
        }
    }

    #[test]
    fn built_session_persistence_payloads_round_trip_on_the_wire() {
        assert_round_trips(PersistenceOperation::SaveUserDrill(sample_drill()));
        assert_round_trips(PersistenceOperation::SaveJournalItem(sample_journal_item()));
        assert_round_trips(PersistenceOperation::SaveBuiltSession(
            sample_built_session(),
        ));
        assert_round_trips(PersistenceOperation::SaveReflection(sample_reflection()));
        assert_round_trips(PersistenceOperation::SaveFeelEntry(sample_feel_entry()));
        assert_round_trips(PersistenceOperation::LoadBuiltSessionData);
        assert_round_trips(PersistenceOutput::BuiltSessionData(BuiltSessionData {
            user_drills: vec![sample_drill()],
            journal_items: vec![sample_journal_item()],
            built_sessions: vec![sample_built_session()],
            play_throughs: vec![sample_play_through()],
            reflections: vec![sample_reflection()],
            feel_entries: vec![sample_feel_entry()],
        }));
    }

    // ── Qualitative capture (Phase D handlers) ────────────────────────

    #[test]
    fn a_feel_is_recorded_once_per_block_and_never_twice() {
        let mut model = Model::test_default();
        model.feel_prompt = Some(capture::FeelPrompt {
            block_id: "b1".into(),
            title: "Rubato in the intro".into(),
        });
        let effects = update(
            &mut model,
            BuiltSessionEvent::RecordFeel {
                block_id: "b1".into(),
                feel: Feel::ItSang,
            },
        );
        assert_eq!(model.feel_entries.len(), 1);
        assert!(model.feel_prompt.is_none(), "the question is answered");
        assert_no_http(&effects);

        let effects = update(
            &mut model,
            BuiltSessionEvent::RecordFeel {
                block_id: "b1".into(),
                feel: Feel::FoughtIt,
            },
        );
        assert_eq!(model.feel_entries.len(), 1, "one per block, ever");
        assert_eq!(model.feel_entries[0].feel, Feel::ItSang);
        assert!(persistence_ops(&effects).is_empty());
    }

    #[test]
    fn skipping_the_feel_writes_nothing_at_all() {
        let mut model = Model::test_default();
        model.feel_prompt = Some(capture::FeelPrompt {
            block_id: "b1".into(),
            title: "Rubato in the intro".into(),
        });
        let effects = update(&mut model, BuiltSessionEvent::SkipFeel);
        assert!(model.feel_prompt.is_none());
        assert!(model.feel_entries.is_empty(), "a skip is not a fourth feel");
        assert!(persistence_ops(&effects).is_empty());
    }

    #[test]
    fn not_tonight_closes_the_reflection_without_a_row() {
        let mut model = Model::test_default();
        model.reflection_prompt = true;
        let effects = update(&mut model, BuiltSessionEvent::DismissReflection);
        assert!(!model.reflection_prompt);
        assert!(model.reflections.is_empty());
        assert!(persistence_ops(&effects).is_empty());
    }

    #[test]
    fn a_kept_reflection_persists_and_closes_the_question() {
        let mut model = Model::test_default();
        model.reflection_prompt = true;
        let effects = update(
            &mut model,
            BuiltSessionEvent::RecordReflection {
                kind: ReflectionKind::SessionClose,
                session_ref: Some("built".into()),
                transcript: None,
                audio_path: Some("reflections/r1.m4a".into()),
                duration_s: Some(24),
            },
        );
        assert!(!model.reflection_prompt);
        assert_eq!(model.reflections.len(), 1);
        let kept = &model.reflections[0];
        assert_eq!(kept.id.len(), 26, "client-minted ulid (invariant 3)");
        assert_eq!(kept.steer, SteerState::Unoffered);
        assert_eq!(
            persistence_ops(&effects),
            vec![PersistenceOperation::SaveReflection(kept.clone())]
        );
        assert_no_http(&effects);
    }

    #[test]
    fn a_transcript_that_lands_late_reaches_the_reflection_it_belongs_to() {
        let mut model = Model::test_default();
        update(
            &mut model,
            BuiltSessionEvent::RecordReflection {
                kind: ReflectionKind::SessionClose,
                session_ref: None,
                transcript: None,
                audio_path: Some("reflections/r1.m4a".into()),
                duration_s: Some(24),
            },
        );
        let id = model.reflections[0].id.clone();
        let effects = update(
            &mut model,
            BuiltSessionEvent::AttachTranscript {
                reflection_id: id.clone(),
                transcript: "The bridge still rushes.".into(),
            },
        );
        assert_eq!(
            model.reflections[0].transcript.as_deref(),
            Some("The bridge still rushes.")
        );
        assert_eq!(
            persistence_ops(&effects),
            vec![PersistenceOperation::SaveReflection(
                model.reflections[0].clone()
            )]
        );
        assert_eq!(model.reflections[0].id, id, "the audio keeps its row");
    }

    /// Carries the library the sample transcript names: without it, #1317.
    fn model_with_unanswered_steer() -> (Model, String) {
        let mut model = Model::test_default();
        model.journal_items.push(JournalItem {
            id: "01J0000000000000000000JRNL".into(),
            name: "The bridge".into(),
            notes: None,
            linked_item_id: None,
            created_at: at(),
            updated_at: at(),
            deleted_at: None,
        });
        let reflection = sample_reflection();
        let id = reflection.id.clone();
        model.reflections.push(reflection);
        (model, id)
    }

    #[test]
    fn accepting_the_morning_steer_spends_the_offer_and_persists_it() {
        let (mut model, id) = model_with_unanswered_steer();
        let effects = update(
            &mut model,
            BuiltSessionEvent::AcceptProposedSteer {
                reflection_id: id.clone(),
            },
        );
        let answered = &model.reflections[0];
        assert_eq!(answered.steer, SteerState::Accepted);
        assert!(answered.steer_at.is_some());
        assert_eq!(
            persistence_ops(&effects),
            vec![PersistenceOperation::SaveReflection(answered.clone())]
        );
        assert_no_http(&effects);
        assert_eq!(answered.id, id);
    }

    /// The `steer` column is what stops the card coming back, so persisting it
    /// on a refusal would spend the offer on a block nobody added (#1317).
    #[test]
    fn accepting_with_nothing_to_place_persists_nothing_and_leaves_the_offer_open() {
        let (mut model, id) = model_with_unanswered_steer();
        model.journal_items.clear();

        let effects = update(
            &mut model,
            BuiltSessionEvent::AcceptProposedSteer {
                reflection_id: id.clone(),
            },
        );

        assert_eq!(model.reflections[0].steer, SteerState::Unoffered);
        assert!(model.reflections[0].steer_at.is_none());
        assert!(persistence_ops(&effects).is_empty());
        assert_eq!(
            model.last_error.as_deref(),
            Some("That doesn't match anything in your library.")
        );
    }

    #[test]
    fn declining_leaves_the_reflection_but_spends_the_offer() {
        let (mut model, id) = model_with_unanswered_steer();
        update(
            &mut model,
            BuiltSessionEvent::DeclineProposedSteer {
                reflection_id: id.clone(),
            },
        );
        assert_eq!(model.reflections.len(), 1, "the reflection itself is kept");
        assert_eq!(model.reflections[0].steer, SteerState::Declined);

        // Changing the answer would move `steer_at` and quietly extend how long
        // the block rides today's plan.
        let declined_at = model.reflections[0].steer_at;
        let effects = update(
            &mut model,
            BuiltSessionEvent::AcceptProposedSteer { reflection_id: id },
        );
        assert_eq!(model.reflections[0].steer, SteerState::Declined);
        assert_eq!(model.reflections[0].steer_at, declined_at);
        assert!(persistence_ops(&effects).is_empty());
    }

    #[test]
    fn feel_prompt_round_trips_on_the_wire() {
        assert_round_trips(capture::FeelPrompt {
            block_id: "01J000000000000000000BREC".into(),
            title: "Rubato in the intro".into(),
        });
    }

    #[test]
    fn the_proposed_steer_round_trips_on_the_wire() {
        assert_round_trips(steer::ProposedSteer {
            reflection_id: "01J000000000000000000REFL".into(),
            quote: "The bridge still rushes when I go from memory.".into(),
            offer: "Give Alice in Wonderland, bridge 8 minutes today?".into(),
            minutes: 8,
        });
    }
}
