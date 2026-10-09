use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::chart::{ChordChart, ScaffoldKind};
use super::key::Key;
pub(crate) use super::link::LinkEdit;
pub use super::link::{ExerciseLink, LinkChange, LinkTarget};
use super::metre::Metre;
pub use super::section::{
    BarRange, BarsInput, ItemSection, SectionChange, SectionEdit, SectionKind,
};
use super::types::{CreateItem, KeyEdit, Tempo, UpdateItem};
use crate::app::{Effect, Event};
use crate::error::LibraryError;
use crate::model::{FormErrorField, FormErrorTarget, Model};
use crate::validation;

/// Discriminates between a piece (repertoire) and an exercise (technique drill).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
#[serde(rename_all = "lowercase")]
pub enum ItemKind {
    Piece,
    Exercise,
}

impl fmt::Display for ItemKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ItemKind::Piece => write!(f, "piece"),
            ItemKind::Exercise => write!(f, "exercise"),
        }
    }
}

/// Major or minor, a `Key`'s mode.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
#[serde(rename_all = "lowercase")]
pub enum Modality {
    Major,
    Minor,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct Item {
    pub id: String,
    pub title: String,
    pub kind: ItemKind,
    pub composer: Option<String>,
    /// The written key (#2106).
    pub key: Option<Key>,
    pub tempo: Option<Tempo>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub priority: bool,
    /// Parsed chord changes (pieces only). Additive + local-first; rides the
    /// piece's `updated_at`. `None` for exercises and un-charted pieces.
    #[serde(default)]
    pub chord_chart: Option<ChordChart>,
    /// An opaque ulid the shell resolves to a file; the bytes never cross the
    /// bridge (`specs/piece-from-photo.md`, key decision 1).
    #[serde(default)]
    pub photo_id: Option<String>,
    /// The piece's time signature, the one source of truth the chart derives
    /// from (#1499). `None` means the piece does not declare one.
    #[serde(default)]
    pub metre: Option<Metre>,
    /// Tombstones included (#2245).
    #[serde(default)]
    pub sections: Vec<ItemSection>,
    /// The library variations this item uses, in the order chosen (#2246).
    #[serde(default)]
    pub variation_ids: Vec<String>,
    /// The keys chosen for practice, in order; the written key is `key`.
    #[serde(default)]
    pub keys: Vec<Key>,
    /// Pieces only; tombstones included (#2248).
    #[serde(default)]
    pub exercise_links: Vec<ExerciseLink>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum ItemEvent {
    Add(CreateItem),
    Update {
        id: String,
        input: UpdateItem,
    },
    Delete {
        id: String,
    },
    /// Parse `raw_chart` and store it on the piece. A parse error surfaces on
    /// `last_error` and stores nothing, never a partial chart.
    SetChordChart {
        piece_id: String,
        raw_chart: String,
    },
    /// Materialise the selected scaffold `kinds` into real exercises linked to
    /// the piece. The core re-derives from the stored chart (deterministic), so
    /// only the ticked `kind`s cross the wire, never spec content (#1106).
    /// Dedups by title against the piece's already-linked exercises; a batch
    /// with nothing new to add is a no-op.
    CommitScaffold {
        piece_id: String,
        kinds: Vec<ScaffoldKind>,
    },
    /// Point the item at the photo the shell has already written to disk,
    /// replacing any it already had. Kept out of `Update` because
    /// `UpdateItem`'s three-state `Option<Option<T>>` is the fiddliest encoding
    /// on the bridge.
    SetPhoto {
        id: String,
        photo_id: String,
    },
    ClearPhoto {
        id: String,
    },
    /// Read a photo the shell has already written to disk into a draft the
    /// user confirms. Takes only the `photo_id`, not an item id: on Add there
    /// is no item to name yet, and the draft is what fills the form
    /// (#1446, spec open question 4).
    ReadPhoto {
        photo_id: String,
    },
    /// Set or clear the item's time signature. Kept out of `Update` for the
    /// same reason as `SetPhoto`. A charted piece's beat split is derived
    /// again from the new metre.
    SetMetre {
        id: String,
        metre: Option<Metre>,
    },
    /// Create a piece with everything the add form could carry: its chord
    /// chart and the exercises written or chosen alongside it, in one save
    /// (#1390). `chart` is raw text, parsed here against the piece's own key,
    /// so a rejected bar leaves no half-made piece.
    AddPieceInFull {
        piece: CreateItem,
        chart: Option<String>,
        exercises: Vec<ScaffoldEntry>,
    },
    /// The item's whole variation set, in order (#2246): `variation_ids` name
    /// library rows, and each of `new_labels` reuses a live row with that
    /// label or mints one, appended after them. Removing one from an item
    /// leaves it in the library with its history.
    UpdateItemVariations {
        id: String,
        variation_ids: Vec<String>,
        new_labels: Vec<String>,
    },
    /// The item's whole list of keys for practice, in order.
    UpdateKeys {
        id: String,
        keys: Vec<Key>,
    },
    /// The exercise's whole set of links across pieces, every changed piece
    /// saved in one batch. Refused whole on the first invalid row.
    SetExerciseLinks {
        exercise_id: String,
        targets: Vec<LinkTarget>,
    },
    /// The piece's exercise picker (#2379): only the ticked exercises and the
    /// ones written there. A kept exercise keeps its sections; a new tick, then
    /// each written one, links to the whole piece after them.
    ChoosePieceExercises {
        piece_id: String,
        exercise_ids: Vec<String>,
        written: Vec<CreateItem>,
    },
    /// The exercise's piece picker (#2379): only the ticked pieces. A kept piece
    /// keeps the exercise's sections on it; a new one links as a whole.
    ChooseExercisePieces {
        exercise_id: String,
        piece_ids: Vec<String>,
    },
    /// The edit form's whole save (#2228): fields and, for an exercise, the
    /// variation set as `UpdateItemVariations` takes it. Every part is checked
    /// before any lands, so a refusal leaves the item as it was.
    Edit {
        id: String,
        input: UpdateItem,
        variation_ids: Vec<String>,
        new_labels: Vec<String>,
    },
    /// One section saved, removed or the list arranged (#2447).
    ChangeSection {
        id: String,
        change: SectionChange,
    },
    /// One exercise's links set, moved or removed on the piece's card (#2447).
    ChangePieceLink {
        piece_id: String,
        change: LinkChange,
    },
}

/// Acts on the library's own variation rows, not on any one item's set.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum VariationEvent {
    Rename {
        id: String,
        label: String,
    },
    /// Tombstones the row: gone from every picker, kept for the plays.
    Delete {
        id: String,
    },
}

/// One exercise a new piece is created with: written alongside it, or chosen
/// from the library.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum ScaffoldEntry {
    New(CreateItem),
    Existing { id: String },
}

