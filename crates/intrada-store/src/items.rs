// ── Items ────────────────────────────────────────────────────────────

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use intrada_core::domain::link::ExerciseLink;
use intrada_core::domain::section::{BarRange, ItemSection};
use intrada_core::persistence::StoredItem;
use intrada_core::{Item, Key, Tempo};
use rusqlite::{params, OptionalExtension, Row, Transaction};
use serde::de::DeserializeOwned;

use crate::codec::{
    self, decode_chord_chart, decode_json, decode_keys, decode_metre, encode_chord_chart,
    encode_json, encode_keys, encode_metre, item_kind, item_kind_text, parse_time, section_kind,
    section_kind_text, stored_key, time_text, try_decode_json, StoredChart, StoredKeyJson,
    StoredMetre, Unreadable,
};
use crate::StoreError;

pub(crate) fn load(tx: &Transaction, unreadable: &mut Unreadable) -> Result<Vec<Item>, StoreError> {
    Ok(
        load_rows(tx, "WHERE deleted_at IS NULL", unreadable, &mut Vec::new())?
            .into_iter()
            .map(|stored| stored.item)
            .collect(),
    )
}

/// Tombstones included, for the sync merge (`specs/icloud-sync.md`).
/// `skipped` gains each piece the merge must not write over: one that will
/// not read, or one with a section or link that will not.
pub(crate) fn load_with_tombstones(
    tx: &Transaction,
    unreadable: &mut Unreadable,
    skipped: &mut Vec<String>,
) -> Result<Vec<StoredItem>, StoreError> {
    load_rows(tx, "", unreadable, skipped)
}

fn load_rows(
    tx: &Transaction,
    filter: &str,
    unreadable: &mut Unreadable,
    skipped: &mut Vec<String>,
) -> Result<Vec<StoredItem>, StoreError> {
    let mut partial = HashSet::new();
    let mut sections = sections_by_item(tx, unreadable, &mut partial)?;
    let mut links = links_by_piece(tx, unreadable, &mut partial)?;
    let mut stmt = tx.prepare(&format!(
        "SELECT * FROM item {filter} ORDER BY created_at DESC"
    ))?;
    let mut rows = stmt.query([])?;
    let mut items = Vec::new();
    while let Some(row) = rows.next()? {
        let id: String = row.get("id")?;
        let item_sections = sections.remove(&id).unwrap_or_default();
        let item_links = links.remove(&id).unwrap_or_default();
        let read = item(row, item_sections, item_links, unreadable).and_then(|item| {
            Ok(StoredItem {
                item,
                deleted_at: optional_time(row, "deleted_at")?,
            })
        });
        if partial.contains(&id) {
            skipped.push(id.clone());
        }
        match read {
            Ok(stored) => items.push(stored),
            // Skipped, not defaulted: the row stays on disk untouched, since no
            // save names an item the core never loaded.
            Err(reason) => {
                unreadable.push(format!("item {id} skipped: {reason}"));
                if !partial.contains(&id) {
                    skipped.push(id);
                }
            }
        }
    }
    Ok(items)
}

/// The merge's copy, its tombstone written exactly as it arrived. The piece
/// wins whole: a section or link only this device has is deleted as of the
/// winner's change.
pub(crate) fn put_merged(
    tx: &Transaction,
    stored: &StoredItem,
    unreadable: &mut Unreadable,
) -> Result<(), StoreError> {
    let item = &stored.item;
    upsert(tx, item, unreadable)?;
    if let Some(deleted_at) = &stored.deleted_at {
        delete(tx, &item.id, deleted_at)?;
    }
    let changed_at = stored
        .deleted_at
        .map_or(item.updated_at, |d| d.max(item.updated_at));
    let sections: Vec<&str> = item.sections.iter().map(|s| s.id.as_str()).collect();
    let links: Vec<&str> = item.exercise_links.iter().map(|l| l.id.as_str()).collect();
    delete_unnamed(tx, "section", "item_id", &item.id, &sections, &changed_at)?;
    delete_unnamed(
        tx,
        "exercise_link",
        "piece_id",
        &item.id,
        &links,
        &changed_at,
    )?;
    Ok(())
}

fn delete_unnamed(
    tx: &Transaction,
    table: &str,
    owner: &str,
    owner_id: &str,
    named: &[&str],
    deleted_at: &DateTime<Utc>,
) -> Result<(), StoreError> {
    tx.execute(
        &format!(
            "UPDATE {table} SET deleted_at = ?1
             WHERE {owner} = ?2 AND deleted_at IS NULL
               AND id NOT IN (SELECT value FROM json_each(?3))"
        ),
        params![time_text(deleted_at), owner_id, encode_json(&named)?],
    )?;
    Ok(())
}

