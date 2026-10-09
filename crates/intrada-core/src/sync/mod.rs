//! iCloud sync: the record two devices exchange and the rule that picks the
//! copy both keep (`specs/icloud-sync.md`).

use std::cmp::Reverse;

use chrono::{DateTime, Utc};
use crux_core::capability::Operation;
use serde::{Deserialize, Serialize};

use crate::domain::item::Item;
use crate::domain::session::PracticeSession;
use crate::domain::variation::Variation;

mod flow;
#[cfg(test)]
mod tests;

pub use flow::{update, upload_for, SyncEvent, SyncState};

/// Bump when any synced type's JSON changes, then re-pin the bodies in
/// `tests.rs`. An older app parks a record written at a newer version.
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum RecordKind {
    Item,
    Variation,
    Session,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct RecordKey {
    pub kind: RecordKind,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
pub struct SyncRecord {
    pub kind: RecordKind,
    pub id: String,
    pub schema_version: u32,
    pub changed_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    /// JSON, not bincode: a newer app must still read an older device's
    /// records, and positional bincode cannot (#2234).
    pub body: Vec<u8>,
}

impl SyncRecord {
    pub fn key(&self) -> RecordKey {
        RecordKey {
            kind: self.kind,
            id: self.id.clone(),
        }
    }
}

/// Fire and forget: the shell carries records to and from iCloud and keeps
/// the parked ones (#2355). Provisional until the trial on #2361 reports.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "facet_typegen", derive(facet::Facet))]
#[cfg_attr(feature = "facet_typegen", repr(C))]
pub enum SyncOperation {
    Upload(Vec<SyncRecord>),
    Park(Vec<SyncRecord>),
    Unpark(Vec<RecordKey>),
}

impl Operation for SyncOperation {
    type Output = ();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Keep,
    Take,
    Park,
}

pub fn decide(local: Option<&SyncRecord>, arrived: &SyncRecord) -> Verdict {
    if arrived.schema_version > SCHEMA_VERSION {
        return Verdict::Park;
    }
    match local {
        Some(local) if rank(arrived) <= rank(local) => Verdict::Keep,
        _ => Verdict::Take,
    }
}

/// One order over every record, so two devices pick the same winner: later
/// first, then a delete over an edit, then the lower body.
fn rank(record: &SyncRecord) -> (DateTime<Utc>, bool, Reverse<&[u8]>) {
    (
        record.changed_at,
        record.deleted_at.is_some(),
        Reverse(record.body.as_slice()),
    )
}

/// A piece, variation or session as this device stores it, tombstone included.
#[derive(Debug, Clone, PartialEq)]
pub enum Synced {
    Item {
        item: Box<Item>,
        deleted_at: Option<DateTime<Utc>>,
    },
    Variation(Variation),
    Session(PracticeSession),
}

impl Synced {
    pub fn key(&self) -> RecordKey {
        let (kind, id) = match self {
            Synced::Item { item, .. } => (RecordKind::Item, &item.id),
            Synced::Variation(v) => (RecordKind::Variation, &v.id),
            Synced::Session(s) => (RecordKind::Session, &s.id),
        };
        RecordKey {
            kind,
            id: id.clone(),
        }
    }

    pub fn record(&self) -> Result<SyncRecord, serde_json::Error> {
        let (body, changed_at, deleted_at) = match self {
            Synced::Item { item, deleted_at } => (
                serde_json::to_vec(item)?,
                later(item.updated_at, *deleted_at),
                *deleted_at,
            ),
            Synced::Variation(v) => (
                serde_json::to_vec(v)?,
                later(v.updated_at, v.deleted_at),
                v.deleted_at,
            ),
            Synced::Session(s) => (serde_json::to_vec(s)?, s.completed_at, None),
        };
        let key = self.key();
        Ok(SyncRecord {
            kind: key.kind,
            id: key.id,
            schema_version: SCHEMA_VERSION,
            changed_at,
            deleted_at,
            body,
        })
    }

    pub fn decode(record: &SyncRecord) -> Result<Self, serde_json::Error> {
        Ok(match record.kind {
            RecordKind::Item => Synced::Item {
                item: serde_json::from_slice(&record.body)?,
                deleted_at: record.deleted_at,
            },
            RecordKind::Variation => Synced::Variation(serde_json::from_slice(&record.body)?),
            RecordKind::Session => Synced::Session(serde_json::from_slice(&record.body)?),
        })
    }
}

fn later(updated_at: DateTime<Utc>, deleted_at: Option<DateTime<Utc>>) -> DateTime<Utc> {
    deleted_at.map_or(updated_at, |d| d.max(updated_at))
}