/// The reconciliation key shared by `CommitScaffold` and the preview's
/// `already_linked` flag: kinds (from the reserved tag, rename-robust) + titles
/// (guarding hand-made exercises) already linked to the piece.
pub(crate) fn linked_scaffold_state(
    model: &Model,
    piece_id: &str,
) -> (
    std::collections::HashSet<ScaffoldKind>,
    std::collections::HashSet<String>,
) {
    let linked_ids: std::collections::HashSet<String> = model
        .items
        .iter()
        .find(|i| i.id == piece_id)
        .map(|p| p.linked_exercise_ids().into_iter().collect())
        .unwrap_or_default();
    let linked = model.items.iter().filter(|i| linked_ids.contains(&i.id));

    let mut kinds = std::collections::HashSet::new();
    let mut titles = std::collections::HashSet::new();
    for item in linked {
        titles.insert(item.title.to_lowercase());
        for tag in &item.tags {
            if let Some(kind) = ScaffoldKind::from_scaffold_tag(tag) {
                kinds.insert(kind);
            }
        }
    }
    (kinds, titles)
}

/// Whether a spec is already linked: by reserved kind (rename-robust) or a
/// hand-made title collision.
pub(crate) fn scaffold_already_linked(
    kinds: &std::collections::HashSet<ScaffoldKind>,
    titles: &std::collections::HashSet<String>,
    kind: ScaffoldKind,
    title: &str,
) -> bool {
    kinds.contains(&kind) || titles.contains(&title.to_lowercase())
}

