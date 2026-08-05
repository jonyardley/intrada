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
use crate::engine::{BlockRecord, WanderRecord};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum PersistenceOperation {
    LoadItems,
    SaveItem(Item),
    /// Upsert a batch atomically — the shell wraps all rows in one transaction,
    /// so a batch create (chart-to-scaffold commit) is all-or-nothing, never a
    /// half-linked piece with orphan exercises (#1106, invariant 5).
    SaveItems(Vec<Item>),
    /// Same `DateTime<Utc>` type as `Item::updated_at` so the tombstone bridges
    /// through the identical codec — byte-identical format under LWW, no drift.
    DeleteItem {
        id: String,
        deleted_at: DateTime<Utc>,
    },
    LoadSessions,
    SaveSession(PracticeSession),
    /// Append coach evidence as it closes (spec §4, #1181). One transaction:
    /// blocks, wanders and their attempts land together or not at all, so the
    /// evidence is never half-written. `updated_at` is core-stamped for the
    /// whole batch — the shell formats no timestamp of its own.
    SaveCoachRecords {
        blocks: Vec<BlockRecord>,
        wanders: Vec<WanderRecord>,
        updated_at: DateTime<Utc>,
    },
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

pub fn save_items(items: Vec<Item>) -> Command<Effect, Event> {
    Command::request_from_shell(PersistenceOperation::SaveItems(items))
        .then_send(Event::StoreWritten)
}

pub fn delete_item(id: String, deleted_at: DateTime<Utc>) -> Command<Effect, Event> {
    Command::request_from_shell(PersistenceOperation::DeleteItem { id, deleted_at })
        .then_send(Event::StoreWritten)
}

/// Takes the whole operation rather than its parts, so the caller can keep the
/// same batch to offer again if the write fails (#1181).
pub fn save_coach_records(batch: PersistenceOperation) -> Command<Effect, Event> {
    Command::request_from_shell(batch).then_send(Event::CoachStoreWritten)
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
            linked_exercise_ids: vec![],
            priority: false,
            chord_chart: None,
            variants: vec![],
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
    fn save_items_requests_one_batch_op_with_all_rows() {
        let mut cmd = save_items(vec![sample_item("a"), sample_item("b")]);
        let ids = cmd
            .effects()
            .find_map(|e| match e {
                Effect::Persistence(req) => match req.operation {
                    PersistenceOperation::SaveItems(items) => {
                        Some(items.iter().map(|i| i.id.clone()).collect::<Vec<_>>())
                    }
                    _ => None,
                },
                _ => None,
            })
            .expect("expected one SaveItems persistence effect");
        assert_eq!(ids, vec!["a".to_string(), "b".to_string()]);
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
            session_score: None,
            reflection_improved: None,
            reflection_still_rough: None,
            reflection_next_target: None,
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

    // ── Coach evidence (#1181) ──────────────────────────────────────────

    mod coach {
        use super::*;
        use crate::app::{AppEffect, Intrada};
        use crate::engine::{BlockRecord, CoachEvent};
        use chrono::TimeZone;

        fn at(second: i64) -> DateTime<Utc> {
            Utc.timestamp_opt(1_754_300_000 + second, 0).unwrap()
        }

        fn local_first() -> (Intrada, Model) {
            let mut model = Model::test_default();
            model.local_first = true;
            (Intrada, model)
        }

        fn send(app: &Intrada, model: &mut Model, event: CoachEvent) -> Command<Effect, Event> {
            app.update(Event::Coach(event), model)
        }

        /// One whole repetition through the app, so the wiring is what is under
        /// test rather than the state machine on its own.
        fn rep(app: &Intrada, model: &mut Model, clean: bool, second: i64) {
            let body = model
                .coach
                .session
                .block()
                .expect("a running block")
                .body_beats();
            let _ = send(app, model, CoachEvent::Beat { beat_index: body });
            let _ = send(
                app,
                model,
                CoachEvent::Tap {
                    clean,
                    now: at(second),
                },
            );
            let _ = send(app, model, CoachEvent::Beat { beat_index: 0 });
        }

        fn coach_records(cmd: &mut Command<Effect, Event>) -> Option<Vec<BlockRecord>> {
            cmd.effects().find_map(|e| match e {
                Effect::Persistence(req) => match req.operation {
                    PersistenceOperation::SaveCoachRecords { blocks, .. } => Some(blocks),
                    _ => None,
                },
                _ => None,
            })
        }

        fn app_effects(cmd: &mut Command<Effect, Event>) -> Vec<AppEffect> {
            cmd.effects()
                .filter_map(|e| match e {
                    Effect::App(req) => Some(req.operation.clone()),
                    _ => None,
                })
                .collect()
        }

        #[test]
        fn a_closed_block_is_appended_to_the_store_without_touching_the_network() {
            let (app, mut model) = local_first();
            let _ = send(
                &app,
                &mut model,
                CoachEvent::StartPlannedSession { now: at(0) },
            );
            for second in [9, 18, 27] {
                rep(&app, &mut model, true, second);
            }

            let mut cmd = send(&app, &mut model, CoachEvent::Tick { now: at(30) });
            let blocks = coach_records(&mut cmd).expect("a SaveCoachRecords op");
            assert_eq!(blocks.len(), 1);
            assert_eq!(
                blocks[0].node, "shells-ii-v-i",
                "today's plan warms up on the material already owned"
            );
            assert_eq!(
                blocks[0].attempts.len(),
                1,
                "shells-cycle asks for one clean pass, so the first tap opens the gate"
            );
            assert!(!has_http(&mut cmd), "evidence never goes over HTTP");
        }

        #[test]
        fn a_tap_rewrites_the_crash_recovery_blob_and_a_beat_does_not() {
            let (app, mut model) = local_first();
            let _ = send(
                &app,
                &mut model,
                CoachEvent::StartPlannedSession { now: at(0) },
            );
            let body = model.coach.session.block().unwrap().body_beats();
            let _ = send(&app, &mut model, CoachEvent::Beat { beat_index: body });

            let mut cmd = send(
                &app,
                &mut model,
                CoachEvent::Tap {
                    clean: true,
                    now: at(9),
                },
            );
            assert!(app_effects(&mut cmd)
                .iter()
                .any(|effect| matches!(effect, AppEffect::SaveCoachSessionInProgress(_))));

            let mut cmd = send(&app, &mut model, CoachEvent::Beat { beat_index: 1 });
            assert!(
                app_effects(&mut cmd).is_empty(),
                "the click reports several beats a second — no write per beat"
            );
        }

        #[test]
        fn the_saved_blob_is_the_session_as_it_stands() {
            let (app, mut model) = local_first();
            let _ = send(
                &app,
                &mut model,
                CoachEvent::StartPlannedSession { now: at(0) },
            );
            let body = model.coach.session.block().unwrap().body_beats();
            let _ = send(&app, &mut model, CoachEvent::Beat { beat_index: body });
            let mut cmd = send(
                &app,
                &mut model,
                CoachEvent::Tap {
                    clean: false,
                    now: at(9),
                },
            );

            let saved = app_effects(&mut cmd)
                .into_iter()
                .find_map(|effect| match effect {
                    AppEffect::SaveCoachSessionInProgress(session) => Some(session),
                    _ => None,
                })
                .expect("a snapshot effect");
            assert_eq!(saved, model.coach.session);
        }

        #[test]
        fn closing_the_session_clears_the_blob() {
            let (app, mut model) = local_first();
            let _ = send(
                &app,
                &mut model,
                CoachEvent::StartPlannedSession { now: at(0) },
            );
            let mut cmd = send(&app, &mut model, CoachEvent::LeaveSession { now: at(20) });

            // `effects()` drains, so read the write before the clear.
            assert!(
                coach_records(&mut cmd).is_some(),
                "the block it ended is still written down first"
            );
            assert!(app_effects(&mut cmd)
                .iter()
                .any(|effect| matches!(effect, AppEffect::ClearCoachSessionInProgress)));
        }

        /// Runs the first block to its gate and past the hold, so a record has
        /// been offered to the store and the retry has something to hold.
        fn close_a_block(app: &Intrada, model: &mut Model) {
            let _ = send(app, model, CoachEvent::StartPlannedSession { now: at(0) });
            rep(app, model, true, 9);
            let _ = send(app, model, CoachEvent::Tick { now: at(30) });
        }

        #[test]
        fn a_failed_evidence_write_is_offered_again_before_it_bothers_anyone() {
            let (app, mut model) = local_first();
            close_a_block(&app, &mut model);

            let mut cmd = app.update(
                Event::CoachStoreWritten(PersistenceOutput::Failed),
                &mut model,
            );
            let retried = coach_records(&mut cmd).expect("the same batch goes back to the store");
            assert_eq!(retried.len(), 1);
            assert_eq!(retried[0].node, "shells-ii-v-i");
            assert!(
                model.last_error.is_none(),
                "a failure the app can have another go at is not the user's problem yet"
            );
        }

        #[test]
        fn a_second_failure_surfaces_instead_of_retrying_forever() {
            let (app, mut model) = local_first();
            close_a_block(&app, &mut model);

            let _ = app.update(
                Event::CoachStoreWritten(PersistenceOutput::Failed),
                &mut model,
            );
            let mut cmd = app.update(
                Event::CoachStoreWritten(PersistenceOutput::Failed),
                &mut model,
            );

            assert!(
                coach_records(&mut cmd).is_none(),
                "one retry, not a loop against a store that is not having it"
            );
            assert!(
                model.last_error.is_some(),
                "the second failure is the user's"
            );
        }

        #[test]
        fn an_acked_write_leaves_nothing_to_offer_again() {
            let (app, mut model) = local_first();
            close_a_block(&app, &mut model);
            let _ = app.update(Event::CoachStoreWritten(PersistenceOutput::Ack), &mut model);

            let mut cmd = app.update(
                Event::CoachStoreWritten(PersistenceOutput::Failed),
                &mut model,
            );
            assert!(
                coach_records(&mut cmd).is_none(),
                "a batch already acked must never be rewritten by a later failure"
            );
            assert!(model.last_error.is_some());
        }

        #[test]
        fn a_failed_evidence_write_surfaces_rather_than_vanishing() {
            let (app, mut model) = local_first();
            let mut cmd = app.update(
                Event::CoachStoreWritten(PersistenceOutput::Failed),
                &mut model,
            );
            assert!(
                model.last_error.is_some(),
                "a failed local write is never a silent success (invariant 5)"
            );
            assert!(
                !cmd.effects().any(|e| matches!(e, Effect::Http(_))),
                "a local failure never falls back to the network"
            );
        }

        #[test]
        fn an_acked_evidence_write_says_nothing() {
            let (app, mut model) = local_first();
            let mut cmd = app.update(Event::CoachStoreWritten(PersistenceOutput::Ack), &mut model);
            assert!(model.last_error.is_none());
            assert!(!cmd.effects().any(|_| true));
        }

        #[test]
        fn the_recovered_session_comes_back_through_the_bridge() {
            let (app, mut model) = local_first();
            let _ = send(
                &app,
                &mut model,
                CoachEvent::StartPlannedSession { now: at(0) },
            );
            let body = model.coach.session.block().unwrap().body_beats();
            let _ = send(&app, &mut model, CoachEvent::Beat { beat_index: body });
            let _ = send(
                &app,
                &mut model,
                CoachEvent::Tap {
                    clean: true,
                    now: at(9),
                },
            );
            let crashed = model.coach.session.clone();

            let (app, mut fresh) = local_first();
            let _ = send(
                &app,
                &mut fresh,
                CoachEvent::RecoverSession {
                    session: crashed,
                    now: at(3600),
                },
            );
            let block = fresh.coach.session.block().expect("the block came back");
            assert_eq!(block.attempts.len(), 1);
            assert_eq!(block.gate_progress.filled(), 1);
        }

        // ── The #846 hazard: the new payloads on the real wire ──

        #[test]
        fn the_evidence_op_survives_the_ffi_wire() {
            let (app, mut model) = local_first();
            let _ = send(
                &app,
                &mut model,
                CoachEvent::StartPlannedSession { now: at(0) },
            );
            for second in [9, 18, 27] {
                rep(&app, &mut model, true, second);
            }
            let mut cmd = send(&app, &mut model, CoachEvent::Tick { now: at(30) });
            let blocks = coach_records(&mut cmd).expect("a SaveCoachRecords op");

            crate::domain::types::assert_round_trips(PersistenceOperation::SaveCoachRecords {
                blocks,
                wanders: vec![],
                updated_at: at(30),
            });
        }

        #[test]
        fn the_crash_recovery_blob_survives_the_ffi_wire() {
            let (app, mut model) = local_first();
            let _ = send(
                &app,
                &mut model,
                CoachEvent::StartPlannedSession { now: at(0) },
            );
            let body = model.coach.session.block().unwrap().body_beats();
            let _ = send(&app, &mut model, CoachEvent::Beat { beat_index: body });
            let _ = send(
                &app,
                &mut model,
                CoachEvent::Tap {
                    clean: true,
                    now: at(9),
                },
            );

            let session = model.coach.session.clone();
            crate::domain::types::assert_round_trips(AppEffect::SaveCoachSessionInProgress(
                session.clone(),
            ));
            crate::domain::types::assert_round_trips(Event::Coach(CoachEvent::RecoverSession {
                session,
                now: at(3600),
            }));
        }
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