fn item(
    row: &Row,
    sections: Vec<ItemSection>,
    exercise_links: Vec<ExerciseLink>,
    unreadable: &mut Unreadable,
) -> Result<Item, String> {
    let id: String = column(row, "id")?;
    let marking: Option<String> = column(row, "tempo_marking")?;
    let bpm = narrow::<u16>(column(row, "tempo_bpm")?, "tempo_bpm")?;
    let key_text: Option<String> = column(row, "key")?;
    let modality: Option<String> = column(row, "modality")?;
    let tags: String = column(row, "tags")?;
    let variation_ids: Option<String> = column(row, "variation_ids")?;
    let chord_chart: Option<String> = column(row, "chord_chart")?;
    let metre: Option<String> = column(row, "metre")?;
    let keys: Option<String> = column(row, "keys")?;
    let priority: i64 = column(row, "priority")?;
    Ok(Item {
        title: column(row, "title")?,
        kind: item_kind(&column::<String>(row, "kind")?, unreadable),
        composer: column(row, "composer")?,
        key: codec::key(key_text.as_deref(), modality.as_deref(), unreadable),
        tempo: Tempo::from_parts(marking, bpm),
        notes: column(row, "notes")?,
        tags: decode_json(&tags, "tags", &id, unreadable).unwrap_or_default(),
        created_at: time(row, "created_at")?,
        updated_at: time(row, "updated_at")?,
        priority: priority != 0,
        chord_chart: decode_chord_chart(chord_chart.as_deref(), &id, unreadable),
        photo_id: column(row, "photo_id")?,
        metre: decode_metre(metre.as_deref(), &id, unreadable),
        sections,
        variation_ids: variation_ids
            .and_then(|json| decode_json(&json, "variation_ids", &id, unreadable))
            .unwrap_or_default(),
        keys: decode_keys(keys.as_deref(), &id, unreadable),
        exercise_links,
        id,
    })
}

fn column<T: rusqlite::types::FromSql>(row: &Row, name: &str) -> Result<T, String> {
    row.get(name).map_err(|e| format!("{name}: {e}"))
}

fn time(row: &Row, name: &str) -> Result<DateTime<Utc>, String> {
    let text: String = column(row, name)?;
    parse_time(&text).ok_or_else(|| format!("{name} {text:?} is not a time"))
}

fn optional_time(row: &Row, name: &str) -> Result<Option<DateTime<Utc>>, String> {
    let text: Option<String> = column(row, name)?;
    text.map(|t| parse_time(&t).ok_or_else(|| format!("{name} {t:?} is not a time")))
        .transpose()
}

fn narrow<T: TryFrom<i64>>(value: Option<i64>, name: &str) -> Result<Option<T>, String> {
    value
        .map(|v| T::try_from(v).map_err(|_| format!("{name} {v} is out of range")))
        .transpose()
}

/// Tombstones included: the core reconciles (#2245).
fn sections_by_item(
    tx: &Transaction,
    unreadable: &mut Unreadable,
    partial: &mut HashSet<String>,
) -> Result<HashMap<String, Vec<ItemSection>>, StoreError> {
    let mut stmt = tx.prepare("SELECT * FROM section ORDER BY item_id, position, id")?;
    let mut rows = stmt.query([])?;
    let mut by_item: HashMap<String, Vec<ItemSection>> = HashMap::new();
    while let Some(row) = rows.next()? {
        let id: String = row.get("id")?;
        match section(row, unreadable) {
            Ok((item_id, section)) => by_item.entry(item_id).or_default().push(section),
            Err(reason) => {
                unreadable.push(format!("section {id} skipped: {reason}"));
                partial.extend(row.get::<_, String>("item_id").ok());
            }
        }
    }
    Ok(by_item)
}

fn section(row: &Row, unreadable: &mut Unreadable) -> Result<(String, ItemSection), String> {
    let first = narrow::<u16>(column(row, "bar_first")?, "bar_first")?;
    let last = narrow::<u16>(column(row, "bar_last")?, "bar_last")?;
    let position = narrow::<usize>(Some(column(row, "position")?), "position")?.unwrap_or(0);
    let section = ItemSection {
        id: column(row, "id")?,
        name: column(row, "name")?,
        bars: first
            .zip(last)
            .map(|(first, last)| BarRange { first, last }),
        kind: section_kind(&column::<String>(row, "kind")?, unreadable),
        target_bpm: narrow::<u16>(column(row, "target_bpm")?, "target_bpm")?,
        position,
        updated_at: time(row, "updated_at")?,
        deleted_at: optional_time(row, "deleted_at")?,
    };
    Ok((column(row, "item_id")?, section))
}

