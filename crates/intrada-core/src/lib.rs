pub mod analytics;
pub mod app;
pub mod domain;
pub mod error;
pub mod model;
pub mod validation;

pub use app::{Effect, Event, Intrada, StorageEffect};
pub use domain::exercise::{Exercise, ExerciseEvent};
pub use domain::piece::{Piece, PieceEvent};
pub use domain::routine::{Routine, RoutineEntry, RoutineEvent};
pub use domain::session::{
    ActiveSession, CompletionStatus, EntryStatus, PracticeSession, SessionEvent, SessionStatus,
    SetlistEntry,
};
pub use domain::types::{
    CreateExercise, CreatePiece, LibraryData, ListQuery, SessionsData, Tempo, UpdateExercise,
    UpdatePiece,
};
pub use error::LibraryError;
pub use model::{
    ActiveSessionView, BuildingSetlistView, ItemPracticeSummary, LibraryItemView, Model,
    PracticeSessionView, RoutineEntryView, RoutineView, ScoreHistoryEntry, SetlistEntryView,
    SummaryView, ViewModel,
};
pub use validation::{
    MAX_BPM, MAX_CATEGORY, MAX_COMPOSER, MAX_NOTES, MAX_ROUTINE_NAME, MAX_TAG, MAX_TEMPO_MARKING,
    MAX_TITLE, MIN_BPM,
};
