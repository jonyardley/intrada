//! Local-first persistence — query/mutation operations the shell fulfils
//! against on-device SQLite (GRDB lands in B2). Unlike `AppEffect`
//! (`Output = ()`), persistence queries return data: the core's first effect
//! with a real typed `Output`, which B1 exists to de-risk across the bridge.

use crux_core::capability::Operation;
use crux_core::command::Command;
use serde::{Deserialize, Serialize};

use crate::app::{Effect, Event};
use crate::domain::item::Item;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum PersistenceOperation {
    LoadItems,
    SaveItem(Item),
    DeleteItem { id: String },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum PersistenceOutput {
    Items(Vec<Item>),
    Ack,
}

impl Operation for PersistenceOperation {
    type Output = PersistenceOutput;
}

pub fn load_items() -> Command<Effect, Event> {
    Command::request_from_shell(PersistenceOperation::LoadItems).then_send(Event::StoreLoaded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::item::ItemKind;
    use crate::model::Model;
    use crux_core::App;

    fn sample_item(id: &str) -> Item {
        let now = chrono::Utc::now();
        Item {
            id: id.to_string(),
            title: "Etude".to_string(),
            kind: ItemKind::Piece,
            composer: None,
            key: None,
            tempo: None,
            notes: None,
            tags: vec![],
            created_at: now,
            updated_at: now,
            priority: false,
        }
    }

    #[test]
    fn load_items_requests_the_load_operation() {
        let mut cmd = load_items();
        let op = cmd
            .effects()
            .find_map(|e| match e {
                Effect::Persistence(req) => Some(req.operation.clone()),
                _ => None,
            })
            .expect("expected a Persistence effect");
        assert_eq!(op, PersistenceOperation::LoadItems);
    }

    #[test]
    fn hydrate_from_store_emits_a_load_effect() {
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        let mut cmd = app.update(Event::HydrateFromStore, &mut model);
        assert!(cmd
            .effects()
            .any(|e| matches!(e, Effect::Persistence(req) if req.operation == PersistenceOperation::LoadItems)));
    }

    #[test]
    fn store_loaded_items_replaces_model_items() {
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        model.items = vec![sample_item("stale")];
        let _ = app.update(
            Event::StoreLoaded(PersistenceOutput::Items(vec![sample_item("fresh")])),
            &mut model,
        );
        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].id, "fresh");
    }

    #[test]
    fn store_loaded_ack_leaves_items_untouched() {
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        model.items = vec![sample_item("keep")];
        let _ = app.update(Event::StoreLoaded(PersistenceOutput::Ack), &mut model);
        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].id, "keep");
    }
}
