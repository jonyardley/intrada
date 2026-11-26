use serde::{Deserialize, Serialize};

use super::{ItemPracticeSummary, ScoreHistoryEntry};
use crate::domain::chart::{ChordChart, ScaffoldKind};
use crate::domain::item::ItemKind;
use crate::domain::key::Key;
use crate::domain::Metre;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct LinkedExerciseView {
    pub id: String,
    pub title: String,
    pub key: Option<Key>,
    /// "E♭ major", as the musician reads it.
    pub key_label: Option<String>,
    pub tempo_marking: Option<String>,
    pub tempo_bpm: Option<u16>,
    /// "Lent · ♩ = 70", and its spoken form.
    pub tempo_line: Option<String>,
    pub tempo_line_spoken: Option<String>,
    pub practice: Option<ItemPracticeSummary>,
    /// This exercise's latest score *on the piece it's linked from* (#1087 B2),
    /// derived from the shared session block, distinct from `practice`, which
    /// is the exercise's flat score across every context. `None` when the pair
    /// has never been practised together (or scored) in a session.
    #[serde(default)]
    pub piece_context_score: Option<u8>,
}

/// The derived scaffold curriculum for a charted piece: the read-only preview
/// (Phase A). Recomputed from the stored chart in `build_view`; the shell renders
/// it and never derives.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ScaffoldPreviewView {
    /// The key the exercises are derived in (the piece's key), as read.
    pub key: String,
    pub specs: Vec<ScaffoldSpecView>,
    /// How many specs hit the arpeggio fallback (out-of-vocab chord).
    pub fallback_total: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ScaffoldSpecView {
    /// Row identity: the shell sends the ticked `kind`s back on commit and the
    /// core re-derives, so spec content never crosses the wire (#1106).
    pub kind: ScaffoldKind,
    pub title: String,
    pub rationale: String,
    pub key: String,
    /// This spec fell back to the chord's arpeggio for at least one change.
    pub fallback: bool,
    /// A matching exercise is already linked to the piece (dedup surface).
    pub already_linked: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PieceRefView {
    pub id: String,
    pub title: String,
    /// Composer, when the piece has one, which disambiguates identically titled
    /// pieces in provenance UI (#1049).
    pub subtitle: Option<String>,
}

/// One piece an exercise is used in, or the "On its own" bucket. A row comes
/// from the piece's live exercise links (an intention), from practising the
/// two together (history, resolved per session from `group_id`, #1087 B1), or
/// from both; `linked` says which (#1363).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ExerciseUsageView {
    /// `None` is the "On its own" bucket: practice with no piece in the block.
    pub piece: Option<PieceRefView>,
    /// Always `false` for "On its own".
    pub linked: bool,
    pub latest_score: Option<u8>,
    /// Distinct sessions the exercise was practised in, in this context.
    pub session_count: usize,
    /// Most recent session date for this context (RFC3339, sorts as a string).
    pub last_practiced_at: Option<String>,
    /// `true` when this row's `piece` no longer exists in the library: the
    /// piece was deleted since it was practised (#1093, decision 2a). The row
    /// is kept (its sessions are real history), but `piece.title` is the
    /// snapshot title and the shell renders the row muted + non-tappable. Always
    /// `false` for the "On its own" bucket.
    #[serde(default)]
    pub piece_removed: bool,
    /// The row names a piece still in the library: it opens, and it counts
    /// towards the card's piece total.
    pub piece_in_library: bool,
    /// The row shows the link button: a piece in the library, practised
    /// alongside but not linked.
    pub offers_link: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct LibraryItemView {
    pub id: String,
    pub item_type: ItemKind,
    pub title: String,
    pub subtitle: String,
    pub key: Option<Key>,
    /// "E♭ major", as the musician reads it.
    pub key_label: Option<String>,
    pub tempo_marking: Option<String>,
    pub tempo_bpm: Option<u16>,
    /// "Lent · ♩ = 70", and its spoken form.
    pub tempo_line: Option<String>,
    pub tempo_line_spoken: Option<String>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
    pub practice: Option<ItemPracticeSummary>,
    pub priority: bool,
    pub linked_exercises: Vec<LinkedExerciseView>,
    /// For exercises: every piece this exercise is used in, plus the "On its
    /// own" bucket, links and practice history merged (#1363). Empty for
    /// pieces.
    #[serde(default)]
    pub used_in: Vec<ExerciseUsageView>,
    /// For a charted piece: the derived scaffold curriculum (read-only preview).
    /// `None` for exercises and un-charted pieces.
    #[serde(default)]
    pub scaffold_preview: Option<ScaffoldPreviewView>,
    /// The parsed chord chart, for pieces that have one; the shell renders the
    /// bar grid from `symbol.raw` and pre-fills the editor from it.
    #[serde(default)]
    pub chord_chart: Option<ChordChart>,
    /// The piece's time signature, which seeds the click's bar (#1499).
    pub metre: Option<Metre>,
    /// The live library variations this item uses, in the order chosen, with
    /// their practice state (#2246).
    pub variations: Vec<VariationView>,
    #[serde(default)]
    pub photo_id: Option<String>,
    /// The keys chosen for practice, in order, with their practice state.
    pub keys: Vec<ItemKeyView>,
    /// "3 of 12 solid" beside the Keys heading; `None` with no keys (#2366).
    pub keys_caption: Option<String>,
    /// The same count beside the Variations heading.
    pub variations_caption: Option<String>,
    /// The wedge the key picker lights for the stored key (#2074).
    pub key_selection: Option<crate::domain::key::KeyWheelSelection>,
    /// Whole minutes this item usually takes, learnt from what is played (#3005).
    pub usual_minutes: u32,
    /// False until a play counts (#3005).
    pub usual_minutes_from_history: bool,
    pub include_in_warm_up: bool,
}

/// One library variation an item uses, with its practice state on that item:
/// the marks of the plays on it (#2246).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct VariationView {
    pub id: String,
    pub label: String,
    pub latest_score: Option<u8>,
    pub score_history: Vec<ScoreHistoryEntry>,
    pub caption: String,
}

