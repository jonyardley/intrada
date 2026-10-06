use super::*;
use crate::domain::section::SectionDraft;

pub(crate) fn update_sections(
    model: &mut Model,
    id: String,
    edits: Vec<SectionEdit>,
) -> Command<Effect, Event> {
    match validation::validate_section_edits(edits) {
        Ok(drafts) => write_sections(model, id, drafts),
        Err(e) => refuse(model, &e),
    }
}

fn write_sections(model: &mut Model, id: String, drafts: Vec<SectionDraft>) -> Command<Effect, Event> {
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

    let removed: Vec<String> = reconciled
        .iter()
        .filter(|s| s.deleted_at.is_some())
        .filter(|s| {
            item.sections
                .iter()
                .any(|was| was.id == s.id && was.deleted_at.is_none())
        })
        .map(|s| s.id.clone())
        .collect();
    item.sections = reconciled;
    // A removed section takes its links with it, never the exercise (#2248).
    for link in item.exercise_links.iter_mut().filter(|l| {
        l.deleted_at.is_none() && l.section_id.as_ref().is_some_and(|id| removed.contains(id))
    }) {
        link.deleted_at = Some(now);
        link.updated_at = now;
    }
    item.updated_at = now;
    let item = item.clone();
    persist_item(model, item)
}

/// Builds the whole list from the stored rows; only the incoming row is
/// validated, so a limit tightened since cannot refuse an unrelated change.
pub(crate) fn change_section(
    model: &mut Model,
    id: String,
    change: SectionChange,
) -> Command<Effect, Event> {
    let Some(item) = model.items.iter().find(|i| i.id == id) else {
        model.raise_error(LibraryError::NotFound { id }.to_string());
        return crux_core::render::render();
    };
    let mut drafts: Vec<SectionDraft> = item
        .live_sections()
        .into_iter()
        .map(SectionDraft::from)
        .collect();

    match change {
        SectionChange::Save(edit) => {
            let draft = match validation::validate_section_edits(vec![edit]) {
                Ok(mut rows) => rows.remove(0),
                Err(e) => return refuse(model, &e),
            };
            match &draft.id {
                None => drafts.push(draft),
                Some(section_id) => {
                    let Some(at) = drafts.iter().position(|d| d.id == draft.id) else {
                        let error = LibraryError::NotFound {
                            id: section_id.clone(),
                        };
                        return refuse(model, &error);
                    };
                    drafts[at] = draft;
                }
            }
        }
        SectionChange::Remove { section_id } => {
            drafts.retain(|d| d.id.as_deref() != Some(section_id.as_str()));
        }
        SectionChange::Arrange { section_ids } => {
            let mut arranged = Vec::with_capacity(section_ids.len());
            for section_id in section_ids {
                let Some(at) = drafts
                    .iter()
                    .position(|d| d.id.as_deref() == Some(section_id.as_str()))
                else {
                    return refuse(model, &LibraryError::NotFound { id: section_id });
                };
                arranged.push(drafts.remove(at));
            }
            drafts = arranged;
        }
    }
    write_sections(model, id, drafts)
}
