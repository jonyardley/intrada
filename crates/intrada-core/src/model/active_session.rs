use serde::{Deserialize, Serialize};

use super::{KeyChoiceView, LastTimeView, SetlistEntryView};
use crate::domain::item::ItemKind;
use crate::domain::key::Key;
use crate::domain::metre::{ClickPresetOption, TempoBand};
use crate::domain::session::{ClickState, ReflectionAnswers, TempoReading};
use crate::domain::Metre;

/// One row of a variation picker, captioned by the core so a variation played
/// earlier in this item never reads as not yet played (#1784).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct PickerVariationView {
    pub id: String,
    pub label: String,
    pub caption: String,
}

/// What the practice screen offers on the current entry beyond the play.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ActiveRecordView {
    pub away_offer: Option<AwayOfferView>,
    /// `Some` while the item-complete sheet is open.
    pub finish: Option<FinishSheetView>,
    pub last_time: Option<LastTimeView>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct AwayOfferView {
    pub minutes: u32,
    /// "Away 6 minutes. Leave it out?"
    pub label: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct FinishSheetView {
    /// One per play, in play order.
    pub rows: Vec<FinishRowView>,
    /// What each row's key can be changed to; the variations are
    /// `current_variations`. Empty when the written key is the only choice.
    pub keys: Vec<KeyChoiceView>,
}

/// What one finish row says was played: the drafted way, else the recorded one.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct FinishRowView {
    pub play_id: String,
    pub label: String,
    /// Spelt as its row in the sheet's `keys`.
    pub key: Option<crate::domain::key::Key>,
    pub variation_ids: Vec<String>,
    pub can_change: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ActiveSessionView {
    pub current_item_title: String,
    pub current_item_type: ItemKind,
    pub current_position: usize,
    pub total_items: usize,
    pub started_at: String,
    /// When today's length runs out (RFC3339 UTC); the shell counts down to it
    /// and shows time over past it. `None` when no length was chosen (#2789).
    pub ends_at: Option<String>,
    /// Wall-clock anchor (RFC3339 UTC) for the *current item*. Resets to
    /// "now" on each item advance (Next / Skip). The shell derives the
    /// per-item elapsed timer from `Utc::now() - current_item_started_at`
    /// rather than incrementing a counter, so it survives WebView suspension /
    /// tab backgrounding without drift.
    pub current_item_started_at: String,
    pub entries: Vec<SetlistEntryView>,
    /// How the open play is being played, so the player can show it (#2246).
    /// Spelt as its row in `current_key_choices`; the written key is `None`,
    /// and an unkeyed play on an item with none reads as its first key.
    pub current_key: Option<Key>,
    /// The key chip's rows: one is shown as its name, two or more as a chip
    /// (#2788); empty when the item keeps no key.
    pub current_key_choices: Vec<KeyChoiceView>,
    /// The chip's text: the row `current_key` matches, else the first row;
    /// `None` only when `current_key_choices` is empty.
    pub current_key_label: Option<String>,
    pub current_variation_ids: Vec<String>,
    /// The open play's variations, named; `None` when it has none.
    pub current_play_label: Option<String>,
    pub current_planned_duration_secs: Option<u32>,
    pub next_item_title: Option<String>,
    /// The current entry's "Aim" note, set in the builder (`EntrySettingsSheet`).
    pub current_item_intention: Option<String>,
    /// The current item's library notes (`Item.notes`), distinct from
    /// `current_item_intention`, which is the session-level "Aim".
    pub current_item_notes: Option<String>,
    /// The anchor piece's title, when the current entry is a related exercise
    /// grouped into that piece's block; `None` for a standalone entry or when
    /// the current entry *is* the anchor piece itself.
    pub current_related_piece_title: Option<String>,
    /// The current item's own declared tempo marking/bpm (from its library
    /// entry): the practice target, distinct from `achieved_tempo` (what was
    /// actually played, logged after completion).
    pub current_item_tempo_marking: Option<String>,
    pub current_item_tempo_bpm: Option<u16>,
    pub current_item_tempo_line: Option<String>,
    pub current_item_tempo_line_spoken: Option<String>,
    /// The beats the click starts on for this item: the musician's default,
    /// fitted to the item's metre or 4/4 (#1915).
    pub current_click_sounding: u16,
    /// The current item's variations with the picker's caption for each (#1784).
    pub current_variations: Vec<PickerVariationView>,
    /// `Some` while the item-complete sheet is open, including after a resume (#2137).
    pub reflection: Option<ReflectionView>,
    /// The bar, tempo and target flag the click reseeds to on this item: 4/4
    /// when the item declares no metre, its BPM inside the band (#2225).
    pub click_seed_metre: Metre,
    pub click_seed_bpm: u16,
    /// False when the item declares no BPM, or one the band moved (#1942).
    pub click_seed_sounds_target: bool,
    /// The seed bar's patterns: an item may carry a valid grouping the
    /// sheet's `click_bars` table does not list.
    pub click_seed_presets: Vec<ClickPresetOption>,
    /// The pattern the item's own bar opens on when chosen again, since the table may not list it.
    pub click_seed_opening: u16,
    pub record: ActiveRecordView,
}

/// The open sheet's saved answers, and the click at its stamp, which seeds
/// the rows that have no stamp of their own.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ReflectionView {
    pub answers: ReflectionAnswers,
    pub reading: TempoReading,
    /// Measured against `current_item_started_at`, it stays the stamped seconds
    /// across a resume (#2297).
    pub stopped_at: String,
    /// One per markable play, in play order (#2230).
    pub tempos: Vec<ReflectionTempoView>,
    /// The item's plays summed, so a resume cannot skew it (#2352).
    pub elapsed_secs: u64,
}

/// A sheet row's tempo as it opens, counted in `click`'s unit (#2230).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct ReflectionTempoView {
    pub play_id: String,
    pub tempo: u16,
    pub click: ClickState,
    pub band: TempoBand,
    pub set_by_hand: bool,
    /// The latest score for this item in this row's key, most recent session first.
    pub last_score: Option<u8>,
    /// `last_score`, else the item's other keys' last scores averaged, else 5.
    pub start_score: u8,
}
