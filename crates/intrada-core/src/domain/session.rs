use crate::app::{AppEffect, Effect, Event};
use crate::domain::item::{Item, ItemKind};
use crate::error::LibraryError;
use crate::model::Model;
use crate::validation;
use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};

// ── Enums ──────────────────────────────────────────────────────────────

/// Completion status of a single setlist entry.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum EntryStatus {
    /// Item was practised and time was recorded.
    Completed,
    /// Item was explicitly skipped (duration_secs = 0).
    Skipped,
    /// Session ended early before reaching this item (duration_secs = 0).
    NotAttempted,
}

/// Whether the session ran to completion or was ended early.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum CompletionStatus {
    /// All items in the setlist were addressed (completed or skipped).
    Completed,
    /// Session was ended before all items were reached.
    EndedEarly,
}

/// A single action in the rep history sequence.
///
/// Values are deltas: `1` = count + 1 (success), `-1` = count − 1 (missed).
/// Enables analytics: sum for net progress, running total for sparkline charts,
/// count of `-1`s for total misses, longest streak of `1`s for best run.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum RepAction {
    /// Failed rep — count decremented.
    Missed,
    /// Successful rep — count incremented.
    Success,
}

// ── Domain Types ───────────────────────────────────────────────────────

/// An individual item within a session's setlist.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SetlistEntry {
    pub id: String,
    pub item_id: String,
    pub item_title: String,
    pub item_type: ItemKind,
    pub position: usize,
    pub duration_secs: u64,
    pub status: EntryStatus,
    pub notes: Option<String>,
    #[serde(default)]
    pub score: Option<u8>,
    #[serde(default)]
    pub intention: Option<String>,
    #[serde(default)]
    pub rep_target: Option<u8>,
    #[serde(default)]
    pub rep_count: Option<u8>,
    #[serde(default)]
    pub rep_target_reached: Option<bool>,
    #[serde(default)]
    pub rep_history: Option<Vec<RepAction>>,
    #[serde(default)]
    pub planned_duration_secs: Option<u32>,
    #[serde(default)]
    pub achieved_tempo: Option<u16>,
    /// Block grouping (building phase): entries pulled in alongside a piece via
    /// its related exercises share one `group_id`. A block is the contiguous run
    /// of entries with the same id; `None` = standalone.
    #[serde(default)]
    pub group_id: Option<String>,
}

/// A completed practice session (persisted to localStorage).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PracticeSession {
    pub id: String,
    pub entries: Vec<SetlistEntry>,
    pub session_notes: Option<String>,
    #[serde(default)]
    pub session_intention: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub total_duration_secs: u64,
    pub completion_status: CompletionStatus,
    #[serde(default)]
    pub session_score: Option<u8>,
    #[serde(default)]
    pub reflection_improved: Option<String>,
    #[serde(default)]
    pub reflection_still_rough: Option<String>,
    #[serde(default)]
    pub reflection_next_target: Option<String>,
}

/// Which of the three structured end-of-session reflection prompts an
/// `UpdateSessionReflection` targets (design-principles T7).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum ReflectionField {
    Improved,
    StillRough,
    NextTarget,
}

// ── Transient State Types ──────────────────────────────────────────────

/// State during setlist assembly (Building phase).
#[derive(Debug, Clone, Default)]
pub struct BuildingSession {
    pub entries: Vec<SetlistEntry>,
    pub session_intention: Option<String>,
    /// Optional session-level time target (in minutes) set via presets.
    /// Purely a UI guide — not enforced.
    pub target_duration_mins: Option<u32>,
    /// Which saved Set this builder was loaded from (if any).
    pub source_set_id: Option<String>,
    /// Ordered item_ids at load time — used to detect modifications.
    pub source_set_entry_snapshot: Vec<String>,
}

/// State during active practice (Active phase).
/// Persisted to `intrada:session-in-progress` for crash recovery.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ActiveSession {
    pub id: String,
    pub entries: Vec<SetlistEntry>,
    pub current_index: usize,
    pub current_item_started_at: DateTime<Utc>,
    pub session_started_at: DateTime<Utc>,
    #[serde(default)]
    pub session_intention: Option<String>,
}

/// State during post-session review (Summary phase).
#[derive(Debug, Clone)]
pub struct SummarySession {
    pub id: String,
    pub entries: Vec<SetlistEntry>,
    pub session_started_at: DateTime<Utc>,
    pub session_ended_at: DateTime<Utc>,
    pub session_notes: Option<String>,
    pub session_intention: Option<String>,
    pub completion_status: CompletionStatus,
    pub session_score: Option<u8>,
    pub reflection_improved: Option<String>,
    pub reflection_still_rough: Option<String>,
    pub reflection_next_target: Option<String>,
}

/// The lifecycle state of a session in the core Model.
#[derive(Debug, Clone, Default)]
pub enum SessionStatus {
    #[default]
    Idle,
    Building(BuildingSession),
    Active(ActiveSession),
    Summary(SummarySession),
}

// ── Events ─────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum SessionEvent {
    // === Building Phase ===
    StartBuilding,
    /// Start building with a session-level time target (in minutes).
    /// The target is a UI guide, not enforced.
    StartBuildingWithTarget {
        target_duration_mins: u32,
    },
    SetSessionIntention {
        intention: Option<String>,
    },
    SetEntryIntention {
        entry_id: String,
        intention: Option<String>,
    },
    /// Set or clear the rep target for an entry during building phase.
    /// `None` disables the counter; `Some(n)` enables it with target `n`.
    SetRepTarget {
        entry_id: String,
        target: Option<u8>,
    },
    /// Set or clear the planned duration for an entry during building phase.
    /// `None` clears the planned duration; `Some(secs)` sets it (range: 60–3600).
    SetEntryDuration {
        entry_id: String,
        duration_secs: Option<u32>,
    },
    AddToSetlist {
        item_id: String,
    },
    /// One-tap "Practise this": from Idle, start building seeded with the
    /// item (a piece brings its related exercises, as `AddToSetlist`).
    StartBuildingWith {
        item_id: String,
    },
    AddNewItemToSetlist {
        title: String,
        item_type: ItemKind,
    },
    RemoveFromSetlist {
        entry_id: String,
    },
    ReorderSetlist {
        entry_id: String,
        new_position: usize,
    },
    /// Move a whole block to a new unit-position (blocks + standalone items,
    /// in order). Keeps the block contiguous.
    ReorderBlock {
        group_id: String,
        new_position: usize,
    },
    /// Drop a block's related exercises, keeping the piece (becomes standalone).
    KeepOnlyPiece {
        group_id: String,
    },
    /// Dissolve a block — its entries stay in place but become standalone.
    UngroupBlock {
        group_id: String,
    },
    /// Dissolve every block.
    UngroupAllBlocks,
    /// Remove a whole block (piece + its related exercises).
    RemoveBlock {
        group_id: String,
    },
    /// Add one more exercise into an already-present block, positioned before
    /// the block's anchor piece. Membership is binary, same idempotency as
    /// `AddToSetlist` (#939): re-adding a present item is a no-op.
    AddExerciseToBlock {
        group_id: String,
        item_id: String,
    },
    StartSession {
        now: DateTime<Utc>,
    },
    /// Set or clear the session-level time target during building phase.
    /// `None` removes the target; `Some(mins)` sets it (validated against
    /// MIN/MAX_SESSION_TARGET_MINS).
    SetTargetDuration {
        target_duration_mins: Option<u32>,
    },
    CancelBuilding,

    // === Active Phase ===
    NextItem {
        now: DateTime<Utc>,
    },
    SkipItem {
        now: DateTime<Utc>,
    },
    AddItemMidSession {
        item_id: String,
    },
    AddNewItemMidSession {
        title: String,
        item_type: ItemKind,
    },
    FinishSession {
        now: DateTime<Utc>,
    },
    EndSessionEarly {
        now: DateTime<Utc>,
    },
    /// Abandon an active session without saving — goes directly to Idle.
    /// Used when the user wants to discard an in-progress session from the
    /// new-session page (e.g. after crash recovery leaves a stale session).
    AbandonSession,
    /// Increment rep count on current entry (capped at target).
    RepGotIt,
    /// Decrement rep count on current entry (floor 0).
    RepMissed,
    /// Initialise rep counter on current entry. Only sets defaults when
    /// no prior rep state exists; otherwise preserves existing state.
    InitRepCounter,

    // === Summary Phase ===
    UpdateEntryNotes {
        entry_id: String,
        notes: Option<String>,
    },
    UpdateEntryScore {
        entry_id: String,
        score: Option<u8>,
    },
    UpdateEntryTempo {
        entry_id: String,
        tempo: Option<u16>,
    },
    UpdateSessionNotes {
        notes: Option<String>,
    },
    UpdateSessionReflection {
        field: ReflectionField,
        text: Option<String>,
    },
    SaveSession {
        now: DateTime<Utc>,
    },
    DiscardSession,

    // === Recovery ===
    RecoverSession {
        session: ActiveSession,
        now: DateTime<Utc>,
    },

    // === History ===
    DeleteSession {
        id: String,
    },
    UpdateSessionScore {
        score: Option<u8>,
    },
}

// ── Helpers ────────────────────────────────────────────────────────────

/// Format seconds into a human-readable duration string.
pub fn format_duration_display(secs: u64) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

/// The builder's planned-duration dialect ("12 min" for whole minutes) —
/// shared by block rows, per-entry planned labels, and the builder total.
pub fn format_planned_duration(secs: u64) -> String {
    if secs % 60 == 0 {
        format!("{} min", secs / 60)
    } else {
        format_duration_display(secs)
    }
}

/// Coarse "45m" / "2h 15m" total (Pencil's pattern) for session-summary lines —
/// minutes floored, seconds dropped. Distinct from `format_duration_display`,
/// which keeps seconds for the live timer and per-entry rows.
pub fn format_duration_summary(secs: u64) -> String {
    let total_minutes = secs / 60;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;
    if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

fn find_item_in_model(model: &Model, item_id: &str) -> Option<(String, ItemKind)> {
    model
        .items
        .iter()
        .find(|i| i.id == item_id)
        .map(|i| (i.title.clone(), i.kind.clone()))
}

fn create_entry(
    item_id: &str,
    item_title: &str,
    item_type: ItemKind,
    position: usize,
) -> SetlistEntry {
    SetlistEntry {
        id: ulid::Ulid::gen().to_string(),
        item_id: item_id.to_string(),
        item_title: item_title.to_string(),
        item_type,
        position,
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
        group_id: None,
    }
}

fn reindex_entries(entries: &mut [SetlistEntry]) {
    for (i, entry) in entries.iter_mut().enumerate() {
        entry.position = i;
    }
}

/// Partition entries into ordered units: a contiguous run sharing a `Some`
/// `group_id` is one unit (a block); every `None` entry is its own unit.
fn into_units(entries: Vec<SetlistEntry>) -> Vec<Vec<SetlistEntry>> {
    let mut units: Vec<Vec<SetlistEntry>> = Vec::new();
    for entry in entries {
        match &entry.group_id {
            Some(g) => {
                let extends = units
                    .last()
                    .and_then(|u| u.first())
                    .and_then(|e| e.group_id.as_deref())
                    == Some(g.as_str());
                if extends {
                    units.last_mut().expect("checked above").push(entry);
                } else {
                    units.push(vec![entry]);
                }
            }
            None => units.push(vec![entry]),
        }
    }
    units
}

/// True when every `group_id` occupies a single contiguous run — the block
/// invariant a reorder must never break.
fn groups_contiguous(entries: &[SetlistEntry]) -> bool {
    let mut closed: std::collections::HashSet<&str> = std::collections::HashSet::new();
    let mut current: Option<&str> = None;
    for entry in entries {
        let g = entry.group_id.as_deref();
        if g != current {
            if let Some(prev) = current {
                closed.insert(prev);
            }
            if let Some(g) = g {
                if closed.contains(g) {
                    return false;
                }
            }
            current = g;
        }
    }
    true
}

/// Clear the `group_id` of any block left without its anchor piece — a block
/// only means "this piece's warm-up", so when the piece goes the related
/// exercises become standalone (§7.4 dissolve).
fn dissolve_pieceless_groups(entries: &mut [SetlistEntry]) {
    let anchored: std::collections::HashSet<&str> = entries
        .iter()
        .filter(|e| e.item_type == ItemKind::Piece)
        .filter_map(|e| e.group_id.as_deref())
        .collect();
    let orphans: std::collections::HashSet<String> = entries
        .iter()
        .filter_map(|e| e.group_id.clone())
        .filter(|g| !anchored.contains(g.as_str()))
        .collect();
    for entry in entries.iter_mut() {
        if entry
            .group_id
            .as_deref()
            .is_some_and(|g| orphans.contains(g))
        {
            entry.group_id = None;
        }
    }
}

fn create_item_from_title(title: &str, kind: ItemKind) -> Item {
    let now = Utc::now();
    Item {
        id: ulid::Ulid::gen().to_string(),
        title: title.to_string(),
        kind,
        composer: None,
        key: None,
        modality: None,
        tempo: None,
        notes: None,
        tags: vec![],
        linked_exercise_ids: vec![],
        created_at: now,
        updated_at: now,
        priority: false,
        chord_chart: None,
        variants: vec![],
    }
}

fn freeze_rep_state(entry: &mut SetlistEntry) {
    if let (Some(target), Some(count)) = (entry.rep_target, entry.rep_count) {
        entry.rep_target_reached = Some(count >= target);
    }
}

/// Find an entry by id in Active *or* Summary phase, so the mid-session
/// reflection sheet can write per-entry data before the summary screen.
fn entry_for_update_mut<'a>(model: &'a mut Model, entry_id: &str) -> Option<&'a mut SetlistEntry> {
    match &mut model.session_status {
        SessionStatus::Active(active) => active.entries.iter_mut().find(|e| e.id == entry_id),
        SessionStatus::Summary(summary) => summary.entries.iter_mut().find(|e| e.id == entry_id),
        SessionStatus::Idle | SessionStatus::Building(_) => None,
    }
}

fn transition_to_summary(
    active: &mut ActiveSession,
    now: DateTime<Utc>,
    completion_status: CompletionStatus,
) -> SummarySession {
    let elapsed = (now - active.current_item_started_at).num_seconds().max(0) as u64;
    if let Some(entry) = active.entries.get_mut(active.current_index) {
        entry.duration_secs = elapsed;
        entry.status = EntryStatus::Completed;
        freeze_rep_state(entry);
    }

    if completion_status == CompletionStatus::EndedEarly {
        for entry in active.entries.iter_mut().skip(active.current_index + 1) {
            entry.status = EntryStatus::NotAttempted;
            entry.duration_secs = 0;
        }
    }

    SummarySession {
        id: active.id.clone(),
        entries: active.entries.clone(),
        session_started_at: active.session_started_at,
        session_ended_at: now,
        session_notes: None,
        session_intention: active.session_intention.clone(),
        completion_status,
        session_score: None,
        reflection_improved: None,
        reflection_still_rough: None,
        reflection_next_target: None,
    }
}

