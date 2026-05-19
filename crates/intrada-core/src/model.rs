use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::analytics::AnalyticsView;
use crate::domain::account::AccountPreferences;
use crate::domain::goal::{Goal, GoalStatus};
use crate::domain::item::{Item, ItemKind};
use crate::domain::mcp_audit::McpAuditEntry;
use crate::domain::mcp_tokens::{CreatedMcpToken, McpToken};
use crate::domain::session::{
    ActiveSession, CompletionStatus, EntryStatus, PracticeSession, RepAction, SessionStatus,
    SetlistEntry, SummarySession,
};
use crate::domain::set::Set;
use crate::domain::ListQuery;

/// Internal application state — not exposed to shells.
#[derive(Debug, Default)]
pub struct Model {
    /// Base URL for the REST API (set via `Event::StartApp`).
    pub api_base_url: String,
    pub items: Vec<Item>,
    pub sessions: Vec<PracticeSession>,
    pub session_status: SessionStatus,
    pub active_query: Option<ListQuery>,
    pub last_error: Option<String>,
    /// Set when the user dismisses the error banner. While true, errors from
    /// HTTP failures routed through [`Model::surface_error`] are silently
    /// swallowed — avoids the "dismiss → next refetch fails → banner
    /// reappears" loop when the underlying problem (network down, auth
    /// expired) hasn't been resolved. Cleared by any confirmed API success
    /// via [`Model::record_success`], signalling the system has recovered
    /// and new failures are worth surfacing again (#346).
    pub error_muted: bool,
    pub sets: Vec<Set>,
    pub goals: Vec<Goal>,
    pub current_goal: Option<Goal>,
    pub practice_summaries: HashMap<String, ItemPracticeSummary>,
    /// Per-user practice defaults; `None` until first load completes.
    pub account_preferences: Option<AccountPreferences>,
    /// True while a `DELETE /api/account` request is outstanding.
    pub delete_in_flight: bool,
    /// One-shot terminal signal: server confirmed the account was
    /// deleted. The shell watches this to sign out + route home.
    /// Does not reset (account is gone; nothing to reset to).
    pub account_deleted: bool,
    /// MCP Personal Access Tokens for the current user. Newest first.
    pub mcp_tokens: Vec<McpToken>,
    /// Set to `true` after the first successful `LoadTokens` so the UI can
    /// distinguish "loading" from "loaded but empty".
    pub mcp_tokens_loaded: bool,
    /// True while a token list / create / revoke request is outstanding.
    pub mcp_tokens_loading: bool,
    /// Set transiently after `CreateToken` succeeds — carries the full
    /// token bytes so the UI can show them once. Cleared by
    /// `DismissCreatedToken` (or naturally when the user navigates away
    /// and the model is reloaded).
    pub just_created_token: Option<CreatedMcpToken>,
    /// Audit-log entries for the current user, newest first.
    pub mcp_audit: Vec<McpAuditEntry>,
    /// True after the first successful `LoadAudit` so the UI can
    /// distinguish "loading" from "loaded but empty".
    pub mcp_audit_loaded: bool,
    /// True while a `LoadAudit` HTTP request is outstanding.
    pub mcp_audit_loading: bool,
    /// True while the OAuth `/oauth/finalize` request is outstanding.
    pub oauth_in_flight: bool,
    /// Set transiently when `/oauth/finalize` returns; the consent view
    /// reacts by navigating the browser to this URL (which contains the
    /// auth code + state for the OAuth client).
    pub oauth_redirect_url: Option<String>,
    /// Monotonic counter that increments each time the server confirms a
    /// `SaveBuildingAsSet` / `SaveSummaryAsSet` write. `SetSaveForm` reads
    /// the value at dispatch time and flips its "Saved" state only after
    /// it observes the counter rise — turning the optimistic-success UX
    /// into a confirmed-success UX. Without this, a failed save would
    /// leave the button stuck on "Saved" while an error toast contradicted
    /// it (#449).
    pub set_saves_committed: u64,
}

impl Model {
    /// Surface an error from a background HTTP failure. Respects the
    /// dismiss-mute state set by [`Model::dismiss_error`]: if the user has
    /// already dismissed the banner and the system has not yet recovered,
    /// the error is silently swallowed to stop the banner re-popping. Also
    /// dedupes identical messages to avoid render storms during burst
    /// failures (#346).
    pub fn surface_error(&mut self, msg: impl Into<String>) {
        if self.error_muted {
            return;
        }
        let msg = msg.into();
        if self.last_error.as_deref() == Some(msg.as_str()) {
            return;
        }
        self.last_error = Some(msg);
    }

