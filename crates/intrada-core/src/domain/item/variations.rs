use super::*;

/// `SetVariants` and `UpdateVariants` share everything past the wire shape.
pub(super) fn update_ladder(
    model: &mut Model,
    id: String,
    edits: Vec<VariantEdit>,
) -> Command<Effect, Event> {
    let edits: Vec<VariantEdit> = edits
        .into_iter()
        .map(|e| VariantEdit {
            id: e.id,
            label: e.label.trim().to_string(),
        })
        .collect();
    if let Err(e) = validation::validate_variant_host(&id, model) {
        return refuse(model, &e);
    }

    let Some(item) = model.items.iter_mut().find(|i| i.id == id) else {
        model.raise_error(LibraryError::NotFound { id }.to_string());
        return crux_core::render::render();
    };

    // The exercise's own key migrates into the ladder as its first
    // rung when this call gives it its first live variation (#1783
    // decision), never on a later call: an exercise that already had
    // variations alongside a key predates that decision and is left
    // alone here.
    let had_live_variant = item.variants.iter().any(|v| v.deleted_at.is_none());
    let (edits, key) = migrate_key_into_edits(item.key.clone(), edits, !had_live_variant);

    let labels: Vec<String> = edits.iter().map(|e| e.label.clone()).collect();
    if let Err(e) = validation::validate_variant_labels(&labels) {
        return refuse(model, &e);
    }

    let now = chrono::Utc::now();
    let existing = std::mem::take(&mut item.variants);
    let reconciled = crate::domain::variant::reconcile_variant_edits(existing.clone(), &edits, now);

    // Order-insensitive: the store loads by position (tombstones
    // interleaved) while reconcile emits live-then-tombstones.
    let sorted_by_id = |mut v: Vec<Variant>| {
        v.sort_by(|a, b| a.id.cmp(&b.id));
        v
    };
    if sorted_by_id(existing.clone()) == sorted_by_id(reconciled.clone()) && key == item.key {
        // No-op: don't bump the parent LWW stamp or write (it could
        // spuriously win a future sync merge).
        item.variants = existing;
        model.last_error = None;
        return crux_core::render::render();
    }

    item.variants = reconciled;
    item.key = key;
    item.updated_at = now;
    model.last_error = None;

    let item = item.clone();
    persist_item(model, item)
}

/// `migrate_key_into_labels` for the Edit form's rows: a folded-in key is a
/// row with no id, since it never was one.
pub(super) fn migrate_key_into_edits(
    key: Option<String>,
    edits: Vec<VariantEdit>,
    eligible: bool,
) -> (Vec<VariantEdit>, Option<String>) {
    let labels: Vec<String> = edits.iter().map(|e| e.label.clone()).collect();
    let (labels, key) = migrate_key_into_labels(key, labels, eligible);
    if labels.len() == edits.len() {
        return (edits, key);
    }
    let mut next = Vec::with_capacity(labels.len());
    next.push(VariantEdit {
        id: None,
        label: labels[0].clone(),
    });
    next.extend(edits);
    (next, key)
}

/// Folds an exercise's own `key` into `labels` as its first rung when it is
/// gaining its first variation: an exercise in several keys has no single
/// key (#1783 decision 1), and dropping the value outright would lose what
/// the musician already recorded. Returns the label set to reconcile against
/// and the key the item should keep afterwards, which is `None` whenever
/// `eligible` and `labels` fold a key in at all, even when the incoming
/// labels already name it case-insensitively (matching
/// `validate_variant_labels`'s own duplicate rule) and so add nothing new: a
/// stale key surviving because the ladder happened to already cover it would
/// still filter and print as if the exercise had one, which decision 1 rules
/// out regardless of how the ladder got there. A no-op, key kept exactly as
/// given, whenever `labels` is empty (no variation is being added) or
/// `eligible` is false (this exercise already had a live variation before
/// this call).
pub(super) fn migrate_key_into_labels(
    key: Option<String>,
    labels: Vec<String>,
    eligible: bool,
) -> (Vec<String>, Option<String>) {
    if !eligible || labels.is_empty() {
        return (labels, key);
    }
    let Some(key) = key else {
        return (labels, None);
    };
    if labels
        .iter()
        .any(|l| l.to_lowercase() == key.to_lowercase())
    {
        return (labels, None);
    }
    let mut next = Vec::with_capacity(labels.len() + 1);
    next.push(key);
    next.extend(labels);
    (next, None)
}

pub(super) fn set_variants(
    model: &mut Model,
    id: String,
    labels: Vec<String>,
) -> Command<Effect, Event> {
    let edits = labels
        .into_iter()
        .map(|label| VariantEdit { id: None, label })
        .collect();
    update_ladder(model, id, edits)
}

pub(super) fn rename_variant(
    model: &mut Model,
    item_id: String,
    variant_id: String,
    new_label: String,
) -> Command<Effect, Event> {
    if let Err(e) = validation::validate_variant_host(&item_id, model) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    let Some(item) = model.items.iter_mut().find(|i| i.id == item_id) else {
        model.raise_error(LibraryError::NotFound { id: item_id }.to_string());
        return crux_core::render::render();
    };

    let Some(variant) = item
        .variants
        .iter()
        .find(|v| v.id == variant_id && v.deleted_at.is_none())
    else {
        model.raise_error(LibraryError::NotFound { id: variant_id }.to_string());
        return crux_core::render::render();
    };

    let new_label = new_label.trim().to_string();
    if new_label == variant.label {
        // No-op: don't bump the parent LWW stamp or write.
        model.last_error = None;
        return crux_core::render::render();
    }

    // Validate the substituted label against the item's other live
    // variations: the same duplicate/length/count checks `SetVariants`
    // runs, applied to the renamed value rather than a whole ladder.
    let mut live: Vec<&Variant> = item
        .variants
        .iter()
        .filter(|v| v.deleted_at.is_none())
        .collect();
    live.sort_by_key(|v| v.position);
    let labels: Vec<String> = live
        .iter()
        .map(|v| {
            if v.id == variant_id {
                new_label.clone()
            } else {
                v.label.clone()
            }
        })
        .collect();

    if let Err(e) = validation::validate_variant_labels(&labels) {
        return refuse(model, &e);
    }

    let now = chrono::Utc::now();
    let variant = item
        .variants
        .iter_mut()
        .find(|v| v.id == variant_id)
        .expect("checked above");
    variant.label = new_label;
    variant.updated_at = now;
    item.updated_at = now;
    model.last_error = None;

    let item = item.clone();
    persist_item(model, item)
}
