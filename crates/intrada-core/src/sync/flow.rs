use std::collections::{BTreeMap, BTreeSet};

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
    /// Never uploaded over: the shell replays these at launch (#2355).
    pub parked: BTreeSet<RecordKey>,
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
        .filter(|s| !model.sync.parked.contains(&s.key()))
        .filter_map(|s| s.record().ok())
        .collect()
}

pub fn update(event: SyncEvent, model: &mut Model) -> Command<Effect, Event> {
    match event {
        SyncEvent::RecordsArrived(records) => arrived(model, records),
        SyncEvent::StoredLoaded { arrived, output } => match output {
            PersistenceOutput::Records(stored) => merge(model, arrived, stored),
            PersistenceOutput::Failed => storage_failed(model),
            _ => Command::done(),
        },
        SyncEvent::MergeWritten {
            touched,
            unpark,
            output,
        } => {
            let refused = !matches!(output, PersistenceOutput::Ack);
            let reloads = persistence::merge_written(model, &touched, refused);
            if refused {
                model.surface_storage_error();
                return Command::all([reloads, crux_core::render::render()]);
            }
            Command::all([reloads, unpark_command(model, unpark)])
        }
        SyncEvent::UploadEverything => {
            Command::request_from_shell(PersistenceOperation::LoadAllRecords)
                .then_send(|output| Event::Sync(SyncEvent::EverythingLoaded(output)))
        }
        SyncEvent::EverythingLoaded(output) => match output {
            PersistenceOutput::Records(stored) => {
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

fn arrived(model: &mut Model, records: Vec<SyncRecord>) -> Command<Effect, Event> {
    let (too_new, readable): (Vec<_>, Vec<_>) = records
        .into_iter()
        .partition(|r| decide(None, r) == Verdict::Park);
    let park = park_command(model, too_new);
    if readable.is_empty() {
        return park;
    }
    let mut keys: Vec<RecordKey> = readable.iter().map(SyncRecord::key).collect();
    keys.sort();
    keys.dedup();
    let load = Command::request_from_shell(PersistenceOperation::LoadRecords(keys)).then_send(
        move |output| {
            Event::Sync(SyncEvent::StoredLoaded {
                arrived: readable,
                output,
            })
        },
    );
    Command::all([park, load])
}

fn merge(
    model: &mut Model,
    arrived: Vec<SyncRecord>,
    stored: StoredRecords,
) -> Command<Effect, Event> {
    let mut current: BTreeMap<RecordKey, SyncRecord> = synced(stored)
        .iter()
        .filter_map(|s| s.record().ok().map(|r| (s.key(), r)))
        .collect();
    let mut winners: BTreeMap<RecordKey, Synced> = BTreeMap::new();
    let mut unreadable = Vec::new();
    let mut unpark = BTreeSet::new();
    for record in arrived {
        let key = record.key();
        match decide(current.get(&key), &record) {
            Verdict::Take => match Synced::decode(&record) {
                Ok(synced) => {
                    winners.insert(key.clone(), synced);
                    current.insert(key.clone(), record);
                    if model.sync.parked.contains(&key) {
                        unpark.insert(key);
                    }
                }
                Err(_) => unreadable.push(record),
            },
            Verdict::Keep if model.sync.parked.contains(&key) => {
                unpark.insert(key);
            }
            Verdict::Keep | Verdict::Park => {}
        }
    }
    let park = park_command(model, unreadable);
    let unpark: Vec<RecordKey> = unpark.into_iter().collect();
    if winners.is_empty() {
        return Command::all([park, unpark_command(model, unpark)]);
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
    Command::all([park, persistence::apply_merged(model, records, unpark)])
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
    model
        .sync
        .parked
        .extend(records.iter().map(SyncRecord::key));
    Command::notify_shell(SyncOperation::Park(records)).into()
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
