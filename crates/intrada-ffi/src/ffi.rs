use std::cell::RefCell;
use std::panic::{self, AssertUnwindSafe};
use std::sync::Once;

use crux_core::{
    bridge::{BincodeFfiFormat, Bridge, EffectId, FfiFormat},
    Core,
};

use crate::stored_session::StoredSession;
use crate::{Intrada, ItemKind, LibrarySort, Modality, SortDirection, SortField};

// Returned (not panicked) so the shell handles it per the no-`try!` contract —
// the crux `counter` example panics but says to do this in production.
#[cfg_attr(feature = "uniffi", derive(uniffi::Error))]
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("core bridge error: {0}")]
    Bridge(String),
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct CoreFFI {
    core: Bridge<Intrada>,
}

impl Default for CoreFFI {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
impl CoreFFI {
    #[cfg_attr(feature = "uniffi", uniffi::constructor)]
    #[must_use]
    pub fn new() -> Self {
        install_panic_hook();
        Self {
            core: Bridge::new(Core::new()),
        }
    }

    pub fn update(&self, data: &[u8]) -> Result<Vec<u8>, CoreError> {
        with_panic_location(|| {
            let mut effects = Vec::new();
            self.core
                .update(data, &mut effects)
                .map_err(|e| CoreError::Bridge(e.to_string()))?;
            Ok(effects)
        })
    }

    pub fn resolve(&self, id: u32, data: &[u8]) -> Result<Vec<u8>, CoreError> {
        with_panic_location(|| {
            let mut effects = Vec::new();
            self.core
                .resolve(EffectId(id), data, &mut effects)
                .map_err(|e| CoreError::Bridge(e.to_string()))?;
            Ok(effects)
        })
    }

    pub fn view(&self) -> Result<Vec<u8>, CoreError> {
        with_panic_location(|| {
            let mut view = Vec::new();
            self.core
                .view(&mut view)
                .map_err(|e| CoreError::Bridge(e.to_string()))?;
            Ok(view)
        })
    }
}

// ── Panic location ──

// UniFFI builds its panic message from the payload alone and symbols are not
// uploaded (#1610), so the location rides in the payload (#2020).
thread_local! {
    static PANIC_LOCATION: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub(crate) fn install_panic_hook() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let previous = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let location = info
                .location()
                .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()));
            let _ = PANIC_LOCATION.try_with(|slot| *slot.borrow_mut() = location);
            previous(info);
        }));
    });
}

pub(crate) fn with_panic_location<R>(work: impl FnOnce() -> R) -> R {
    panic::catch_unwind(AssertUnwindSafe(work)).unwrap_or_else(|payload| {
        let message = payload
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| payload.downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "unknown panic".to_string());
        let located = match PANIC_LOCATION.with(|slot| slot.borrow_mut().take()) {
            Some(location) => format!("{message} at {location}"),
            None => message,
        };
        panic::resume_unwind(Box::new(located))
    })
}

// ── Picker candidates ──

/// A small, flat type crossing the plain FFI boundary, not the full
/// `LibraryItemView`, so this shape stays stable regardless of what fields
/// the view gains (#1653).
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone)]
pub struct PickerCandidateArg {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub last_practiced_at: Option<String>,
    pub kind: PickerKind,
    pub priority: bool,
}

impl From<PickerCandidateArg> for crate::view::library::PickerCandidate {
    fn from(c: PickerCandidateArg) -> Self {
        Self {
            id: c.id,
            title: c.title,
            subtitle: c.subtitle,
            notes: c.notes,
            tags: c.tags,
            created_at: c.created_at,
            last_practiced_at: c.last_practiced_at,
            kind: c.kind.into(),
            priority: c.priority,
        }
    }
}

/// The core stays UniFFI-agnostic, so the kind crosses the plain call as its
/// own enum.
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerKind {
    Piece,
    Exercise,
}

impl From<PickerKind> for ItemKind {
    fn from(kind: PickerKind) -> Self {
        match kind {
            PickerKind::Piece => Self::Piece,
            PickerKind::Exercise => Self::Exercise,
        }
    }
}

