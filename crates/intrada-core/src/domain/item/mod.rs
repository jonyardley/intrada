use chrono::{DateTime, Utc};
use crux_core::Command;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::chart::{ChordChart, ScaffoldKind};
use super::metre::Metre;
use super::types::{CreateItem, Tempo, UpdateItem};
pub use super::variant::Variant;
use super::variant::VariantEdit;
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

/// Major/minor tonality, paired with `Item.key` (the tonic, e.g. "F#").
/// Selection/spelling logic lives in the shell's key picker.
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
    pub key: Option<String>,
    #[serde(default)]
    pub modality: Option<Modality>,
    pub tempo: Option<Tempo>,
    pub notes: Option<String>,
    pub tags: Vec<String>,
    // `#[serde(default)]` so absent fields (old clients / bincode) default to `[]`.
    #[serde(default)]
    pub linked_exercise_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(default)]
    pub priority: bool,
    /// Parsed chord changes (pieces only). Additive + local-first; rides the
    /// piece's `updated_at`. `None` for exercises and un-charted pieces.
    #[serde(default)]
    pub chord_chart: Option<ChordChart>,
    /// Ordered variation ladder (exercises only), tombstones included; appended
    /// last + `#[serde(default)]` so old rows / bincode snapshots decode to
    /// an empty ladder (#846). Persisted to the `variant` child table (#1083).
    #[serde(default)]
    pub variants: Vec<Variant>,
    /// An opaque ulid the shell resolves to a file; the bytes never cross the
    /// bridge (`specs/piece-from-photo.md`, key decision 1).
    #[serde(default)]
    pub photo_id: Option<String>,
    /// The piece's time signature, the one source of truth the chart derives
    /// from (#1499). `None` means the piece does not declare one.
    #[serde(default)]
    pub metre: Option<Metre>,
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
    LinkExercise {
        piece_id: String,
        exercise_id: String,
    },
    /// Create one hand-written exercise already linked to `piece_id`. Single
    /// event so the shell never has to learn the minted ulid: `Add` alone
    /// leaves the exercise unlinked and the shell with no id to link it by
    /// (#1431). Local-first only: the online create path reassigns ids
    /// server-side, which is the dangling-link trap #1108 marks inside
    /// `CommitScaffold`.
    AddLinkedExercise {
        piece_id: String,
        input: CreateItem,
    },
    UnlinkExercise {
        piece_id: String,
        exercise_id: String,
    },
    ReorderLinkedExercises {
        piece_id: String,
        ordered_ids: Vec<String>,
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
    /// Define an exercise's whole variation ladder from ordered `labels`,
    /// reconciled by case-insensitive label: matching variants keep their id
    /// (and score history), removed labels tombstone, re-added labels
    /// resurrect. Empty = clear the ladder. Local-first only until sync
    /// (#1083 C1).
    SetVariants {
        id: String,
        labels: Vec<String>,
    },
    /// Rename a variation in place, matched by `variant_id` rather than label:
    /// `SetVariants`'s label-keyed reconciliation can't express a rename
    /// (a changed label is indistinguishable from remove+add, which would
    /// drop score history). `new_label` is validated against the item's
    /// other live variations for duplicates (#1083 C4).
    RenameVariant {
        item_id: String,
        variant_id: String,
        new_label: String,
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
    /// so a rejected bar leaves no half-made piece. Local-first only, like
    /// `AddLinkedExercise`.
    AddPieceInFull {
        piece: CreateItem,
        chart: Option<String>,
        exercises: Vec<ScaffoldEntry>,
    },
    /// The Edit form's whole ladder in one write (#1783): rows carry the id
    /// they started from, so renames, reorders, removals and additions land
    /// together, and a swap of two labels is not a duplicate. Tombstones and
    /// the key migration follow `SetVariants`. Appended last: positional wire.
    UpdateVariants {
        id: String,
        variants: Vec<VariantEdit>,
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
    let linked_ids: std::collections::HashSet<&String> = model
        .items
        .iter()
        .find(|i| i.id == piece_id)
        .map(|p| p.linked_exercise_ids.iter().collect())
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
        crate::persistence::save_item(item),
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
        "labels" | "variant_labels" => Some(FormErrorField::Variations),
        _ => None,
    }
}

mod create;
mod edit;
mod links;
mod metre;
mod photo;
#[cfg(test)]
mod tests;
mod variations;

pub fn handle_item_event(event: ItemEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        ItemEvent::Add(input) => create::add(model, input),
        ItemEvent::AddLinkedExercise { piece_id, input } => {
            create::add_linked_exercise(model, piece_id, input)
        }
        ItemEvent::AddPieceInFull {
            piece,
            chart,
            exercises,
        } => create::add_piece_in_full(model, piece, chart, exercises),
        ItemEvent::Update { id, input } => edit::update(model, id, input),
        ItemEvent::Delete { id } => edit::delete(model, id),
        ItemEvent::SetPhoto { id, photo_id } => photo::set_photo(model, id, Some(photo_id)),
        ItemEvent::ClearPhoto { id } => photo::set_photo(model, id, None),
        ItemEvent::ReadPhoto { photo_id } => photo::read_photo(model, photo_id),
        ItemEvent::SetMetre { id, metre } => metre::set_metre(model, id, metre),
        ItemEvent::LinkExercise {
            piece_id,
            exercise_id,
        } => links::link_exercise(model, piece_id, exercise_id),
        ItemEvent::UnlinkExercise {
            piece_id,
            exercise_id,
        } => links::unlink_exercise(model, piece_id, exercise_id),
        ItemEvent::ReorderLinkedExercises {
            piece_id,
            ordered_ids,
        } => links::reorder_linked_exercises(model, piece_id, ordered_ids),
        ItemEvent::SetChordChart {
            piece_id,
            raw_chart,
        } => edit::set_chord_chart(model, piece_id, raw_chart),
        ItemEvent::SetVariants { id, labels } => variations::set_variants(model, id, labels),
        ItemEvent::UpdateVariants { id, variants } => {
            variations::update_ladder(model, id, variants)
        }
        ItemEvent::CommitScaffold { piece_id, kinds } => {
            links::commit_scaffold(model, piece_id, kinds)
        }
        ItemEvent::RenameVariant {
            item_id,
            variant_id,
            new_label,
        } => variations::rename_variant(model, item_id, variant_id, new_label),
    }
}
