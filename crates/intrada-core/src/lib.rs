pub mod analytics;
pub mod app;
pub mod domain;
pub mod error;
pub mod model;
pub mod persistence;
pub mod practice_weeks;
pub(crate) mod priorities;
pub mod recognition;
pub(crate) mod sample;
pub(crate) mod staleness;
pub mod stored_session;
pub mod suggestion;
pub mod sync;
pub mod validation;
pub mod view;

pub use app::{AppEffect, Effect, Event, Intrada};
pub use domain::item::{Item, ItemEvent, ItemKind, Modality, VariationEvent};
pub use domain::key::{Accidental, Key, Letter};
pub use domain::session::{
    ActiveSession, CompletionStatus, EntryStatus, PracticeSession, SessionEvent, SessionStatus,
    SetlistEntry,
};
pub use domain::types::{
    CreateItem, KeyEdit, LibraryData, LibrarySort, ListQuery, SessionsData, SortDirection,
    SortField, Tempo, TempoInput, UpdateItem,
};
pub use error::LibraryError;
pub use model::{
    ActiveSessionView, BuildingSetlistView, ItemPracticeSummary, LibraryItemView, LimitsView,
    Model, PhotoRecognition, PhotoRecognitionStatus, PhotoRecognitionView, PracticeSessionView,
    ScoreHistoryEntry, SetlistEntryView, SummaryView, TempoTrendPoint, TempoTrendView, ViewModel,
};
pub use persistence::{PersistenceOperation, PersistenceOutput};
pub use recognition::{
    fill_form, read_fields, DraftSource, FieldFill, FieldNow, PageReading, PhotoDraft, ReadField,
    RecognisedLine, RecognitionOperation, RecognitionOutput, SuggestedFields, TempoDraftField,
    TextDraftField, LOW_CONFIDENCE,
};
pub use validation::{
    MAX_ACHIEVED_TEMPO, MAX_BPM, MAX_COMPOSER, MAX_NOTES, MAX_TAG, MAX_TEMPO_MARKING, MAX_TITLE,
    MIN_ACHIEVED_TEMPO, MIN_BPM,
};
