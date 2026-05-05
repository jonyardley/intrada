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
pub enum RepAction {
    /// Failed rep — count decremented.
    Missed,
    /// Successful rep — count incremented.
    Success,
}

// ── Domain Types ───────────────────────────────────────────────────────

/// An individual item within a session's setlist.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
}

/// A completed practice session (persisted to localStorage).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
}

// ── Transient State Types ──────────────────────────────────────────────

/// State during setlist assembly (Building phase).
#[derive(Debug, Clone)]
pub struct BuildingSession {
    pub entries: Vec<SetlistEntry>,
    pub session_intention: Option<String>,
    /// Optional session-level time target (in minutes) set via presets.
    /// Purely a UI guide — not enforced.
    pub target_duration_mins: Option<u32>,
}

/// State during active practice (Active phase).
/// Persisted to `intrada:session-in-progress` for crash recovery.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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
    StartSession {
        now: DateTime<Utc>,
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
    SaveSession {
        now: DateTime<Utc>,
    },
    DiscardSession,

    // === Recovery ===
    RecoverSession {
        session: ActiveSession,
    },

    // === History ===
    DeleteSession {
        id: String,
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

/// Look up a library item by ID and return (title, kind).
fn find_item_in_model(model: &Model, item_id: &str) -> Option<(String, ItemKind)> {
    model
        .items
        .iter()
        .find(|i| i.id == item_id)
        .map(|i| (i.title.clone(), i.kind.clone()))
}

/// Create a new SetlistEntry from a library item lookup.
fn create_entry(
    item_id: &str,
    item_title: &str,
    item_type: ItemKind,
    position: usize,
) -> SetlistEntry {
    SetlistEntry {
        id: ulid::Ulid::new().to_string(),
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
    }
}

/// Re-index entry positions after a mutation.
fn reindex_entries(entries: &mut [SetlistEntry]) {
    for (i, entry) in entries.iter_mut().enumerate() {
        entry.position = i;
    }
}

/// Create a minimal Item from title-only input.
fn create_item_from_title(title: &str, kind: ItemKind) -> Item {
    let now = Utc::now();
    Item {
        id: ulid::Ulid::new().to_string(),
        title: title.to_string(),
        kind,
        composer: None,
        key: None,
        tempo: None,
        notes: None,
        tags: vec![],
        created_at: now,
        updated_at: now,
    }
}

/// Freeze rep state on an entry: set `rep_target_reached` based on whether count >= target.
fn freeze_rep_state(entry: &mut SetlistEntry) {
    if let (Some(target), Some(count)) = (entry.rep_target, entry.rep_count) {
        entry.rep_target_reached = Some(count >= target);
    }
}

/// Find a setlist entry by id, mutably, regardless of which session phase
/// the model is currently in. Returns `None` if not in an Active or Summary
/// phase, or if the entry id is unknown.
///
/// Used by `UpdateEntryScore` / `UpdateEntryTempo` / `UpdateEntryNotes` so the
/// mid-session reflection sheet can write per-entry data the moment the user
/// completes an item, not just on the post-session summary screen.
fn entry_for_update_mut<'a>(model: &'a mut Model, entry_id: &str) -> Option<&'a mut SetlistEntry> {
    match &mut model.session_status {
        SessionStatus::Active(active) => active.entries.iter_mut().find(|e| e.id == entry_id),
        SessionStatus::Summary(summary) => summary.entries.iter_mut().find(|e| e.id == entry_id),
        SessionStatus::Idle | SessionStatus::Building(_) => None,
    }
}