// ── Event Handler ──────────────────────────────────────────────────────

pub fn handle_session_event(event: SessionEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        // ── Building Phase ─────────────────────────────────────────
        SessionEvent::StartBuilding => {
            if !matches!(model.session_status, SessionStatus::Idle) {
                model.last_error = Some("A practice is already in progress".to_string());
                return crux_core::render::render();
            }
            model.session_status = SessionStatus::Building(BuildingSession::default());
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::StartBuildingWithTarget {
            target_duration_mins,
        } => {
            if !matches!(model.session_status, SessionStatus::Idle) {
                model.last_error = Some("A practice is already in progress".to_string());
                return crux_core::render::render();
            }
            if !(validation::MIN_SESSION_TARGET_MINS..=validation::MAX_SESSION_TARGET_MINS)
                .contains(&target_duration_mins)
            {
                model.last_error = Some(format!(
                    "Session target must be between {} and {} minutes",
                    validation::MIN_SESSION_TARGET_MINS,
                    validation::MAX_SESSION_TARGET_MINS
                ));
                return crux_core::render::render();
            }
            model.session_status = SessionStatus::Building(BuildingSession {
                target_duration_mins: Some(target_duration_mins),
                ..Default::default()
            });
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::SetSessionIntention { intention } => {
            let SessionStatus::Building(ref mut building) = model.session_status else {
                // No-op when not in Building state
                return crux_core::render::render();
            };

            if let Err(e) = validation::validate_intention(&intention) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            building.session_intention = intention;
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::SetTargetDuration {
            target_duration_mins,
        } => {
            let SessionStatus::Building(ref mut building) = model.session_status else {
                return crux_core::render::render();
            };

            if let Some(mins) = target_duration_mins {
                if !(validation::MIN_SESSION_TARGET_MINS..=validation::MAX_SESSION_TARGET_MINS)
                    .contains(&mins)
                {
                    model.last_error = Some(format!(
                        "Session target must be between {} and {} minutes",
                        validation::MIN_SESSION_TARGET_MINS,
                        validation::MAX_SESSION_TARGET_MINS
                    ));
                    return crux_core::render::render();
                }
            }

            building.target_duration_mins = target_duration_mins;
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::SetEntryIntention {
            entry_id,
            intention,
        } => {
            let SessionStatus::Building(ref mut building) = model.session_status else {
                // No-op when not in Building state
                return crux_core::render::render();
            };

            if let Err(e) = validation::validate_intention(&intention) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(entry) = building.entries.iter_mut().find(|e| e.id == entry_id) else {
                model.last_error = Some(format!("Entry '{entry_id}' not found in setlist"));
                return crux_core::render::render();
            };

            entry.intention = intention;
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::SetRepTarget { entry_id, target } => {
            let SessionStatus::Building(ref mut building) = model.session_status else {
                // No-op when not in Building state
                return crux_core::render::render();
            };

            if let Some(t) = target {
                if let Err(e) = validation::validate_rep_target(&Some(t)) {
                    model.last_error = Some(e.to_string());
                    return crux_core::render::render();
                }
            }

            let Some(entry) = building.entries.iter_mut().find(|e| e.id == entry_id) else {
                model.last_error = Some(format!("Entry '{entry_id}' not found in setlist"));
                return crux_core::render::render();
            };

            entry.rep_target = target;
            // Changing the target invalidates any prior progress.
            entry.rep_count = None;
            entry.rep_target_reached = None;
            entry.rep_history = None;
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::SetEntryDuration {
            entry_id,
            duration_secs,
        } => {
            let SessionStatus::Building(ref mut building) = model.session_status else {
                // No-op when not in Building state
                return crux_core::render::render();
            };

            if let Err(e) = validation::validate_planned_duration(&duration_secs) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(entry) = building.entries.iter_mut().find(|e| e.id == entry_id) else {
                model.last_error = Some(format!("Entry '{entry_id}' not found in setlist"));
                return crux_core::render::render();
            };

            entry.planned_duration_secs = duration_secs;
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::StartBuildingWith { item_id } => {
            if !matches!(model.session_status, SessionStatus::Idle) {
                model.last_error = Some("A practice is already in progress".to_string());
                return crux_core::render::render();
            }
            if !model.items.iter().any(|i| i.id == item_id) {
                model.last_error = Some(LibraryError::NotFound { id: item_id }.to_string());
                return crux_core::render::render();
            }
            model.session_status = SessionStatus::Building(BuildingSession::default());
            handle_session_event(SessionEvent::AddToSetlist { item_id }, model)
        }

        SessionEvent::AddToSetlist { item_id } => {
            if !matches!(model.session_status, SessionStatus::Building(_)) {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            }

            // Membership is binary (the picker/sheet toggle relies on it):
            // re-adding a present item is an idempotent no-op, not a duplicate (#939).
            if let SessionStatus::Building(ref building) = model.session_status {
                if building.entries.iter().any(|e| e.item_id == item_id) {
                    model.last_error = None;
                    return crux_core::render::render();
                }
            }

            // Resolve the item and — for a piece — its related exercises as owned
            // tuples before taking the mutable Building borrow.
            let Some(item) = model.items.iter().find(|i| i.id == item_id) else {
                model.last_error = Some(LibraryError::NotFound { id: item_id }.to_string());
                return crux_core::render::render();
            };
            let piece = (item.id.clone(), item.title.clone(), item.kind.clone());
            let related: Vec<(String, String, ItemKind)> = if item.kind == ItemKind::Piece {
                item.linked_exercise_ids
                    .iter()
                    .filter_map(|ex_id| {
                        model
                            .items
                            .iter()
                            .find(|i| &i.id == ex_id && i.kind == ItemKind::Exercise)
                            .map(|i| (i.id.clone(), i.title.clone(), i.kind.clone()))
                    })
                    .collect()
            } else {
                Vec::new()
            };

            let SessionStatus::Building(ref mut building) = model.session_status else {
                model.last_error = Some("Internal error: expected Building state".to_string());
                return crux_core::render::render();
            };

            // Skip related exercises already in the setlist — don't duplicate.
            let existing: std::collections::HashSet<String> =
                building.entries.iter().map(|e| e.item_id.clone()).collect();
            let related_to_add: Vec<(String, String, ItemKind)> = related
                .into_iter()
                .filter(|(id, _, _)| !existing.contains(id))
                .collect();

            // A block forms only when ≥1 related exercise actually comes along.
            let group_id = if related_to_add.is_empty() {
                None
            } else {
                Some(ulid::Ulid::gen().to_string())
            };

            // Related first (warm-up order), then the piece.
            for (id, title, kind) in &related_to_add {
                let position = building.entries.len();
                let mut entry = create_entry(id, title, kind.clone(), position);
                entry.group_id.clone_from(&group_id);
                building.entries.push(entry);
            }
            let position = building.entries.len();
            let mut piece_entry = create_entry(&piece.0, &piece.1, piece.2, position);
            piece_entry.group_id = group_id;
            building.entries.push(piece_entry);

            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::AddNewItemToSetlist { title, item_type } => {
            let SessionStatus::Building(ref mut building) = model.session_status else {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            };

            if let Err(e) = validation::validate_title(&title) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let item = create_item_from_title(&title, item_type.clone());
            let new_item_id = item.id.clone();
            model.items.push(item.clone());

            let position = building.entries.len();
            let entry = create_entry(&new_item_id, &title, item_type, position);
            building.entries.push(entry);
            model.last_error = None;

            Command::all([
                crate::http::create_item(&model.api_base_url, &item, &new_item_id),
                crux_core::render::render(),
            ])
        }

        SessionEvent::RemoveFromSetlist { entry_id } => {
            let SessionStatus::Building(ref mut building) = model.session_status else {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            };

            let len_before = building.entries.len();
            building.entries.retain(|e| e.id != entry_id);

            if building.entries.len() == len_before {
                model.last_error = Some(format!("Entry '{entry_id}' not found in setlist"));
                return crux_core::render::render();
            }

            dissolve_pieceless_groups(&mut building.entries);
            reindex_entries(&mut building.entries);
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::ReorderSetlist {
            entry_id,
            new_position,
        } => {
            let SessionStatus::Building(ref mut building) = model.session_status else {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            };

            let Some(current_index) = building.entries.iter().position(|e| e.id == entry_id) else {
                model.last_error = Some(format!("Entry '{entry_id}' not found in setlist"));
                return crux_core::render::render();
            };

            if new_position >= building.entries.len() {
                model.last_error = Some(format!(
                    "Invalid position: {new_position} (max: {})",
                    building.entries.len().saturating_sub(1)
                ));
                return crux_core::render::render();
            }

            let entry = building.entries.remove(current_index);
            building.entries.insert(new_position, entry);
            if !groups_contiguous(&building.entries) {
                // Revert — the move would split a block.
                let entry = building.entries.remove(new_position);
                building.entries.insert(current_index, entry);
                model.last_error = Some("Can't move an item out of its block".to_string());
                return crux_core::render::render();
            }
            reindex_entries(&mut building.entries);
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::ReorderBlock {
            group_id,
            new_position,
        } => {
            let SessionStatus::Building(ref mut building) = model.session_status else {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            };

            let mut units = into_units(std::mem::take(&mut building.entries));
            let Some(current) = units.iter().position(|u| {
                u.first().and_then(|e| e.group_id.as_deref()) == Some(group_id.as_str())
            }) else {
                building.entries = units.into_iter().flatten().collect();
                model.last_error = Some(format!("Block '{group_id}' not found in setlist"));
                return crux_core::render::render();
            };

            let target = new_position.min(units.len().saturating_sub(1));
            let unit = units.remove(current);
            units.insert(target, unit);
            building.entries = units.into_iter().flatten().collect();
            reindex_entries(&mut building.entries);
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::KeepOnlyPiece { group_id } => {
            let SessionStatus::Building(ref mut building) = model.session_status else {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            };

            let in_block = |e: &SetlistEntry| e.group_id.as_deref() == Some(group_id.as_str());
            if !building.entries.iter().any(in_block) {
                model.last_error = Some(format!("Block '{group_id}' not found in setlist"));
                return crux_core::render::render();
            }

            building
                .entries
                .retain(|e| !(in_block(e) && e.item_type == ItemKind::Exercise));
            // The lone piece left behind is no longer a block.
            for entry in building.entries.iter_mut().filter(|e| in_block(e)) {
                entry.group_id = None;
            }
            reindex_entries(&mut building.entries);
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::UngroupBlock { group_id } => {
            let SessionStatus::Building(ref mut building) = model.session_status else {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            };

            let mut found = false;
            for entry in building
                .entries
                .iter_mut()
                .filter(|e| e.group_id.as_deref() == Some(group_id.as_str()))
            {
                entry.group_id = None;
                found = true;
            }
            if !found {
                model.last_error = Some(format!("Block '{group_id}' not found in setlist"));
                return crux_core::render::render();
            }
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::UngroupAllBlocks => {
            let SessionStatus::Building(ref mut building) = model.session_status else {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            };
            for entry in &mut building.entries {
                entry.group_id = None;
            }
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::RemoveBlock { group_id } => {
            let SessionStatus::Building(ref mut building) = model.session_status else {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            };

            let len_before = building.entries.len();
            building
                .entries
                .retain(|e| e.group_id.as_deref() != Some(group_id.as_str()));
            if building.entries.len() == len_before {
                model.last_error = Some(format!("Block '{group_id}' not found in setlist"));
                return crux_core::render::render();
            }
            reindex_entries(&mut building.entries);
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::AddExerciseToBlock { group_id, item_id } => {
            let SessionStatus::Building(ref building) = model.session_status else {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            };

            // Membership is binary, same idempotency as `AddToSetlist` (#939).
            if building.entries.iter().any(|e| e.item_id == item_id) {
                model.last_error = None;
                return crux_core::render::render();
            }

            let Some(anchor_index) = building.entries.iter().position(|e| {
                e.group_id.as_deref() == Some(group_id.as_str()) && e.item_type == ItemKind::Piece
            }) else {
                model.last_error = Some(format!("Block '{group_id}' not found in setlist"));
                return crux_core::render::render();
            };

            let Some(item) = model.items.iter().find(|i| i.id == item_id) else {
                model.last_error = Some(LibraryError::NotFound { id: item_id }.to_string());
                return crux_core::render::render();
            };
            if item.kind != ItemKind::Exercise {
                model.last_error = Some("Only an exercise can be added to a block".to_string());
                return crux_core::render::render();
            }
            let (id, title, kind) = (item.id.clone(), item.title.clone(), item.kind.clone());

            let SessionStatus::Building(ref mut building) = model.session_status else {
                model.last_error = Some("Internal error: expected Building state".to_string());
                return crux_core::render::render();
            };
            let mut entry = create_entry(&id, &title, kind, anchor_index);
            entry.group_id = Some(group_id);
            building.entries.insert(anchor_index, entry);
            reindex_entries(&mut building.entries);
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::StartSession { now } => {
            let SessionStatus::Building(ref building) = model.session_status else {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            };

            if let Err(e) = validation::validate_entries_not_empty(&building.entries, "Setlist") {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let mut entries = building.entries.clone();
            for entry in &mut entries {
                if entry.rep_target.is_some() {
                    entry.rep_count = Some(0);
                    entry.rep_target_reached = Some(false);
                    entry.rep_history = Some(vec![]);
                }
            }

            let active = ActiveSession {
                id: ulid::Ulid::gen().to_string(),
                entries,
                current_index: 0,
                current_item_started_at: now,
                session_started_at: now,
                session_intention: building.session_intention.clone(),
            };

            let save_effect = AppEffect::SaveSessionInProgress(active.clone());
            model.session_status = SessionStatus::Active(active);
            model.last_error = None;

            Command::all([
                Command::notify_shell(save_effect).into(),
                crux_core::render::render(),
            ])
        }

        SessionEvent::CancelBuilding => {
            // Idempotent: already-Idle cancel is a no-op success, not a silent error (#944).
            match model.session_status {
                SessionStatus::Building(_) | SessionStatus::Idle => {
                    model.session_status = SessionStatus::Idle;
                    model.last_error = None;
                }
                _ => {
                    model.last_error = Some("Not in building state".to_string());
                }
            }
            crux_core::render::render()
        }

        // ── Active Phase ───────────────────────────────────────────
        SessionEvent::NextItem { now } => {
            let SessionStatus::Active(ref mut active) = model.session_status else {
                model.last_error = Some("Not in active state".to_string());
                return crux_core::render::render();
            };

            if active.current_index >= active.entries.len() - 1 {
                let summary = transition_to_summary(active, now, CompletionStatus::Completed);
                model.session_status = SessionStatus::Summary(summary);
                model.last_error = None;
                return crux_core::render::render();
            }

            let elapsed = (now - active.current_item_started_at).num_seconds().max(0) as u64;

            if let Some(entry) = active.entries.get_mut(active.current_index) {
                entry.duration_secs = elapsed;
                entry.status = EntryStatus::Completed;
                freeze_rep_state(entry);
            }

            active.current_index += 1;
            active.current_item_started_at = now;
            model.last_error = None;

            let save_effect = AppEffect::SaveSessionInProgress(active.clone());
            Command::all([
                Command::notify_shell(save_effect).into(),
                crux_core::render::render(),
            ])
        }

        SessionEvent::SkipItem { now } => {
            let SessionStatus::Active(ref mut active) = model.session_status else {
                model.last_error = Some("Not in active state".to_string());
                return crux_core::render::render();
            };

            if let Some(entry) = active.entries.get_mut(active.current_index) {
                entry.duration_secs = 0;
                entry.status = EntryStatus::Skipped;
                freeze_rep_state(entry);
            }

            if active.current_index >= active.entries.len() - 1 {
                let summary = SummarySession {
                    id: active.id.clone(),
                    entries: active.entries.clone(),
                    session_started_at: active.session_started_at,
                    session_ended_at: now,
                    session_notes: None,
                    session_intention: active.session_intention.clone(),
                    completion_status: CompletionStatus::Completed,
                    session_score: None,
                    reflection_improved: None,
                    reflection_still_rough: None,
                    reflection_next_target: None,
                };
                model.session_status = SessionStatus::Summary(summary);
                model.last_error = None;
                return crux_core::render::render();
            }

            active.current_index += 1;
            active.current_item_started_at = now;
            model.last_error = None;

            let save_effect = AppEffect::SaveSessionInProgress(active.clone());
            Command::all([
                Command::notify_shell(save_effect).into(),
                crux_core::render::render(),
            ])
        }

        SessionEvent::AddItemMidSession { item_id } => {
            if !matches!(model.session_status, SessionStatus::Active(_)) {
                model.last_error = Some("Not in active state".to_string());
                return crux_core::render::render();
            }

            let Some((title, item_type)) = find_item_in_model(model, &item_id) else {
                model.last_error = Some(LibraryError::NotFound { id: item_id }.to_string());
                return crux_core::render::render();
            };

            let SessionStatus::Active(ref mut active) = model.session_status else {
                model.last_error = Some("Internal error: expected Active state".to_string());
                return crux_core::render::render();
            };
            let position = active.entries.len();
            let entry = create_entry(&item_id, &title, item_type, position);
            active.entries.push(entry);
            model.last_error = None;

            let save_effect = AppEffect::SaveSessionInProgress(active.clone());
            Command::all([
                Command::notify_shell(save_effect).into(),
                crux_core::render::render(),
            ])
        }

        SessionEvent::AddNewItemMidSession { title, item_type } => {
            let SessionStatus::Active(ref mut active) = model.session_status else {
                model.last_error = Some("Not in active state".to_string());
                return crux_core::render::render();
            };

            if let Err(e) = validation::validate_title(&title) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let item = create_item_from_title(&title, item_type.clone());
            let new_item_id = item.id.clone();
            model.items.push(item.clone());

            let position = active.entries.len();
            let entry = create_entry(&new_item_id, &title, item_type, position);
            active.entries.push(entry);
            model.last_error = None;

            let save_effect_session = AppEffect::SaveSessionInProgress(active.clone());
            Command::all([
                crate::http::create_item(&model.api_base_url, &item, &new_item_id),
                Command::notify_shell(save_effect_session).into(),
                crux_core::render::render(),
            ])
        }

        SessionEvent::FinishSession { now } => {
            let SessionStatus::Active(ref mut active) = model.session_status else {
                model.last_error = Some("Not in active state".to_string());
                return crux_core::render::render();
            };

            let summary = transition_to_summary(active, now, CompletionStatus::Completed);
            model.session_status = SessionStatus::Summary(summary);
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::EndSessionEarly { now } => {
            let SessionStatus::Active(ref mut active) = model.session_status else {
                model.last_error = Some("Not in active state".to_string());
                return crux_core::render::render();
            };

            let summary = transition_to_summary(active, now, CompletionStatus::EndedEarly);
            model.session_status = SessionStatus::Summary(summary);
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::AbandonSession => {
            if !matches!(model.session_status, SessionStatus::Active(_)) {
                model.last_error = Some("No active practice to abandon".to_string());
                return crux_core::render::render();
            }

            model.session_status = SessionStatus::Idle;
            model.last_error = None;

            Command::all([
                Command::notify_shell(AppEffect::ClearSessionInProgress).into(),
                crux_core::render::render(),
            ])
        }

        SessionEvent::RepGotIt => {
            let SessionStatus::Active(ref mut active) = model.session_status else {
                return crux_core::render::render();
            };

            let Some(entry) = active.entries.get_mut(active.current_index) else {
                return crux_core::render::render();
            };

            // No-op if counter is not active or target already reached
            let (Some(target), Some(count)) = (entry.rep_target, entry.rep_count) else {
                return crux_core::render::render();
            };
            if entry.rep_target_reached == Some(true) {
                return crux_core::render::render();
            }

            let new_count = (count + 1).min(target);
            entry.rep_count = Some(new_count);
            if new_count >= target {
                entry.rep_target_reached = Some(true);
            }

            if let Some(ref mut history) = entry.rep_history {
                if history.len() < crate::validation::MAX_REP_HISTORY {
                    history.push(RepAction::Success);
                }
            }

            model.last_error = None;
            let save_effect = AppEffect::SaveSessionInProgress(active.clone());
            Command::all([
                Command::notify_shell(save_effect).into(),
                crux_core::render::render(),
            ])
        }

        SessionEvent::RepMissed => {
            let SessionStatus::Active(ref mut active) = model.session_status else {
                return crux_core::render::render();
            };

            let Some(entry) = active.entries.get_mut(active.current_index) else {
                return crux_core::render::render();
            };

            // No-op if counter is not active or target already reached
            let (Some(_target), Some(count)) = (entry.rep_target, entry.rep_count) else {
                return crux_core::render::render();
            };
            if entry.rep_target_reached == Some(true) {
                return crux_core::render::render();
            }

            entry.rep_count = Some(count.saturating_sub(1));

            if let Some(ref mut history) = entry.rep_history {
                if history.len() < crate::validation::MAX_REP_HISTORY {
                    history.push(RepAction::Missed);
                }
            }

            model.last_error = None;
            let save_effect = AppEffect::SaveSessionInProgress(active.clone());
            Command::all([
                Command::notify_shell(save_effect).into(),
                crux_core::render::render(),
            ])
        }

        SessionEvent::InitRepCounter => {
            let SessionStatus::Active(ref mut active) = model.session_status else {
                return crux_core::render::render();
            };

            let Some(entry) = active.entries.get_mut(active.current_index) else {
                return crux_core::render::render();
            };

            // Preserve existing rep state (e.g. after hide/show); only seed defaults when absent.
            if entry.rep_target.is_none() {
                entry.rep_target = Some(validation::DEFAULT_REP_TARGET);
                entry.rep_count = Some(0);
                entry.rep_target_reached = Some(false);
                entry.rep_history = Some(vec![]);
            }

            model.last_error = None;
            let save_effect = AppEffect::SaveSessionInProgress(active.clone());
            Command::all([
                Command::notify_shell(save_effect).into(),
                crux_core::render::render(),
            ])
        }

        // ── Entry Updates (Active or Summary) ──────────────────────
        // Accepted in both phases so the mid-session reflection sheet can record
        // as the user moves on. Invariant: only Completed entries can be scored.
        SessionEvent::UpdateEntryScore { entry_id, score } => {
            if let Some(s) = score {
                if !(validation::MIN_SCORE..=validation::MAX_SCORE).contains(&s) {
                    return crux_core::render::render();
                }
            }

            let Some(entry) = entry_for_update_mut(model, &entry_id) else {
                return crux_core::render::render();
            };

            if entry.status != EntryStatus::Completed {
                return crux_core::render::render();
            }

            entry.score = score;
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::UpdateEntryTempo { entry_id, tempo } => {
            if let Err(_e) = validation::validate_achieved_tempo(&tempo) {
                return crux_core::render::render();
            }

            let Some(entry) = entry_for_update_mut(model, &entry_id) else {
                return crux_core::render::render();
            };

            if entry.status != EntryStatus::Completed {
                return crux_core::render::render();
            }

            entry.achieved_tempo = tempo;
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::UpdateEntryNotes { entry_id, notes } => {
            if let Err(e) = validation::validate_entry_notes(&notes) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let Some(entry) = entry_for_update_mut(model, &entry_id) else {
                model.last_error = Some(format!("Entry '{entry_id}' not found"));
                return crux_core::render::render();
            };

            entry.notes = notes;
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::UpdateSessionReflection { field, text } => {
            let SessionStatus::Summary(ref mut summary) = model.session_status else {
                model.last_error = Some("Not in summary state".to_string());
                return crux_core::render::render();
            };

            if let Err(e) = validation::validate_reflection(&text) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            let text = text.map(|t| t.trim().to_string()).filter(|t| !t.is_empty());
            match field {
                ReflectionField::Improved => summary.reflection_improved = text,
                ReflectionField::StillRough => summary.reflection_still_rough = text,
                ReflectionField::NextTarget => summary.reflection_next_target = text,
            }
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::UpdateSessionNotes { notes } => {
            let SessionStatus::Summary(ref mut summary) = model.session_status else {
                model.last_error = Some("Not in summary state".to_string());
                return crux_core::render::render();
            };

            if let Err(e) = validation::validate_session_notes(&notes) {
                model.last_error = Some(e.to_string());
                return crux_core::render::render();
            }

            summary.session_notes = notes;
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::UpdateSessionScore { score } => {
            if let Some(s) = score {
                if !(validation::MIN_SCORE..=validation::MAX_SCORE).contains(&s) {
                    return crux_core::render::render();
                }
            }
            let SessionStatus::Summary(ref mut summary) = model.session_status else {
                model.last_error = Some("Not in summary state".to_string());
                return crux_core::render::render();
            };
            summary.session_score = score;
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::SaveSession { now } => {
            let SessionStatus::Summary(ref summary) = model.session_status else {
                model.last_error = Some("Not in summary state".to_string());
                return crux_core::render::render();
            };

            let total_duration_secs: u64 = summary.entries.iter().map(|e| e.duration_secs).sum();

            let practice_session = PracticeSession {
                id: summary.id.clone(),
                entries: summary.entries.clone(),
                session_notes: summary.session_notes.clone(),
                session_intention: summary.session_intention.clone(),
                started_at: summary.session_started_at,
                completed_at: now,
                total_duration_secs,
                completion_status: summary.completion_status.clone(),
                session_score: summary.session_score,
                reflection_improved: summary.reflection_improved.clone(),
                reflection_still_rough: summary.reflection_still_rough.clone(),
                reflection_next_target: summary.reflection_next_target.clone(),
            };

            model.sessions.push(practice_session.clone());
            model.practice_summaries = crate::app::build_practice_summaries(&model.sessions);
            model.session_status = SessionStatus::Idle;
            model.last_error = None;

            let clear = Command::notify_shell(AppEffect::ClearSessionInProgress).into();
            if model.local_first {
                // No server callback to clear the dismiss-mute, so record success here.
                model.record_success();
                Command::all([
                    crate::persistence::save_session(practice_session),
                    clear,
                    crux_core::render::render(),
                ])
            } else {
                Command::all([
                    crate::http::create_session(&model.api_base_url, &practice_session),
                    clear,
                    crux_core::render::render(),
                ])
            }
        }

        SessionEvent::DiscardSession => {
            if !matches!(model.session_status, SessionStatus::Summary(_)) {
                model.last_error = Some("Not in summary state".to_string());
                return crux_core::render::render();
            }

            model.session_status = SessionStatus::Idle;
            model.last_error = None;

            Command::all([
                Command::notify_shell(AppEffect::ClearSessionInProgress).into(),
                crux_core::render::render(),
            ])
        }

        // ── Recovery ───────────────────────────────────────────────
        SessionEvent::RecoverSession { session, now } => {
            if !matches!(model.session_status, SessionStatus::Idle) {
                model.last_error =
                    Some("Cannot recover: a practice is already in progress".to_string());
                return crux_core::render::render();
            }

            // Re-anchor the running item's wall-clock timer: the blob's anchor
            // is from before the kill, so resuming hours later would otherwise
            // show that gap as elapsed practice (#962).
            let mut session = session;
            session.current_item_started_at = now;
            model.session_status = SessionStatus::Active(session);
            model.last_error = None;
            crux_core::render::render()
        }

        // ── History ────────────────────────────────────────────────
        SessionEvent::DeleteSession { id } => {
            let len_before = model.sessions.len();
            model.sessions.retain(|s| s.id != id);

            if model.sessions.len() == len_before {
                model.last_error = Some(LibraryError::NotFound { id: id.clone() }.to_string());
                return crux_core::render::render();
            }

            model.practice_summaries = crate::app::build_practice_summaries(&model.sessions);
            model.last_error = None;

            Command::all([
                crate::http::delete_session(&model.api_base_url, &id),
                crux_core::render::render(),
            ])
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Intrada;
    use crux_core::App;

    fn model_with_library() -> Model {
        let now = Utc::now();
        Model {
            items: vec![
                Item {
                    id: "piece-1".to_string(),
                    title: "Moonlight Sonata".to_string(),
                    kind: ItemKind::Piece,
                    composer: Some("Beethoven".to_string()),
                    key: None,
                    modality: None,
                    tempo: None,
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                    linked_exercise_ids: vec![],
                    priority: false,
                    chord_chart: None,
                    variants: vec![],
                },
                Item {
                    id: "piece-2".to_string(),
                    title: "Clair de Lune".to_string(),
                    kind: ItemKind::Piece,
                    composer: Some("Debussy".to_string()),
                    key: None,
                    modality: None,
                    tempo: None,
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                    linked_exercise_ids: vec![],
                    priority: false,
                    chord_chart: None,
                    variants: vec![],
                },
                Item {
                    id: "exercise-1".to_string(),
                    title: "C Major Scale".to_string(),
                    kind: ItemKind::Exercise,
                    composer: None,
                    key: None,
                    modality: None,
                    tempo: None,
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                    linked_exercise_ids: vec![],
                    priority: false,
                    chord_chart: None,
                    variants: vec![],
                },
            ],
            api_base_url: "http://localhost:3001".to_string(),
            ..Default::default()
        }
    }

    fn update(model: &mut Model, event: Event) {
        let app = Intrada;
        let _cmd = app.update(event, model);
    }

    // --- Block grouping (related exercises travel with a piece) ---

    fn linked_model() -> Model {
        let now = Utc::now();
        let mk = |id: &str, title: &str, kind: ItemKind, linked: &[&str]| Item {
            id: id.to_string(),
            title: title.to_string(),
            kind,
            composer: None,
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
            linked_exercise_ids: linked.iter().map(|s| s.to_string()).collect(),
            created_at: now,
            updated_at: now,
            priority: false,
            chord_chart: None,
            variants: vec![],
        };
        Model {
            items: vec![
                mk("piece-P", "Sonata", ItemKind::Piece, &["ex-A", "ex-B"]),
                mk("piece-Q", "Nocturne", ItemKind::Piece, &[]),
                mk("piece-R", "Etude", ItemKind::Piece, &["ex-C"]),
                mk("ex-A", "Scales", ItemKind::Exercise, &[]),
                mk("ex-B", "Arpeggios", ItemKind::Exercise, &[]),
                mk("ex-C", "Sight-reading", ItemKind::Exercise, &[]),
                mk("ex-D", "Trills", ItemKind::Exercise, &[]),
            ],
            api_base_url: "http://localhost:3001".to_string(),
            ..Default::default()
        }
    }

    fn building_entries(model: &Model) -> &[SetlistEntry] {
        match &model.session_status {
            SessionStatus::Building(b) => &b.entries,
            _ => panic!("expected Building state"),
        }
    }

    fn ids(model: &Model) -> Vec<String> {
        building_entries(model)
            .iter()
            .map(|e| e.item_id.clone())
            .collect()
    }

    fn add(model: &mut Model, item_id: &str) {
        update(
            model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: item_id.to_string(),
            }),
        );
    }

    fn group_of(model: &Model, item_id: &str) -> Option<String> {
        building_entries(model)
            .iter()
            .find(|e| e.item_id == item_id)
            .and_then(|e| e.group_id.clone())
    }

    #[test]
    fn start_building_with_seeds_exercise_from_idle() {
        let mut m = linked_model();
        update(
            &mut m,
            Event::Session(SessionEvent::StartBuildingWith {
                item_id: "ex-C".to_string(),
            }),
        );
        let e = building_entries(&m);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].item_id, "ex-C");
        assert_eq!(e[0].group_id, None);
        assert_eq!(m.last_error, None);
    }

    #[test]
    fn start_building_with_piece_forms_block() {
        let mut m = linked_model();
        update(
            &mut m,
            Event::Session(SessionEvent::StartBuildingWith {
                item_id: "piece-P".to_string(),
            }),
        );
        assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"]);
        let e = building_entries(&m);
        assert!(e[0].group_id.is_some(), "block has a group_id");
        assert!(e.iter().all(|x| x.group_id == e[0].group_id));
    }

    #[test]
    fn start_building_with_rejects_when_not_idle() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "ex-A");
        update(
            &mut m,
            Event::Session(SessionEvent::StartBuildingWith {
                item_id: "ex-C".to_string(),
            }),
        );
        assert_eq!(
            m.last_error.as_deref(),
            Some("A practice is already in progress")
        );
        assert_eq!(ids(&m), ["ex-A"], "existing setlist untouched");
    }

    #[test]
    fn start_building_with_unknown_item_stays_idle() {
        let mut m = linked_model();
        update(
            &mut m,
            Event::Session(SessionEvent::StartBuildingWith {
                item_id: "nope".to_string(),
            }),
        );
        assert!(m.last_error.is_some());
        assert!(
            matches!(m.session_status, SessionStatus::Idle),
            "a failed seed must not leave an empty building session"
        );
    }

    #[test]
    fn add_to_setlist_is_idempotent_by_item_id() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "ex-C");
        add(&mut m, "ex-C");
        assert_eq!(ids(&m), ["ex-C"], "second add of the same item is a no-op");
        assert_eq!(m.last_error, None);
    }

    #[test]
    fn add_piece_twice_does_not_duplicate_block() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-P");
        add(&mut m, "piece-P");
        assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"]);
        assert_eq!(m.last_error, None);
    }

    #[test]
    fn re_adding_present_piece_is_a_full_no_op_even_with_new_relateds() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-Q");
        if let Some(piece) = m.items.iter_mut().find(|i| i.id == "piece-Q") {
            piece.linked_exercise_ids = vec!["ex-C".to_string()];
        }
        add(&mut m, "piece-Q");
        assert_eq!(
            ids(&m),
            ["piece-Q"],
            "membership no-op wins; newly linked relateds come in by removing and re-adding"
        );
    }

    #[test]
    fn add_piece_pulls_related_into_a_block_related_first() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-P");
        let e = building_entries(&m);
        assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"]);
        let g = e[0].group_id.clone();
        assert!(g.is_some(), "block has a group_id");
        assert!(
            e.iter().all(|x| x.group_id == g),
            "all three share the group"
        );
        assert_eq!((e[0].position, e[1].position, e[2].position), (0, 1, 2));
    }

    #[test]
    fn add_piece_without_related_is_standalone() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-Q");
        let e = building_entries(&m);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].group_id, None);
    }

    #[test]
    fn add_exercise_directly_is_standalone() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "ex-C");
        let e = building_entries(&m);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].group_id, None);
    }

    #[test]
    fn add_piece_skips_already_present_related() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "ex-A");
        add(&mut m, "piece-P");
        let e = building_entries(&m);
        assert_eq!(
            e.iter().filter(|x| x.item_id == "ex-A").count(),
            1,
            "ex-A not duplicated"
        );
        assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"]);
        assert_eq!(
            e[0].group_id, None,
            "the pre-existing ex-A stays standalone"
        );
        assert!(e[1].group_id.is_some());
        assert_eq!(e[1].group_id, e[2].group_id, "ex-B + piece form the block");
    }

    #[test]
    fn add_exercise_to_block_inserts_before_anchor_piece() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-P");
        let g = group_of(&m, "piece-P").unwrap();
        update(
            &mut m,
            Event::Session(SessionEvent::AddExerciseToBlock {
                group_id: g.clone(),
                item_id: "ex-D".to_string(),
            }),
        );
        assert_eq!(m.last_error, None);
        assert_eq!(ids(&m), ["ex-A", "ex-B", "ex-D", "piece-P"]);
        assert_eq!(group_of(&m, "ex-D"), Some(g.clone()));
        assert!(groups_contiguous(building_entries(&m)));
    }

    #[test]
    fn add_exercise_to_block_rejects_unknown_group() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-P");
        update(
            &mut m,
            Event::Session(SessionEvent::AddExerciseToBlock {
                group_id: "no-such-group".to_string(),
                item_id: "ex-D".to_string(),
            }),
        );
        assert!(m.last_error.is_some());
        assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"], "setlist untouched");
    }

    #[test]
    fn add_exercise_to_block_rejects_non_exercise_item() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-P");
        let g = group_of(&m, "piece-P").unwrap();
        update(
            &mut m,
            Event::Session(SessionEvent::AddExerciseToBlock {
                group_id: g,
                item_id: "piece-Q".to_string(),
            }),
        );
        assert!(m.last_error.is_some());
        assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"], "setlist untouched");
    }

    #[test]
    fn add_exercise_to_block_is_idempotent_by_item_id() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-P");
        let g = group_of(&m, "piece-P").unwrap();
        for _ in 0..2 {
            update(
                &mut m,
                Event::Session(SessionEvent::AddExerciseToBlock {
                    group_id: g.clone(),
                    item_id: "ex-D".to_string(),
                }),
            );
        }
        assert_eq!(m.last_error, None);
        assert_eq!(ids(&m), ["ex-A", "ex-B", "ex-D", "piece-P"]);
    }

    #[test]
    fn remove_block_removes_piece_and_related() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "ex-C");
        add(&mut m, "piece-P");
        let g = group_of(&m, "piece-P").unwrap();
        update(
            &mut m,
            Event::Session(SessionEvent::RemoveBlock { group_id: g }),
        );
        assert!(m.last_error.is_none());
        assert_eq!(ids(&m), ["ex-C"]);
    }

    #[test]
    fn keep_only_piece_drops_related_and_destandalones() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-P");
        let g = group_of(&m, "piece-P").unwrap();
        update(
            &mut m,
            Event::Session(SessionEvent::KeepOnlyPiece { group_id: g }),
        );
        let e = building_entries(&m);
        assert_eq!(ids(&m), ["piece-P"]);
        assert_eq!(e[0].group_id, None, "lone piece is no longer a block");
    }

    #[test]
    fn ungroup_block_keeps_items_in_place() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-P");
        let g = group_of(&m, "piece-P").unwrap();
        update(
            &mut m,
            Event::Session(SessionEvent::UngroupBlock { group_id: g }),
        );
        let e = building_entries(&m);
        assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P"]);
        assert!(e.iter().all(|x| x.group_id.is_none()));
    }

    #[test]
    fn ungroup_all_clears_every_group() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-P");
        add(&mut m, "ex-C");
        update(&mut m, Event::Session(SessionEvent::UngroupAllBlocks));
        assert!(building_entries(&m).iter().all(|x| x.group_id.is_none()));
    }

    #[test]
    fn reorder_block_moves_the_whole_unit() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-P");
        add(&mut m, "ex-C");
        let g = group_of(&m, "piece-P").unwrap();
        update(
            &mut m,
            Event::Session(SessionEvent::ReorderBlock {
                group_id: g,
                new_position: 1,
            }),
        );
        assert_eq!(ids(&m), ["ex-C", "ex-A", "ex-B", "piece-P"]);
    }

    #[test]
    fn reorder_within_block_is_allowed() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-P");
        let ex_b = building_entries(&m)[1].id.clone();
        update(
            &mut m,
            Event::Session(SessionEvent::ReorderSetlist {
                entry_id: ex_b,
                new_position: 0,
            }),
        );
        assert!(m.last_error.is_none());
        let e = building_entries(&m);
        assert_eq!(ids(&m), ["ex-B", "ex-A", "piece-P"]);
        assert!(e.iter().all(|x| x.group_id == e[0].group_id));
    }

    #[test]
    fn reorder_that_splits_a_block_is_rejected() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "ex-C");
        add(&mut m, "piece-P");
        let ex_c = building_entries(&m)[0].id.clone();
        update(
            &mut m,
            Event::Session(SessionEvent::ReorderSetlist {
                entry_id: ex_c,
                new_position: 2,
            }),
        );
        assert!(m.last_error.is_some(), "splitting move rejected");
        assert_eq!(
            ids(&m),
            ["ex-C", "ex-A", "ex-B", "piece-P"],
            "order unchanged"
        );
    }

    #[test]
    fn building_view_projects_blocks_and_standalones() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-P");
        add(&mut m, "ex-C");
        let vm = Intrada.view(&m);
        let b = vm.building_setlist.expect("building view");
        assert_eq!(b.item_count, 4);
        assert_eq!(b.block_count, 2);
        let block = &b.blocks[0];
        assert!(block.group_id.is_some());
        assert_eq!(block.piece_title.as_deref(), Some("Sonata"));
        assert_eq!(block.related_count, 2);
        assert_eq!(block.entries.len(), 3);
        let solo = &b.blocks[1];
        assert_eq!(solo.group_id, None);
        assert_eq!(solo.piece_title, None);
        assert_eq!(solo.related_count, 0);
    }

    #[test]
    fn two_adjacent_blocks_project_separately() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-P"); // block G1: ex-A, ex-B, piece-P
        add(&mut m, "piece-R"); // block G2: ex-C, piece-R
        assert_eq!(ids(&m), ["ex-A", "ex-B", "piece-P", "ex-C", "piece-R"]);
        let g1 = group_of(&m, "piece-P");
        let g2 = group_of(&m, "piece-R");
        assert!(g1.is_some() && g2.is_some() && g1 != g2, "distinct blocks");
        let b = Intrada.view(&m).building_setlist.unwrap();
        assert_eq!(b.block_count, 2);
        assert_eq!(b.item_count, 5);
        assert_eq!(b.blocks[0].piece_title.as_deref(), Some("Sonata"));
        assert_eq!(b.blocks[0].related_count, 2);
        assert_eq!(b.blocks[1].piece_title.as_deref(), Some("Etude"));
        assert_eq!(b.blocks[1].related_count, 1);
    }

    #[test]
    fn standalone_can_move_between_two_blocks() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-P"); // G1: 0,1,2
        add(&mut m, "piece-R"); // G2: 3,4
        add(&mut m, "ex-D"); // standalone: 5
        let ex_d = building_entries(&m)
            .iter()
            .find(|e| e.item_id == "ex-D")
            .unwrap()
            .id
            .clone();
        update(
            &mut m,
            Event::Session(SessionEvent::ReorderSetlist {
                entry_id: ex_d,
                new_position: 3,
            }),
        );
        assert!(
            m.last_error.is_none(),
            "a standalone between blocks splits neither"
        );
        assert_eq!(
            ids(&m),
            ["ex-A", "ex-B", "piece-P", "ex-D", "ex-C", "piece-R"]
        );
        assert!(groups_contiguous(building_entries(&m)));
    }

    #[test]
    fn removing_the_piece_dissolves_the_block_to_standalone() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-P"); // ex-A, ex-B, piece-P (group)
        let piece = building_entries(&m)
            .iter()
            .find(|e| e.item_id == "piece-P")
            .unwrap()
            .id
            .clone();
        update(
            &mut m,
            Event::Session(SessionEvent::RemoveFromSetlist { entry_id: piece }),
        );
        assert!(m.last_error.is_none());
        assert_eq!(ids(&m), ["ex-A", "ex-B"]);
        assert!(
            building_entries(&m).iter().all(|x| x.group_id.is_none()),
            "related become standalone when their piece is removed"
        );
        let b = Intrada.view(&m).building_setlist.unwrap();
        assert_eq!(
            b.block_count, 2,
            "two standalone units, not one pieceless block"
        );
    }

    #[test]
    fn removing_a_related_keeps_the_block() {
        let mut m = linked_model();
        update(&mut m, Event::Session(SessionEvent::StartBuilding));
        add(&mut m, "piece-P"); // ex-A, ex-B, piece-P
        let ex_a = building_entries(&m)[0].id.clone();
        update(
            &mut m,
            Event::Session(SessionEvent::RemoveFromSetlist { entry_id: ex_a }),
        );
        assert!(m.last_error.is_none());
        assert_eq!(ids(&m), ["ex-B", "piece-P"]);
        let e = building_entries(&m);
        assert!(
            e[0].group_id.is_some() && e[0].group_id == e[1].group_id,
            "the piece + remaining related stay one block"
        );
    }

    // --- Building Phase Tests ---

    #[test]
    fn test_start_building() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));

        assert!(model.last_error.is_none());
        assert!(matches!(model.session_status, SessionStatus::Building(_)));
    }

    #[test]
    fn test_start_building_when_already_building() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(&mut model, Event::Session(SessionEvent::StartBuilding));

        assert!(model.last_error.is_some());
    }

    #[test]
    fn test_start_building_with_target() {
        let mut model = model_with_library();
        update(
            &mut model,
            Event::Session(SessionEvent::StartBuildingWithTarget {
                target_duration_mins: 20,
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.target_duration_mins, Some(20));
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_start_building_with_target_when_already_building() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::StartBuildingWithTarget {
                target_duration_mins: 15,
            }),
        );

        assert!(model.last_error.is_some());
    }

    #[test]
    fn test_start_building_with_target_out_of_range() {
        let mut model = model_with_library();
        update(
            &mut model,
            Event::Session(SessionEvent::StartBuildingWithTarget {
                target_duration_mins: 0,
            }),
        );

        assert!(model.last_error.is_some());
        assert!(matches!(model.session_status, SessionStatus::Idle));

        update(
            &mut model,
            Event::Session(SessionEvent::StartBuildingWithTarget {
                target_duration_mins: 999,
            }),
        );

        assert!(model.last_error.is_some());
        assert!(matches!(model.session_status, SessionStatus::Idle));
    }

    #[test]
    fn test_start_building_without_target_has_none() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));

        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.target_duration_mins, None);
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_set_target_duration_during_building() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::SetTargetDuration {
                target_duration_mins: Some(20),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.target_duration_mins, Some(20));
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_set_target_duration_clear() {
        let mut model = model_with_library();
        update(
            &mut model,
            Event::Session(SessionEvent::StartBuildingWithTarget {
                target_duration_mins: 15,
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::SetTargetDuration {
                target_duration_mins: None,
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.target_duration_mins, None);
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_set_target_duration_out_of_range() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::SetTargetDuration {
                target_duration_mins: Some(999),
            }),
        );

        assert!(model.last_error.is_some());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.target_duration_mins, None);
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_set_target_duration_when_not_building() {
        let mut model = model_with_library();
        update(
            &mut model,
            Event::Session(SessionEvent::SetTargetDuration {
                target_duration_mins: Some(20),
            }),
        );
        assert!(matches!(model.session_status, SessionStatus::Idle));
        assert!(model.last_error.is_none());
    }

    #[test]
    fn test_add_to_setlist() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries.len(), 1);
            assert_eq!(b.entries[0].item_title, "Moonlight Sonata");
            assert_eq!(b.entries[0].item_type, ItemKind::Piece);
            assert_eq!(b.entries[0].position, 0);
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_add_to_setlist_item_not_found() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "nonexistent".to_string(),
            }),
        );

        assert!(model.last_error.is_some());
    }

    #[test]
    fn test_remove_from_setlist() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-2".to_string(),
            }),
        );

        let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
            b.entries[0].id.clone()
        } else {
            panic!("Expected Building state");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::RemoveFromSetlist { entry_id }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries.len(), 1);
            assert_eq!(b.entries[0].item_title, "Clair de Lune");
            assert_eq!(b.entries[0].position, 0); // Re-indexed
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_reorder_setlist() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-2".to_string(),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "exercise-1".to_string(),
            }),
        );

        let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
            b.entries[2].id.clone() // exercise-1 at position 2
        } else {
            panic!("Expected Building state");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::ReorderSetlist {
                entry_id,
                new_position: 0,
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries[0].item_title, "C Major Scale");
            assert_eq!(b.entries[1].item_title, "Moonlight Sonata");
            assert_eq!(b.entries[2].item_title, "Clair de Lune");
            assert_eq!(b.entries[0].position, 0);
            assert_eq!(b.entries[1].position, 1);
            assert_eq!(b.entries[2].position, 2);
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_start_session_empty_setlist() {
        let mut model = model_with_library();
        let now = Utc::now();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::StartSession { now }),
        );

        assert!(model.last_error.is_some());
        assert!(matches!(model.session_status, SessionStatus::Building(_)));
    }

    #[test]
    fn test_start_session_with_items() {
        let mut model = model_with_library();
        let now = Utc::now();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::StartSession { now }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Active(ref active) = model.session_status {
            assert_eq!(active.current_index, 0);
            assert_eq!(active.entries.len(), 1);
            assert_eq!(active.session_started_at, now);
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_cancel_building() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(&mut model, Event::Session(SessionEvent::CancelBuilding));

        assert!(model.last_error.is_none());
        assert!(matches!(model.session_status, SessionStatus::Idle));
    }

    #[test]
    fn test_cancel_building_when_idle_is_noop_success() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::CancelBuilding));

        assert!(model.last_error.is_none());
        assert!(matches!(model.session_status, SessionStatus::Idle));
    }

    #[test]
    fn test_cancel_building_is_idempotent_when_called_twice() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(&mut model, Event::Session(SessionEvent::CancelBuilding));
        update(&mut model, Event::Session(SessionEvent::CancelBuilding));

        assert!(model.last_error.is_none());
        assert!(matches!(model.session_status, SessionStatus::Idle));
    }

    #[test]
    fn test_cancel_building_from_active_is_a_wrong_state_error() {
        // Cancelling the builder must not silently nuke an Active session (#944).
        let (mut model, _) = model_with_active_session(2);

        update(&mut model, Event::Session(SessionEvent::CancelBuilding));

        assert_eq!(model.last_error.as_deref(), Some("Not in building state"));
        assert!(matches!(model.session_status, SessionStatus::Active(_)));
    }

    // --- Active Phase Tests ---

    fn model_with_active_session(item_count: usize) -> (Model, DateTime<Utc>) {
        let mut model = model_with_library();
        let now = Utc::now();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));

        let items = ["piece-1", "piece-2", "exercise-1"];
        for item_id in items.iter().take(item_count.min(3)) {
            update(
                &mut model,
                Event::Session(SessionEvent::AddToSetlist {
                    item_id: (*item_id).to_string(),
                }),
            );
        }

        update(
            &mut model,
            Event::Session(SessionEvent::StartSession { now }),
        );
        (model, now)
    }

    #[test]
    fn test_next_item() {
        let (mut model, start) = model_with_active_session(3);
        let now = start + chrono::Duration::seconds(30);

        update(&mut model, Event::Session(SessionEvent::NextItem { now }));

        assert!(model.last_error.is_none());
        if let SessionStatus::Active(ref active) = model.session_status {
            assert_eq!(active.current_index, 1);
            assert_eq!(active.entries[0].duration_secs, 30);
            assert_eq!(active.entries[0].status, EntryStatus::Completed);
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_next_item_on_last_transitions_to_summary() {
        let (mut model, start) = model_with_active_session(1);
        let now = start + chrono::Duration::seconds(60);

        update(&mut model, Event::Session(SessionEvent::NextItem { now }));

        assert!(model.last_error.is_none());
        assert!(matches!(model.session_status, SessionStatus::Summary(_)));
    }

    #[test]
    fn test_finish_session() {
        let (mut model, start) = model_with_active_session(2);
        let t1 = start + chrono::Duration::seconds(30);
        let t2 = t1 + chrono::Duration::seconds(45);

        update(
            &mut model,
            Event::Session(SessionEvent::NextItem { now: t1 }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now: t2 }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Summary(ref summary) = model.session_status {
            assert_eq!(summary.entries[0].duration_secs, 30);
            assert_eq!(summary.entries[1].duration_secs, 45);
            assert_eq!(summary.completion_status, CompletionStatus::Completed);
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_end_session_early() {
        let (mut model, start) = model_with_active_session(3);
        let t1 = start + chrono::Duration::seconds(30);
        let t2 = t1 + chrono::Duration::seconds(20);

        update(
            &mut model,
            Event::Session(SessionEvent::NextItem { now: t1 }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::EndSessionEarly { now: t2 }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Summary(ref summary) = model.session_status {
            assert_eq!(summary.entries[0].duration_secs, 30);
            assert_eq!(summary.entries[0].status, EntryStatus::Completed);
            assert_eq!(summary.entries[1].duration_secs, 20);
            assert_eq!(summary.entries[1].status, EntryStatus::Completed);
            assert_eq!(summary.entries[2].duration_secs, 0);
            assert_eq!(summary.entries[2].status, EntryStatus::NotAttempted);
            assert_eq!(summary.completion_status, CompletionStatus::EndedEarly);
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_skip_item() {
        let (mut model, start) = model_with_active_session(3);
        let now = start + chrono::Duration::seconds(10);

        update(&mut model, Event::Session(SessionEvent::SkipItem { now }));

        assert!(model.last_error.is_none());
        if let SessionStatus::Active(ref active) = model.session_status {
            assert_eq!(active.current_index, 1);
            assert_eq!(active.entries[0].duration_secs, 0);
            assert_eq!(active.entries[0].status, EntryStatus::Skipped);
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_skip_last_item_transitions_to_summary() {
        let (mut model, start) = model_with_active_session(1);
        let now = start + chrono::Duration::seconds(10);

        update(&mut model, Event::Session(SessionEvent::SkipItem { now }));

        assert!(model.last_error.is_none());
        if let SessionStatus::Summary(ref summary) = model.session_status {
            assert_eq!(summary.entries[0].status, EntryStatus::Skipped);
            assert_eq!(summary.entries[0].duration_secs, 0);
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_add_item_mid_session() {
        let (mut model, _start) = model_with_active_session(2);

        update(
            &mut model,
            Event::Session(SessionEvent::AddItemMidSession {
                item_id: "exercise-1".to_string(),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Active(ref active) = model.session_status {
            assert_eq!(active.entries.len(), 3);
            assert_eq!(active.entries[2].item_title, "C Major Scale");
            assert_eq!(active.current_index, 0); // Timer not interrupted
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_add_new_item_mid_session() {
        let (mut model, _start) = model_with_active_session(2);

        update(
            &mut model,
            Event::Session(SessionEvent::AddNewItemMidSession {
                title: "New Scale".to_string(),
                item_type: ItemKind::Exercise,
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Active(ref active) = model.session_status {
            assert_eq!(active.entries.len(), 3);
            assert_eq!(active.entries[2].item_title, "New Scale");
        } else {
            panic!("Expected Active state");
        }
        // Verify item was added to library (3 original + 1 new)
        assert_eq!(model.items.len(), 4);
    }

    // --- Summary Phase Tests ---

    fn model_with_summary() -> Model {
        let (mut model, start) = model_with_active_session(2);
        let t1 = start + chrono::Duration::seconds(30);
        let t2 = t1 + chrono::Duration::seconds(45);

        update(
            &mut model,
            Event::Session(SessionEvent::NextItem { now: t1 }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now: t2 }),
        );
        model
    }

    #[test]
    fn test_update_entry_notes() {
        let mut model = model_with_summary();

        let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
            s.entries[0].id.clone()
        } else {
            panic!("Expected Summary state");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryNotes {
                entry_id,
                notes: Some("Needs more practice".to_string()),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].notes, Some("Needs more practice".to_string()));
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_update_entry_notes_too_long() {
        let mut model = model_with_summary();

        let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
            s.entries[0].id.clone()
        } else {
            panic!("Expected Summary state");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryNotes {
                entry_id,
                notes: Some("x".repeat(5001)),
            }),
        );

        assert!(model.last_error.is_some());
    }

    #[test]
    fn test_update_session_reflection_sets_each_field_in_summary() {
        let mut model = model_with_summary();

        for (field, text) in [
            (ReflectionField::Improved, "Thumb-unders even at 92"),
            (ReflectionField::StillRough, "Bars 12-14 rush past 88"),
            (
                ReflectionField::NextTarget,
                "Bars 12-14 at 80, hands together",
            ),
        ] {
            update(
                &mut model,
                Event::Session(SessionEvent::UpdateSessionReflection {
                    field,
                    text: Some(text.to_string()),
                }),
            );
            assert!(model.last_error.is_none());
        }

        let SessionStatus::Summary(ref s) = model.session_status else {
            panic!("Expected Summary state");
        };
        assert_eq!(
            s.reflection_improved,
            Some("Thumb-unders even at 92".to_string())
        );
        assert_eq!(
            s.reflection_still_rough,
            Some("Bars 12-14 rush past 88".to_string())
        );
        assert_eq!(
            s.reflection_next_target,
            Some("Bars 12-14 at 80, hands together".to_string())
        );
    }

    #[test]
    fn test_update_session_reflection_rejected_outside_summary() {
        let (mut model, _start) = model_with_active_session(2);

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateSessionReflection {
                field: ReflectionField::Improved,
                text: Some("mid-session thought".to_string()),
            }),
        );

        assert_eq!(model.last_error, Some("Not in summary state".to_string()));
        let SessionStatus::Active(_) = model.session_status else {
            panic!("Active session must be untouched");
        };
    }

    #[test]
    fn test_update_session_reflection_blank_normalises_to_none() {
        let mut model = model_with_summary();

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateSessionReflection {
                field: ReflectionField::Improved,
                text: Some("real note".to_string()),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateSessionReflection {
                field: ReflectionField::Improved,
                text: Some("   ".to_string()),
            }),
        );

        assert!(model.last_error.is_none());
        let SessionStatus::Summary(ref s) = model.session_status else {
            panic!("Expected Summary state");
        };
        assert_eq!(s.reflection_improved, None);
    }

    #[test]
    fn test_update_session_reflection_trims_retained_text() {
        let mut model = model_with_summary();

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateSessionReflection {
                field: ReflectionField::NextTarget,
                text: Some("  bridge at 80  ".to_string()),
            }),
        );

        let SessionStatus::Summary(ref s) = model.session_status else {
            panic!("Expected Summary state");
        };
        assert_eq!(s.reflection_next_target, Some("bridge at 80".to_string()));
    }

    #[test]
    fn test_update_session_reflection_over_cap_rejected() {
        let mut model = model_with_summary();

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateSessionReflection {
                field: ReflectionField::NextTarget,
                text: Some("x".repeat(validation::MAX_REFLECTION + 1)),
            }),
        );

        assert!(model.last_error.is_some());
        let SessionStatus::Summary(ref s) = model.session_status else {
            panic!("Expected Summary state");
        };
        assert_eq!(s.reflection_next_target, None);
    }

    #[test]
    fn test_save_session_carries_reflections() {
        let mut model = model_with_summary();

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateSessionReflection {
                field: ReflectionField::StillRough,
                text: Some("left hand collapses in the bridge".to_string()),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::SaveSession { now: Utc::now() }),
        );

        assert_eq!(model.sessions.len(), 1);
        assert_eq!(model.sessions[0].reflection_improved, None);
        assert_eq!(
            model.sessions[0].reflection_still_rough,
            Some("left hand collapses in the bridge".to_string())
        );
        assert_eq!(model.sessions[0].reflection_next_target, None);
    }

    #[test]
    fn test_update_session_notes() {
        let mut model = model_with_summary();

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateSessionNotes {
                notes: Some("Great practice today".to_string()),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.session_notes, Some("Great practice today".to_string()));
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_save_session() {
        let mut model = model_with_summary();

        let now = Utc::now();
        update(
            &mut model,
            Event::Session(SessionEvent::SaveSession { now }),
        );

        assert!(model.last_error.is_none());
        assert!(matches!(model.session_status, SessionStatus::Idle));
        assert_eq!(model.sessions.len(), 1);
        assert_eq!(model.sessions[0].total_duration_secs, 75); // 30 + 45
        assert_eq!(
            model.sessions[0].completion_status,
            CompletionStatus::Completed
        );
    }

    #[test]
    fn local_first_save_session_persists_and_skips_http() {
        let mut model = model_with_summary();
        model.local_first = true;
        let app = Intrada;
        let mut cmd = app.update(
            Event::Session(SessionEvent::SaveSession { now: Utc::now() }),
            &mut model,
        );
        let id = model.sessions[0].id.clone();
        assert!(
            cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
                if matches!(&req.operation, crate::persistence::PersistenceOperation::SaveSession(s) if s.id == id))),
            "local-first save persists the session locally"
        );
        assert!(
            !cmd.effects().any(|e| matches!(e, Effect::Http(_))),
            "local-first save makes no HTTP request"
        );
    }

    #[test]
    fn online_save_session_uses_http_not_persistence() {
        let mut model = model_with_summary();
        let app = Intrada;
        let mut cmd = app.update(
            Event::Session(SessionEvent::SaveSession { now: Utc::now() }),
            &mut model,
        );
        assert!(
            cmd.effects().any(|e| matches!(e, Effect::Http(_))),
            "online save POSTs to the server"
        );
        assert!(
            !cmd.effects().any(|e| matches!(e, Effect::Persistence(_))),
            "online save makes no local persistence write"
        );
    }

    #[test]
    fn test_save_session_updates_practice_summaries_in_view() {
        // Regression test for #247: practice data must be visible in the
        // ViewModel immediately after SaveSession, without a re-fetch.
        let mut model = model_with_summary();

        // Score the first entry before saving
        if let SessionStatus::Summary(ref mut summary) = model.session_status {
            summary.entries[0].score = Some(4);
        }

        let now = Utc::now();
        update(
            &mut model,
            Event::Session(SessionEvent::SaveSession { now }),
        );

        // The model should have updated practice_summaries
        assert!(
            !model.practice_summaries.is_empty(),
            "practice_summaries should be populated after save"
        );

        // Build the view — this is what the shell sees
        let app = crate::app::Intrada;
        let vm = <crate::app::Intrada as crux_core::App>::view(&app, &model);

        // Find the item that was practised (piece-1)
        let practised_item = vm.items.iter().find(|i| i.id == "piece-1");
        assert!(
            practised_item.is_some(),
            "piece-1 should be in the ViewModel"
        );

        let practice = practised_item.unwrap().practice.as_ref();
        assert!(
            practice.is_some(),
            "piece-1 should have practice data after SaveSession"
        );

        let practice = practice.unwrap();
        assert_eq!(practice.session_count, 1);
        assert_eq!(practice.latest_score, Some(4));
    }

    #[test]
    fn test_discard_session() {
        let mut model = model_with_summary();

        update(&mut model, Event::Session(SessionEvent::DiscardSession));

        assert!(model.last_error.is_none());
        assert!(matches!(model.session_status, SessionStatus::Idle));
        assert!(model.sessions.is_empty());
    }

    #[test]
    fn test_abandon_session_from_active() {
        let (mut model, _) = model_with_active_session(2);

        update(&mut model, Event::Session(SessionEvent::AbandonSession));

        assert!(model.last_error.is_none());
        assert!(matches!(model.session_status, SessionStatus::Idle));
    }

    #[test]
    fn test_abandon_session_not_active() {
        let mut model = model_with_library();

        update(&mut model, Event::Session(SessionEvent::AbandonSession));

        assert_eq!(
            model.last_error.as_deref(),
            Some("No active practice to abandon")
        );
    }

    // --- Recovery Tests ---

    #[test]
    fn test_recover_session() {
        let mut model = model_with_library();
        let now = Utc::now();

        let active = ActiveSession {
            id: "recovered-session".to_string(),
            entries: vec![create_entry(
                "piece-1",
                "Moonlight Sonata",
                ItemKind::Piece,
                0,
            )],
            current_index: 0,
            current_item_started_at: now,
            session_started_at: now,
            session_intention: None,
        };

        update(
            &mut model,
            Event::Session(SessionEvent::RecoverSession {
                session: active,
                now,
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.id, "recovered-session");
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_recover_session_reanchors_current_item_timer() {
        let mut model = model_with_library();
        let started_yesterday = Utc::now() - chrono::Duration::hours(20);
        let now = Utc::now();

        let active = ActiveSession {
            id: "stale-session".to_string(),
            entries: vec![create_entry(
                "piece-1",
                "Moonlight Sonata",
                ItemKind::Piece,
                0,
            )],
            current_index: 0,
            current_item_started_at: started_yesterday,
            session_started_at: started_yesterday,
            session_intention: None,
        };

        update(
            &mut model,
            Event::Session(SessionEvent::RecoverSession {
                session: active,
                now,
            }),
        );

        let SessionStatus::Active(ref a) = model.session_status else {
            panic!("Expected Active state");
        };
        assert_eq!(
            a.current_item_started_at, now,
            "resume must re-anchor the running item's timer, not show 20h elapsed"
        );
        assert_eq!(
            a.session_started_at, started_yesterday,
            "the session's historical start stays untouched"
        );
    }

    #[test]
    fn test_recover_session_when_not_idle() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));

        let now = Utc::now();
        let active = ActiveSession {
            id: "recovered".to_string(),
            entries: vec![],
            current_index: 0,
            current_item_started_at: now,
            session_started_at: now,
            session_intention: None,
        };

        update(
            &mut model,
            Event::Session(SessionEvent::RecoverSession {
                session: active,
                now,
            }),
        );

        assert!(model.last_error.is_some());
    }

    // --- Delete Session Test ---

    #[test]
    fn test_delete_session() {
        let mut model = model_with_summary();
        let now = Utc::now();

        update(
            &mut model,
            Event::Session(SessionEvent::SaveSession { now }),
        );

        let session_id = model.sessions[0].id.clone();
        update(
            &mut model,
            Event::Session(SessionEvent::DeleteSession { id: session_id }),
        );

        assert!(model.last_error.is_none());
        assert!(model.sessions.is_empty());
    }

    #[test]
    fn test_delete_session_not_found() {
        let mut model = model_with_library();

        update(
            &mut model,
            Event::Session(SessionEvent::DeleteSession {
                id: "nonexistent".to_string(),
            }),
        );

        assert!(model.last_error.is_some());
    }

    // --- Edge Case Tests ---

    #[test]
    fn test_all_items_skipped() {
        let (mut model, start) = model_with_active_session(2);
        let t1 = start + chrono::Duration::seconds(5);
        let t2 = t1 + chrono::Duration::seconds(5);

        update(
            &mut model,
            Event::Session(SessionEvent::SkipItem { now: t1 }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::SkipItem { now: t2 }),
        );

        assert!(matches!(model.session_status, SessionStatus::Summary(_)));

        let save_time = Utc::now();
        update(
            &mut model,
            Event::Session(SessionEvent::SaveSession { now: save_time }),
        );

        assert_eq!(model.sessions.len(), 1);
        assert_eq!(model.sessions[0].total_duration_secs, 0);
    }

    #[test]
    fn test_single_item_setlist() {
        let (mut model, start) = model_with_active_session(1);
        let now = start + chrono::Duration::seconds(120);

        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries.len(), 1);
            assert_eq!(s.entries[0].duration_secs, 120);
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_complete_lifecycle() {
        let mut model = model_with_library();
        let t0 = Utc::now();

        // 1. Start building
        update(&mut model, Event::Session(SessionEvent::StartBuilding));

        // 2. Add 3 items
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-2".to_string(),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "exercise-1".to_string(),
            }),
        );

        // 3. Start session
        update(
            &mut model,
            Event::Session(SessionEvent::StartSession { now: t0 }),
        );

        // 4. Practice first item for 30s, then Next
        let t1 = t0 + chrono::Duration::seconds(30);
        update(
            &mut model,
            Event::Session(SessionEvent::NextItem { now: t1 }),
        );

        // 5. Skip second item
        let t2 = t1 + chrono::Duration::seconds(5);
        update(
            &mut model,
            Event::Session(SessionEvent::SkipItem { now: t2 }),
        );

        // 6. Finish on third item after 60s
        let t3 = t2 + chrono::Duration::seconds(60);
        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now: t3 }),
        );

        // 7. Add notes
        let entry_id_0 = if let SessionStatus::Summary(ref s) = model.session_status {
            s.entries[0].id.clone()
        } else {
            panic!("Expected Summary");
        };
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryNotes {
                entry_id: entry_id_0,
                notes: Some("Good tempo control".to_string()),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateSessionNotes {
                notes: Some("Focused session".to_string()),
            }),
        );

        // 8. Save
        let t_save = Utc::now();
        update(
            &mut model,
            Event::Session(SessionEvent::SaveSession { now: t_save }),
        );

        // Verify final state
        assert!(matches!(model.session_status, SessionStatus::Idle));
        assert_eq!(model.sessions.len(), 1);

        let session = &model.sessions[0];
        assert_eq!(session.entries.len(), 3);
        assert_eq!(session.entries[0].duration_secs, 30);
        assert_eq!(session.entries[0].status, EntryStatus::Completed);
        assert_eq!(
            session.entries[0].notes,
            Some("Good tempo control".to_string())
        );
        assert_eq!(session.entries[1].duration_secs, 0);
        assert_eq!(session.entries[1].status, EntryStatus::Skipped);
        assert_eq!(session.entries[2].duration_secs, 60);
        assert_eq!(session.entries[2].status, EntryStatus::Completed);
        assert_eq!(session.session_notes, Some("Focused session".to_string()));
        assert_eq!(session.total_duration_secs, 90); // 30 + 0 + 60
        assert_eq!(session.completion_status, CompletionStatus::Completed);
    }

    // --- UpdateEntryScore Tests ---

    #[test]
    fn test_update_entry_score_on_completed_entry() {
        let mut model = model_with_summary();

        let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
            s.entries[0].id.clone()
        } else {
            panic!("Expected Summary state");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryScore {
                entry_id: entry_id.clone(),
                score: Some(4),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].score, Some(4));
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_update_entry_score_toggle_clears_score() {
        let mut model = model_with_summary();

        let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
            s.entries[0].id.clone()
        } else {
            panic!("Expected Summary state");
        };

        // Set score to 4
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryScore {
                entry_id: entry_id.clone(),
                score: Some(4),
            }),
        );

        // Clear score by setting to None (toggle)
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryScore {
                entry_id: entry_id.clone(),
                score: None,
            }),
        );

        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].score, None);
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_update_entry_score_ignored_on_skipped_entry() {
        // Create a session where one item is skipped
        let (mut model, start) = model_with_active_session(2);
        let t1 = start + chrono::Duration::seconds(5);
        let t2 = t1 + chrono::Duration::seconds(30);

        // Skip the first item
        update(
            &mut model,
            Event::Session(SessionEvent::SkipItem { now: t1 }),
        );
        // Finish the second item
        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now: t2 }),
        );

        let skipped_entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].status, EntryStatus::Skipped);
            s.entries[0].id.clone()
        } else {
            panic!("Expected Summary state");
        };

        // Try to score the skipped entry — should be a no-op
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryScore {
                entry_id: skipped_entry_id.clone(),
                score: Some(3),
            }),
        );

        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].score, None); // Score not set
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_update_entry_score_out_of_range_rejected() {
        let mut model = model_with_summary();

        let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
            s.entries[0].id.clone()
        } else {
            panic!("Expected Summary state");
        };

        // Score 0 — out of range
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryScore {
                entry_id: entry_id.clone(),
                score: Some(0),
            }),
        );

        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].score, None); // Score not set
        }

        // Score 11 — out of range
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryScore {
                entry_id: entry_id.clone(),
                score: Some(11),
            }),
        );

        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].score, None); // Score still not set
        }
    }

    #[test]
    fn test_update_entry_score_rejected_on_pending_entry() {
        // The current item (the one in progress) has status NotAttempted
        // until NextItem / SkipItem flips it. Scoring it would let the user
        // rate work they haven't done — invariant: only Completed entries
        // can be scored, regardless of session phase.
        let (mut model, _start) = model_with_active_session(2);

        let entry_id = if let SessionStatus::Active(ref a) = model.session_status {
            a.entries[0].id.clone()
        } else {
            panic!("Expected Active state");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryScore {
                entry_id: entry_id.clone(),
                score: Some(3),
            }),
        );

        // Score not set — entry is still NotAttempted
        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].score, None);
            assert_eq!(a.entries[0].status, EntryStatus::NotAttempted);
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_update_entry_score_works_mid_session_on_completed_entry() {
        // The mid-session reflection sheet's flow: NextItem flips the
        // just-completed entry to Completed, THEN the sheet dispatches
        // UpdateEntryScore for that entry. Session is still Active (we're
        // moving on to item 2, not finishing). The score should land.
        let (mut model, start) = model_with_active_session(2);
        let t1 = start + chrono::Duration::seconds(45);

        // Advance — entry[0] becomes Completed, current_index moves to 1
        update(
            &mut model,
            Event::Session(SessionEvent::NextItem { now: t1 }),
        );

        // Capture the just-completed entry id (still in Active phase)
        let entry_id = if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].status, EntryStatus::Completed);
            assert_eq!(a.current_index, 1);
            a.entries[0].id.clone()
        } else {
            panic!("Expected Active state — only one of two items advanced");
        };

        // Score the just-completed entry mid-session
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryScore {
                entry_id: entry_id.clone(),
                score: Some(4),
            }),
        );

        // Score persisted, session still Active
        assert!(model.last_error.is_none());
        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].score, Some(4));
        } else {
            panic!("Expected Active state — session shouldn't have ended");
        }
    }

    #[test]
    fn test_mid_session_entry_score_survives_into_summary() {
        // A per-entry score set mid-session must still be present once the
        // session finishes and projects into the Summary — the reconciliation
        // the reflection hand-off (Phase 6) relies on.
        let (mut model, start) = model_with_active_session(2);
        let t1 = start + chrono::Duration::seconds(45);
        let t2 = t1 + chrono::Duration::seconds(30);

        update(
            &mut model,
            Event::Session(SessionEvent::NextItem { now: t1 }),
        );
        let entry_id = if let SessionStatus::Active(ref a) = model.session_status {
            a.entries[0].id.clone()
        } else {
            panic!("Expected Active state after advancing one of two items");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryScore {
                entry_id,
                score: Some(4),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now: t2 }),
        );

        if let SessionStatus::Summary(ref summary) = model.session_status {
            assert_eq!(summary.entries[0].score, Some(4));
        } else {
            panic!("Expected Summary state after finishing the session");
        }
    }

    #[test]
    fn test_update_entry_tempo_works_mid_session_on_completed_entry() {
        let (mut model, start) = model_with_active_session(2);
        let t1 = start + chrono::Duration::seconds(45);

        update(
            &mut model,
            Event::Session(SessionEvent::NextItem { now: t1 }),
        );

        let entry_id = if let SessionStatus::Active(ref a) = model.session_status {
            a.entries[0].id.clone()
        } else {
            panic!("Expected Active state");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryTempo {
                entry_id,
                tempo: Some(120),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].achieved_tempo, Some(120));
        }
    }

    #[test]
    fn test_update_entry_notes_works_mid_session_on_completed_entry() {
        let (mut model, start) = model_with_active_session(2);
        let t1 = start + chrono::Duration::seconds(45);

        update(
            &mut model,
            Event::Session(SessionEvent::NextItem { now: t1 }),
        );

        let entry_id = if let SessionStatus::Active(ref a) = model.session_status {
            a.entries[0].id.clone()
        } else {
            panic!("Expected Active state");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryNotes {
                entry_id,
                notes: Some("felt solid".to_string()),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].notes.as_deref(), Some("felt solid"));
        }
    }

    #[test]
    fn test_update_entry_score_works_on_last_item_after_finish_session() {
        // The last-item path through the reflection sheet: NextItem to the
        // final item, FinishSession → transitions to Summary, then the sheet
        // dispatches UpdateEntryScore. `transition_to_summary` clones entries
        // into `summary.entries`, so the entry id must still resolve via
        // `entry_for_update_mut`. Pinning this so a future refactor of the
        // transition can't silently drop scoring on the last item.
        let (mut model, start) = model_with_active_session(2);
        let t1 = start + chrono::Duration::seconds(45);
        let t2 = t1 + chrono::Duration::seconds(60);

        // Advance to the last item, then finish the session
        update(
            &mut model,
            Event::Session(SessionEvent::NextItem { now: t1 }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now: t2 }),
        );

        // Should be in Summary phase now
        let last_entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[1].status, EntryStatus::Completed);
            s.entries[1].id.clone()
        } else {
            panic!("Expected Summary state after FinishSession");
        };

        // Score the last item — same code path the reflection sheet takes
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryScore {
                entry_id: last_entry_id,
                score: Some(5),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[1].score, Some(5));
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_update_entry_score_unknown_entry_id_is_silent_noop() {
        // The sheet snapshots the entry id at open time. If the session is
        // cleared (recovery, new session) before Continue, the id won't
        // match anything in the new model. `entry_for_update_mut` returns
        // None — the dispatch must be silent (no last_error, no panic).
        let mut model = model_with_summary();

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryScore {
                entry_id: "no-such-entry".to_string(),
                score: Some(3),
            }),
        );

        // Same shape for tempo
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryTempo {
                entry_id: "no-such-entry".to_string(),
                tempo: Some(120),
            }),
        );

        // Score and tempo handlers don't surface an error for unknown ids
        // (they're silent no-ops, matching the existing pattern).
        assert!(model.last_error.is_none());
    }

    #[test]
    fn test_update_entry_score_rejected_on_skipped_entry_in_active_phase() {
        // SkipItem produces EntryStatus::Skipped, not Completed. The
        // status gate in the handlers should reject scoring a skipped
        // entry mid-session, just as it does in the summary phase.
        let (mut model, start) = model_with_active_session(2);
        let t1 = start + chrono::Duration::seconds(5);

        update(
            &mut model,
            Event::Session(SessionEvent::SkipItem { now: t1 }),
        );

        let skipped_entry_id = if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].status, EntryStatus::Skipped);
            assert_eq!(a.current_index, 1, "Should have advanced past skipped item");
            a.entries[0].id.clone()
        } else {
            panic!("Expected Active state — second item should still be running");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryScore {
                entry_id: skipped_entry_id,
                score: Some(3),
            }),
        );

        // Score not applied — entry is Skipped, not Completed
        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].score, None);
            assert_eq!(a.entries[0].status, EntryStatus::Skipped);
        }
    }

    #[test]
    fn test_update_entry_score_boundary_values() {
        let mut model = model_with_summary();

        let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
            s.entries[0].id.clone()
        } else {
            panic!("Expected Summary state");
        };

        // Score 1 — minimum valid
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryScore {
                entry_id: entry_id.clone(),
                score: Some(1),
            }),
        );

        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].score, Some(1));
        }

        // Score 10 — maximum valid
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryScore {
                entry_id: entry_id.clone(),
                score: Some(10),
            }),
        );

        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].score, Some(10));
        }
    }

    // --- UpdateEntryTempo Tests ---

    #[test]
    fn test_update_entry_tempo_on_completed_entry() {
        let mut model = model_with_summary();

        let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
            s.entries[0].id.clone()
        } else {
            panic!("Expected Summary state");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryTempo {
                entry_id: entry_id.clone(),
                tempo: Some(120),
            }),
        );

        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].achieved_tempo, Some(120));
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_update_entry_tempo_none_clears() {
        let mut model = model_with_summary();

        let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
            s.entries[0].id.clone()
        } else {
            panic!("Expected Summary state");
        };

        // Set tempo to 120
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryTempo {
                entry_id: entry_id.clone(),
                tempo: Some(120),
            }),
        );

        // Clear tempo by setting to None
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryTempo {
                entry_id: entry_id.clone(),
                tempo: None,
            }),
        );

        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].achieved_tempo, None);
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_update_entry_tempo_rejected_on_skipped() {
        // Build a summary with a skipped entry: skip item 1, complete item 2
        let (mut model, start) = model_with_active_session(2);
        let t1 = start + chrono::Duration::seconds(10);
        let t2 = t1 + chrono::Duration::seconds(30);

        // Skip first item
        update(
            &mut model,
            Event::Session(SessionEvent::SkipItem { now: t1 }),
        );
        // Finish session (completes second item, transitions to summary)
        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now: t2 }),
        );

        // Find the skipped entry
        let skipped_entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
            s.entries
                .iter()
                .find(|e| e.status == EntryStatus::Skipped)
                .expect("Should have a skipped entry")
                .id
                .clone()
        } else {
            panic!("Expected Summary state");
        };

        // Try to set tempo on the skipped entry — should be a no-op
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryTempo {
                entry_id: skipped_entry_id.clone(),
                tempo: Some(100),
            }),
        );

        if let SessionStatus::Summary(ref s) = model.session_status {
            let skipped = s.entries.iter().find(|e| e.id == skipped_entry_id).unwrap();
            assert_eq!(skipped.achieved_tempo, None);
        }
    }

    #[test]
    fn test_update_entry_tempo_rejected_out_of_range() {
        let mut model = model_with_summary();

        let entry_id = if let SessionStatus::Summary(ref s) = model.session_status {
            s.entries[0].id.clone()
        } else {
            panic!("Expected Summary state");
        };

        // Tempo 0 — out of range
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryTempo {
                entry_id: entry_id.clone(),
                tempo: Some(0),
            }),
        );

        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].achieved_tempo, None);
        }

        // Tempo 501 — out of range
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryTempo {
                entry_id: entry_id.clone(),
                tempo: Some(501),
            }),
        );

        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].achieved_tempo, None);
        }
    }

    // --- format_duration_display Tests ---

    #[test]
    fn test_format_duration_seconds_only() {
        assert_eq!(format_duration_display(0), "0s");
        assert_eq!(format_duration_display(45), "45s");
        assert_eq!(format_duration_display(59), "59s");
    }

    #[test]
    fn test_format_duration_minutes_and_seconds() {
        assert_eq!(format_duration_display(60), "1m 0s");
        assert_eq!(format_duration_display(90), "1m 30s");
        assert_eq!(format_duration_display(3599), "59m 59s");
    }

    #[test]
    fn test_format_duration_hours() {
        assert_eq!(format_duration_display(3600), "1h 0m 0s");
        assert_eq!(format_duration_display(3661), "1h 1m 1s");
        assert_eq!(format_duration_display(7200), "2h 0m 0s");
    }

    // --- format_duration_summary Tests ---

    #[test]
    fn test_format_duration_summary() {
        assert_eq!(format_duration_summary(0), "0m");
        assert_eq!(format_duration_summary(45), "0m");
        assert_eq!(format_duration_summary(1800), "30m");
        assert_eq!(format_duration_summary(2700), "45m");
        assert_eq!(format_duration_summary(3600), "1h 0m");
        assert_eq!(format_duration_summary(8100), "2h 15m");
    }

    // --- SessionsData Serialization Test ---

    #[test]
    fn test_sessions_data_serialization() {
        use crate::domain::types::SessionsData;

        let data = SessionsData { sessions: vec![] };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: SessionsData = serde_json::from_str(&json).unwrap();
        assert!(parsed.sessions.is_empty());
    }

    // --- AddNewItemToSetlist Tests ---

    #[test]
    fn test_add_new_item_to_setlist_piece() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddNewItemToSetlist {
                title: "New Piece".to_string(),
                item_type: ItemKind::Piece,
            }),
        );

        assert!(model.last_error.is_none());
        // Verify new item in library (3 original + 1 new)
        assert_eq!(model.items.len(), 4);
        assert_eq!(model.items[3].title, "New Piece");
        // Verify in setlist
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries.len(), 1);
            assert_eq!(b.entries[0].item_title, "New Piece");
            assert_eq!(b.entries[0].item_type, ItemKind::Piece);
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_add_new_item_to_setlist_exercise() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddNewItemToSetlist {
                title: "New Exercise".to_string(),
                item_type: ItemKind::Exercise,
            }),
        );

        assert!(model.last_error.is_none());
        // 3 original + 1 new
        assert_eq!(model.items.len(), 4);
    }

    // test_add_new_item_invalid_type removed — item_type is now ItemKind enum,
    // invalid values are prevented at compile time.

    // --- Intention Tests ---

    #[test]
    fn test_set_session_intention() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::SetSessionIntention {
                intention: Some("Focus on dynamics".to_string()),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.session_intention, Some("Focus on dynamics".to_string()));
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_set_entry_intention() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );

        let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
            b.entries[0].id.clone()
        } else {
            panic!("Expected Building state");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::SetEntryIntention {
                entry_id,
                intention: Some("Work on left hand".to_string()),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(
                b.entries[0].intention,
                Some("Work on left hand".to_string())
            );
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_intention_too_long() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));

        let long_text = "a".repeat(501);
        update(
            &mut model,
            Event::Session(SessionEvent::SetSessionIntention {
                intention: Some(long_text),
            }),
        );

        assert!(model.last_error.is_some());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.session_intention, None);
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_intention_threaded_to_active() {
        let mut model = model_with_library();
        let now = Utc::now();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );

        // Set session intention
        update(
            &mut model,
            Event::Session(SessionEvent::SetSessionIntention {
                intention: Some("Session intention".to_string()),
            }),
        );

        // Set entry intention
        let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
            b.entries[0].id.clone()
        } else {
            panic!("Expected Building state");
        };
        update(
            &mut model,
            Event::Session(SessionEvent::SetEntryIntention {
                entry_id,
                intention: Some("Entry intention".to_string()),
            }),
        );

        // Start session
        update(
            &mut model,
            Event::Session(SessionEvent::StartSession { now }),
        );

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.session_intention, Some("Session intention".to_string()));
            assert_eq!(a.entries[0].intention, Some("Entry intention".to_string()));
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_intention_threaded_to_summary() {
        let mut model = model_with_library();
        let now = Utc::now();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );

        update(
            &mut model,
            Event::Session(SessionEvent::SetSessionIntention {
                intention: Some("Summary test".to_string()),
            }),
        );

        let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
            b.entries[0].id.clone()
        } else {
            panic!("Expected Building state");
        };
        update(
            &mut model,
            Event::Session(SessionEvent::SetEntryIntention {
                entry_id,
                intention: Some("Entry summary test".to_string()),
            }),
        );

        update(
            &mut model,
            Event::Session(SessionEvent::StartSession { now }),
        );

        let t1 = now + chrono::Duration::seconds(30);
        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now: t1 }),
        );

        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.session_intention, Some("Summary test".to_string()));
            assert_eq!(
                s.entries[0].intention,
                Some("Entry summary test".to_string())
            );
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_intention_persisted_in_save() {
        let mut model = model_with_library();
        let now = Utc::now();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );

        update(
            &mut model,
            Event::Session(SessionEvent::SetSessionIntention {
                intention: Some("Save test".to_string()),
            }),
        );

        let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
            b.entries[0].id.clone()
        } else {
            panic!("Expected Building state");
        };
        update(
            &mut model,
            Event::Session(SessionEvent::SetEntryIntention {
                entry_id,
                intention: Some("Entry save test".to_string()),
            }),
        );

        update(
            &mut model,
            Event::Session(SessionEvent::StartSession { now }),
        );

        let t1 = now + chrono::Duration::seconds(30);
        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now: t1 }),
        );

        let t2 = t1 + chrono::Duration::seconds(5);
        update(
            &mut model,
            Event::Session(SessionEvent::SaveSession { now: t2 }),
        );

        assert_eq!(model.sessions.len(), 1);
        assert_eq!(
            model.sessions[0].session_intention,
            Some("Save test".to_string())
        );
        assert_eq!(
            model.sessions[0].entries[0].intention,
            Some("Entry save test".to_string())
        );
    }

    #[test]
    fn test_set_intention_outside_building() {
        let (mut model, _start) = model_with_active_session(2);

        // In Active state, try to set session intention — should be no-op
        update(
            &mut model,
            Event::Session(SessionEvent::SetSessionIntention {
                intention: Some("Should not stick".to_string()),
            }),
        );

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.session_intention, None);
        } else {
            panic!("Expected Active state");
        }
    }

    // --- Rep Counter Tests ---

    /// Helper: create an active session with a rep target on the first item.
    fn model_with_active_session_and_rep(target: u8) -> (Model, DateTime<Utc>) {
        let mut model = model_with_library();
        let now = Utc::now();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-2".to_string(),
            }),
        );

        // Set rep target on first entry during building
        if let SessionStatus::Building(ref mut b) = model.session_status {
            b.entries[0].rep_target = Some(target);
        } else {
            panic!("Expected Building state");
        }

        update(
            &mut model,
            Event::Session(SessionEvent::StartSession { now }),
        );
        (model, now)
    }

    #[test]
    fn test_rep_initialized_on_start_session() {
        let (model, _now) = model_with_active_session_and_rep(5);

        if let SessionStatus::Active(ref a) = model.session_status {
            // First entry has rep target → initialized
            assert_eq!(a.entries[0].rep_target, Some(5));
            assert_eq!(a.entries[0].rep_count, Some(0));
            assert_eq!(a.entries[0].rep_target_reached, Some(false));
            // Second entry has no rep target → not initialized
            assert_eq!(a.entries[1].rep_target, None);
            assert_eq!(a.entries[1].rep_count, None);
            assert_eq!(a.entries[1].rep_target_reached, None);
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_rep_got_it_increments() {
        let (mut model, _now) = model_with_active_session_and_rep(5);

        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepGotIt));

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].rep_count, Some(3));
            assert_eq!(a.entries[0].rep_target_reached, Some(false));
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_rep_got_it_reaches_target() {
        let (mut model, _now) = model_with_active_session_and_rep(3);

        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepGotIt));

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].rep_count, Some(3));
            assert_eq!(a.entries[0].rep_target_reached, Some(true));
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_rep_got_it_frozen_after_target_reached() {
        let (mut model, _now) = model_with_active_session_and_rep(3);

        // Reach target
        for _ in 0..3 {
            update(&mut model, Event::Session(SessionEvent::RepGotIt));
        }

        // Additional got-it should not increase count
        update(&mut model, Event::Session(SessionEvent::RepGotIt));

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].rep_count, Some(3));
            assert_eq!(a.entries[0].rep_target_reached, Some(true));
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_rep_missed_decrements() {
        let (mut model, _now) = model_with_active_session_and_rep(5);

        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepGotIt));

        update(&mut model, Event::Session(SessionEvent::RepMissed));

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].rep_count, Some(2));
            assert_eq!(a.entries[0].rep_target_reached, Some(false));
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_rep_missed_floor_zero() {
        let (mut model, _now) = model_with_active_session_and_rep(5);

        // Miss with count at 0
        update(&mut model, Event::Session(SessionEvent::RepMissed));

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].rep_count, Some(0));
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_rep_missed_frozen_after_target_reached() {
        let (mut model, _now) = model_with_active_session_and_rep(3);

        // Reach target
        for _ in 0..3 {
            update(&mut model, Event::Session(SessionEvent::RepGotIt));
        }

        // Miss should not decrement after target reached
        update(&mut model, Event::Session(SessionEvent::RepMissed));

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].rep_count, Some(3));
            assert_eq!(a.entries[0].rep_target_reached, Some(true));
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_enable_rep_counter_sets_default_target() {
        let (mut model, _now) = model_with_active_session(2);

        // Current entry has no rep target
        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].rep_target, None);
        }

        update(&mut model, Event::Session(SessionEvent::InitRepCounter));

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].rep_target, Some(5)); // DEFAULT_REP_TARGET
            assert_eq!(a.entries[0].rep_count, Some(0));
            assert_eq!(a.entries[0].rep_target_reached, Some(false));
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_init_rep_counter_preserves_existing_state() {
        let (mut model, _now) = model_with_active_session_and_rep(7);

        // Record some progress
        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepGotIt));

        // Re-init should preserve existing state (target, count, history)
        update(&mut model, Event::Session(SessionEvent::InitRepCounter));

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].rep_target, Some(7)); // preserved original target
            assert_eq!(a.entries[0].rep_count, Some(2)); // preserved count
            assert_eq!(
                a.entries[0].rep_history,
                Some(vec![RepAction::Success, RepAction::Success])
            ); // preserved history
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_rep_state_frozen_on_next_item() {
        let (mut model, start) = model_with_active_session_and_rep(5);

        // Got-it 3 times (partial progress)
        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepGotIt));

        let now = start + chrono::Duration::seconds(30);
        update(&mut model, Event::Session(SessionEvent::NextItem { now }));

        if let SessionStatus::Active(ref a) = model.session_status {
            // First entry frozen: 3/5, target not reached
            assert_eq!(a.entries[0].rep_count, Some(3));
            assert_eq!(a.entries[0].rep_target_reached, Some(false));
            // Now on second item
            assert_eq!(a.current_index, 1);
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_rep_state_frozen_on_skip_item() {
        let (mut model, start) = model_with_active_session_and_rep(5);

        // Got-it once
        update(&mut model, Event::Session(SessionEvent::RepGotIt));

        let now = start + chrono::Duration::seconds(10);
        update(&mut model, Event::Session(SessionEvent::SkipItem { now }));

        if let SessionStatus::Active(ref a) = model.session_status {
            // First entry frozen: 1/5, target not reached
            assert_eq!(a.entries[0].rep_count, Some(1));
            assert_eq!(a.entries[0].rep_target_reached, Some(false));
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_rep_state_in_summary_after_finish() {
        let (mut model, start) = model_with_active_session_and_rep(3);

        // Reach target
        for _ in 0..3 {
            update(&mut model, Event::Session(SessionEvent::RepGotIt));
        }

        let now = start + chrono::Duration::seconds(60);
        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now }),
        );

        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].rep_target, Some(3));
            assert_eq!(s.entries[0].rep_count, Some(3));
            assert_eq!(s.entries[0].rep_target_reached, Some(true));
        } else {
            panic!("Expected Summary state");
        }
    }

    #[test]
    fn test_rep_state_persisted_through_save() {
        let (mut model, start) = model_with_active_session_and_rep(3);

        // Reach target
        for _ in 0..3 {
            update(&mut model, Event::Session(SessionEvent::RepGotIt));
        }

        let t1 = start + chrono::Duration::seconds(60);
        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now: t1 }),
        );

        let t2 = t1 + chrono::Duration::seconds(5);
        update(
            &mut model,
            Event::Session(SessionEvent::SaveSession { now: t2 }),
        );

        assert_eq!(model.sessions.len(), 1);
        assert_eq!(model.sessions[0].entries[0].rep_target, Some(3));
        assert_eq!(model.sessions[0].entries[0].rep_count, Some(3));
        assert_eq!(model.sessions[0].entries[0].rep_target_reached, Some(true));
    }

    #[test]
    fn test_rep_no_counter_entry_unaffected() {
        let (mut model, _now) = model_with_active_session(2);

        // RepGotIt on entry without counter — no-op
        update(&mut model, Event::Session(SessionEvent::RepGotIt));

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].rep_target, None);
            assert_eq!(a.entries[0].rep_count, None);
            assert_eq!(a.entries[0].rep_target_reached, None);
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_rep_got_it_capped_at_target() {
        let (mut model, _now) = model_with_active_session_and_rep(3);

        // Try to go beyond target
        for _ in 0..10 {
            update(&mut model, Event::Session(SessionEvent::RepGotIt));
        }

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].rep_count, Some(3)); // capped at target
            assert_eq!(a.entries[0].rep_target_reached, Some(true));
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_rep_state_frozen_on_end_session_early() {
        let (mut model, now) = model_with_active_session_and_rep(5);

        // Increment rep count twice on item 1
        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepGotIt));

        // End session early (item 2 is never reached)
        let t1 = now + chrono::Duration::seconds(30);
        update(
            &mut model,
            Event::Session(SessionEvent::EndSessionEarly { now: t1 }),
        );

        if let SessionStatus::Summary(ref s) = model.session_status {
            // Item 1: rep state frozen — 2/5, not reached
            assert_eq!(s.entries[0].rep_target, Some(5));
            assert_eq!(s.entries[0].rep_count, Some(2));
            assert_eq!(s.entries[0].rep_target_reached, Some(false));
            assert_eq!(s.entries[0].status, EntryStatus::Completed);

            // Item 2: no rep target set, marked not_attempted
            assert_eq!(s.entries[1].rep_target, None);
            assert_eq!(s.entries[1].rep_count, None);
            assert_eq!(s.entries[1].rep_target_reached, None);
            assert_eq!(s.entries[1].status, EntryStatus::NotAttempted);
        } else {
            panic!("Expected Summary state");
        }
    }

    // ── SetRepTarget (Building phase) tests ──────────────────────────

    #[test]
    fn test_set_rep_target_in_building() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );

        let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
            b.entries[0].id.clone()
        } else {
            panic!("Expected Building state");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::SetRepTarget {
                entry_id: entry_id.clone(),
                target: Some(7),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries[0].rep_target, Some(7));
            assert_eq!(b.entries[0].rep_count, None);
            assert_eq!(b.entries[0].rep_target_reached, None);
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_set_rep_target_clear() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );

        let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
            b.entries[0].id.clone()
        } else {
            panic!("Expected Building state");
        };

        // Set target first
        update(
            &mut model,
            Event::Session(SessionEvent::SetRepTarget {
                entry_id: entry_id.clone(),
                target: Some(5),
            }),
        );

        // Then clear it
        update(
            &mut model,
            Event::Session(SessionEvent::SetRepTarget {
                entry_id,
                target: None,
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries[0].rep_target, None);
        } else {
            panic!("Expected Building state");
        }
    }

    #[test]
    fn test_set_rep_target_invalid_value() {
        let mut model = model_with_library();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );

        let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
            b.entries[0].id.clone()
        } else {
            panic!("Expected Building state");
        };

        // Target below minimum (3)
        update(
            &mut model,
            Event::Session(SessionEvent::SetRepTarget {
                entry_id: entry_id.clone(),
                target: Some(1),
            }),
        );

        assert!(model.last_error.is_some());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries[0].rep_target, None); // unchanged
        } else {
            panic!("Expected Building state");
        }

        // Target above maximum (10)
        model.last_error = None;
        update(
            &mut model,
            Event::Session(SessionEvent::SetRepTarget {
                entry_id,
                target: Some(15),
            }),
        );

        assert!(model.last_error.is_some());
    }

    #[test]
    fn test_set_rep_target_flows_to_active() {
        let mut model = model_with_library();
        let now = Utc::now();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );

        let entry_id = if let SessionStatus::Building(ref b) = model.session_status {
            b.entries[0].id.clone()
        } else {
            panic!("Expected Building state");
        };

        update(
            &mut model,
            Event::Session(SessionEvent::SetRepTarget {
                entry_id,
                target: Some(8),
            }),
        );

        update(
            &mut model,
            Event::Session(SessionEvent::StartSession { now }),
        );

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].rep_target, Some(8));
            assert_eq!(a.entries[0].rep_count, Some(0)); // initialized
            assert_eq!(a.entries[0].rep_target_reached, Some(false)); // initialized
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_set_rep_target_no_op_outside_building() {
        let (mut model, _now) = model_with_active_session_and_rep(5);

        // SetRepTarget should no-op in Active state
        update(
            &mut model,
            Event::Session(SessionEvent::SetRepTarget {
                entry_id: "whatever".to_string(),
                target: Some(10),
            }),
        );

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].rep_target, Some(5)); // unchanged
        } else {
            panic!("Expected Active state");
        }
    }

    // --- Rep History Tests (US1) ---

    #[test]
    fn test_rep_history_initialised_on_start_session() {
        let (model, _now) = model_with_active_session_and_rep(5);

        if let SessionStatus::Active(ref a) = model.session_status {
            // Entry with rep_target should have rep_history initialised
            assert_eq!(a.entries[0].rep_history, Some(vec![]));
            // Entry without rep_target should have no history
            assert_eq!(a.entries[1].rep_history, None);
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_rep_history_appended_on_got_it() {
        let (mut model, _now) = model_with_active_session_and_rep(5);

        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepGotIt));

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(
                a.entries[0].rep_history,
                Some(vec![RepAction::Success, RepAction::Success])
            );
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_rep_history_appended_on_missed() {
        let (mut model, _now) = model_with_active_session_and_rep(5);

        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepMissed));

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(
                a.entries[0].rep_history,
                Some(vec![RepAction::Success, RepAction::Missed])
            );
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_rep_history_frozen_on_next_item() {
        let (mut model, start) = model_with_active_session_and_rep(5);

        // Record some actions
        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepMissed));
        update(&mut model, Event::Session(SessionEvent::RepGotIt));

        // Move to next item
        let next_time = start + chrono::Duration::seconds(60);
        update(
            &mut model,
            Event::Session(SessionEvent::NextItem { now: next_time }),
        );

        if let SessionStatus::Active(ref a) = model.session_status {
            // First entry's history should be frozen with the recorded actions
            assert_eq!(
                a.entries[0].rep_history,
                Some(vec![
                    RepAction::Success,
                    RepAction::Missed,
                    RepAction::Success
                ])
            );
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_rep_history_none_without_counter() {
        let (model, _now) = model_with_active_session_and_rep(5);

        if let SessionStatus::Active(ref a) = model.session_status {
            // Second entry has no rep target, so no history
            assert_eq!(a.entries[1].rep_target, None);
            assert_eq!(a.entries[1].rep_history, None);
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_rep_history_initialised_on_enable_counter() {
        let mut model = model_with_library();
        let now = Utc::now();
        update(&mut model, Event::Session(SessionEvent::StartBuilding));
        update(
            &mut model,
            Event::Session(SessionEvent::AddToSetlist {
                item_id: "piece-1".to_string(),
            }),
        );
        // Start session WITHOUT rep target set during building
        update(
            &mut model,
            Event::Session(SessionEvent::StartSession { now }),
        );

        // Init counter mid-session
        update(&mut model, Event::Session(SessionEvent::InitRepCounter));

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.entries[0].rep_history, Some(vec![]));
            assert!(a.entries[0].rep_target.is_some());
        } else {
            panic!("Expected Active state");
        }
    }

    #[test]
    fn test_update_session_score_sets_and_validates() {
        let mut model = model_with_summary();

        update(
            &mut model,
            Event::Session(SessionEvent::UpdateSessionScore { score: Some(8) }),
        );
        assert!(model.last_error.is_none());
        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.session_score, Some(8));
        } else {
            panic!("Expected Summary state");
        }

        // Out of range is rejected, leaving the prior value intact.
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateSessionScore { score: Some(11) }),
        );
        assert!(model.last_error.is_none());
        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.session_score, Some(8));
        }
    }

    #[test]
    fn test_rep_history_persisted_through_save() {
        let (mut model, start) = model_with_active_session_and_rep(3);

        // Hit got it 3 times to reach target
        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepGotIt));

        // Finish session
        let end_time = start + chrono::Duration::seconds(120);
        update(
            &mut model,
            Event::Session(SessionEvent::FinishSession { now: end_time }),
        );

        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(
                s.entries[0].rep_history,
                Some(vec![
                    RepAction::Success,
                    RepAction::Success,
                    RepAction::Success
                ])
            );
        } else {
            panic!("Expected Summary state");
        }
    }

    /// `group_id` rides `SetlistEntry` across the bincode FFI wire (in the
    /// persisted `PracticeSession` and the ViewModel). It is the sole input to
    /// the B1 context derivation, so a silent drop on the wire would erase every
    /// piece context. Guard the whole entry, with `group_id` set, against the
    /// #846 class.
    #[test]
    fn setlist_entry_with_group_id_round_trips_on_ffi_bincode_wire() {
        crate::domain::types::assert_round_trips(SetlistEntry {
            id: "e1".to_string(),
            item_id: "ex-1".to_string(),
            item_title: "Scales".to_string(),
            item_type: ItemKind::Exercise,
            position: 0,
            duration_secs: 300,
            status: EntryStatus::Completed,
            notes: Some("warm up".to_string()),
            score: Some(6),
            intention: Some("even tone".to_string()),
            rep_target: Some(5),
            rep_count: Some(5),
            rep_target_reached: Some(true),
            rep_history: Some(vec![RepAction::Success, RepAction::Missed]),
            planned_duration_secs: Some(300),
            achieved_tempo: Some(96),
            group_id: Some("g1".to_string()),
        });
    }
}
