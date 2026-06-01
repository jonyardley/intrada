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

/// Upsert an item into the local store (write-through). The `Ack` lands in
/// `Event::StoreLoaded` and is a no-op — the model was already updated.
pub fn save_item(item: Item) -> Command<Effect, Event> {
    Command::request_from_shell(PersistenceOperation::SaveItem(item)).then_send(Event::StoreLoaded)
}

/// Soft-delete an item from the local store (write-through).
pub fn delete_item(id: String) -> Command<Effect, Event> {
    Command::request_from_shell(PersistenceOperation::DeleteItem { id })
        .then_send(Event::StoreLoaded)
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

    // ── Write-through (B3a) ─────────────────────────────────────────────

    fn emits_save(cmd: &mut Command<Effect, Event>, expected_id: &str) -> bool {
        cmd.effects().any(|e| {
            matches!(e, Effect::Persistence(req)
                if matches!(&req.operation, PersistenceOperation::SaveItem(item) if item.id == expected_id))
        })
    }

    #[test]
    fn save_item_requests_a_save_op() {
        let mut cmd = save_item(sample_item("p1"));
        assert!(emits_save(&mut cmd, "p1"));
    }

    #[test]
    fn delete_item_requests_a_delete_op() {
        let mut cmd = delete_item("gone".to_string());
        assert!(cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
            if req.operation == PersistenceOperation::DeleteItem { id: "gone".to_string() })));
    }

    #[test]
    fn item_created_writes_through_to_store() {
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        let mut cmd = app.update(
            Event::ItemCreated {
                temp_id: "tmp".into(),
                item: sample_item("server-id"),
            },
            &mut model,
        );
        assert!(emits_save(&mut cmd, "server-id"));
    }

    #[test]
    fn item_updated_writes_through_to_store() {
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        model.items = vec![sample_item("p1")];
        let mut cmd = app.update(
            Event::ItemUpdated {
                item: sample_item("p1"),
            },
            &mut model,
        );
        assert!(emits_save(&mut cmd, "p1"));
    }

    #[test]
    fn delete_writes_through_a_tombstone() {
        use crate::domain::item::ItemEvent;
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        model.items = vec![sample_item("p1")];
        let mut cmd = app.update(
            Event::Item(ItemEvent::Delete { id: "p1".into() }),
            &mut model,
        );
        assert!(cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
            if req.operation == PersistenceOperation::DeleteItem { id: "p1".to_string() })));
    }

    #[test]
    fn optimistic_create_does_not_persist_yet() {
        // B3a persists creates only on confirmation (real id), so the optimistic
        // Add must emit no Persistence effect (until B3b client-ulids — #818).
        use crate::domain::item::ItemEvent;
        use crate::domain::types::CreateItem;
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        let mut cmd = app.update(
            Event::Item(ItemEvent::Add(CreateItem {
                title: "New".into(),
                kind: ItemKind::Piece,
                composer: None,
                key: None,
                tempo: None,
                notes: None,
                tags: vec![],
            })),
            &mut model,
        );
        assert!(!cmd.effects().any(|e| matches!(e, Effect::Persistence(_))));
    }
}