/// One of an item's keys for practice, with the marks of the plays in it,
/// either spelling (#2106).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ItemKeyView {
    pub key: Key,
    pub label: String,
    pub latest_score: Option<u8>,
    pub caption: String,
}

/// A live row of the library's variation list, for the pickers (#2246).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct VariationOptionView {
    pub id: String,
    pub label: String,
    /// "On Clair de lune" for one item, "On 2 items" for more, `None` when
    /// no item uses it, so a rename shows how far it reaches (#2366).
    pub usage: Option<String>,
}

/// A key or variation counts as solid once its latest mark reaches this, of
/// 10 (#1762).
pub(crate) const SOLID_MIN: u8 = 8;

/// "3 of 12 solid" over the latest marks of an item's keys or variations;
/// `None` when it holds none (#2366).
pub(crate) fn solid_caption(latest: impl Iterator<Item = Option<u8>>) -> Option<String> {
    let (solid, total) = latest.fold((0, 0), |(solid, total), mark| {
        (
            solid + usize::from(mark.is_some_and(|m| m >= SOLID_MIN)),
            total + 1,
        )
    });
    (total > 0).then(|| format!("{solid} of {total} solid"))
}

/// The saved mark in the musician's words: the one wording the Library row and
/// the picker's out-of-session fallback share (#1809).
pub(crate) fn saved_mark_caption(latest_score: Option<u8>) -> String {
    match latest_score {
        Some(score) => format!("{score} of {}", crate::validation::MAX_SCORE),
        None => "Not yet played".to_string(),
    }
}

// ── Test fixtures ─────────────────────────────────────────────────────
// One place per many-field view type, composed with struct update, so a new
// field costs one edit rather than one per call site (code-and-tests.md, Tests).

#[cfg(test)]
impl LibraryItemView {
    pub(crate) fn fixture(id: &str, title: &str, item_type: ItemKind) -> Self {
        Self {
            id: id.to_string(),
            item_type,
            title: title.to_string(),
            subtitle: String::new(),
            key: None,
            key_label: None,
            tempo_marking: None,
            tempo_bpm: None,
            tempo_line: None,
            tempo_line_spoken: None,
            notes: None,
            tags: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
            practice: None,
            priority: false,
            linked_exercises: Vec::new(),
            used_in: Vec::new(),
            scaffold_preview: None,
            chord_chart: None,
            metre: None,
            variations: Vec::new(),
            photo_id: None,
            keys: Vec::new(),
            keys_caption: None,
            variations_caption: None,
            key_selection: None,
            usual_minutes: 10,
            usual_minutes_from_history: false,
            include_in_warm_up: false,
        }
    }
}