    /// Mark a confirmed API success. Clears any active error and exits the
    /// dismiss-mute state — the system has demonstrably recovered, so
    /// future failures are worth showing again. Call from any handler that
    /// receives a successful API response.
    pub fn record_success(&mut self) {
        self.last_error = None;
        self.error_muted = false;
    }

    /// User explicitly dismissed the error banner. Clears the active error
    /// and enters the mute state so subsequent background failures don't
    /// immediately re-pop the banner.
    pub fn dismiss_error(&mut self) {
        self.last_error = None;
        self.error_muted = true;
    }

    /// Reset all user-scoped state on sign-out so a subsequent sign-in
    /// (potentially as a different user on the same browser) starts from a
    /// clean slate. Preserves `api_base_url` (set once at app startup; not
    /// per-user). Without this, the next user briefly sees the previous
    /// user's data — most concerning for MCP tokens / audit which are a
    /// soft information disclosure until the first refetch overwrites them
    /// (#645).
    pub fn reset_for_sign_out(&mut self) {
        let api_base_url = std::mem::take(&mut self.api_base_url);
        *self = Self {
            api_base_url,
            ..Self::default()
        };
    }
}

#[cfg(test)]
impl Model {
    /// Create a test model with a valid API base URL.
    ///
    /// crux_http requires absolute URLs, so tests must use this instead of
    /// `Model::default()` when the handler under test produces HTTP effects.
    pub fn test_default() -> Self {
        Self {
            api_base_url: "http://localhost:3001".to_string(),
            ..Default::default()
        }
    }
}

/// View-layer representation of session lifecycle state.
///
/// Mirrors the internal `SessionStatus` enum but is serializable and
/// exposed to shells via the `ViewModel`. Using an enum instead of a
/// String gives compile-time safety in both Rust and generated Swift code.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum SessionStatusView {
    #[default]
    Idle,
    Building,
    Active,
    Summary,
}

/// Serializable view state sent to shells for rendering.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ViewModel {
    pub items: Vec<LibraryItemView>,
    pub sessions: Vec<PracticeSessionView>,
    pub active_session: Option<ActiveSessionView>,
    pub building_setlist: Option<BuildingSetlistView>,
    pub summary: Option<SummaryView>,
    pub session_status: SessionStatusView,
    pub error: Option<String>,
    pub analytics: Option<AnalyticsView>,
    pub sets: Vec<SetView>,
    pub goals: Vec<GoalView>,
    pub current_goal: Option<GoalView>,
    pub account_preferences: Option<AccountPreferences>,
    pub delete_in_flight: bool,
    pub account_deleted: bool,
    pub mcp_tokens: Vec<McpToken>,
    pub mcp_tokens_loaded: bool,
    pub mcp_tokens_loading: bool,
    pub just_created_token: Option<CreatedMcpToken>,
    pub mcp_audit: Vec<McpAuditEntry>,
    pub mcp_audit_loaded: bool,
    pub mcp_audit_loading: bool,
    pub oauth_in_flight: bool,
    pub oauth_redirect_url: Option<String>,
    /// Mirrors `Model::set_saves_committed`. See that field for the contract.
    pub set_saves_committed: u64,
}

// Sanity-check that ViewModel field names mirror Model where applicable.
// (No code; just keeps model + viewmodel in sync mentally.)

/// Represents a goal for display in the UI.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GoalView {
    pub id: String,
    pub title: Option<String>,
    pub date: String,
    pub notes: Option<String>,
    pub notes_preview: String,
    pub deadline: Option<String>,
    pub status: GoalStatus,
    pub completed_at: Option<String>,
    pub is_overdue: bool,
    pub items: Vec<GoalItemView>,
    pub photos: Vec<GoalPhotoView>,
    pub has_photos: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Photo metadata for display in the UI.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GoalPhotoView {
    pub id: String,
    pub url: String,
}

/// A linked library item for display in the UI.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GoalItemView {
    pub item_id: String,
    pub item_title: String,
    pub item_type: ItemKind,
}