/// Transition from Active to Summary, computing final duration for the current item.
fn transition_to_summary(
    active: &mut ActiveSession,
    now: DateTime<Utc>,
    completion_status: CompletionStatus,
) -> SummarySession {
    // Record duration for current item
    let elapsed = (now - active.current_item_started_at).num_seconds().max(0) as u64;
    if let Some(entry) = active.entries.get_mut(active.current_index) {
        entry.duration_secs = elapsed;
        entry.status = EntryStatus::Completed;
        freeze_rep_state(entry);
    }

    // Mark remaining items as NotAttempted if ending early
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
            model.session_status = SessionStatus::Building(BuildingSession {
                entries: vec![],
                session_intention: None,
                target_duration_mins: None,
            });
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
                entries: vec![],
                session_intention: None,
                target_duration_mins: Some(target_duration_mins),
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

            // Validate target if provided
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
            // Clear all rep state when changing target in building phase
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

            // Validate duration if provided
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

        SessionEvent::AddToSetlist { item_id } => {
            if !matches!(model.session_status, SessionStatus::Building(_)) {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            }

            let Some((title, item_type)) = find_item_in_model(model, &item_id) else {
                model.last_error = Some(LibraryError::NotFound { id: item_id }.to_string());
                return crux_core::render::render();
            };

            let SessionStatus::Building(ref mut building) = model.session_status else {
                model.last_error = Some("Internal error: expected Building state".to_string());
                return crux_core::render::render();
            };
            let position = building.entries.len();
            let entry = create_entry(&item_id, &title, item_type, position);
            building.entries.push(entry);
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
                crate::http::create_item(&model.api_base_url, &item),
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
            // Initialize rep counter state for entries with a rep_target set
            for entry in &mut entries {
                if entry.rep_target.is_some() {
                    entry.rep_count = Some(0);
                    entry.rep_target_reached = Some(false);
                    entry.rep_history = Some(vec![]);
                }
            }

            let active = ActiveSession {
                id: ulid::Ulid::new().to_string(),
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
            if !matches!(model.session_status, SessionStatus::Building(_)) {
                model.last_error = Some("Not in building state".to_string());
                return crux_core::render::render();
            }
            model.session_status = SessionStatus::Idle;
            model.last_error = None;
            crux_core::render::render()
        }

        // ── Active Phase ───────────────────────────────────────────
        SessionEvent::NextItem { now } => {
            let SessionStatus::Active(ref mut active) = model.session_status else {
                model.last_error = Some("Not in active state".to_string());
                return crux_core::render::render();
            };

            // If this was the last item, transition to Summary
            // (transition_to_summary handles duration, status, and freeze)
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

            // If this was the last item, transition to Summary
            if active.current_index >= active.entries.len() - 1 {
                let summary = SummarySession {
                    id: active.id.clone(),
                    entries: active.entries.clone(),
                    session_started_at: active.session_started_at,
                    session_ended_at: now,
                    session_notes: None,
                    session_intention: active.session_intention.clone(),
                    completion_status: CompletionStatus::Completed,
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
                crate::http::create_item(&model.api_base_url, &item),
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

            // Append to rep history (capped at MAX_REP_HISTORY)
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

            // Append to rep history (capped at MAX_REP_HISTORY)
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

            // Only initialise defaults when no prior rep state exists.
            // If rep state already exists (e.g. after hide/show), preserve it.
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
        // These accept dispatches from both phases so the mid-session
        // reflection sheet can record per-entry data the moment the user
        // moves on to the next item, not just retroactively from the
        // post-session summary screen. The core invariant is unchanged:
        // only entries that have actually been Completed can be scored /
        // tempo'd / noted.
        SessionEvent::UpdateEntryScore { entry_id, score } => {
            // Validate score range if present
            if let Some(s) = score {
                if !(validation::MIN_SCORE..=validation::MAX_SCORE).contains(&s) {
                    // Silent no-op for out-of-range score
                    return crux_core::render::render();
                }
            }

            let Some(entry) = entry_for_update_mut(model, &entry_id) else {
                // Silent no-op if not in a session phase that has entries,
                // or if the entry id is unknown.
                return crux_core::render::render();
            };

            // Only allow scoring on completed entries
            if entry.status != EntryStatus::Completed {
                return crux_core::render::render();
            }

            entry.score = score;
            model.last_error = None;
            crux_core::render::render()
        }

        SessionEvent::UpdateEntryTempo { entry_id, tempo } => {
            // Validate tempo range if present
            if let Err(_e) = validation::validate_achieved_tempo(&tempo) {
                // Silent no-op for out-of-range tempo
                return crux_core::render::render();
            }

            let Some(entry) = entry_for_update_mut(model, &entry_id) else {
                return crux_core::render::render();
            };

            // Only allow tempo on completed entries
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
            };

            model.sessions.push(practice_session.clone());
            model.practice_summaries = crate::app::build_practice_summaries(&model.sessions);
            model.session_status = SessionStatus::Idle;
            model.last_error = None;

            Command::all([
                crate::http::create_session(&model.api_base_url, &practice_session),
                Command::notify_shell(AppEffect::ClearSessionInProgress).into(),
                crux_core::render::render(),
            ])
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
        SessionEvent::RecoverSession { session } => {
            if !matches!(model.session_status, SessionStatus::Idle) {
                model.last_error =
                    Some("Cannot recover: a practice is already in progress".to_string());
                return crux_core::render::render();
            }

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
                    tempo: None,
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                },
                Item {
                    id: "piece-2".to_string(),
                    title: "Clair de Lune".to_string(),
                    kind: ItemKind::Piece,
                    composer: Some("Debussy".to_string()),
                    key: None,
                    tempo: None,
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
                },
                Item {
                    id: "exercise-1".to_string(),
                    title: "C Major Scale".to_string(),
                    kind: ItemKind::Exercise,
                    composer: None,
                    key: None,
                    tempo: None,
                    notes: None,
                    tags: vec![],
                    created_at: now,
                    updated_at: now,
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

        // Also test above max
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
    fn test_add_duplicate_items_to_setlist() {
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
                item_id: "piece-1".to_string(),
            }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Building(ref b) = model.session_status {
            assert_eq!(b.entries.len(), 2);
            // Each entry has a unique ID
            assert_ne!(b.entries[0].id, b.entries[1].id);
        } else {
            panic!("Expected Building state");
        }
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
            // Verify positions are re-indexed
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

        // Complete first item
        update(
            &mut model,
            Event::Session(SessionEvent::NextItem { now: t1 }),
        );
        // End early on second item
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
            Event::Session(SessionEvent::RecoverSession { session: active }),
        );

        assert!(model.last_error.is_none());
        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.id, "recovered-session");
        } else {
            panic!("Expected Active state");
        }
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
            Event::Session(SessionEvent::RecoverSession { session: active }),
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

        // Should be in summary state
        assert!(matches!(model.session_status, SessionStatus::Summary(_)));

        // Save it
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

        // Score 6 — out of range
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryScore {
                entry_id: entry_id.clone(),
                score: Some(6),
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

        // Score 5 — maximum valid
        update(
            &mut model,
            Event::Session(SessionEvent::UpdateEntryScore {
                entry_id: entry_id.clone(),
                score: Some(5),
            }),
        );

        if let SessionStatus::Summary(ref s) = model.session_status {
            assert_eq!(s.entries[0].score, Some(5));
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
                intention: Some("Session goal".to_string()),
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
                intention: Some("Entry goal".to_string()),
            }),
        );

        // Start session
        update(
            &mut model,
            Event::Session(SessionEvent::StartSession { now }),
        );

        if let SessionStatus::Active(ref a) = model.session_status {
            assert_eq!(a.session_intention, Some("Session goal".to_string()));
            assert_eq!(a.entries[0].intention, Some("Entry goal".to_string()));
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

        // Got-it 3 times
        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepGotIt));
        update(&mut model, Event::Session(SessionEvent::RepGotIt));

        // Miss once
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
}
