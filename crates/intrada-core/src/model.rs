use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::analytics::AnalyticsView;
use crate::domain::item::Item;
use crate::domain::routine::Routine;
use crate::domain::session::{
    ActiveSession, CompletionStatus, EntryStatus, PracticeSession, RepAction, SessionStatus,
    SetlistEntry, SummarySession,
};
use crate::domain::ListQuery;

/// Internal application state — not exposed to shells.
#[derive(Debug, Default)]
pub struct Model {
    pub items: Vec<Item>,
    pub sessions: Vec<PracticeSession>,
    pub session_status: SessionStatus,
    pub active_query: Option<ListQuery>,
    pub last_error: Option<String>,
    pub routines: Vec<Routine>,
    pub practice_summaries: HashMap<String, ItemPracticeSummary>,
}

/// Serializable view state sent to shells for rendering.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ViewModel {
    pub items: Vec<LibraryItemView>,
    pub sessions: Vec<PracticeSessionView>,
    pub active_session: Option<ActiveSessionView>,
    pub building_setlist: Option<BuildingSetlistView>,
    pub summary: Option<SummaryView>,
    pub session_status: String,
    pub error: Option<String>,
    pub analytics: Option<AnalyticsView>,
    pub routines: Vec<RoutineView>,
}

/// Represents a routine for display in the UI.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RoutineView {
    pub id: String,
    pub name: String,
    pub entry_count: usize,
    pub entries: Vec<RoutineEntryView>,
}

/// Represents a single entry within a routine for display.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct RoutineEntryView {
    pub id: String,
    pub item_id: String,
    pub item_title: String,
    pub item_type: String,
    pub position: usize,
}

/// Flattened representation of a piece or exercise for display.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LibraryItemView {
    pub id: String,
    pub item_type: String,
    pub title: String,
    pub subtitle: String,
    pub category: Option<String>,
    pub key: Option<String>,
    pub tempo: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub practice: Option<ItemPracticeSummary>,
}

/// Practice summary for a library item.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ItemPracticeSummary {
    pub session_count: usize,
    pub total_minutes: u32,
    pub latest_score: Option<u8>,
    pub score_history: Vec<ScoreHistoryEntry>,
}

/// A single score data point for an item's progress history.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ScoreHistoryEntry {
    pub session_date: String,
    pub score: u8,
    pub session_id: String,
}

/// A completed practice session in history view.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PracticeSessionView {
    pub id: String,
    pub started_at: String,
    pub finished_at: String,
    pub total_duration_display: String,
    pub completion_status: String,
    pub notes: Option<String>,
    pub entries: Vec<SetlistEntryView>,
    pub session_intention: Option<String>,
}

/// A single entry within a session view.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SetlistEntryView {
    pub id: String,
    pub item_id: String,
    pub item_title: String,
    pub item_type: String,
    pub position: usize,
    pub duration_display: String,
    pub status: String,
    pub notes: Option<String>,
    pub score: Option<u8>,
    pub intention: Option<String>,
    pub rep_target: Option<u8>,
    pub rep_count: Option<u8>,
    pub rep_target_reached: Option<bool>,
    pub rep_history: Option<Vec<RepAction>>,
    pub planned_duration_secs: Option<u32>,
    pub planned_duration_display: Option<String>,
}

/// View for the in-progress active session.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ActiveSessionView {
    pub current_item_title: String,
    pub current_item_type: String,
    pub current_position: usize,
    pub total_items: usize,
    pub started_at: String,
    pub entries: Vec<SetlistEntryView>,
    pub session_intention: Option<String>,
    pub current_rep_target: Option<u8>,
    pub current_rep_count: Option<u8>,
    pub current_rep_target_reached: Option<bool>,
    pub current_rep_history: Option<Vec<RepAction>>,
    pub current_planned_duration_secs: Option<u32>,
    pub next_item_title: Option<String>,
}

/// View for the building phase setlist.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BuildingSetlistView {
    pub entries: Vec<SetlistEntryView>,
    pub item_count: usize,
    pub session_intention: Option<String>,
}