pub fn goal_to_view(goal: &Goal) -> GoalView {
    let notes_preview = goal
        .notes
        .as_deref()
        .unwrap_or("")
        .chars()
        .take(100)
        .collect::<String>();

    let is_overdue = matches!(goal.status, GoalStatus::Active)
        && goal.deadline.as_ref().is_some_and(|d| {
            chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d")
                .map(|deadline_date| deadline_date < chrono::Utc::now().date_naive())
                .unwrap_or(false)
        });

    GoalView {
        id: goal.id.clone(),
        title: goal.title.clone(),
        date: goal.date.clone(),
        notes: goal.notes.clone(),
        notes_preview,
        deadline: goal.deadline.clone(),
        status: goal.status.clone(),
        completed_at: goal.completed_at.map(|dt| dt.to_rfc3339()),
        is_overdue,
        items: goal
            .items
            .iter()
            .map(|i| GoalItemView {
                item_id: i.item_id.clone(),
                item_title: i.item_title.clone(),
                item_type: i.item_type.clone(),
            })
            .collect(),
        photos: goal
            .photos
            .iter()
            .map(|p| GoalPhotoView {
                id: p.id.clone(),
                url: p.url.clone(),
            })
            .collect(),
        has_photos: !goal.photos.is_empty(),
        created_at: goal.created_at.to_rfc3339(),
        updated_at: goal.updated_at.to_rfc3339(),
    }
}

/// Represents a set for display in the UI.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SetView {
    pub id: String,
    pub name: String,
    pub entry_count: usize,
    pub entries: Vec<SetEntryView>,
}

/// Represents a single entry within a set for display.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SetEntryView {
    pub id: String,
    pub item_id: String,
    pub item_title: String,
    pub item_type: ItemKind,
    pub position: usize,
}

/// Flattened representation of a piece or exercise for display.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct LibraryItemView {
    pub id: String,
    pub item_type: ItemKind,
    pub title: String,
    pub subtitle: String,
    pub key: Option<String>,
    pub tempo: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub practice: Option<ItemPracticeSummary>,
    pub latest_achieved_tempo: Option<u16>,
}

/// Practice summary for a library item.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ItemPracticeSummary {
    pub session_count: usize,
    pub total_minutes: u32,
    pub latest_score: Option<u8>,
    pub score_history: Vec<ScoreHistoryEntry>,
    pub latest_tempo: Option<u16>,
    pub tempo_history: Vec<TempoHistoryEntry>,
}

/// A single score data point for an item's progress history.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ScoreHistoryEntry {
    pub session_date: String,
    pub score: u8,
    pub session_id: String,
}

/// A single tempo data point for an item's tempo progress history.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TempoHistoryEntry {
    pub session_date: String,
    pub tempo: u16,
    pub session_id: String,
}

/// A completed practice session in history view.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct PracticeSessionView {
    pub id: String,
    pub started_at: String,
    pub finished_at: String,
    pub total_duration_display: String,
    pub completion_status: CompletionStatus,
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
    pub item_type: ItemKind,
    pub position: usize,
    pub duration_display: String,
    pub status: EntryStatus,
    pub notes: Option<String>,
    pub score: Option<u8>,
    pub intention: Option<String>,
    pub rep_target: Option<u8>,
    pub rep_count: Option<u8>,
    pub rep_target_reached: Option<bool>,
    pub rep_history: Option<Vec<RepAction>>,
    pub planned_duration_secs: Option<u32>,
    pub planned_duration_display: Option<String>,
    pub achieved_tempo: Option<u16>,
}

/// View for the in-progress active session.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ActiveSessionView {
    pub current_item_title: String,
    pub current_item_type: ItemKind,
    pub current_position: usize,
    pub total_items: usize,
    pub started_at: String,
    /// Wall-clock anchor (RFC3339 UTC) for the *current item*. Resets to
    /// "now" on each item advance (Next / Skip). The shell derives the
    /// per-item elapsed timer from `Utc::now() - current_item_started_at`
    /// rather than incrementing a counter — survives WebView suspension /
    /// tab backgrounding without drift.
    pub current_item_started_at: String,
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
/// Whether the builder's entries originate from, and relate to, a saved Set.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub enum SetSourceStatus {
    #[default]
    NoSource,
    UnmodifiedFromSource {
        set_id: String,
        set_name: String,
    },
    ModifiedFromSource {
        set_id: String,
        set_name: String,
    },
}

/// View for the building phase setlist.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BuildingSetlistView {
    pub entries: Vec<SetlistEntryView>,
    pub item_count: usize,
    pub session_intention: Option<String>,
    pub target_duration_mins: Option<u32>,
    pub source_status: SetSourceStatus,
}