/// Tombstones included: the core reconciles (#2248).
fn links_by_piece(
    tx: &Transaction,
    unreadable: &mut Unreadable,
    partial: &mut HashSet<String>,
) -> Result<HashMap<String, Vec<ExerciseLink>>, StoreError> {
    let mut stmt = tx.prepare("SELECT * FROM exercise_link ORDER BY piece_id, position, id")?;
    let mut rows = stmt.query([])?;
    let mut by_piece: HashMap<String, Vec<ExerciseLink>> = HashMap::new();
    while let Some(row) = rows.next()? {
        let id: String = row.get("id")?;
        match exercise_link(row) {
            Ok((piece_id, link)) => by_piece.entry(piece_id).or_default().push(link),
            Err(reason) => {
                unreadable.push(format!("exercise link {id} skipped: {reason}"));
                partial.extend(row.get::<_, String>("piece_id").ok());
            }
        }
    }
    Ok(by_piece)
}

fn exercise_link(row: &Row) -> Result<(String, ExerciseLink), String> {
    let link = ExerciseLink {
        id: column(row, "id")?,
        exercise_id: column(row, "exercise_id")?,
        section_id: column(row, "section_id")?,
        position: narrow::<usize>(Some(column(row, "position")?), "position")?.unwrap_or(0),
        updated_at: time(row, "updated_at")?,
        deleted_at: optional_time(row, "deleted_at")?,
    };
    Ok((column(row, "piece_id")?, link))
}

// ── Saving ───────────────────────────────────────────────────────────

