//! The notebook's on-device database, shared by both phones (#2421,
//! `specs/android-shell.md`, The shared store). The core decides every read
//! and write; this crate executes its `PersistenceOperation`s against SQLite.

mod codec;
mod items;
mod migrations;
mod sessions;
#[cfg(test)]
mod tests;

use std::path::Path;

use intrada_core::persistence::StoredRecords;
use intrada_core::stored_session::StoredSessionError;
use intrada_core::sync::{RecordKey, RecordKind};
use intrada_core::{PersistenceOperation, PersistenceOutput};
use rusqlite::{Connection, Transaction, TransactionBehavior};

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Session(#[from] StoredSessionError),
    #[error("{0} is out of range")]
    OutOfRange(&'static str),
}

/// What the store made of one operation.
#[derive(Debug, Clone, PartialEq)]
pub struct Answer {
    /// `Failed` when the operation did not complete; never a faked `Ack` (#816).
    pub output: PersistenceOutput,
    /// Stored values the store could not read, for the phone to report (#949).
    pub unreadable: Vec<String>,
    /// Why the operation failed, for the phone to report.
    pub error: Option<String>,
}

pub struct Store {
    conn: Connection,
    /// Rows a migration skipped, handed over with the first answer.
    pending: Vec<String>,
}

impl Store {
    /// Opens, or creates, the database file and brings it to the latest
    /// migration.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        Self::new(Connection::open(path)?, migrations::latest())
    }

    /// For tests, previews and the fallback when the file will not open.
    pub fn in_memory() -> Result<Self, StoreError> {
        Self::new(Connection::open_in_memory()?, migrations::latest())
    }

    fn new(mut conn: Connection, target: &str) -> Result<Self, StoreError> {
        let mut pending = Vec::new();
        migrations::migrate(&mut conn, target, &mut pending)?;
        Ok(Self { conn, pending })
    }

    pub fn handle(&mut self, operation: &PersistenceOperation) -> Answer {
        let mut unreadable = std::mem::take(&mut self.pending);
        match self.run(operation, &mut unreadable) {
            Ok(output) => Answer {
                output,
                unreadable,
                error: None,
            },
            Err(e) => Answer {
                output: PersistenceOutput::Failed,
                unreadable,
                error: Some(e.to_string()),
            },
        }
    }

    /// Each operation is one transaction: a batch lands whole or not at all
    /// (#1106, invariant 5).
    fn run(
        &mut self,
        operation: &PersistenceOperation,
        unreadable: &mut Vec<String>,
    ) -> Result<PersistenceOutput, StoreError> {
        let behaviour = match operation {
            PersistenceOperation::LoadItems
            | PersistenceOperation::LoadSessions
            | PersistenceOperation::LoadVariations
            | PersistenceOperation::LoadRecords(_)
            | PersistenceOperation::LoadAllRecords => TransactionBehavior::Deferred,
            _ => TransactionBehavior::Immediate,
        };
        let tx = self.conn.transaction_with_behavior(behaviour)?;
        let output = run_in(&tx, operation, unreadable)?;
        tx.commit()?;
        Ok(output)
    }
}

fn run_in(
    tx: &Transaction,
    operation: &PersistenceOperation,
    unreadable: &mut Vec<String>,
) -> Result<PersistenceOutput, StoreError> {
    Ok(match operation {
        PersistenceOperation::LoadItems => PersistenceOutput::Items(items::load(tx, unreadable)?),
        PersistenceOperation::SaveItem(item) => {
            items::upsert(tx, item, unreadable)?;
            PersistenceOutput::Ack
        }
        PersistenceOperation::SaveItems(batch) => {
            for item in batch {
                items::upsert(tx, item, unreadable)?;
            }
            PersistenceOutput::Ack
        }
        PersistenceOperation::DeleteItem { id, deleted_at } => {
            items::delete(tx, id, deleted_at)?;
            PersistenceOutput::Ack
        }
        PersistenceOperation::LoadSessions => {
            PersistenceOutput::Sessions(sessions::load(tx, unreadable)?)
        }
        PersistenceOperation::SaveSession(session) => {
            sessions::save(tx, session)?;
            PersistenceOutput::Ack
        }
        PersistenceOperation::LoadVariations => {
            PersistenceOutput::Variations(sessions::load_variations(tx, unreadable)?)
        }
        PersistenceOperation::SaveVariations(variations) => {
            sessions::save_variations(tx, variations)?;
            PersistenceOutput::Ack
        }
        PersistenceOperation::LoadRecords(keys) => {
            PersistenceOutput::Records(records(tx, Some(keys), unreadable)?)
        }
        PersistenceOperation::LoadAllRecords => {
            PersistenceOutput::Records(records(tx, None, unreadable)?)
        }
        PersistenceOperation::ApplyMerged(merged) => {
            for item in &merged.items {
                items::put_merged(tx, item, unreadable)?;
            }
            sessions::save_variations(tx, &merged.variations)?;
            for session in &merged.sessions {
                sessions::save(tx, session)?;
            }
            PersistenceOutput::Ack
        }
    })
}

/// Every stored copy the keys name, or everything when there are none, with
/// the named rows this app cannot read.
fn records(
    tx: &Transaction,
    keys: Option<&Vec<RecordKey>>,
    unreadable: &mut Vec<String>,
) -> Result<StoredRecords, StoreError> {
    let wanted = |kind: RecordKind, id: &str| {
        keys.is_none_or(|keys| keys.iter().any(|k| k.kind == kind && k.id == id))
    };
    let mut skipped = [Vec::new(), Vec::new(), Vec::new()];
    let [skipped_items, skipped_variations, skipped_sessions] = &mut skipped;
    let records = StoredRecords {
        items: items::load_with_tombstones(tx, unreadable, skipped_items)?
            .into_iter()
            .filter(|s| wanted(RecordKind::Item, &s.item.id))
            .collect(),
        variations: sessions::variation_rows(tx, unreadable, skipped_variations)?
            .into_iter()
            .filter(|v| wanted(RecordKind::Variation, &v.id))
            .collect(),
        sessions: sessions::load_with_tombstones(tx, unreadable, skipped_sessions)?
            .into_iter()
            .filter(|s| wanted(RecordKind::Session, &s.id))
            .collect(),
        unreadable: Vec::new(),
    };
    let unreadable_keys = [RecordKind::Item, RecordKind::Variation, RecordKind::Session]
        .into_iter()
        .zip(skipped)
        .flat_map(|(kind, ids)| ids.into_iter().map(move |id| RecordKey { kind, id }))
        .filter(|key| wanted(key.kind, &key.id))
        .collect();
    Ok(StoredRecords {
        unreadable: unreadable_keys,
        ..records
    })
}