/// A refused write, with the form field it points at when it has one.
fn refuse(model: &mut Model, error: &LibraryError) -> Command<Effect, Event> {
    model.last_error_target = form_field(error).map(|field| FormErrorTarget::Piece { field });
    model.raise_error(error.to_string());
    crux_core::render::render()
}

fn persist_item(model: &mut Model, item: Item) -> Command<Effect, Event> {
    model.clear_error();
    Command::all([
        crate::persistence::save_item(model, item),
        crux_core::render::render(),
    ])
}

/// The form field a validation failure belongs to, where the add form has one
/// to mark (#1595). Anything else, including a missing item, points nowhere.
fn form_field(error: &LibraryError) -> Option<FormErrorField> {
    let LibraryError::Validation { field, .. } = error else {
        return None;
    };
    match field.as_str() {
        "title" => Some(FormErrorField::Title),
        "composer" => Some(FormErrorField::Composer),
        "tempo" => Some(FormErrorField::Tempo),
        "notes" => Some(FormErrorField::Notes),
        "tags" => Some(FormErrorField::Tags),
        "labels" | "variation_labels" => Some(FormErrorField::Variations),
        "sections" => Some(FormErrorField::Sections),
        _ => None,
    }
}

mod create;
mod edit;
mod links;
mod metre;
mod photo;
mod sections;
pub(crate) use sections::update_sections;
#[cfg(test)]
impl Item {
    pub(crate) fn fixture(id: &str) -> Self {
        let at = DateTime::<Utc>::from_timestamp(1_790_000_000, 0).expect("fixed time");
        Self {
            id: id.to_string(),
            title: "Etude".to_string(),
            kind: ItemKind::Piece,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: Vec::new(),
            created_at: at,
            updated_at: at,
            priority: false,
            chord_chart: None,
            photo_id: None,
            metre: None,
            sections: Vec::new(),
            variation_ids: Vec::new(),
            keys: Vec::new(),
            exercise_links: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests;
mod variations;

pub fn handle_item_event(event: ItemEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        ItemEvent::Add(input) => create::add(model, input),
        ItemEvent::AddPieceInFull {
            piece,
            chart,
            exercises,
        } => create::add_piece_in_full(model, piece, chart, exercises),
        ItemEvent::Update { id, input } => edit::update(model, id, input),
        ItemEvent::Edit {
            id,
            input,
            variation_ids,
            new_labels,
        } => edit::edit(model, id, input, variation_ids, new_labels),
        ItemEvent::Delete { id } => edit::delete(model, id),
        ItemEvent::SetPhoto { id, photo_id } => photo::set_photo(model, id, Some(photo_id)),
        ItemEvent::ClearPhoto { id } => photo::set_photo(model, id, None),
        ItemEvent::ReadPhoto { photo_id } => photo::read_photo(model, photo_id),
        ItemEvent::SetMetre { id, metre } => metre::set_metre(model, id, metre),
        ItemEvent::SetChordChart {
            piece_id,
            raw_chart,
        } => edit::set_chord_chart(model, piece_id, raw_chart),
        ItemEvent::UpdateItemVariations {
            id,
            variation_ids,
            new_labels,
        } => variations::update_item_variations(model, id, variation_ids, new_labels),
        ItemEvent::UpdateKeys { id, keys } => variations::update_keys(model, id, keys),
        ItemEvent::CommitScaffold { piece_id, kinds } => {
            links::commit_scaffold(model, piece_id, kinds)
        }
        ItemEvent::SetExerciseLinks {
            exercise_id,
            targets,
        } => links::set_exercise_links(model, exercise_id, targets),
        ItemEvent::ChoosePieceExercises {
            piece_id,
            exercise_ids,
            written,
        } => links::choose_piece_exercises(model, piece_id, exercise_ids, written),
        ItemEvent::ChooseExercisePieces {
            exercise_id,
            piece_ids,
        } => links::choose_exercise_pieces(model, exercise_id, piece_ids),
        ItemEvent::ChangeSection { id, change } => sections::change_section(model, id, change),
        ItemEvent::ChangePieceLink { piece_id, change } => {
            links::change_piece_link(model, piece_id, change)
        }
    }
}

pub fn handle_variation_event(event: VariationEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        VariationEvent::Rename { id, label } => variations::rename(model, id, label),
        VariationEvent::Delete { id } => variations::delete(model, id),
    }
}
