use super::*;
use crate::domain::variation::{self, Variation};

pub(super) fn update_item_variations(
    model: &mut Model,
    id: String,
    variation_ids: Vec<String>,
    new_labels: Vec<String>,
) -> Command<Effect, Event> {
    let now = chrono::Utc::now();
    let (ids, minted) = match resolve_variations(model, variation_ids, new_labels, now) {
        Ok(set) => set,
        Err(e) => return refuse(model, &e),
    };

    let Some(item) = model.items.iter_mut().find(|i| i.id == id) else {
        model.raise_error(LibraryError::NotFound { id }.to_string());
        return crux_core::render::render();
    };
    if item.variation_ids == ids && minted.is_empty() {
        model.last_error = None;
        return crux_core::render::render();
    }
    item.variation_ids = ids;
    item.updated_at = now;
    let item = item.clone();
    model.last_error = None;
    save_with_minted(model, item, minted)
}

/// The item's variation ids in order, and the library rows to mint for labels
/// no live row has yet.
pub(super) fn resolve_variations(
    model: &Model,
    variation_ids: Vec<String>,
    new_labels: Vec<String>,
    now: chrono::DateTime<chrono::Utc>,
) -> Result<(Vec<String>, Vec<Variation>), LibraryError> {
    let new_labels = validation::distinct_ignoring_case(new_labels);
    validation::validate_variation_labels(&new_labels)?;
    validation::validate_variation_ids(model, &variation_ids)?;
    let (minted_ids, minted) = variation::ids_for_labels(&model.variations, &new_labels, now);
    let mut ids = variation_ids;
    for minted_id in minted_ids {
        if !ids.contains(&minted_id) {
            ids.push(minted_id);
        }
    }
    if ids.len() > validation::MAX_VARIATIONS {
        return Err(LibraryError::Validation {
            field: "labels".to_string(),
            message: format!(
                "An item can have at most {} variations",
                validation::MAX_VARIATIONS
            ),
        });
    }
    Ok((ids, minted))
}

pub(super) fn save_with_minted(
    model: &mut Model,
    item: Item,
    minted: Vec<Variation>,
) -> Command<Effect, Event> {
    if minted.is_empty() {
        return persist_item(model, item);
    }
    model.variations.extend(minted.iter().cloned());
    Command::all([
        crate::persistence::save_variations(model, minted),
        persist_item(model, item),
    ])
}

pub(super) fn update_keys(model: &mut Model, id: String, keys: Vec<Key>) -> Command<Effect, Event> {
    if let Err(e) = validation::validate_keys(&keys) {
        return refuse(model, &e);
    }
    let Some(item) = model.items.iter_mut().find(|i| i.id == id) else {
        model.raise_error(LibraryError::NotFound { id }.to_string());
        return crux_core::render::render();
    };
    if item.keys == keys {
        model.last_error = None;
        return crux_core::render::render();
    }
    item.keys = keys;
    item.updated_at = chrono::Utc::now();
    let item = item.clone();
    persist_item(model, item)
}

pub(super) fn rename(model: &mut Model, id: String, label: String) -> Command<Effect, Event> {
    let label = label.trim().to_string();
    if let Err(e) = validation::validate_variation_label(&label)
        .and_then(|()| validation::validate_variation_label_free(&model.variations, &label, &id))
    {
        return refuse(model, &e);
    }
    let Some(row) = live_row(model, &id) else {
        model.raise_error(LibraryError::NotFound { id }.to_string());
        return crux_core::render::render();
    };
    if row.label == label {
        model.last_error = None;
        return crux_core::render::render();
    }
    row.label = label;
    row.updated_at = chrono::Utc::now();
    let row = row.clone();
    save_row(model, row)
}

pub(super) fn delete(model: &mut Model, id: String) -> Command<Effect, Event> {
    let Some(row) = live_row(model, &id) else {
        model.raise_error(LibraryError::NotFound { id }.to_string());
        return crux_core::render::render();
    };
    let now = chrono::Utc::now();
    row.deleted_at = Some(now);
    row.updated_at = now;
    let row = row.clone();
    save_row(model, row)
}

fn live_row<'a>(model: &'a mut Model, id: &str) -> Option<&'a mut Variation> {
    model
        .variations
        .iter_mut()
        .find(|v| v.id == id && v.deleted_at.is_none())
}

fn save_row(model: &mut Model, row: Variation) -> Command<Effect, Event> {
    model.clear_error();
    Command::all([
        crate::persistence::save_variations(model, vec![row]),
        crux_core::render::render(),
    ])
}