/// Exhaustive over `ItemKind`, so a new kind is a compile error here.
impl From<ItemKind> for PickerKind {
    fn from(kind: ItemKind) -> Self {
        match kind {
            ItemKind::Piece => Self::Piece,
            ItemKind::Exercise => Self::Exercise,
        }
    }
}

/// The picker's star, tag and type narrowing, read by the same rule as the
/// Library's filter (#1999).
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, Default)]
pub struct PickerFilterArg {
    pub kind: Option<PickerKind>,
    pub priority_only: bool,
    pub tags: Vec<String>,
}

impl From<PickerFilterArg> for crate::view::library::PickerFilter {
    fn from(f: PickerFilterArg) -> Self {
        Self {
            kind: f.kind.map(Into::into),
            priority_only: f.priority_only,
            tags: f.tags,
        }
    }
}

/// Core stays UniFFI-agnostic (CLAUDE.md), so `SortField` (which carries
/// `facet::Facet` for the crux typegen path, a separate derive system) needs
/// its own copy at this plain-call boundary.
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerSortField {
    DateAdded,
    LastPracticed,
    Title,
}

impl From<PickerSortField> for SortField {
    fn from(field: PickerSortField) -> Self {
        match field {
            PickerSortField::DateAdded => Self::DateAdded,
            PickerSortField::LastPracticed => Self::LastPracticed,
            PickerSortField::Title => Self::Title,
        }
    }
}

/// Exhaustive over `SortField`, so a variant added there without a matching
/// arm here is a compile error, not a silently unreachable picker option.
impl From<SortField> for PickerSortField {
    fn from(field: SortField) -> Self {
        match field {
            SortField::DateAdded => Self::DateAdded,
            SortField::LastPracticed => Self::LastPracticed,
            SortField::Title => Self::Title,
        }
    }
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerSortDirection {
    Ascending,
    Descending,
}

impl From<PickerSortDirection> for SortDirection {
    fn from(direction: PickerSortDirection) -> Self {
        match direction {
            PickerSortDirection::Ascending => Self::Ascending,
            PickerSortDirection::Descending => Self::Descending,
        }
    }
}

impl From<SortDirection> for PickerSortDirection {
    fn from(direction: SortDirection) -> Self {
        match direction {
            SortDirection::Ascending => Self::Ascending,
            SortDirection::Descending => Self::Descending,
        }
    }
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, Copy)]
pub struct PickerSortArg {
    pub field: PickerSortField,
    pub direction: PickerSortDirection,
}

impl From<PickerSortArg> for LibrarySort {
    fn from(sort: PickerSortArg) -> Self {
        Self {
            field: sort.field.into(),
            direction: sort.direction.into(),
        }
    }
}

/// The picker sheet's own sort and search (#1653): a plain call, not an
/// `Event` round trip, since it runs on every keystroke and must not disturb
/// the Library screen's own `ListQuery` state. See
/// `intrada_core::view::library::sort_and_filter_candidates` for the shared comparator
/// and search predicate this calls into. Returns candidate ids in filtered,
/// sorted order; the shell reorders its own list by them.
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[must_use]
pub fn sort_and_filter_picker_candidates(
    candidates: Vec<PickerCandidateArg>,
    sort: PickerSortArg,
    search: String,
    filter: PickerFilterArg,
) -> Vec<String> {
    let candidates: Vec<crate::view::library::PickerCandidate> =
        candidates.into_iter().map(Into::into).collect();
    crate::view::library::sort_and_filter_candidates(
        &candidates,
        &sort.into(),
        &search,
        &filter.into(),
    )
}

// ── Key wheel ──

/// The core stays UniFFI-agnostic, so the modality crosses the plain call as
/// its own enum.
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WheelMode {
    Major,
    Minor,
}

impl From<WheelMode> for Modality {
    fn from(mode: WheelMode) -> Self {
        match mode {
            WheelMode::Major => Self::Major,
            WheelMode::Minor => Self::Minor,
        }
    }
}

