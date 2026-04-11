pub mod analytics;
pub mod app;
pub mod domain;
pub mod error;
pub mod http;
pub mod model;
pub mod validation;

pub use app::{AppEffect, Effect, Event, Intrada};
pub use domain::item::{Item, ItemEvent, ItemKind};
pub use domain::lesson::{Lesson, LessonEvent, LessonPhoto};
pub use domain::routine::{Routine, RoutineEntry, RoutineEvent};
pub use domain::session::{
    ActiveSession, CompletionStatus, EntryStatus, PracticeSession, SessionEvent, SessionStatus,
    SetlistEntry,
};
pub use domain::types::{
    CreateItem, CreateLesson, LibraryData, ListQuery, SessionsData, Tempo, UpdateItem, UpdateLesson,
};
pub use error::LibraryError;

// Re-export crux_http protocol types so shells can handle HTTP effects
// without a direct crux_http dependency.
pub use crux_http::protocol::{HttpHeader, HttpResponse, HttpResult};
pub use crux_http::{HttpError, HttpRequest};
pub use model::{
    ActiveSessionView, BuildingSetlistView, ItemPracticeSummary, LessonPhotoView, LessonView,
    LibraryItemView, Model, PracticeSessionView, RoutineEntryView, RoutineView, ScoreHistoryEntry,
    SessionStatusView, SetlistEntryView, SummaryView, TempoHistoryEntry, ViewModel,
};
pub use validation::{
    MAX_ACHIEVED_TEMPO, MAX_BPM, MAX_COMPOSER, MAX_LESSON_NOTES, MAX_NOTES, MAX_ROUTINE_NAME,
    MAX_TAG, MAX_TEMPO_MARKING, MAX_TITLE, MIN_ACHIEVED_TEMPO, MIN_BPM,
};
