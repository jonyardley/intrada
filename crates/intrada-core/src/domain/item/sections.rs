use super::*;

pub(super) fn update_sections(
    model: &mut Model,
    id: String,
    edits: Vec<SectionEdit>,
) -> Command<Effect, Event> {
    let drafts = match validation::validate_section_edits(edits) {
        Ok(drafts) => drafts,
        Err(e) => return refuse(model, &e),
    };
    let Some(item) = model.items.iter_mut().find(|i| i.id == id) else {
        model.raise_error(LibraryError::NotFound { id }.to_string());
        return crux_core::render::render();
    };

    let now = chrono::Utc::now();
    let reconciled = crate::domain::section::reconcile_sections(item.sections.clone(), drafts, now);

    // Order-insensitive: the store loads by position with tombstones
    // interleaved, while reconcile emits live rows then tombstones.
    let sorted_by_id = |mut s: Vec<ItemSection>| {
        s.sort_by(|a, b| a.id.cmp(&b.id));
        s
    };
    if sorted_by_id(item.sections.clone()) == sorted_by_id(reconciled.clone()) {
        model.last_error = None;
        return crux_core::render::render();
    }

    item.sections = reconciled;
    item.updated_at = now;
    let item = item.clone();
    persist_item(model, item)
}
