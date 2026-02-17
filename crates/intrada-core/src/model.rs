use serde::{Deserialize, Serialize};

use crate::domain::session::{
    ActiveSession, CompletionStatus, EntryStatus, PracticeSession, SessionStatus, SetlistEntry,
    SummarySession,
};
use crate::domain::{Exercise, ListQuery, Piece};

/// Internal application state — not exposed to shells.
#[derive(Debug, Default)]
pub struct Model {
    pub pieces: Vec<Piece>,
    pub exercises: Vec<Exercise>,
    pub sessions: Vec<PracticeSession>,
    pub session_status: SessionStatus,
    pub active_query: Option<ListQuery>,
    pub last_error: Option<String>,
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
}

/// View for the building phase setlist.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BuildingSetlistView {
    pub entries: Vec<SetlistEntryView>,
    pub item_count: usize,
}

/// View for the end-of-session summary.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SummaryView {
    pub total_duration_display: String,
    pub completion_status: String,
    pub notes: Option<String>,
    pub entries: Vec<SetlistEntryView>,
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
    }
}