/// Insert or update by id; clears any tombstone (an upsert revives a row).
pub(crate) fn upsert(
    tx: &Transaction,
    item: &Item,
    unreadable: &mut Unreadable,
) -> Result<(), StoreError> {
    let chord_chart = match encode_chord_chart(
        item.chord_chart.as_ref(),
        stored_chart_key_if_unreadable(tx, &item.id, unreadable)?,
    )? {
        Some(json) => Some(json),
        None => stored_if_unreadable::<StoredChart>(tx, "chord_chart", &item.id)?,
    };
    let keys = match stored_keys_if_unreadable(tx, &item.id, &item.keys, unreadable)? {
        Some(json) => json,
        None => encode_keys(&item.keys)?,
    };
    let metre = match encode_metre(item.metre.as_ref())? {
        Some(json) => Some(json),
        None => stored_if_unreadable::<StoredMetre>(tx, "metre", &item.id)?,
    };
    let key = match &item.key {
        Some(key) => Some(stored_key(key)),
        None => stored_key_if_unreadable(tx, &item.id, unreadable)?,
    };
    tx.execute(
        &format!(
            "INSERT INTO item
               (id, title, kind, composer, key, modality, tempo_marking, tempo_bpm, notes, tags,
                created_at, updated_at, priority, chord_chart, photo_id,
                metre, variation_ids, keys, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                     ?17, ?18, NULL)
             ON CONFLICT(id) DO UPDATE SET
               title = excluded.title, kind = excluded.kind, composer = excluded.composer,
               key = excluded.key, modality = excluded.modality,
               tempo_marking = excluded.tempo_marking,
               tempo_bpm = excluded.tempo_bpm, notes = excluded.notes,
               tags = {tags},
               updated_at = excluded.updated_at, priority = excluded.priority,
               chord_chart = excluded.chord_chart, photo_id = excluded.photo_id,
               metre = excluded.metre,
               variation_ids = {variation_ids},
               keys = excluded.keys, deleted_at = NULL",
            tags = keeping_unreadable("tags"),
            variation_ids = keeping_unreadable("variation_ids"),
        ),
        params![
            item.id,
            item.title,
            item_kind_text(&item.kind),
            item.composer,
            key.as_ref().map(|k| &k.key),
            key.as_ref().and_then(|k| k.modality.as_ref()),
            item.tempo.as_ref().and_then(|t| t.marking.as_ref()),
            item.tempo.as_ref().and_then(|t| t.bpm),
            item.notes,
            encode_json(&item.tags)?,
            time_text(&item.created_at),
            time_text(&item.updated_at),
            item.priority,
            chord_chart,
            item.photo_id,
            metre,
            encode_json(&item.variation_ids)?,
            keys,
        ],
    )?;
    for s in &item.sections {
        tx.execute(
            "INSERT INTO section
               (id, item_id, name, bar_first, bar_last, kind, target_bpm, position, updated_at,
                deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
               item_id = excluded.item_id, name = excluded.name,
               bar_first = excluded.bar_first, bar_last = excluded.bar_last,
               kind = excluded.kind, target_bpm = excluded.target_bpm,
               position = excluded.position, updated_at = excluded.updated_at,
               deleted_at = excluded.deleted_at",
            params![
                s.id,
                item.id,
                s.name,
                s.bars.map(|b| b.first),
                s.bars.map(|b| b.last),
                section_kind_text(s.kind),
                s.target_bpm,
                position(s.position)?,
                time_text(&s.updated_at),
                s.deleted_at.as_ref().map(time_text),
            ],
        )?;
    }
    for l in &item.exercise_links {
        tx.execute(
            "INSERT INTO exercise_link
               (id, piece_id, exercise_id, section_id, position, updated_at, deleted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(id) DO UPDATE SET
               piece_id = excluded.piece_id, exercise_id = excluded.exercise_id,
               section_id = excluded.section_id, position = excluded.position,
               updated_at = excluded.updated_at, deleted_at = excluded.deleted_at",
            params![
                l.id,
                item.id,
                l.exercise_id,
                l.section_id,
                position(l.position)?,
                time_text(&l.updated_at),
                l.deleted_at.as_ref().map(time_text),
            ],
        )?;
    }
    Ok(())
}

fn position(position: usize) -> Result<i64, StoreError> {
    i64::try_from(position).map_err(|_| StoreError::OutOfRange("position"))
}

/// Soft delete: the core-stamped tombstone, so the deletion can win a later
/// last-write-wins sync.
pub(crate) fn delete(
    tx: &Transaction,
    id: &str,
    deleted_at: &DateTime<Utc>,
) -> Result<(), StoreError> {
    tx.execute(
        "UPDATE item SET deleted_at = ?1 WHERE id = ?2",
        params![time_text(deleted_at), id],
    )?;
    Ok(())
}

/// A list column that will not decode reads back empty (#1117), so writing
/// that empty list back would destroy the only copy; keep it until the list
/// really changes.
fn keeping_unreadable(column: &str) -> String {
    format!(
        "CASE WHEN excluded.{column} = '[]' AND NOT (
           json_valid(item.{column}) AND json_type(item.{column}) = 'array'
           AND NOT EXISTS (SELECT 1 FROM json_each(item.{column}) WHERE type <> 'text')
         ) THEN item.{column} ELSE excluded.{column} END"
    )
}

fn stored_text(tx: &Transaction, column: &str, id: &str) -> Result<Option<String>, StoreError> {
    Ok(tx
        .query_row(
            &format!("SELECT {column} FROM item WHERE id = ?1"),
            [id],
            |row| row.get::<_, Option<String>>(0),
        )
        .optional()?
        .flatten())
}

/// A value that will not decode reads back as none, so writing none back
/// would destroy the only copy (#2097); keep it until a real value replaces it.
fn stored_if_unreadable<T: DeserializeOwned>(
    tx: &Transaction,
    column: &str,
    id: &str,
) -> Result<Option<String>, StoreError> {
    Ok(stored_text(tx, column, id)?.filter(|stored| try_decode_json::<T>(stored).is_none()))
}

/// A key list holding one the core cannot read loads without it; while the
/// item's list is what loaded, the stored one stays so that key is not lost
/// (#2097, #2106).
fn stored_keys_if_unreadable(
    tx: &Transaction,
    id: &str,
    keys: &[Key],
    unreadable: &mut Unreadable,
) -> Result<Option<String>, StoreError> {
    let Some(stored) = stored_text(tx, "keys", id)? else {
        return Ok(None);
    };
    let Some(all) = try_decode_json::<Vec<StoredKeyJson>>(&stored) else {
        return Ok(keys.is_empty().then_some(stored));
    };
    let readable = decode_keys(Some(&stored), id, unreadable);
    Ok((readable.len() < all.len() && readable == keys).then_some(stored))
}

fn stored_chart_key_if_unreadable(
    tx: &Transaction,
    id: &str,
    unreadable: &mut Unreadable,
) -> Result<Option<StoredKeyJson>, StoreError> {
    let Some(chart) = stored_text(tx, "chord_chart", id)?
        .and_then(|stored| try_decode_json::<StoredChart>(&stored))
    else {
        return Ok(None);
    };
    let unread = !chart.key.is_empty()
        && codec::key(Some(&chart.key), Some(&chart.modality), unreadable).is_none();
    Ok(unread.then_some(StoredKeyJson {
        key: chart.key,
        modality: Some(chart.modality),
    }))
}

/// A key the core could not read loads as none, so writing none back would
/// lose the musician's text; it stays until they pick a key (#2106).
fn stored_key_if_unreadable(
    tx: &Transaction,
    id: &str,
    unreadable: &mut Unreadable,
) -> Result<Option<StoredKeyJson>, StoreError> {
    let Some((text, modality)) = tx
        .query_row(
            "SELECT key, modality FROM item WHERE id = ?1",
            [id],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, Option<String>>(1)?,
                ))
            },
        )
        .optional()?
    else {
        return Ok(None);
    };
    let Some(text) = text else { return Ok(None) };
    let unread = codec::key(Some(&text), modality.as_deref(), unreadable).is_none();
    Ok(unread.then_some(StoredKeyJson {
        key: text,
        modality,
    }))
}