/// View for the end-of-session summary.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SummaryView {
    pub total_duration_display: String,
    pub completion_status: String,
    pub notes: Option<String>,
    pub entries: Vec<SetlistEntryView>,
    pub session_intention: Option<String>,
}

// ── View helpers ──────────────────────────────────────────────────────

/// Convert a `SetlistEntry` into a `SetlistEntryView`.
pub fn entry_to_view(entry: &SetlistEntry) -> SetlistEntryView {
    SetlistEntryView {
        id: entry.id.clone(),
        item_id: entry.item_id.clone(),
        item_title: entry.item_title.clone(),
        item_type: entry.item_type.clone(),
        position: entry.position,
        duration_display: crate::domain::session::format_duration_display(entry.duration_secs),
        status: match entry.status {
            EntryStatus::Completed => "completed".to_string(),
            EntryStatus::Skipped => "skipped".to_string(),
            EntryStatus::NotAttempted => "not_attempted".to_string(),
        },
        notes: entry.notes.clone(),
        score: entry.score,
        intention: entry.intention.clone(),
        rep_target: entry.rep_target,
        rep_count: entry.rep_count,
        rep_target_reached: entry.rep_target_reached,
        rep_history: entry.rep_history.clone(),
        planned_duration_secs: entry.planned_duration_secs,
        planned_duration_display: entry.planned_duration_secs.map(|secs| {
            let mins = secs / 60;
            if secs % 60 == 0 {
                format!("{mins} min")
            } else {
                crate::domain::session::format_duration_display(secs as u64)
            }
        }),
    }
}

/// Build views from an `ActiveSession`.
pub fn build_active_session_view(active: &ActiveSession) -> ActiveSessionView {
    let safe_index = active
        .current_index
        .min(active.entries.len().saturating_sub(1));
    let current = &active.entries[safe_index];
    ActiveSessionView {
        current_item_title: current.item_title.clone(),
        current_item_type: current.item_type.clone(),
        current_position: active.current_index,
        total_items: active.entries.len(),
        started_at: active.session_started_at.to_rfc3339(),
        entries: active.entries.iter().map(entry_to_view).collect(),
        session_intention: active.session_intention.clone(),
        current_rep_target: current.rep_target,
        current_rep_count: current.rep_count,
        current_rep_target_reached: current.rep_target_reached,
        current_rep_history: current.rep_history.clone(),
        current_planned_duration_secs: current.planned_duration_secs,
        next_item_title: active
            .entries
            .get(safe_index + 1)
            .map(|e| e.item_title.clone()),
    }
}

/// Build view from a `SummarySession`.
pub fn build_summary_view(summary: &SummarySession) -> SummaryView {
    let total_secs: u64 = summary.entries.iter().map(|e| e.duration_secs).sum();
    SummaryView {
        total_duration_display: crate::domain::session::format_duration_display(total_secs),
        completion_status: match summary.completion_status {
            CompletionStatus::Completed => "completed".to_string(),
            CompletionStatus::EndedEarly => "ended_early".to_string(),
        },
        notes: summary.session_notes.clone(),
        entries: summary.entries.iter().map(entry_to_view).collect(),
        session_intention: summary.session_intention.clone(),
    }
}

/// Build view from completed `PracticeSession`.
pub fn session_to_view(session: &PracticeSession) -> PracticeSessionView {
    PracticeSessionView {
        id: session.id.clone(),
        started_at: session.started_at.to_rfc3339(),
        finished_at: session.completed_at.to_rfc3339(),
        total_duration_display: crate::domain::session::format_duration_display(
            session.total_duration_secs,
        ),
        completion_status: match session.completion_status {
            CompletionStatus::Completed => "completed".to_string(),
            CompletionStatus::EndedEarly => "ended_early".to_string(),
        },
        notes: session.session_notes.clone(),
        entries: session.entries.iter().map(entry_to_view).collect(),
        session_intention: session.session_intention.clone(),
    }
}