/// View for the end-of-session summary.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SummaryView {
    pub total_duration_display: String,
    pub completion_status: CompletionStatus,
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
        status: entry.status.clone(),
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
        achieved_tempo: entry.achieved_tempo,
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
        current_item_type: current.item_type.clone(), // Now ItemKind
        current_position: active.current_index,
        total_items: active.entries.len(),
        started_at: active.session_started_at.to_rfc3339(),
        current_item_started_at: active.current_item_started_at.to_rfc3339(),
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
        completion_status: summary.completion_status.clone(),
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
        completion_status: session.completion_status.clone(),
        notes: session.session_notes.clone(),
        entries: session.entries.iter().map(entry_to_view).collect(),
        session_intention: session.session_intention.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::goal::{GoalItem, GoalPhoto};
    use chrono::{TimeZone, Utc};

    fn make_entry(id: &str, item_id: &str, title: &str, position: usize) -> SetlistEntry {
        SetlistEntry {
            id: id.to_string(),
            item_id: item_id.to_string(),
            item_title: title.to_string(),
            item_type: ItemKind::Piece,
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

    // ── Model error methods ────────────────────────────────────────────

    #[test]
    fn surface_error_sets_last_error() {
        let mut model = Model::default();
        model.surface_error("network down");
        assert_eq!(model.last_error.as_deref(), Some("network down"));
    }

    #[test]
    fn surface_error_dedupes_identical_messages() {
        let mut model = Model::default();
        model.surface_error("fail");
        model.surface_error("fail");
        assert_eq!(model.last_error.as_deref(), Some("fail"));
    }

    #[test]
    fn surface_error_muted_after_dismiss() {
        let mut model = Model::default();
        model.surface_error("first");
        model.dismiss_error();
        model.surface_error("second");
        assert!(model.last_error.is_none());
        assert!(model.error_muted);
    }

    #[test]
    fn record_success_clears_error_and_unmutes() {
        let mut model = Model::default();
        model.surface_error("oops");
        model.dismiss_error();
        model.record_success();
        assert!(model.last_error.is_none());
        assert!(!model.error_muted);
        model.surface_error("new error");
        assert_eq!(model.last_error.as_deref(), Some("new error"));
    }

    #[test]
    fn reset_for_sign_out_preserves_api_base_url() {
        let mut model = Model {
            api_base_url: "https://api.example.com".to_string(),
            ..Default::default()
        };
        model.items.push(Item {
            id: "item-1".to_string(),
            title: "leftover".to_string(),
            kind: ItemKind::Piece,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        });
        model.set_saves_committed = 5;
        model.reset_for_sign_out();
        assert_eq!(model.api_base_url, "https://api.example.com");
        assert!(model.items.is_empty());
        assert_eq!(model.set_saves_committed, 0);
    }

    // ── entry_to_view ──────────────────────────────────────────────────

    #[test]
    fn entry_to_view_formats_duration() {
        let mut entry = make_entry("e1", "i1", "Scale", 0);
        entry.duration_secs = 125;
        let view = entry_to_view(&entry);
        assert_eq!(view.duration_display, "2m 5s");
    }

    #[test]
    fn entry_to_view_planned_duration_whole_minutes() {
        let mut entry = make_entry("e1", "i1", "Scale", 0);
        entry.planned_duration_secs = Some(300);
        let view = entry_to_view(&entry);
        assert_eq!(view.planned_duration_display.as_deref(), Some("5 min"));
    }

    #[test]
    fn entry_to_view_planned_duration_partial_minutes() {
        let mut entry = make_entry("e1", "i1", "Scale", 0);
        entry.planned_duration_secs = Some(90);
        let view = entry_to_view(&entry);
        assert_eq!(view.planned_duration_display.as_deref(), Some("1m 30s"));
    }

    // ── build_active_session_view ──────────────────────────────────────

    #[test]
    fn active_session_view_next_item_title() {
        let active = ActiveSession {
            id: "as1".to_string(),
            entries: vec![
                make_entry("e1", "i1", "Scale", 0),
                make_entry("e2", "i2", "Etude", 1),
            ],
            current_index: 0,
            session_started_at: Utc::now(),
            current_item_started_at: Utc::now(),
            session_intention: None,
        };
        let view = build_active_session_view(&active);
        assert_eq!(view.next_item_title.as_deref(), Some("Etude"));
    }

    #[test]
    fn active_session_view_last_item_has_no_next() {
        let active = ActiveSession {
            id: "as2".to_string(),
            entries: vec![
                make_entry("e1", "i1", "Scale", 0),
                make_entry("e2", "i2", "Etude", 1),
            ],
            current_index: 1,
            session_started_at: Utc::now(),
            current_item_started_at: Utc::now(),
            session_intention: None,
        };
        let view = build_active_session_view(&active);
        assert!(view.next_item_title.is_none());
    }

    // ── build_summary_view ─────────────────────────────────────────────

    #[test]
    fn summary_view_total_duration() {
        let mut e1 = make_entry("e1", "i1", "Scale", 0);
        e1.duration_secs = 60;
        let mut e2 = make_entry("e2", "i2", "Etude", 1);
        e2.duration_secs = 90;
        let summary = crate::domain::session::SummarySession {
            id: "sum1".to_string(),
            entries: vec![e1, e2],
            session_started_at: Utc::now(),
            session_ended_at: Utc::now(),
            completion_status: CompletionStatus::Completed,
            session_notes: None,
            session_intention: Some("focus".to_string()),
        };
        let view = build_summary_view(&summary);
        assert_eq!(view.total_duration_display, "2m 30s");
        assert_eq!(view.session_intention.as_deref(), Some("focus"));
    }

    // ── goal_to_view ──────────────────────────────────────────────────

    #[test]
    fn goal_notes_preview_truncated_at_100_chars() {
        let long_notes = "a".repeat(200);
        let goal = Goal {
            id: "g1".to_string(),
            title: Some("Practice scales".to_string()),
            date: "2026-01-15".to_string(),
            notes: Some(long_notes),
            deadline: None,
            status: GoalStatus::Active,
            completed_at: None,
            items: vec![],
            photos: vec![],
            created_at: Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap(),
            target_confidence: None,
        };
        let view = goal_to_view(&goal);
        assert_eq!(view.notes_preview.len(), 100);
    }

    #[test]
    fn goal_has_photos_flag() {
        let goal = Goal {
            id: "g1".to_string(),
            title: None,
            date: "2026-01-15".to_string(),
            notes: None,
            deadline: None,
            status: GoalStatus::Active,
            completed_at: None,
            items: vec![],
            photos: vec![GoalPhoto {
                id: "p1".to_string(),
                url: "https://example.com/photo.jpg".to_string(),
                created_at: Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap(),
            }],
            created_at: Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap(),
            target_confidence: None,
        };
        let view = goal_to_view(&goal);
        assert!(view.has_photos);
        assert_eq!(view.photos.len(), 1);
    }

    #[test]
    fn goal_is_overdue_when_deadline_in_past_and_active() {
        let yesterday = (chrono::Utc::now().date_naive() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();
        let goal = Goal {
            id: "g1".to_string(),
            title: None,
            date: "2026-01-15".to_string(),
            notes: None,
            deadline: Some(yesterday),
            status: GoalStatus::Active,
            completed_at: None,
            items: vec![],
            photos: vec![],
            created_at: Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap(),
            target_confidence: None,
        };
        let view = goal_to_view(&goal);
        assert!(view.is_overdue);
    }

    #[test]
    fn goal_not_overdue_when_completed() {
        let yesterday = (chrono::Utc::now().date_naive() - chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();
        let goal = Goal {
            id: "g1".to_string(),
            title: None,
            date: "2026-01-15".to_string(),
            notes: None,
            deadline: Some(yesterday),
            status: GoalStatus::Completed,
            completed_at: Some(Utc.with_ymd_and_hms(2026, 1, 14, 10, 0, 0).unwrap()),
            items: vec![],
            photos: vec![],
            created_at: Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap(),
            target_confidence: None,
        };
        let view = goal_to_view(&goal);
        assert!(!view.is_overdue);
    }

    #[test]
    fn goal_items_appear_in_view() {
        let goal = Goal {
            id: "g1".to_string(),
            title: Some("Learn sonata".to_string()),
            date: "2026-01-15".to_string(),
            notes: None,
            deadline: None,
            status: GoalStatus::Active,
            completed_at: None,
            items: vec![GoalItem {
                item_id: "item-1".to_string(),
                item_title: "Moonlight Sonata".to_string(),
                item_type: ItemKind::Piece,
                target_date: None,
                target_confidence: None,
            }],
            photos: vec![],
            created_at: Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2026, 1, 15, 10, 0, 0).unwrap(),
            target_confidence: None,
        };
        let view = goal_to_view(&goal);
        assert_eq!(view.items.len(), 1);
        assert_eq!(view.items[0].item_title, "Moonlight Sonata");
    }
}
