use super::*;

/// Kicks off recognition. The bytes are already on disk (phase A writes them
/// shell-side), so the core only ever names the file.
pub(super) fn read_photo(model: &mut Model, photo_id: String) -> Command<Effect, Event> {
    if let Err(e) = validation::validate_photo_id(&photo_id) {
        model.raise_error(e.to_string());
        return crux_core::render::render();
    }

    model.last_error = None;
    model.photo_recognition = crate::model::PhotoRecognition::Reading {
        photo_id: photo_id.clone(),
    };
    Command::all([
        crate::recognition::read_page(photo_id),
        crux_core::render::render(),
    ])
}

/// `next` is `None` for a clear. The file the item stops pointing at is left on
/// disk for the reaping pass (#1442): a leaked file costs disk, an eagerly
/// deleted one costs the user their photo, and the write that would justify
/// deleting can still fail after the delete has run (spec, key decision 2).
pub(super) fn set_photo(
    model: &mut Model,
    id: String,
    next: Option<String>,
) -> Command<Effect, Event> {
    if let Some(photo_id) = next.as_deref() {
        if let Err(e) = validation::validate_photo_id(photo_id) {
            model.raise_error(e.to_string());
            return crux_core::render::render();
        }
    }

    let Some(item) = model.items.iter_mut().find(|i| i.id == id) else {
        model.raise_error(LibraryError::NotFound { id }.to_string());
        return crux_core::render::render();
    };

    model.last_error = None;
    if item.photo_id == next {
        return crux_core::render::render();
    }

    item.photo_id = next;
    item.updated_at = chrono::Utc::now();
    let item = item.clone();

    model.clear_error();
    Command::all([
        crate::persistence::save_item(model, item),
        crux_core::render::render(),
    ])
}
