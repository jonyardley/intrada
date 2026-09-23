use crux_core::{
    bridge::{Bridge, EffectId},
    Core,
};

use crate::{Intrada, ItemKind, LibrarySort, SortDirection, SortField};

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
        Self {
            core: Bridge::new(Core::new()),
        }
    }

    pub fn update(&self, data: &[u8]) -> Result<Vec<u8>, CoreError> {
        let mut effects = Vec::new();
        self.core
            .update(data, &mut effects)
            .map_err(|e| CoreError::Bridge(e.to_string()))?;
        Ok(effects)
    }

    pub fn resolve(&self, id: u32, data: &[u8]) -> Result<Vec<u8>, CoreError> {
        let mut effects = Vec::new();
        self.core
            .resolve(EffectId(id), data, &mut effects)
            .map_err(|e| CoreError::Bridge(e.to_string()))?;
        Ok(effects)
    }

    pub fn view(&self) -> Result<Vec<u8>, CoreError> {
        let mut view = Vec::new();
        self.core
            .view(&mut view)
            .map_err(|e| CoreError::Bridge(e.to_string()))?;
        Ok(view)
    }
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

/// `ItemKind`'s copy at the plain-call boundary, as `PickerSortField` is.
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

/// The Add form has no saved exercise to read `LibraryItemView.shows_key`
/// from, so it asks with its own unsaved row count (#1783 decision 1).
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[must_use]
pub fn exercise_form_shows_key(live_variant_count: u32) -> bool {
    crate::domain::variant::shows_key_field(live_variant_count as usize)
}

/// The shell builds the crash-recovery blob's storage key from this, so a
/// shape change in the core retires the old key on its own (#1116).
#[cfg_attr(feature = "uniffi", uniffi::export)]
#[must_use]
pub fn session_blob_version() -> u32 {
    crate::domain::session::ActiveSession::BLOB_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
