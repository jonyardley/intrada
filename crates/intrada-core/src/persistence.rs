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
use crate::domain::variation::Variation;
use crate::model::Model;
use crate::sync::{self, RecordKey, RecordKind, SyncOperation, SyncRecord, Synced};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
// Boxing a variant changes the generated Swift type and the bincode wire (#846).
#[allow(clippy::large_enum_variant)]
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
    /// Tombstones included: plays name deleted variations too (#2246).
    LoadVariations,
    /// Upsert the rows in one transaction.
    SaveVariations(Vec<Variation>),
    /// Tombstones included, so a merge sees a delete (`specs/icloud-sync.md`).
    LoadRecords(Vec<RecordKey>),
    LoadAllRecords,
    /// The merge's winners in one transaction, `deleted_at` written as given.
    ApplyMerged(StoredRecords),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct StoredItem {
    pub item: Item,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct StoredRecords {
    pub items: Vec<StoredItem>,
    pub variations: Vec<Variation>,
    pub sessions: Vec<PracticeSession>,
    /// Asked for but on disk in a shape this app cannot read, so a merge must
    /// not write over them.
    pub unreadable: Vec<RecordKey>,
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
    Variations(Vec<Variation>),
    Records(StoredRecords),
}

impl Operation for PersistenceOperation {
    type Output = PersistenceOutput;
}

/// What one list has out with the store, so a load that may predate an edit
/// never replaces it (#2067, `specs/list-reload-race.md`).
#[derive(Debug, Default)]
pub struct ListSync {
    writes_out: u32,
    loads_out: u32,
    stale: bool,
    loaded: bool,
    generation: u64,
}

#[derive(Debug, PartialEq)]
pub enum Landed {
    Apply,
    Drop,
    Reload,
}

impl ListSync {
    fn write_sent(&mut self) {
        self.writes_out += 1;
        self.generation += 1;
        if self.loads_out > 0 {
            self.stale = true;
        }
    }

    fn load_sent(&mut self) {
        self.loads_out += 1;
    }

    fn reload_due(&mut self) -> bool {
        let due = self.stale && self.writes_out == 0 && self.loads_out == 0;
        if due {
            self.stale = false;
        }
        due
    }

    /// True once a load has replaced the list, so an empty list means empty
    /// and not unanswered.
    pub fn has_loaded(&self) -> bool {
        self.loaded
    }

    pub fn load_landed(&mut self) -> Landed {
        self.loads_out = self.loads_out.saturating_sub(1);
        if !self.stale && self.writes_out == 0 {
            self.loaded = true;
            return Landed::Apply;
        }
        self.stale = true;
        if self.reload_due() {
            Landed::Reload
        } else {
            Landed::Drop
        }
    }

    /// A load that brought back no list. Asks again only for an edit it may
    /// have missed; the stale mark is cleared as that load goes, so a broken
    /// store fails it once and stops (#825).
    fn load_ended(&mut self) -> bool {
        self.loads_out = self.loads_out.saturating_sub(1);
        self.reload_due()
    }

    /// True when the list must be loaded again now. A refused write needs the
    /// rollback load, but only once nothing else is out to land over it.
    pub fn write_settled(&mut self, refused: bool) -> bool {
        self.writes_out = self.writes_out.saturating_sub(1);
        self.stale |= refused;
        self.reload_due()
    }
}

pub fn items_load_ended(model: &mut Model, then: Command<Effect, Event>) -> Command<Effect, Event> {
    if model.items_sync.load_ended() {
        Command::all([then, load_items(model)])
    } else {
        then
    }
}

pub fn sessions_load_ended(
    model: &mut Model,
    then: Command<Effect, Event>,
) -> Command<Effect, Event> {
    if model.sessions_sync.load_ended() {
        Command::all([then, load_sessions(model)])
    } else {
        then
    }
}

pub fn load_items(model: &mut Model) -> Command<Effect, Event> {
    model.items_sync.load_sent();
    Command::request_from_shell(PersistenceOperation::LoadItems).then_send(Event::StoreLoaded)
}

/// Uploads only what the store kept: a refused write never reaches the other
/// devices.
fn write_then_upload(
    operation: PersistenceOperation,
    upload: Vec<SyncRecord>,
    written: fn(PersistenceOutput) -> Event,
) -> Command<Effect, Event> {
    Command::new(|ctx| async move {
        let output = ctx.request_from_shell(operation).await;
        if output == PersistenceOutput::Ack && !upload.is_empty() {
            ctx.notify_shell(SyncOperation::Upload(upload));
        }
        ctx.send_event(written(output));
    })
}

fn live_items(items: &[Item]) -> Vec<Synced> {
    items
        .iter()
        .map(|item| Synced::Item {
            item: Box::new(item.clone()),
            deleted_at: None,
        })
        .collect()
}

pub fn save_item(model: &mut Model, item: Item) -> Command<Effect, Event> {
    model.items_sync.write_sent();
    let upload = sync::upload_for(model, &live_items(std::slice::from_ref(&item)));
    write_then_upload(
        PersistenceOperation::SaveItem(item),
        upload,
        Event::StoreWritten,
    )
}

pub fn save_items(model: &mut Model, items: Vec<Item>) -> Command<Effect, Event> {
    model.items_sync.write_sent();
    let upload = sync::upload_for(model, &live_items(&items));
    write_then_upload(
        PersistenceOperation::SaveItems(items),
        upload,
        Event::StoreWritten,
    )
}

pub fn delete_item(
    model: &mut Model,
    item: Item,
    deleted_at: DateTime<Utc>,
) -> Command<Effect, Event> {
    model.items_sync.write_sent();
    let id = item.id.clone();
    let upload = sync::upload_for(
        model,
        &[Synced::Item {
            item: Box::new(item),
            deleted_at: Some(deleted_at),
        }],
    );
    write_then_upload(
        PersistenceOperation::DeleteItem { id, deleted_at },
        upload,
        Event::StoreWritten,
    )
}

pub fn load_sessions(model: &mut Model) -> Command<Effect, Event> {
    model.sessions_sync.load_sent();
    Command::request_from_shell(PersistenceOperation::LoadSessions)
        .then_send(Event::SessionsStoreLoaded)
}

pub fn variations_load_ended(
    model: &mut Model,
    then: Command<Effect, Event>,
) -> Command<Effect, Event> {
    if model.variations_sync.load_ended() {
        Command::all([then, load_variations(model)])
    } else {
        then
    }
}

pub fn load_variations(model: &mut Model) -> Command<Effect, Event> {
    model.variations_sync.load_sent();
    Command::request_from_shell(PersistenceOperation::LoadVariations)
        .then_send(Event::VariationsStoreLoaded)
}

pub fn save_variations(model: &mut Model, rows: Vec<Variation>) -> Command<Effect, Event> {
    model.variations_sync.write_sent();
    let changed: Vec<Synced> = rows.iter().cloned().map(Synced::Variation).collect();
    let upload = sync::upload_for(model, &changed);
    write_then_upload(
        PersistenceOperation::SaveVariations(rows),
        upload,
        Event::VariationsStoreWritten,
    )
}

pub fn save_session(model: &mut Model, session: PracticeSession) -> Command<Effect, Event> {
    model.sessions_sync.write_sent();
    let upload = sync::upload_for(model, &[Synced::Session(session.clone())]);
    write_then_upload(
        PersistenceOperation::SaveSession(session),
        upload,
        Event::SessionStoreWritten,
    )
}

fn touched(records: &StoredRecords) -> Vec<RecordKind> {
    [
        (RecordKind::Item, records.items.is_empty()),
        (RecordKind::Variation, records.variations.is_empty()),
        (RecordKind::Session, records.sessions.is_empty()),
    ]
    .into_iter()
    .filter_map(|(kind, empty)| (!empty).then_some(kind))
    .collect()
}

/// Moves on with every write sent on the list, so a merge can tell that its
/// stored copy may be out of date.
pub fn write_generation(model: &Model, kind: RecordKind) -> u64 {
    match kind {
        RecordKind::Item => model.items_sync.generation,
        RecordKind::Variation => model.variations_sync.generation,
        RecordKind::Session => model.sessions_sync.generation,
    }
}

fn list_sync(model: &mut Model, kind: RecordKind) -> &mut ListSync {
    match kind {
        RecordKind::Item => &mut model.items_sync,
        RecordKind::Variation => &mut model.variations_sync,
        RecordKind::Session => &mut model.sessions_sync,
    }
}

fn load_list(model: &mut Model, kind: RecordKind) -> Command<Effect, Event> {
    match kind {
        RecordKind::Item => load_items(model),
        RecordKind::Variation => load_variations(model),
        RecordKind::Session => load_sessions(model),
    }
}

/// Counted as a write on each list it changes, so a load already out cannot
/// land over the merge (#2067).
pub fn apply_merged(
    model: &mut Model,
    records: StoredRecords,
    unpark: Vec<RecordKey>,
) -> Command<Effect, Event> {
    let touched = touched(&records);
    for kind in &touched {
        list_sync(model, *kind).write_sent();
    }
    Command::request_from_shell(PersistenceOperation::ApplyMerged(records)).then_send(
        move |output| {
            Event::Sync(sync::SyncEvent::MergeWritten {
                touched,
                unpark,
                output,
            })
        },
    )
}

/// Reloads each list the merge changed, so the screens show the winners.
pub fn merge_written(
    model: &mut Model,
    touched: &[RecordKind],
    refused: bool,
) -> Command<Effect, Event> {
    let reloads: Vec<_> = touched
        .iter()
        .filter_map(|kind| {
            let due = list_sync(model, *kind).write_settled(refused);
            (due || !refused).then(|| load_list(model, *kind))
        })
        .collect();
    Command::all(reloads)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::item::ItemKind;
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
            exercise_links: vec![],
            priority: false,
            chord_chart: None,
            variation_ids: vec![],
            keys: vec![],
            sections: vec![],
            photo_id: None,
            metre: None,
        }
    }

    #[test]
    fn load_items_requests_the_load_operation() {
        let mut cmd = load_items(&mut Model::default());
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
    fn store_loaded_items_replaces_model_items() {
        let app = crate::app::Intrada;
        let mut model = Model {
            items: vec![sample_item("stale")].into(),
            ..Default::default()
        };
        let _ = app.update(
            Event::StoreLoaded(PersistenceOutput::Items(vec![sample_item("fresh")])),
            &mut model,
        );
        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].id, "fresh");
    }

    fn variation(id: &str, deleted: bool) -> Variation {
        let now = chrono::Utc::now();
        Variation {
            id: id.to_string(),
            label: id.to_string(),
            updated_at: now,
            deleted_at: deleted.then_some(now),
        }
    }

    fn saved_variations(cmd: &mut Command<Effect, Event>) -> Option<Vec<Variation>> {
        cmd.effects().find_map(|e| match e {
            Effect::Persistence(req) => match req.operation {
                PersistenceOperation::SaveVariations(rows) => Some(rows),
                _ => None,
            },
            _ => None,
        })
    }

    #[test]
    fn start_app_loads_the_variations_too() {
        let app = crate::app::Intrada;
        let mut cmd = app.update(Event::StartApp, &mut Model::default());
        assert!(cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
            if req.operation == PersistenceOperation::LoadVariations)));
    }

    /// The built-ins seed an empty library once, and are written so a second
    /// launch finds them; a library of tombstones is not empty (#2246).
    #[test]
    fn an_empty_library_is_seeded_with_the_built_ins_and_saved() {
        let app = crate::app::Intrada;
        let mut model = Model::default();
        let _ = load_variations(&mut model);
        let mut cmd = app.update(
            Event::VariationsStoreLoaded(PersistenceOutput::Variations(vec![])),
            &mut model,
        );

        let saved = saved_variations(&mut cmd).expect("the seed is written");
        assert_eq!(saved.len(), 4);
        assert_eq!(model.variations.len(), 4);

        let mut model = Model::default();
        let _ = load_variations(&mut model);
        let mut cmd = app.update(
            Event::VariationsStoreLoaded(PersistenceOutput::Variations(vec![variation(
                crate::domain::variation::BUILT_INS[0].0,
                true,
            )])),
            &mut model,
        );
        assert_eq!(
            saved_variations(&mut cmd),
            None,
            "a deleted built-in stays deleted"
        );
        assert_eq!(model.variations.len(), 1);
    }

    #[test]
    fn a_failed_seed_write_is_not_retried_on_the_reload() {
        let app = crate::app::Intrada;
        let mut model = Model::default();
        let _ = load_variations(&mut model);
        let mut cmd = app.update(
            Event::VariationsStoreLoaded(PersistenceOutput::Variations(vec![])),
            &mut model,
        );
        assert!(saved_variations(&mut cmd).is_some());
        let _ = app.update(
            Event::VariationsStoreWritten(PersistenceOutput::Failed),
            &mut model,
        );

        let mut cmd = app.update(
            Event::VariationsStoreLoaded(PersistenceOutput::Variations(vec![])),
            &mut model,
        );
        assert_eq!(saved_variations(&mut cmd), None, "no write loop");
        assert!(model.variations.is_empty());
    }

    #[test]
    fn a_failed_variation_write_surfaces_and_reloads() {
        let app = crate::app::Intrada;
        let mut model = Model::default();
        let _ = save_variations(&mut model, vec![variation("v1", false)]);
        let mut cmd = app.update(
            Event::VariationsStoreWritten(PersistenceOutput::Failed),
            &mut model,
        );
        assert!(model.last_error.is_some(), "never a silent success");
        assert!(cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
            if req.operation == PersistenceOperation::LoadVariations)));
    }

    #[test]
    fn store_loaded_failed_surfaces_an_error() {
        let app = crate::app::Intrada;
        let mut model = Model::default();
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
        let mut model = Model::default();
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
        let mut model = Model::default();
        let mut cmd = app.update(Event::StoreWritten(PersistenceOutput::Ack), &mut model);
        assert!(model.last_error.is_none());
        assert!(!cmd.effects().any(|e| matches!(e, Effect::Persistence(_))));
    }

    #[test]
    fn store_loaded_ack_leaves_items_untouched() {
        let app = crate::app::Intrada;
        let mut model = Model {
            items: vec![sample_item("keep")].into(),
            ..Default::default()
        };
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

    fn has_persistence(cmd: &mut Command<Effect, Event>) -> bool {
        cmd.effects().any(|e| matches!(e, Effect::Persistence(_)))
    }

    fn has_delete(cmd: &mut Command<Effect, Event>, id: &str) -> bool {
        cmd.effects().any(|e| {
            matches!(e, Effect::Persistence(req)
            if matches!(&req.operation, PersistenceOperation::DeleteItem { id: op_id, .. } if op_id == id))
        })
    }

    #[test]
    fn save_item_requests_a_save_op() {
        let mut cmd = save_item(&mut Model::default(), sample_item("p1"));
        assert!(has_save(&mut cmd, "p1"));
    }

    #[test]
    fn save_items_requests_one_batch_op_with_all_rows() {
        let mut cmd = save_items(
            &mut Model::default(),
            vec![sample_item("a"), sample_item("b")],
        );
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
        let mut cmd = delete_item(
            &mut Model::default(),
            Item::fixture("gone"),
            chrono::Utc::now(),
        );
        assert!(has_delete(&mut cmd, "gone"));
    }

    #[test]
    fn delete_stamps_the_delete_instant() {
        let app = crate::app::Intrada;
        let mut model = Model {
            items: vec![sample_item("p1")].into(),
            ..Default::default()
        };
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
            tempo: None,
            notes: None,
            tags: vec![],
            photo_id: None,
            variation_labels: Vec::new(),
        }
    }

    #[test]
    fn add_persists_with_the_client_ulid() {
        use crate::domain::item::ItemEvent;
        let app = crate::app::Intrada;
        let mut model = Model::default();
        let mut cmd = app.update(Event::Item(ItemEvent::Add(create_item())), &mut model);
        let id = model.items[0].id.clone();
        assert!(
            has_save(&mut cmd, &id),
            "create persists with the client ulid"
        );
    }

    #[test]
    fn update_persists_the_edited_row() {
        use crate::domain::item::ItemEvent;
        use crate::domain::types::UpdateItem;
        let app = crate::app::Intrada;
        let mut model = Model {
            items: vec![sample_item("p1")].into(),
            ..Default::default()
        };
        let input = UpdateItem {
            title: Some("Renamed".into()),
            kind: None,
            composer: None,
            key: crate::domain::types::KeyEdit::Keep,
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
    }

    #[test]
    fn delete_persists_a_tombstone() {
        use crate::domain::item::ItemEvent;
        let app = crate::app::Intrada;
        let mut model = Model {
            items: vec![sample_item("p1")].into(),
            ..Default::default()
        };
        let mut cmd = app.update(
            Event::Item(ItemEvent::Delete { id: "p1".into() }),
            &mut model,
        );
        assert!(has_delete(&mut cmd, "p1"));
    }

    #[test]
    fn a_failed_delete_rolls_back_via_store_reload() {
        // End-to-end rollback (#834): an optimistic delete whose store write
        // fails must never be a silent success (invariant #5) — it surfaces an
        // error and reloads from the store, which still holds the row the
        // failed write never removed.
        use crate::domain::item::ItemEvent;
        let app = crate::app::Intrada;
        let mut model = Model {
            items: vec![sample_item("p1")].into(),
            ..Default::default()
        };
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

    // ── Storage failures and the dismiss mute (#1936) ─────────────────

    type Arm = (&'static str, fn() -> Event);

    fn failure_arms() -> [Arm; 4] {
        [
            ("items read", || {
                Event::StoreLoaded(PersistenceOutput::Failed)
            }),
            ("items write", || {
                Event::StoreWritten(PersistenceOutput::Failed)
            }),
            ("sessions read", || {
                Event::SessionsStoreLoaded(PersistenceOutput::Failed)
            }),
            ("sessions write", || {
                Event::SessionStoreWritten(PersistenceOutput::Failed)
            }),
        ]
    }

    fn ack_arms() -> [Arm; 2] {
        [
            ("items write", || {
                Event::StoreWritten(PersistenceOutput::Ack)
            }),
            ("sessions write", || {
                Event::SessionStoreWritten(PersistenceOutput::Ack)
            }),
        ]
    }

    #[test]
    fn every_storage_failure_arm_reaches_the_banner_and_the_sequence() {
        let app = crate::app::Intrada;
        for (arm, fail) in failure_arms() {
            let mut model = Model::default();
            let _ = app.update(fail(), &mut model);
            assert_eq!(
                model.last_error.as_deref(),
                Some("Couldn't access local storage."),
                "{arm}"
            );
            assert_eq!(app.view(&model).error_seq, 1, "{arm}");
        }
    }

    #[test]
    fn an_acknowledged_write_after_dismiss_lets_the_next_failure_show() {
        let app = crate::app::Intrada;
        for (ack, ack_event) in ack_arms() {
            for (arm, fail) in failure_arms() {
                let mut model = Model::default();
                let _ = app.update(fail(), &mut model);
                let _ = app.update(Event::ClearError, &mut model);
                let _ = app.update(ack_event(), &mut model);
                let _ = app.update(fail(), &mut model);
                assert!(
                    model.last_error.is_some(),
                    "{arm} after a dismiss and an acknowledged {ack}"
                );
            }
        }
    }

    #[test]
    fn a_write_the_store_has_not_confirmed_keeps_the_mute() {
        use crate::domain::item::ItemEvent;
        let app = crate::app::Intrada;
        let mut model = Model::default();
        let _ = app.update(Event::StoreWritten(PersistenceOutput::Failed), &mut model);
        let _ = app.update(Event::ClearError, &mut model);
        let _ = app.update(Event::Item(ItemEvent::Add(create_item())), &mut model);
        let _ = app.update(Event::StoreWritten(PersistenceOutput::Failed), &mut model);
        assert!(
            model.last_error.is_none(),
            "a still-broken store must not re-pop a dismissed banner (#346)"
        );
    }

    #[test]
    fn an_acknowledged_write_leaves_a_standing_banner_alone() {
        let app = crate::app::Intrada;
        for (ack, ack_event) in ack_arms() {
            let mut model = Model::default();
            let _ = app.update(Event::StoreLoaded(PersistenceOutput::Failed), &mut model);
            let _ = app.update(ack_event(), &mut model);
            assert!(model.last_error.is_some(), "{ack}");
        }
    }

    #[test]
    fn start_app_hydrates_from_store() {
        let app = crate::app::Intrada;
        let mut model = Model::default();
        let mut cmd = app.update(Event::StartApp, &mut model);
        assert!(cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
            if req.operation == PersistenceOperation::LoadItems)));
    }

    /// Offline-first invariant 1 (`.claude/rules/offline-first.md`): every step
    /// of the lifecycle (launch, create, update, delete) reaches the on-device
    /// store, so nothing the musician does lives only in memory.
    #[test]
    fn offline_invariant_lifecycle_reaches_the_store_at_every_step() {
        use crate::domain::item::ItemEvent;
        use crate::domain::types::UpdateItem;
        let app = crate::app::Intrada;
        let mut model = Model::default();

        let mut launch = app.update(Event::StartApp, &mut model);
        assert!(has_persistence(&mut launch), "launch");

        let mut add = app.update(Event::Item(ItemEvent::Add(create_item())), &mut model);
        assert!(has_persistence(&mut add), "create");
        let id = model.items[0].id.clone();

        let input = UpdateItem {
            title: Some("Renamed".into()),
            kind: None,
            composer: None,
            key: crate::domain::types::KeyEdit::Keep,
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
        assert!(has_persistence(&mut update), "update");

        let mut delete = app.update(Event::Item(ItemEvent::Delete { id }), &mut model);
        assert!(has_persistence(&mut delete), "delete");
    }

    // ── A load never overwrites a newer edit (#2067) ────────────────────

    fn asks_for_items(cmd: &mut Command<Effect, Event>) -> bool {
        cmd.effects().any(|e| {
            matches!(e, Effect::Persistence(req) if req.operation == PersistenceOperation::LoadItems)
        })
    }

    fn items_loaded(items: Vec<Item>) -> Event {
        Event::StoreLoaded(PersistenceOutput::Items(items))
    }

    fn add(app: &crate::app::Intrada, model: &mut Model) -> String {
        use crate::domain::item::ItemEvent;
        let _ = app.update(Event::Item(ItemEvent::Add(create_item())), model);
        model.items.last().expect("the added item").id.clone()
    }

    #[test]
    fn a_load_out_when_an_edit_is_sent_is_dropped_and_asked_again_after_the_ack() {
        let app = crate::app::Intrada;
        let mut model = Model::default();
        let _ = app.update(Event::StartApp, &mut model);
        let id = add(&app, &mut model);

        let mut landed = app.update(items_loaded(vec![]), &mut model);
        assert_eq!(
            model.items.len(),
            1,
            "the older list must not drop the edit"
        );
        assert!(!asks_for_items(&mut landed), "the write is still out");

        let mut acked = app.update(Event::StoreWritten(PersistenceOutput::Ack), &mut model);
        assert!(
            asks_for_items(&mut acked),
            "ask again once the write settles"
        );

        let _ = app.update(items_loaded(vec![sample_item(&id)]), &mut model);
        assert_eq!(model.items.len(), 1);
        assert_eq!(model.items[0].title, "Etude", "the fresh load applies");
    }

    #[test]
    fn an_edit_acknowledged_before_the_older_load_lands_still_drops_that_load() {
        let app = crate::app::Intrada;
        let mut model = Model::default();
        let _ = app.update(Event::StartApp, &mut model);
        let _ = add(&app, &mut model);

        let mut acked = app.update(Event::StoreWritten(PersistenceOutput::Ack), &mut model);
        assert!(!asks_for_items(&mut acked), "a load is still out");

        let mut landed = app.update(items_loaded(vec![]), &mut model);
        assert_eq!(model.items.len(), 1);
        assert!(asks_for_items(&mut landed), "nothing is out, so ask again");
    }

    #[test]
    fn a_load_landing_while_an_earlier_edit_is_unconfirmed_waits_for_it() {
        let app = crate::app::Intrada;
        let mut model = Model::default();
        let _ = add(&app, &mut model);
        let _ = app.update(Event::StartApp, &mut model);

        let mut landed = app.update(items_loaded(vec![]), &mut model);
        assert_eq!(model.items.len(), 1);
        assert!(!asks_for_items(&mut landed));

        let mut acked = app.update(Event::StoreWritten(PersistenceOutput::Ack), &mut model);
        assert!(asks_for_items(&mut acked));
    }

    #[test]
    fn a_refused_edit_while_a_load_is_out_rolls_back_after_that_load() {
        let app = crate::app::Intrada;
        let mut model = Model::default();
        let _ = app.update(Event::StartApp, &mut model);
        let _ = add(&app, &mut model);

        let mut refused = app.update(Event::StoreWritten(PersistenceOutput::Failed), &mut model);
        assert!(model.last_error.is_some());
        assert!(
            !asks_for_items(&mut refused),
            "a load already out could land over the rollback"
        );

        let mut landed = app.update(items_loaded(vec![]), &mut model);
        assert!(asks_for_items(&mut landed), "the rollback load goes now");

        let _ = app.update(items_loaded(vec![]), &mut model);
        assert!(model.items.is_empty(), "the refused add rolls back");
    }

    #[test]
    fn two_loads_around_an_edit_both_drop_and_ask_once() {
        let app = crate::app::Intrada;
        let mut model = Model::default();
        let _ = app.update(Event::StartApp, &mut model);
        let _ = add(&app, &mut model);
        let _ = app.update(Event::StartApp, &mut model);

        let mut first = app.update(items_loaded(vec![]), &mut model);
        let mut acked = app.update(Event::StoreWritten(PersistenceOutput::Ack), &mut model);
        assert!(!asks_for_items(&mut first));
        assert!(!asks_for_items(&mut acked), "the second load is still out");

        let mut second = app.update(items_loaded(vec![]), &mut model);
        assert_eq!(model.items.len(), 1);
        assert!(asks_for_items(&mut second));
    }

    #[test]
    fn a_failed_load_never_asks_again_but_the_edit_it_missed_does() {
        let app = crate::app::Intrada;
        let mut model = Model::default();
        let _ = app.update(Event::StartApp, &mut model);
        let _ = add(&app, &mut model);

        let mut failed = app.update(Event::StoreLoaded(PersistenceOutput::Failed), &mut model);
        assert!(model.last_error.is_some());
        assert!(
            !asks_for_items(&mut failed),
            "a broken store must not loop (#825)"
        );
        assert_eq!(model.items.len(), 1);

        let mut acked = app.update(Event::StoreWritten(PersistenceOutput::Ack), &mut model);
        assert!(
            asks_for_items(&mut acked),
            "the failed load is no longer out"
        );
    }

    #[test]
    fn a_refused_edit_still_rolls_back_when_the_load_out_fails() {
        let app = crate::app::Intrada;
        let mut model = Model::default();
        let _ = app.update(Event::StartApp, &mut model);
        let _ = add(&app, &mut model);
        let _ = app.update(Event::StoreWritten(PersistenceOutput::Failed), &mut model);

        let mut failed = app.update(Event::StoreLoaded(PersistenceOutput::Failed), &mut model);
        assert!(
            asks_for_items(&mut failed),
            "the refused add must roll back"
        );

        let mut again = app.update(Event::StoreLoaded(PersistenceOutput::Failed), &mut model);
        assert!(
            !asks_for_items(&mut again),
            "a broken store must not loop (#825)"
        );
    }

    #[test]
    fn a_load_answered_with_no_list_is_no_longer_out() {
        let app = crate::app::Intrada;
        let mut model = Model::default();
        let _ = app.update(Event::StartApp, &mut model);
        let _ = add(&app, &mut model);
        let _ = app.update(Event::StoreLoaded(PersistenceOutput::Ack), &mut model);

        let mut acked = app.update(Event::StoreWritten(PersistenceOutput::Ack), &mut model);
        assert!(asks_for_items(&mut acked));
    }

    // ── Sessions ────────────────────────────────────────────────────────

    fn sample_session(id: &str) -> PracticeSession {
        let now = chrono::Utc::now();
        PracticeSession {
            id: id.to_string(),
            entries: vec![],
            session_notes: None,
            started_at: now,
            completed_at: now,
            total_duration_secs: 0,
            completion_status: crate::domain::session::CompletionStatus::Completed,
            session_score: None,
            capture_version: None,
        }
    }

    #[test]
    fn save_session_requests_a_save_op() {
        let mut cmd = save_session(&mut Model::default(), sample_session("s1"));
        assert!(cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
            if matches!(&req.operation, PersistenceOperation::SaveSession(s) if s.id == "s1"))));
    }

    #[test]
    fn load_sessions_requests_the_load_operation() {
        let mut cmd = load_sessions(&mut Model::default());
        assert!(cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
            if req.operation == PersistenceOperation::LoadSessions)));
    }

    #[test]
    fn sessions_store_loaded_replaces_model_sessions() {
        let app = crate::app::Intrada;
        let mut model = Model {
            sessions: vec![sample_session("stale")].into(),
            ..Default::default()
        };
        let _ = app.update(
            Event::SessionsStoreLoaded(PersistenceOutput::Sessions(vec![sample_session("fresh")])),
            &mut model,
        );
        assert_eq!(model.sessions.len(), 1);
        assert_eq!(model.sessions[0].id, "fresh");
    }

    #[test]
    fn start_app_also_loads_sessions() {
        let app = crate::app::Intrada;
        let mut model = Model::default();
        let mut cmd = app.update(Event::StartApp, &mut model);
        assert!(cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
            if req.operation == PersistenceOperation::LoadSessions)));
    }

    #[test]
    fn session_store_written_failed_reloads_sessions() {
        let app = crate::app::Intrada;
        let mut model = Model::default();
        let mut cmd = app.update(
            Event::SessionStoreWritten(PersistenceOutput::Failed),
            &mut model,
        );
        assert!(model.last_error.is_some());
        assert!(cmd.effects().any(|e| matches!(e, Effect::Persistence(req)
            if req.operation == PersistenceOperation::LoadSessions)));
    }
}
