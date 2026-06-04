//! Local-first persistence — query/mutation operations the shell fulfils
//! against on-device SQLite. Unlike `AppEffect` (`Output = ()`), these
//! queries return data: the core's first effect with a real typed `Output`.

use chrono::{DateTime, Utc};
use crux_core::capability::Operation;
use crux_core::command::Command;
use serde::{Deserialize, Serialize};

use crate::app::{Effect, Event};
use crate::domain::item::Item;
use crate::domain::session::PracticeSession;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum PersistenceOperation {
    LoadItems,
    SaveItem(Item),
    /// Same `DateTime<Utc>` type as `Item::updated_at` so the tombstone bridges
    /// through the identical codec — byte-identical format under LWW, no drift.
    DeleteItem {
        id: String,
        deleted_at: DateTime<Utc>,
    },
    LoadSessions,
    SaveSession(PracticeSession),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum PersistenceOutput {
    Items(Vec<Item>),
    Sessions(Vec<PracticeSession>),
    Ack,
    /// Local store failed the op — surfaced, not trusted as success (#816).
    Failed,
}

impl Operation for PersistenceOperation {
    type Output = PersistenceOutput;
}

pub fn load_items() -> Command<Effect, Event> {
    Command::request_from_shell(PersistenceOperation::LoadItems).then_send(Event::StoreLoaded)
}

pub fn save_item(item: Item) -> Command<Effect, Event> {
    Command::request_from_shell(PersistenceOperation::SaveItem(item)).then_send(Event::StoreWritten)
}

pub fn delete_item(id: String, deleted_at: DateTime<Utc>) -> Command<Effect, Event> {
    Command::request_from_shell(PersistenceOperation::DeleteItem { id, deleted_at })
        .then_send(Event::StoreWritten)
}

pub fn load_sessions() -> Command<Effect, Event> {
    Command::request_from_shell(PersistenceOperation::LoadSessions)
        .then_send(Event::SessionsStoreLoaded)
}

pub fn save_session(session: PracticeSession) -> Command<Effect, Event> {
    Command::request_from_shell(PersistenceOperation::SaveSession(session))
        .then_send(Event::SessionStoreWritten)
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
            modality: None,
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
    fn store_loaded_failed_surfaces_an_error() {
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        let mut cmd = app.update(Event::StoreLoaded(PersistenceOutput::Failed), &mut model);
        assert!(
            model.last_error.is_some(),
            "a failed read must surface an error"
        );
        assert!(!cmd.effects().any(|e| matches!(e, Effect::Persistence(_))));
    }

    #[test]
    fn store_written_failed_surfaces_and_rehydrates() {
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        let mut cmd = app.update(Event::StoreWritten(PersistenceOutput::Failed), &mut model);
        assert!(
            model.last_error.is_some(),
            "a failed write must surface an error"
        );
        assert!(cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
            if req.operation == PersistenceOperation::LoadItems)));
    }

    #[test]
    fn store_written_ack_is_a_noop() {
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        let mut cmd = app.update(Event::StoreWritten(PersistenceOutput::Ack), &mut model);
        assert!(model.last_error.is_none());
        assert!(!cmd.effects().any(|e| matches!(e, Effect::Persistence(_))));
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

    // ── Builders ────────────────────────────────────────────────────────

    fn has_save(cmd: &mut Command<Effect, Event>, id: &str) -> bool {
        cmd.effects().any(|e| {
            matches!(e, Effect::Persistence(req)
                if matches!(&req.operation, PersistenceOperation::SaveItem(item) if item.id == id))
        })
    }

    fn has_delete(cmd: &mut Command<Effect, Event>, id: &str) -> bool {
        cmd.effects().any(|e| {
            matches!(e, Effect::Persistence(req)
            if matches!(&req.operation, PersistenceOperation::DeleteItem { id: op_id, .. } if op_id == id))
        })
    }

    fn has_http(cmd: &mut Command<Effect, Event>) -> bool {
        cmd.effects().any(|e| matches!(e, Effect::Http(_)))
    }

    #[test]
    fn save_item_requests_a_save_op() {
        let mut cmd = save_item(sample_item("p1"));
        assert!(has_save(&mut cmd, "p1"));
    }

    #[test]
    fn delete_item_requests_a_delete_op() {
        let mut cmd = delete_item("gone".to_string(), chrono::Utc::now());
        assert!(has_delete(&mut cmd, "gone"));
    }

    #[test]
    fn local_first_delete_stamps_the_delete_instant() {
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        model.local_first = true;
        model.items = vec![sample_item("p1")];

        let before = chrono::Utc::now();
        let mut cmd = app.update(
            Event::Item(crate::domain::item::ItemEvent::Delete {
                id: "p1".to_string(),
            }),
            &mut model,
        );
        let after = chrono::Utc::now();

        let deleted_at = cmd
            .effects()
            .find_map(|e| match e {
                Effect::Persistence(req) => match req.operation {
                    PersistenceOperation::DeleteItem { deleted_at, .. } => Some(deleted_at),
                    _ => None,
                },
                _ => None,
            })
            .expect("a DeleteItem persistence op");
        // Core stamps the delete instant (DateTime<Utc>, same type as updated_at).
        assert!(deleted_at >= before && deleted_at <= after);
    }

    // ── Local-first mode: writes persist locally, no HTTP ───────────────

    fn create_item() -> crate::domain::types::CreateItem {
        crate::domain::types::CreateItem {
            title: "New".into(),
            kind: ItemKind::Piece,
            composer: Some("Chopin".into()), // pieces require a composer (validation)
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: vec![],
        }
    }

    #[test]
    fn local_first_add_persists_and_skips_http() {
        use crate::domain::item::ItemEvent;
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        model.local_first = true;
        let mut cmd = app.update(Event::Item(ItemEvent::Add(create_item())), &mut model);
        let id = model.items[0].id.clone();
        assert!(
            has_save(&mut cmd, &id),
            "local-first create persists with the client ulid"
        );
        assert!(
            !has_http(&mut cmd),
            "local-first create makes no HTTP request"
        );
    }

    #[test]
    fn local_first_update_persists_and_skips_http() {
        use crate::domain::item::ItemEvent;
        use crate::domain::types::UpdateItem;
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        model.local_first = true;
        model.items = vec![sample_item("p1")];
        let input = UpdateItem {
            title: Some("Renamed".into()),
            kind: None,
            composer: None,
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: None,
            priority: None,
        };
        let mut cmd = app.update(
            Event::Item(ItemEvent::Update {
                id: "p1".into(),
                input,
            }),
            &mut model,
        );
        assert!(has_save(&mut cmd, "p1"));
        assert!(!has_http(&mut cmd));
    }

    #[test]
    fn local_first_delete_persists_tombstone_and_skips_http() {
        use crate::domain::item::ItemEvent;
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        model.local_first = true;
        model.items = vec![sample_item("p1")];
        let mut cmd = app.update(
            Event::Item(ItemEvent::Delete { id: "p1".into() }),
            &mut model,
        );
        assert!(has_delete(&mut cmd, "p1"));
        assert!(!has_http(&mut cmd));
    }

    #[test]
    fn local_first_failed_delete_rolls_back_via_store_reload() {
        // End-to-end rollback (#834): an optimistic delete whose store write
        // fails must never be a silent success (invariant #5) — it surfaces an
        // error and reloads from the store, which still holds the row the
        // failed write never removed.
        use crate::domain::item::ItemEvent;
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        model.local_first = true;
        model.items = vec![sample_item("p1")];

        let mut cmd = app.update(
            Event::Item(ItemEvent::Delete { id: "p1".into() }),
            &mut model,
        );
        assert!(
            model.items.is_empty(),
            "delete optimistically removes the row"
        );
        assert!(has_delete(&mut cmd, "p1"));

        let mut cmd = app.update(Event::StoreWritten(PersistenceOutput::Failed), &mut model);
        assert!(
            model.last_error.is_some(),
            "a failed delete must surface an error, not vanish silently"
        );
        assert!(
            cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
                if req.operation == PersistenceOperation::LoadItems)),
            "a failed write reloads from the store to roll back"
        );

        let _ = app.update(
            Event::StoreLoaded(PersistenceOutput::Items(vec![sample_item("p1")])),
            &mut model,
        );
        assert_eq!(
            model.items.len(),
            1,
            "the rolled-back delete restores the row"
        );
        assert_eq!(model.items[0].id, "p1");
    }

    #[test]
    fn online_add_uses_http_not_persistence() {
        use crate::domain::item::ItemEvent;
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        let mut cmd = app.update(Event::Item(ItemEvent::Add(create_item())), &mut model);
        assert!(has_http(&mut cmd), "online create POSTs to the server");
        assert!(!cmd.effects().any(|e| matches!(e, Effect::Persistence(_))));
    }

    #[test]
    fn local_first_write_clears_the_dismiss_mute() {
        // Local-first has no server callback, so a successful local write
        // must record the success itself.
        use crate::domain::item::ItemEvent;
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        model.local_first = true;
        model.dismiss_error();
        let _ = app.update(Event::Item(ItemEvent::Add(create_item())), &mut model);
        assert!(
            !model.error_muted,
            "a successful local write should un-mute"
        );
    }

    #[test]
    fn start_app_local_first_hydrates_from_store() {
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        let mut cmd = app.update(
            Event::StartApp {
                api_base_url: "http://x".into(),
                local_first: true,
            },
            &mut model,
        );
        assert!(model.local_first);
        assert!(cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
            if req.operation == PersistenceOperation::LoadItems)));
        assert!(
            !has_http(&mut cmd),
            "local-first launch hydrates from the store, no HTTP"
        );
    }

    #[test]
    fn start_app_online_fetches_over_http() {
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        let mut cmd = app.update(
            Event::StartApp {
                api_base_url: "http://x".into(),
                local_first: false,
            },
            &mut model,
        );
        assert!(!model.local_first);
        assert!(has_http(&mut cmd));
        assert!(!cmd.effects().any(|e| matches!(e, Effect::Persistence(_))));
    }

    /// Offline-first invariant #1 (CLAUDE.md): the full local-first lifecycle
    /// (launch/create/update/delete) emits zero HTTP — a regression sentinel.
    #[test]
    fn offline_invariant_local_first_lifecycle_makes_no_http() {
        use crate::domain::item::ItemEvent;
        use crate::domain::types::UpdateItem;
        let app = crate::app::Intrada;
        let mut model = Model::test_default();

        let mut launch = app.update(
            Event::StartApp {
                api_base_url: "http://x".into(),
                local_first: true,
            },
            &mut model,
        );
        assert!(!has_http(&mut launch), "launch");

        let mut add = app.update(Event::Item(ItemEvent::Add(create_item())), &mut model);
        assert!(!has_http(&mut add), "create");
        let id = model.items[0].id.clone();

        let input = UpdateItem {
            title: Some("Renamed".into()),
            kind: None,
            composer: None,
            key: None,
            modality: None,
            tempo: None,
            notes: None,
            tags: None,
            priority: None,
        };
        let mut update = app.update(
            Event::Item(ItemEvent::Update {
                id: id.clone(),
                input,
            }),
            &mut model,
        );
        assert!(!has_http(&mut update), "update");

        let mut delete = app.update(Event::Item(ItemEvent::Delete { id }), &mut model);
        assert!(!has_http(&mut delete), "delete");
    }

    // ── Sessions ────────────────────────────────────────────────────────

    fn sample_session(id: &str) -> PracticeSession {
        let now = chrono::Utc::now();
        PracticeSession {
            id: id.to_string(),
            entries: vec![],
            session_notes: None,
            session_intention: None,
            started_at: now,
            completed_at: now,
            total_duration_secs: 0,
            completion_status: crate::domain::session::CompletionStatus::Completed,
        }
    }

    #[test]
    fn save_session_requests_a_save_op() {
        let mut cmd = save_session(sample_session("s1"));
        assert!(cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
            if matches!(&req.operation, PersistenceOperation::SaveSession(s) if s.id == "s1"))));
    }

    #[test]
    fn load_sessions_requests_the_load_operation() {
        let mut cmd = load_sessions();
        assert!(cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
            if req.operation == PersistenceOperation::LoadSessions)));
    }

    #[test]
    fn sessions_store_loaded_replaces_model_sessions() {
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        model.sessions = vec![sample_session("stale")];
        let _ = app.update(
            Event::SessionsStoreLoaded(PersistenceOutput::Sessions(vec![sample_session("fresh")])),
            &mut model,
        );
        assert_eq!(model.sessions.len(), 1);
        assert_eq!(model.sessions[0].id, "fresh");
    }

    #[test]
    fn start_app_local_first_also_loads_sessions() {
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        let mut cmd = app.update(
            Event::StartApp {
                api_base_url: "http://x".into(),
                local_first: true,
            },
            &mut model,
        );
        assert!(cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
            if req.operation == PersistenceOperation::LoadSessions)));
        assert!(!has_http(&mut cmd), "local-first launch makes no HTTP");
    }

    #[test]
    fn session_store_written_failed_reloads_sessions() {
        let app = crate::app::Intrada;
        let mut model = Model::test_default();
        let mut cmd = app.update(
            Event::SessionStoreWritten(PersistenceOutput::Failed),
            &mut model,
        );
        assert!(model.last_error.is_some());
        assert!(cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
            if req.operation == PersistenceOperation::LoadSessions)));
    }
}
