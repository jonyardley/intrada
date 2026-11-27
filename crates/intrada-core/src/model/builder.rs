use serde::{Deserialize, Serialize};

use super::{PickerVariationView, SetlistEntryView};

/// The one-tap offer of how the item was last played.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct LastTimeView {
    pub entry_id: String,
    pub variation_ids: Vec<String>,
    /// "Last time: Dotted".
    pub label: String,
    pub key: Option<crate::domain::key::Key>,
}

/// One row of a key picker: `key: None` is the written key.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct KeyChoiceView {
    pub key: Option<crate::domain::key::Key>,
    pub label: String,
    /// "Written key" under the written key's name; `None` on every other row.
    pub caption: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct EntryKeysView {
    pub entry_id: String,
    pub keys: Vec<KeyChoiceView>,
    /// The planned key, spelt as its row in `keys`.
    pub current: Option<crate::domain::key::Key>,
    pub current_label: String,
}

/// A unit in the builder queue: a block (a piece with its related exercises) or
/// a single standalone item. `group_id == None` means standalone.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SetlistBlockView {
    pub group_id: Option<String>,
    /// The block's anchor-piece title; `None` for a standalone item.
    pub piece_title: Option<String>,
    pub related_count: usize,
    /// The planned total ("10 min"); empty when nothing is planned.
    pub duration_display: String,
    pub entries: Vec<SetlistEntryView>,
    /// The anchor piece and its related exercises, in order; `None` and empty
    /// for a standalone item.
    pub piece: Option<SetlistEntryView>,
    pub related: Vec<SetlistEntryView>,
    /// Items in the session outside this block, which it cannot take (#2075).
    pub taken_elsewhere: Vec<String>,
    /// The row's second line (#2467): "Piece · 10 min" for a standalone item,
    /// "2 related, then piece · 10 min", "Piece only" or "10 min" for a block.
    pub caption: String,
    /// A collapsed block's line: the planned total, `None` when nothing is.
    pub planned_display: Option<String>,
    /// "+2 related", or "piece only".
    pub related_label: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct BuildingSetlistView {
    pub entries: Vec<SetlistEntryView>,
    pub item_count: usize,
    /// The same entries grouped into ordered units (blocks + standalone items).
    pub blocks: Vec<SetlistBlockView>,
    /// Sum of the entries' planned durations; `None` when nothing is planned
    /// so shells can fall back to counts-only copy.
    pub total_duration_display: Option<String>,
    pub total_duration_summary: Option<String>,
    pub length_mins: Option<u16>,
    /// "20 of 30 min planned"; `None` without a length (#1736) or before anything
    /// is planned, since the stepper above it already shows the length (#3043).
    pub length_summary: Option<String>,
    /// The variations each entry can be tagged to, for the entries whose item
    /// has any. Looked up in the whole library, so a search cannot empty it.
    pub entry_variations: Vec<EntryVariationsView>,
    /// For the entries whose item was last played in a key or on variations.
    pub last_times: Vec<LastTimeView>,
    /// For the entries whose item keeps keys of its own.
    pub entry_keys: Vec<EntryKeysView>,
    /// "10 min · 2 blocks · 3 items": the time only once some is planned,
    /// the blocks only once there is one (#2467).
    pub summary: String,
    /// "Start session · 10 min", or "Start session".
    pub start_title: String,
    /// The plan against today's length (#3009); `None` without a length.
    pub fit: Option<FitView>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct FitView {
    /// Timed rows only, each to the nearest whole minute.
    pub used_mins: u32,
    pub length_mins: u16,
    pub state: FitState,
    /// "72 of 60 min", then ", plus 2 untimed" when any row has no minutes.
    pub summary: String,
    pub offers: Vec<FitOffer>,
    /// "12 min over", "45 min left" or "Fits exactly".
    pub state_line: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum FitState {
    Under,
    Exact,
    Over,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum FitOffer {
    FillWithSuggestions,
    StretchToFit,
    TrimToFit,
    DropLastItem,
}

/// What an entry's plan can name: its item's variations.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct EntryVariationsView {
    pub entry_id: String,
    pub variations: Vec<PickerVariationView>,
}