#[cfg(test)]
impl VariationView {
    pub(crate) fn fixture(id: &str, label: &str) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            latest_score: None,
            score_history: Vec::new(),
            caption: saved_mark_caption(None),
        }
    }

    pub(crate) fn scored(self, score: u8) -> Self {
        Self {
            latest_score: Some(score),
            caption: saved_mark_caption(Some(score)),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Solid counts (#2366) ──

    #[test]
    fn solid_caption_counts_latest_marks_of_eight_or_more() {
        let cases: [(&[Option<u8>], Option<&str>); 4] = [
            (&[], None),
            (&[None, Some(7)], Some("0 of 2 solid")),
            (&[Some(8), Some(10), Some(7), None], Some("2 of 4 solid")),
            (&[Some(9)], Some("1 of 1 solid")),
        ];
        for (marks, expected) in cases {
            assert_eq!(
                solid_caption(marks.iter().copied()).as_deref(),
                expected,
                "{marks:?}"
            );
        }
    }

    /// `ExerciseUsageView` crosses the bincode FFI bridge inside the
    /// ViewModel; guard it against the #846 silent-drop class.
    #[test]
    fn exercise_usage_view_round_trips_on_ffi_bincode_wire() {
        crate::domain::types::assert_round_trips(ExerciseUsageView {
            piece: Some(PieceRefView {
                id: "P".to_string(),
                title: "Sonata".to_string(),
                subtitle: Some("Beethoven".to_string()),
            }),
            linked: true,
            latest_score: Some(6),
            session_count: 2,
            last_practiced_at: Some("2026-07-01T00:00:00+00:00".to_string()),
            piece_removed: true,
            piece_in_library: false,
            offers_link: true,
        });
    }

    /// `ScaffoldPreviewView` crosses the bincode wire inside the ViewModel
    /// (`LibraryItemView.scaffold_preview`); guard it against the #846 drop class.
    #[test]
    fn scaffold_preview_view_round_trips_on_ffi_bincode_wire() {
        crate::domain::types::assert_round_trips(ScaffoldPreviewView {
            key: "G".to_string(),
            specs: vec![ScaffoldSpecView {
                kind: ScaffoldKind::Shells,
                title: "Shells".to_string(),
                rationale: "3rd + 7th of every chord".to_string(),
                key: "G".to_string(),
                fallback: false,
                already_linked: true,
            }],
            fallback_total: 0,
        });
    }

    /// The item's variations and keys cross the bincode wire inside
    /// `LibraryItemView`; guard them against the #846 drop class.
    #[test]
    fn variation_and_key_views_round_trip_on_ffi_bincode_wire() {
        crate::domain::types::assert_round_trips(VariationView {
            id: "v-1".to_string(),
            label: "Hands separately".to_string(),
            latest_score: Some(8),
            score_history: vec![ScoreHistoryEntry {
                session_date: "2026-07-01T00:00:00+00:00".to_string(),
                score: 8,
                session_id: "s1".to_string(),
            }],
            caption: "8 of 10".to_string(),
        });
        let key = crate::domain::key::Key::parse("Eb minor").expect("a key");
        crate::domain::types::assert_round_trips(ItemKeyView {
            key,
            label: key.label(),
            latest_score: None,
            caption: "Not yet played".to_string(),
        });
    }

    /// `LinkedExerciseView` crosses the same bincode wire; `piece_context_score`
    /// (#1087 B2) is a trailing `Option`; guard it against the #846 drop class.
    #[test]
    fn linked_exercise_view_round_trips_on_ffi_bincode_wire() {
        crate::domain::types::assert_round_trips(LinkedExerciseView {
            id: "E".to_string(),
            title: "Enclosures".to_string(),
            key: crate::domain::key::Key::parse("C minor"),
            key_label: Some("C minor".to_string()),
            tempo_marking: Some("Allegro".to_string()),
            tempo_bpm: Some(132),
            tempo_line: Some("Allegro · ♩ = 132".to_string()),
            tempo_line_spoken: Some("Allegro, 132 beats per minute".to_string()),
            practice: None,
            piece_context_score: Some(7),
        });
    }
}
