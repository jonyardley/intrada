use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use crux_core::command::Command;
use serde::{Deserialize, Serialize};

use super::{decide, RecordKey, RecordKind, SyncOperation, SyncRecord, Synced, Verdict};
use crate::app::{Effect, Event};
use crate::model::Model;
use crate::persistence::{
    self, PersistenceOperation, PersistenceOutput, StoredItem, StoredRecords,
};

#[derive(Debug, Default)]
pub struct SyncState {
    /// Set by an account change; what follows is #2356's.
    pub paused: bool,
    /// Never uploaded over: the shell replays these at launch (#2355). Each
    /// keeps the change time of the copy parked, so an older arrival cannot
    /// unpark it.
    pub parked: BTreeMap<RecordKey, DateTime<Utc>>,
    /// The merge in flight, with each list's write generation as its stored
    /// copies were asked for. One at a time, so two never race.
    merging: Option<BTreeMap<RecordKind, u64>>,
    queued: Vec<SyncRecord>,
    /// A load or merge write failed since the core last settled, so the
    /// shell must fetch those batches again.
    failed: bool,
    /// Rows the store cannot read: uploading the readable part would delete
    /// the rest on the other device.
    unreadable: BTreeSet<RecordKey>,
}

/// Provisional until the trial on #2361 reports.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum SyncEvent {
    RecordsArrived(Vec<SyncRecord>),
    UploadEverything,
    AccountChanged,
    StoredLoaded {
        arrived: Vec<SyncRecord>,
        output: PersistenceOutput,
    },
    EverythingLoaded(PersistenceOutput),
    MergeWritten {
        touched: Vec<RecordKind>,
        unpark: Vec<RecordKey>,
        output: PersistenceOutput,
    },
}

/// The records a local write should upload once the store keeps it.
pub fn upload_for(model: &Model, changed: &[Synced]) -> Vec<SyncRecord> {
    if model.sync.paused {
        return Vec::new();
    }
    changed
        .iter()
        .filter(|s| !model.sync.parked.contains_key(&s.key()))
        .filter(|s| !model.sync.unreadable.contains(&s.key()))
        .filter_map(|s| s.record().ok())
        .collect()
}

pub fn update(event: SyncEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        SyncEvent::RecordsArrived(records) => {
            if model.sync.merging.is_some() {
                model.sync.queued.extend(records);
                return Command::done();
            }
            start(model, records)
        }
        SyncEvent::StoredLoaded { arrived, output } => match output {
            PersistenceOutput::Records(_) if written_since_loaded(model, &arrived) => {
                load_stored(model, arrived)
            }
            PersistenceOutput::Records(stored) => merge(model, arrived, stored),
            PersistenceOutput::Failed => {
                model.sync.failed = true;
                let failed = storage_failed(model);
                Command::all([failed, finish(model)])
            }
            _ => {
                model.sync.failed = true;
                finish(model)
            }
        },
        SyncEvent::MergeWritten {
            touched,
            unpark,
            output,
        } => {
            let refused = !matches!(output, PersistenceOutput::Ack);
            let reloads = persistence::merge_written(model, &touched, refused);
            let shown = if refused {
                model.sync.failed = true;
                model.surface_storage_error();
                crux_core::render::render()
            } else {
                unpark_command(model, unpark)
            };
            Command::all([reloads, shown, finish(model)])
        }
        SyncEvent::UploadEverything => {
            Command::request_from_shell(PersistenceOperation::LoadAllRecords)
                .then_send(|output| Event::Sync(SyncEvent::EverythingLoaded(output)))
        }
        SyncEvent::EverythingLoaded(output) => match output {
            PersistenceOutput::Records(mut stored) => {
                model.sync.unreadable =
                    std::mem::take(&mut stored.unreadable).into_iter().collect();
                let records = upload_for(model, &synced(stored));
                if records.is_empty() {
                    Command::done()
                } else {
                    Command::notify_shell(SyncOperation::Upload(records)).into()
                }
            }
            PersistenceOutput::Failed => storage_failed(model),
            _ => Command::done(),
        },
        SyncEvent::AccountChanged => {
            model.sync.paused = true;
            Command::done()
        }
    }
}

fn start(model: &mut Model, records: Vec<SyncRecord>) -> Command<Effect, Event> {
    let (too_new, readable): (Vec<_>, Vec<_>) = records
        .into_iter()
        .partition(|r| decide(None, r) == Verdict::Park);
    let park = park_command(model, too_new);
    if readable.is_empty() {
        return Command::all([park, finish(model)]);
    }
    Command::all([park, load_stored(model, readable)])
}

fn load_stored(model: &mut Model, arrived: Vec<SyncRecord>) -> Command<Effect, Event> {
    let generations = arrived
        .iter()
        .map(|r| (r.kind, persistence::write_generation(model, r.kind)))
        .collect();
    model.sync.merging = Some(generations);
    let mut keys: Vec<RecordKey> = arrived.iter().map(SyncRecord::key).collect();
    keys.sort();
    keys.dedup();
    Command::request_from_shell(PersistenceOperation::LoadRecords(keys))
        .then_send(move |output| Event::Sync(SyncEvent::StoredLoaded { arrived, output }))
}