impl From<Modality> for WheelMode {
    fn from(modality: Modality) -> Self {
        match modality {
            Modality::Major => Self::Major,
            Modality::Minor => Self::Minor,
        }
    }
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WheelWedge {
    pub ring: u8,
    pub mode: WheelMode,
    pub primary: String,
    pub alt: Option<String>,
}

/// `key` is the generated bincode of the core's `Key`, so there is one
/// description of a key, not a mirror here (#2106).
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WheelTap {
    pub key: Vec<u8>,
    pub flipped: bool,
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredKey {
    pub text: String,
    pub mode: Option<WheelMode>,
}

fn decode_key(bytes: &[u8]) -> Result<crate::Key, CoreError> {
    BincodeFfiFormat::deserialize(bytes).map_err(|e| CoreError::Bridge(format!("key: {e}")))
}

fn encode_key(key: &crate::Key) -> Result<Vec<u8>, CoreError> {
    let mut bytes = Vec::new();
    BincodeFfiFormat::serialize(&mut bytes, key)
        .map_err(|e| CoreError::Bridge(format!("key: {e}")))?;
    Ok(bytes)
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WheelSelection {
    pub ring: u8,
    pub mode: WheelMode,
    pub spelling: String,
}

/// The picker edits an unsaved form, so it asks with plain calls rather than
/// an `Event` round trip per tap (#2226).
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[must_use]
pub fn key_wheel() -> Vec<WheelWedge> {
    crate::domain::key::wheel()
        .into_iter()
        .map(|w| WheelWedge {
            ring: w.ring,
            mode: w.modality.into(),
            primary: w.primary.to_string(),
            alt: w.alt.map(str::to_string),
        })
        .collect()
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn key_next_on_tap(
    current: Option<Vec<u8>>,
    ring: u8,
    mode: WheelMode,
) -> Result<Option<WheelTap>, CoreError> {
    let current = current.as_deref().map(decode_key).transpose()?;
    crate::domain::key::next_on_tap(current.as_ref(), ring, mode.into())
        .map(|t| {
            Ok(WheelTap {
                key: encode_key(&t.key)?,
                flipped: t.flipped,
            })
        })
        .transpose()
}

fn decode_keys(keys: &[Vec<u8>]) -> Result<Vec<crate::Key>, CoreError> {
    keys.iter().map(|k| decode_key(k)).collect()
}

fn encode_keys(keys: &[crate::Key]) -> Result<Vec<Vec<u8>>, CoreError> {
    keys.iter().map(encode_key).collect()
}

/// A plain call, not an Event: the sheet edits an unsaved list (#2226, #2372).
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn key_set_tap(
    keys: Vec<Vec<u8>>,
    ring: u8,
    mode: WheelMode,
) -> Result<Vec<Vec<u8>>, CoreError> {
    encode_keys(&crate::domain::key::tap_key_set(
        &decode_keys(&keys)?,
        ring,
        mode.into(),
    ))
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn key_set_add_all(keys: Vec<Vec<u8>>, mode: WheelMode) -> Result<Vec<Vec<u8>>, CoreError> {
    encode_keys(&crate::domain::key::add_all_to_key_set(
        &decode_keys(&keys)?,
        mode.into(),
    ))
}

/// Which spoke the form's unsaved key lights.
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn key_wheel_selection(key: Vec<u8>) -> Result<Option<WheelSelection>, CoreError> {
    Ok(
        crate::domain::key::wheel_selection(&decode_key(&key)?).map(|s| WheelSelection {
            ring: s.ring,
            mode: s.modality.into(),
            spelling: s.spelling,
        }),
    )
}

/// The form's unsaved key as the musician reads it: "E♭ major".
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn key_label(key: Vec<u8>) -> Result<String, CoreError> {
    Ok(decode_key(&key)?.label())
}

// ── Stored keys ──

/// The store's two columns into a key; `None` for text the core cannot
/// read, which the store keeps in its column untouched (#2106).
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn key_from_stored(
    text: Option<String>,
    mode: Option<WheelMode>,
) -> Result<Option<Vec<u8>>, CoreError> {
    crate::domain::key::key_from_stored(text.as_deref(), mode.map(Into::into))
        .map(|key| encode_key(&key))
        .transpose()
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn key_to_stored(key: Vec<u8>) -> Result<StoredKey, CoreError> {
    let (text, mode) = crate::domain::key::key_to_stored(&decode_key(&key)?);
    Ok(StoredKey {
        text,
        mode: mode.map(Into::into),
    })
}

// ── Stored sessions ──

/// The `session` table's columns (#2234); the shell maps them, the core reads
/// what they mean.
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq)]
pub struct StoredSessionRow {
    pub id: String,
    pub started_at: String,
    pub completed_at: String,
    pub total_duration_secs: i64,
    pub completion_status: String,
    pub session_notes: Option<String>,
    pub entries: String,
    pub session_score: Option<i64>,
    pub capture_version: Option<i64>,
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq)]
pub struct StoredSessionRead {
    /// A bincode `PracticeSession`.
    pub session: Vec<u8>,
    /// Values the core replaced, for the shell to log (#949).
    pub unreadable: Vec<String>,
}

/// An `Err` is a row the core refuses; the shell skips it.
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn session_from_stored(row: StoredSessionRow) -> Result<StoredSessionRead, CoreError> {
    let read = crate::stored_session::session_from_stored(&StoredSession {
        id: row.id,
        started_at: row.started_at,
        completed_at: row.completed_at,
        total_duration_secs: row.total_duration_secs,
        completion_status: row.completion_status,
        session_notes: row.session_notes,
        entries: row.entries,
        session_score: row.session_score,
        capture_version: row.capture_version,
    })
    .map_err(|e| CoreError::Bridge(e.to_string()))?;
    let mut session = Vec::new();
    BincodeFfiFormat::serialize(&mut session, &read.session)
        .map_err(|e| CoreError::Bridge(format!("session: {e}")))?;
    Ok(StoredSessionRead {
        session,
        unreadable: read.unreadable,
    })
}

#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn session_to_stored(session: Vec<u8>) -> Result<StoredSessionRow, CoreError> {
    let session: crate::PracticeSession = BincodeFfiFormat::deserialize(&session)
        .map_err(|e| CoreError::Bridge(format!("session: {e}")))?;
    let row = crate::stored_session::session_to_stored(&session)
        .map_err(|e| CoreError::Bridge(e.to_string()))?;
    Ok(StoredSessionRow {
        id: row.id,
        started_at: row.started_at,
        completed_at: row.completed_at,
        total_duration_secs: row.total_duration_secs,
        completion_status: row.completion_status,
        session_notes: row.session_notes,
        entries: row.entries,
        session_score: row.session_score,
        capture_version: row.capture_version,
    })
}

// ── Filling the form from a read ──

#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormReadField {
    Title,
    Composer,
    Marking,
    Bpm,
    Chart,
}

impl From<FormReadField> for crate::ReadField {
    fn from(field: FormReadField) -> Self {
        match field {
            FormReadField::Title => Self::Title,
            FormReadField::Composer => Self::Composer,
            FormReadField::Marking => Self::Marking,
            FormReadField::Bpm => Self::Bpm,
            FormReadField::Chart => Self::Chart,
        }
    }
}

impl From<crate::ReadField> for FormReadField {
    fn from(field: crate::ReadField) -> Self {
        match field {
            crate::ReadField::Title => Self::Title,
            crate::ReadField::Composer => Self::Composer,
            crate::ReadField::Marking => Self::Marking,
            crate::ReadField::Bpm => Self::Bpm,
            crate::ReadField::Chart => Self::Chart,
        }
    }
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormFieldNow {
    pub field: FormReadField,
    pub text: String,
    pub holds_read: bool,
}

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormFieldFill {
    pub field: FormReadField,
    pub value: String,
    pub weak: bool,
}

/// The add form is unsaved, so it asks on each read rather than the core
/// holding it (#2229). The draft crosses as the generated bincode the view
/// model gave the shell, so there is one description of it, not a mirror here.
#[cfg_attr(feature = "uniffi", uniffi::export)]
pub fn fill_form_from_read(
    draft: Vec<u8>,
    fields: Vec<FormFieldNow>,
) -> Result<Vec<FormFieldFill>, CoreError> {
    let draft: crate::PhotoDraft = BincodeFfiFormat::deserialize(&draft)
        .map_err(|e| CoreError::Bridge(format!("photo draft: {e}")))?;
    let fields: Vec<crate::FieldNow> = fields
        .into_iter()
        .map(|now| crate::FieldNow {
            field: now.field.into(),
            text: now.text,
            holds_read: now.holds_read,
        })
        .collect();
    Ok(crate::fill_form(&draft, &fields)
        .into_iter()
        .map(|fill| FormFieldFill {
            field: fill.field.into(),
            value: fill.value,
            weak: fill.weak,
        })
        .collect())
}

/// The shell builds the crash-recovery blob's storage key from this, so a
/// shape change in the core retires the old key on its own (#1116).
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[must_use]
pub fn session_blob_version() -> u32 {
    crate::domain::session::ActiveSession::BLOB_VERSION
}

/// The shell builds the profile blob's storage key from this, so a shape
/// change in the core retires the old key on its own (#2026).
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[must_use]
pub fn profile_blob_version() -> u32 {
    crate::domain::profile::Profile::BLOB_VERSION
}

/// The shell builds the library sort's storage key from this (#2089).
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[must_use]
pub fn library_sort_blob_version() -> u32 {
    crate::domain::types::LibrarySort::BLOB_VERSION
}

/// The shell builds the first-run blob's storage key from this (#2116).
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[must_use]
pub fn first_run_blob_version() -> u32 {
    crate::domain::first_run::FirstRun::BLOB_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    fn panic_payload(work: impl FnOnce()) -> String {
        let payload = panic::catch_unwind(AssertUnwindSafe(work)).expect_err("work should panic");
        payload
            .downcast_ref::<String>()
            .cloned()
            .expect("payload should be the String UniFFI downcasts")
    }

    #[test]
    fn a_core_panic_payload_names_its_file_and_line() {
        let _core = CoreFFI::new();
        let line = line!() + 1;
        let message = panic_payload(|| with_panic_location(|| panic!("index out of bounds")));
        let expected = format!("index out of bounds at {}:{line}:", file!());
        assert!(message.starts_with(&expected), "{message}");
    }

    #[test]
    fn a_formatted_panic_message_keeps_its_text_alongside_the_location() {
        let _core = CoreFFI::new();
        let len = 3;
        let message = panic_payload(|| with_panic_location(|| panic!("index {} of {len}", 7)));
        let expected = format!("index 7 of 3 at {}:", file!());
        assert!(message.starts_with(&expected), "{message}");
    }

    #[test]
    fn bridge_serializes_initial_view() {
        let core = CoreFFI::new();
        let view = core.view().expect("initial view should serialize");
        assert!(!view.is_empty(), "serialized ViewModel should be non-empty");
    }

    /// Exercises the FFI-side duplicate types a caller actually constructs,
    /// not the core types directly.
    #[test]
    fn sort_and_filter_picker_candidates_round_trips_through_the_ffi_types() {
        let candidates = vec![
            PickerCandidateArg {
                id: "p1".to_string(),
                title: "Clair de Lune".to_string(),
                subtitle: "Debussy".to_string(),
                notes: None,
                tags: vec![],
                created_at: "2026-01-01".to_string(),
                last_practiced_at: None,
                kind: PickerKind::Piece,
                priority: false,
            },
            PickerCandidateArg {
                id: "p2".to_string(),
                title: "Ballade".to_string(),
                subtitle: "Chopin".to_string(),
                notes: None,
                tags: vec![],
                created_at: "2026-01-02".to_string(),
                last_practiced_at: None,
                kind: PickerKind::Piece,
                priority: false,
            },
        ];
        let sort = PickerSortArg {
            field: PickerSortField::Title,
            direction: PickerSortDirection::Ascending,
        };

        let ids = sort_and_filter_picker_candidates(
            candidates,
            sort,
            String::new(),
            PickerFilterArg::default(),
        );

        assert_eq!(
            ids,
            vec!["p2", "p1"],
            "ascending title order across the FFI boundary"
        );
    }

    #[test]
    fn sort_and_filter_picker_candidates_filters_by_search() {
        let candidates = vec![PickerCandidateArg {
            id: "p1".to_string(),
            title: "Clair de Lune".to_string(),
            subtitle: "Debussy".to_string(),
            notes: None,
            tags: vec![],
            created_at: "2026-01-01".to_string(),
            last_practiced_at: None,
            kind: PickerKind::Piece,
            priority: false,
        }];
        let sort = PickerSortArg {
            field: PickerSortField::Title,
            direction: PickerSortDirection::Ascending,
        };

        let ids = sort_and_filter_picker_candidates(
            candidates,
            sort,
            "nonexistent".to_string(),
            PickerFilterArg::default(),
        );

        assert!(ids.is_empty());
    }

    /// The filter crosses as FFI types and reaches the core's rule: every
    /// `PickerKind` arm converts to the kind it names.
    #[test]
    fn sort_and_filter_picker_candidates_scopes_through_the_ffi_types() {
        let candidate =
            |id: &str, kind: PickerKind, priority: bool, tag: &str| PickerCandidateArg {
                id: id.to_string(),
                title: id.to_string(),
                subtitle: String::new(),
                notes: None,
                tags: vec![tag.to_string()],
                created_at: "2026-01-01".to_string(),
                last_practiced_at: None,
                kind,
                priority,
            };
        let candidates = vec![
            candidate("p1", PickerKind::Piece, true, "recital"),
            candidate("p2", PickerKind::Piece, false, "recital"),
            candidate("e1", PickerKind::Exercise, true, "warm-up"),
            candidate("e2", PickerKind::Exercise, false, "Warm-up"),
        ];
        let sort = PickerSortArg {
            field: PickerSortField::Title,
            direction: PickerSortDirection::Ascending,
        };
        let cases: [(PickerFilterArg, &[&str]); 4] = [
            (
                PickerFilterArg {
                    kind: Some(PickerKind::Piece),
                    ..Default::default()
                },
                &["p1", "p2"],
            ),
            (
                PickerFilterArg {
                    kind: Some(PickerKind::Exercise),
                    ..Default::default()
                },
                &["e1", "e2"],
            ),
            (
                PickerFilterArg {
                    priority_only: true,
                    ..Default::default()
                },
                &["e1", "p1"],
            ),
            (
                PickerFilterArg {
                    tags: vec!["WARM-UP".to_string()],
                    ..Default::default()
                },
                &["e1", "e2"],
            ),
        ];
        for (filter, expected) in cases {
            let ids = sort_and_filter_picker_candidates(
                candidates.clone(),
                sort,
                String::new(),
                filter.clone(),
            );
            assert_eq!(ids, expected, "{filter:?}");
        }
        for kind in [ItemKind::Piece, ItemKind::Exercise] {
            assert_eq!(ItemKind::from(PickerKind::from(kind.clone())), kind);
        }
    }

    /// Every `PickerSortField` and `PickerSortDirection` combination, driven
    /// through the core enums and the reverse `From` conversions, so a
    /// transposed arm in either `From` table is caught rather than surviving
    /// on `Title`/`Ascending` alone.
    #[test]
    fn sort_and_filter_picker_candidates_covers_every_field_and_direction() {
        let candidates = vec![
            PickerCandidateArg {
                id: "a".to_string(),
                title: "Bravo".to_string(),
                subtitle: String::new(),
                notes: None,
                tags: vec![],
                created_at: "2026-01-03".to_string(),
                last_practiced_at: Some("2026-02-02".to_string()),
                kind: PickerKind::Piece,
                priority: false,
            },
            PickerCandidateArg {
                id: "b".to_string(),
                title: "Alpha".to_string(),
                subtitle: String::new(),
                notes: None,
                tags: vec![],
                created_at: "2026-01-01".to_string(),
                last_practiced_at: Some("2026-02-03".to_string()),
                kind: PickerKind::Piece,
                priority: false,
            },
            PickerCandidateArg {
                id: "c".to_string(),
                title: "Charlie".to_string(),
                subtitle: String::new(),
                notes: None,
                tags: vec![],
                created_at: "2026-01-02".to_string(),
                last_practiced_at: Some("2026-02-01".to_string()),
                kind: PickerKind::Piece,
                priority: false,
            },
        ];

        let cases: [(SortField, SortDirection, [&str; 3]); 6] = [
            (
                SortField::DateAdded,
                SortDirection::Ascending,
                ["b", "c", "a"],
            ),
            (
                SortField::DateAdded,
                SortDirection::Descending,
                ["a", "c", "b"],
            ),
            (SortField::Title, SortDirection::Ascending, ["b", "a", "c"]),
            (SortField::Title, SortDirection::Descending, ["c", "a", "b"]),
            (
                SortField::LastPracticed,
                SortDirection::Ascending,
                ["c", "a", "b"],
            ),
            (
                SortField::LastPracticed,
                SortDirection::Descending,
                ["b", "a", "c"],
            ),
        ];

        for (field, direction, expected) in cases {
            let sort = PickerSortArg {
                field: field.into(),
                direction: direction.into(),
            };
            let ids = sort_and_filter_picker_candidates(
                candidates.clone(),
                sort,
                String::new(),
                PickerFilterArg::default(),
            );
            assert_eq!(
                ids, expected,
                "field {field:?} direction {direction:?} should order {expected:?}"
            );
        }
    }

    #[test]
    fn the_six_oclock_spoke_flips_gb_to_f_sharp_across_the_plain_call() {
        let first = key_next_on_tap(None, 6, WheelMode::Major)
            .expect("decodes")
            .expect("on the wheel");
        let first_key = decode_key(&first.key).expect("a key");
        assert_eq!(
            (first_key.spelling().as_str(), first.flipped),
            ("Gb", false)
        );
        let second = key_next_on_tap(Some(first.key), 6, WheelMode::Major)
            .expect("decodes")
            .expect("on the wheel");
        let second_key = decode_key(&second.key).expect("a key");
        assert_eq!(
            (second_key.spelling().as_str(), second.flipped),
            ("F#", true)
        );
        assert_eq!(
            key_label(second.key.clone()).expect("decodes"),
            "F\u{266f} major"
        );
        assert_eq!(
            key_wheel_selection(second.key).expect("decodes"),
            Some(WheelSelection {
                ring: 6,
                mode: WheelMode::Major,
                spelling: "F#".into()
            })
        );
        let minor_six = &key_wheel()[18];
        assert_eq!(
            (
                minor_six.mode,
                minor_six.primary.as_str(),
                minor_six.alt.as_deref()
            ),
            (WheelMode::Minor, "Eb", Some("D#"))
        );
    }

    #[test]
    fn a_set_of_keys_switches_and_fills_across_the_plain_calls() {
        let added = key_set_tap(vec![], 6, WheelMode::Major).expect("decodes");
        let switched = key_set_tap(added, 6, WheelMode::Major).expect("decodes");
        let filled = key_set_add_all(switched, WheelMode::Major).expect("decodes");
        let labels: Vec<String> = filled
            .into_iter()
            .map(|k| key_label(k).expect("decodes"))
            .collect();
        assert_eq!(labels.len(), 12);
        assert_eq!(labels[..2], ["F\u{266f} major", "C major"]);
        assert!(matches!(
            key_set_tap(vec![vec![0xff]], 0, WheelMode::Major),
            Err(CoreError::Bridge(_))
        ));
    }

    /// The store's columns cross the plain call and come back as written;
    /// text the core cannot read is `None`, never a guess (#2106).
    #[test]
    fn a_stored_key_round_trips_through_the_plain_calls() {
        let bytes = key_from_stored(Some("Eb".to_string()), Some(WheelMode::Minor))
            .expect("decodes")
            .expect("a key");
        assert_eq!(
            key_to_stored(bytes).expect("decodes"),
            StoredKey {
                text: "Eb".to_string(),
                mode: Some(WheelMode::Minor)
            }
        );
        assert_eq!(
            key_from_stored(Some("C dorian".to_string()), None).expect("decodes"),
            None
        );
        assert!(key_to_stored(vec![9, 9]).is_err(), "bad bytes are an error");
    }

    /// The shell's columns cross both plain calls and come back as written;
    /// a time that does not parse refuses the row rather than guessing.
    #[test]
    fn a_stored_session_row_round_trips_through_the_plain_calls() {
        let row = StoredSessionRow {
            id: "s1".to_string(),
            started_at: "2026-09-01T10:00:00Z".to_string(),
            completed_at: "2026-09-01T10:10:00Z".to_string(),
            total_duration_secs: 600,
            completion_status: "ended_early".to_string(),
            session_notes: Some("tired".to_string()),
            entries: r#"[{"id":"e1","itemId":"i1","itemTitle":"Scales","itemType":"exercise","position":0,"durationSecs":60,"status":"completed","plannedVariationIds":[],"plays":[{"id":"p1","variationIds":[],"startedAt":"2026-09-01T10:00:00Z","seconds":60,"tempoChanges":[],"score":3,"away":[]}],"segments":[],"gotInTheWay":[],"notePoints":[]}]"#.to_string(),
            session_score: Some(4),
            capture_version: Some(1),
        };
        let read = session_from_stored(row.clone()).expect("reads");
        assert!(read.unreadable.is_empty());
        assert_eq!(session_to_stored(read.session).expect("writes"), row);

        let bad = StoredSessionRow {
            started_at: "never".to_string(),
            ..row
        };
        assert!(session_from_stored(bad).is_err());
        assert!(
            session_to_stored(vec![9, 9]).is_err(),
            "bad bytes are an error"
        );
    }

    fn draft_bytes(draft: &crate::PhotoDraft) -> Vec<u8> {
        let mut bytes = Vec::new();
        BincodeFfiFormat::serialize(&mut bytes, draft).expect("serialize");
        bytes
    }

    fn blank(field: FormReadField) -> FormFieldNow {
        FormFieldNow {
            field,
            text: String::new(),
            holds_read: false,
        }
    }

    /// The draft the shell holds crosses as bytes and reaches the core's rule
    /// intact: the tempo still splits and each weak mark survives (#2229).
    #[test]
    fn fill_form_from_read_decodes_the_draft_the_view_model_sent() {
        let draft = crate::PhotoDraft {
            title: Some(crate::TextDraftField {
                value: "Autumn Leaves".to_string(),
                source: crate::DraftSource::Recognised,
                confidence: 0.9,
                weak: false,
            }),
            composer: None,
            tempo: Some(crate::TempoDraftField {
                value: crate::Tempo {
                    marking: Some("Moderato".to_string()),
                    bpm: Some(120),
                },
                source: crate::DraftSource::Suggested,
                confidence: 0.3,
                weak: true,
            }),
            chart_text: None,
        };
        let fields = vec![
            blank(FormReadField::Title),
            FormFieldNow {
                field: FormReadField::Marking,
                text: "Slowly".to_string(),
                holds_read: false,
            },
            blank(FormReadField::Bpm),
        ];

        let fills = fill_form_from_read(draft_bytes(&draft), fields).expect("decodes");

        assert_eq!(
            fills,
            vec![
                FormFieldFill {
                    field: FormReadField::Title,
                    value: "Autumn Leaves".to_string(),
                    weak: false,
                },
                FormFieldFill {
                    field: FormReadField::Bpm,
                    value: "120".to_string(),
                    weak: true,
                },
            ]
        );
    }

    #[test]
    fn fill_form_from_read_refuses_bytes_that_are_not_a_draft() {
        let result = fill_form_from_read(vec![7, 0, 0], vec![blank(FormReadField::Title)]);
        assert!(matches!(result, Err(CoreError::Bridge(_))));
    }
}
