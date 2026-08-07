//! Fixtures the built-session tests share across `compose`, `blocks` and the
//! handlers, so a change to an entity's shape lands in one place.

use chrono::{DateTime, TimeZone, Utc};

use super::{BuiltBlock, BuiltSession, BuiltTarget, JournalItem, UserDrill};
use crate::domain::item::{Item, ItemKind};

pub fn at() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 8, 7, 10, 0, 0).unwrap()
}

pub fn user_drill(id: &str, name: &str) -> UserDrill {
    UserDrill {
        id: id.into(),
        name: name.into(),
        criterion: "Three clean passes at 72".into(),
        tempo_bpm: Some(72),
        keys: vec![],
        passes_to_open: 3,
        serves: None,
        created_at: at(),
        updated_at: at(),
        deleted_at: None,
    }
}

pub fn journal_item(id: &str, name: &str) -> JournalItem {
    JournalItem {
        id: id.into(),
        name: name.into(),
        notes: None,
        linked_item_id: None,
        created_at: at(),
        updated_at: at(),
        deleted_at: None,
    }
}

pub fn sample_item(id: &str, title: &str, kind: ItemKind) -> Item {
    Item {
        id: id.into(),
        title: title.into(),
        kind,
        composer: None,
        key: None,
        modality: None,
        tempo: None,
        notes: None,
        tags: vec![],
        linked_exercise_ids: vec![],
        created_at: at(),
        updated_at: at(),
        priority: false,
        chord_chart: None,
        variants: vec![],
    }
}

pub fn built_block(id: &str, target: BuiltTarget) -> BuiltBlock {
    BuiltBlock {
        id: id.into(),
        target,
        minutes: None,
    }
}

pub fn built_session() -> BuiltSession {
    BuiltSession {
        id: "01J000000000000000000BSESH".into(),
        source: Some("From Friday's lesson".into()),
        blocks: vec![],
        created_at: at(),
        updated_at: at(),
        deleted_at: None,
    }
}
