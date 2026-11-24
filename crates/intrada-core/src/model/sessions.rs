use serde::{Deserialize, Serialize};

use crate::domain::item::ItemKind;
use crate::domain::key::Key;
use crate::domain::session::{ClickState, CompletionStatus, EntryStatus};

/// One stretch of an item played one way, as the sheet and the Progress
/// screen read it (#1739, #2246). Labels are resolved here, tombstones
/// included, because a play on a variation since deleted still has to say
/// what it was.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PlayView {
    pub id: String,
    pub key: Option<Key>,
    pub variation_ids: Vec<String>,
    /// "E♭ major · Hands separately"; `None` for the item plain, in the
    /// written key.
    pub label: Option<String>,
    pub seconds: u64,
    pub duration_display: String,
    /// "4m 0s", "92 bpm": only what this play recorded, the duration always
    /// first, since rows that show it elsewhere drop it (#2515).
    pub meta_parts: Vec<String>,
    pub achieved_tempo: Option<u16>,
    pub click_pattern: Option<ClickState>,
    /// `achieved_tempo` in the unit of the click that stamped it, crotchets
    /// when no click did, so the shell never converts a tempo (#1761).
    pub tempo_display: Option<u16>,
    pub score: Option<u8>,
    /// Whether marking this play now would stick: `false` means the pending
    /// terminal transition will drop it as a stray tap on the picker before a
    /// mark could ever be attached (#1758). Only accurate on the reflection
    /// sheet, after `PrepareReflection`; mid-item it reads the open play as
    /// incidental regardless of how long it has actually run.
    pub is_markable: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PracticeSessionView {
    pub id: String,
    pub total_duration_display: String,
    pub total_duration_summary: String,
    pub completion_status: CompletionStatus,
    pub notes: Option<String>,
    pub entries: Vec<SetlistEntryView>,
    pub session_score: Option<u8>,
    /// The card's line of what was played, e.g. "Major scales in C, G and
    /// D · Arpeggios in E♭ major" (#1785). Built here rather than the shell
    /// so a long session is cut off honestly, at a fragment boundary with
    /// "and N more", instead of a mid-word ellipsis.
    pub played_summary: String,
    /// `Today`, `Yesterday` or `Sat 30 May`, on the week strip's clock (#2053).
    pub day_label: String,
    /// "3 pieces", "2 exercises", or "5 items" when the session spans both (#2515).
    pub item_count_display: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SetlistEntryView {
    pub id: String,
    pub item_id: String,
    pub item_title: String,
    pub item_type: ItemKind,
    pub position: usize,
    pub duration_display: String,
    pub status: EntryStatus,
    pub notes: Option<String>,
    pub intention: Option<String>,
    pub planned_duration_secs: Option<u32>,
    pub planned_duration_display: Option<String>,
    /// The block this entry belongs to in the builder; `None` = standalone.
    pub group_id: Option<String>,
    /// False for a block's piece: removing it alone would dissolve the block,
    /// so the block's own remove offers that instead.
    pub removable: bool,
    /// What the builder planned (#2246): any variations.
    pub planned_variation_ids: Vec<String>,
    /// "Hands separately"; `None` with no plan.
    pub planned_label: Option<String>,
    /// What was actually practised, in order.
    pub plays: Vec<PlayView>,
    /// The mean of the plays that carry a mark, rounded to nearest, and `None`
    /// when none do. The only place several marks collapse into one (#1739
    /// decision 9): per-variation history reads `plays` and never this.
    pub score_summary: Option<u8>,
    pub planned_key: Option<crate::domain::key::Key>,
    /// The builder's tags under the row: the key, then the variations still
    /// on the item (#2467). Empty outside the builder.
    pub plan_tags: Vec<String>,
}
