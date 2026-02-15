use std::cell::RefCell;

use crux_core::Core;
use leptos::prelude::{RwSignal, Set};

use intrada_core::{Effect, Event, Intrada, LibraryData, StorageEffect, ViewModel};

use crate::data::create_stub_data;

const STORAGE_KEY: &str = "intrada:library";

thread_local! {
    static LIBRARY: RefCell<LibraryData> = RefCell::new(LibraryData::default());
}

fn get_local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

fn load_from_local_storage() -> LibraryData {
    let Some(storage) = get_local_storage() else {
        return LibraryData::default();
    };

    match storage.get_item(STORAGE_KEY) {
        Ok(Some(json)) => serde_json::from_str(&json).unwrap_or_default(),
        _ => LibraryData::default(),
    }
}

fn save_to_local_storage(data: &LibraryData) {
    let Some(storage) = get_local_storage() else {
        return;
    };

    match serde_json::to_string(data) {
        Ok(json) => {
            if storage.set_item(STORAGE_KEY, &json).is_err() {
                web_sys::console::warn_1(
                    &"intrada: localStorage write failed (storage may be full)".into(),
                );
            }
        }
        Err(e) => {
            web_sys::console::warn_1(
                &format!("intrada: failed to serialise library data: {e}").into(),
            );
        }
    }
}

/// Load library data from localStorage (or seed with stub data on first run).
///
/// Called by `App()` during initialisation, mirroring the CLI shell's `load_data()`.
pub fn load_library_data() -> (Vec<intrada_core::Piece>, Vec<intrada_core::Exercise>) {
    let mut data = load_from_local_storage();

    // If localStorage was empty, seed with stub data
    if data.pieces.is_empty() && data.exercises.is_empty() {
        let (pieces, exercises) = create_stub_data();
        data.pieces = pieces;
        data.exercises = exercises;
        save_to_local_storage(&data);
    }

    LIBRARY.with(|lib| *lib.borrow_mut() = data.clone());
    (data.pieces, data.exercises)
}

/// Process effects returned by the Crux core.
pub fn process_effects(
    core: &Core<Intrada>,
    effects: Vec<Effect>,
    view_model: &RwSignal<ViewModel>,
) {
    for effect in effects {
        match effect {
            Effect::Render(_) => {}
            Effect::Storage(boxed_request) => match &boxed_request.operation {
                StorageEffect::LoadAll => {
                    let (pieces, exercises) = load_library_data();
                    let inner_effects = core.process_event(Event::DataLoaded { pieces, exercises });
                    process_effects(core, inner_effects, view_model);
                }
                StorageEffect::SavePiece(piece) => {
                    LIBRARY.with(|lib| {
                        let mut data = lib.borrow_mut();
                        data.pieces.push(piece.clone());
                        save_to_local_storage(&data);
                    });
                }
                StorageEffect::SaveExercise(exercise) => {
                    LIBRARY.with(|lib| {
                        let mut data = lib.borrow_mut();
                        data.exercises.push(exercise.clone());
                        save_to_local_storage(&data);
                    });
                }
                StorageEffect::UpdatePiece(piece) => {
                    LIBRARY.with(|lib| {
                        let mut data = lib.borrow_mut();
                        if let Some(existing) = data.pieces.iter_mut().find(|p| p.id == piece.id) {
                            *existing = piece.clone();
                        }
                        save_to_local_storage(&data);
                    });
                }
                StorageEffect::UpdateExercise(exercise) => {
                    LIBRARY.with(|lib| {
                        let mut data = lib.borrow_mut();
                        if let Some(existing) =
                            data.exercises.iter_mut().find(|e| e.id == exercise.id)
                        {
                            *existing = exercise.clone();
                        }
                        save_to_local_storage(&data);
                    });
                }
                StorageEffect::DeleteItem { id } => {
                    LIBRARY.with(|lib| {
                        let mut data = lib.borrow_mut();
                        data.pieces.retain(|p| p.id != *id);
                        data.exercises.retain(|e| e.id != *id);
                        save_to_local_storage(&data);
                    });
                }
            },
        }
    }
    view_model.set(core.view());
}
