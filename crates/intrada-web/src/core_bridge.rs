use crux_core::Core;
use leptos::prelude::{RwSignal, Set};

use intrada_core::app::{Effect, StorageEffect};
use intrada_core::{Event, Intrada, ViewModel};

use crate::data::create_stub_data;

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
                    let (pieces, exercises) = create_stub_data();
                    let inner_effects = core.process_event(Event::DataLoaded { pieces, exercises });
                    process_effects(core, inner_effects, view_model);
                }
                StorageEffect::SavePiece(_)
                | StorageEffect::SaveExercise(_)
                | StorageEffect::UpdatePiece(_)
                | StorageEffect::UpdateExercise(_)
                | StorageEffect::DeleteItem { .. } => {}
            },
        }
    }
    view_model.set(core.view());
}
