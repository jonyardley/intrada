pub mod analytics;
pub mod app;
pub mod domain;
pub mod error;
pub mod http;
pub mod model;
pub mod validation;

pub use app::{AppEffect, Effect, Event, Intrada};
pub use domain::account::{AccountEvent, AccountPreferences};
pub use domain::goal::{Goal, GoalEvent, GoalItem, GoalPhoto, GoalStatus};
pub use domain::item::{Item, ItemEvent, ItemKind};
pub use domain::mcp_audit::{McpAuditEntry, McpAuditEvent};
pub use domain::mcp_tokens::{CreatedMcpToken, McpToken, McpTokenEvent};
pub use domain::oauth::{OAuthEvent, OAuthFinalizeParams};
pub use domain::session::{
    ActiveSession, CompletionStatus, EntryStatus, PracticeSession, SessionEvent, SessionStatus,
    SetlistEntry,
};
pub use domain::set::{Set, SetEntry, SetEvent};
pub use domain::types::{
    CreateGoal, CreateItem, LibraryData, LinkGoalItem, ListQuery, SessionsData, Tempo, UpdateGoal,
    UpdateItem,
};
pub use error::LibraryError;

// Re-export crux_http protocol types so shells can handle HTTP effects
// without a direct crux_http dependency.
pub use crux_http::protocol::{HttpHeader, HttpResponse, HttpResult};
pub use crux_http::{HttpError, HttpRequest};
pub use model::{
    ActiveSessionView, BuildingSetlistView, GoalItemView, GoalPhotoView, GoalView,
    ItemPracticeSummary, LibraryItemView, Model, PracticeSessionView, ScoreHistoryEntry,
    SessionStatusView, SetEntryView, SetSourceStatus, SetView, SetlistEntryView, SummaryView,
    TempoHistoryEntry, ViewModel,
};
pub use validation::{
    MAX_ACHIEVED_TEMPO, MAX_BPM, MAX_COMPOSER, MAX_GOAL_NOTES, MAX_GOAL_TITLE, MAX_NOTES,
    MAX_SET_NAME, MAX_TAG, MAX_TEMPO_MARKING, MAX_TITLE, MIN_ACHIEVED_TEMPO, MIN_BPM,
};