/// A local write sent after the stored copies were asked for may land before
/// the merge, which would then overwrite it with an older decision.
fn written_since_loaded(model: &Model, arrived: &[SyncRecord]) -> bool {
    let Some(loaded) = &model.sync.merging else {
        return false;
    };
    arrived
        .iter()
        .any(|r| loaded.get(&r.kind) != Some(&persistence::write_generation(model, r.kind)))
}

fn finish(model: &mut Model) -> Command<Effect, Event> {
    model.sync.merging = None;
    let queued = std::mem::take(&mut model.sync.queued);
    if !queued.is_empty() {
        return start(model, queued);
    }
    if std::mem::take(&mut model.sync.failed) {
        return Command::done();
    }
    Command::notify_shell(SyncOperation::Settled).into()
}

fn merge(
    model: &mut Model,
    arrived: Vec<SyncRecord>,
    mut stored: StoredRecords,
) -> Command<Effect, Event> {
    let unreadable_here: BTreeSet<RecordKey> =
        std::mem::take(&mut stored.unreadable).into_iter().collect();
    for record in &arrived {
        model.sync.unreadable.remove(&record.key());
    }
    let mut current: BTreeMap<RecordKey, SyncRecord> = synced(stored)
        .iter()
        .filter_map(|s| s.record().ok().map(|r| (s.key(), r)))
        .collect();
    let mut winners: BTreeMap<RecordKey, Synced> = BTreeMap::new();
    let mut to_park = Vec::new();
    let mut unpark_taken = BTreeSet::new();
    let mut unpark_kept = BTreeSet::new();
    for record in arrived {
        let key = record.key();
        if unreadable_here.contains(&key) {
            to_park.push(record);
            continue;
        }
        let unparks = model
            .sync
            .parked
            .get(&key)
            .is_some_and(|parked_at| record.changed_at >= *parked_at);
        match decide(current.get(&key), &record) {
            Verdict::Take => match Synced::decode(&record) {
                Ok(synced) => {
                    winners.insert(key.clone(), synced);
                    current.insert(key.clone(), record);
                    if unparks {
                        unpark_taken.insert(key);
                    }
                }
                Err(_) => to_park.push(record),
            },
            Verdict::Keep if unparks => {
                unpark_kept.insert(key);
            }
            Verdict::Keep | Verdict::Park => {}
        }
    }
    let park = park_command(model, to_park);
    let kept: Vec<SyncRecord> = unpark_kept
        .into_iter()
        .filter(|key| !winners.contains_key(key))
        .filter_map(|key| current.remove(&key))
        .collect();
    let kept = unpark_and_upload(model, kept);
    let unpark: Vec<RecordKey> = unpark_taken.into_iter().collect();
    if winners.is_empty() {
        let unpark = unpark_command(model, unpark);
        return Command::all([park, kept, unpark, finish(model)]);
    }
    let mut records = StoredRecords::default();
    for synced in winners.into_values() {
        match synced {
            Synced::Item { item, deleted_at } => records.items.push(StoredItem {
                item: *item,
                deleted_at,
            }),
            Synced::Variation(v) => records.variations.push(v),
            Synced::Session(s) => records.sessions.push(s),
        }
    }
    Command::all([
        park,
        kept,
        persistence::apply_merged(model, records, unpark),
    ])
}

fn synced(stored: StoredRecords) -> Vec<Synced> {
    let items = stored.items.into_iter().map(|s| Synced::Item {
        item: Box::new(s.item),
        deleted_at: s.deleted_at,
    });
    let variations = stored.variations.into_iter().map(Synced::Variation);
    let sessions = stored.sessions.into_iter().map(Synced::Session);
    items.chain(variations).chain(sessions).collect()
}

fn park_command(model: &mut Model, records: Vec<SyncRecord>) -> Command<Effect, Event> {
    if records.is_empty() {
        return Command::done();
    }
    for record in &records {
        let parked_at = model
            .sync
            .parked
            .entry(record.key())
            .or_insert(record.changed_at);
        *parked_at = (*parked_at).max(record.changed_at);
    }
    Command::notify_shell(SyncOperation::Park(records)).into()
}

/// The local copy won, and was held back while its id was parked.
fn unpark_and_upload(model: &mut Model, kept: Vec<SyncRecord>) -> Command<Effect, Event> {
    let unpark = unpark_command(model, kept.iter().map(SyncRecord::key).collect());
    if kept.is_empty() || model.sync.paused {
        return unpark;
    }
    Command::all([
        unpark,
        Command::notify_shell(SyncOperation::Upload(kept)).into(),
    ])
}

fn unpark_command(model: &mut Model, keys: Vec<RecordKey>) -> Command<Effect, Event> {
    if keys.is_empty() {
        return Command::done();
    }
    for key in &keys {
        model.sync.parked.remove(key);
    }
    Command::notify_shell(SyncOperation::Unpark(keys)).into()
}

fn storage_failed(model: &mut Model) -> Command<Effect, Event> {
    model.surface_storage_error();
    crux_core::render::render()
}
