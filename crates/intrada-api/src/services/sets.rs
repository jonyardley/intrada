use libsql::Connection;

use intrada_core::domain::set::Set;
use intrada_core::validation;

use crate::db;
use crate::db::sets::{CreateSetEntry, CreateSetRequest, UpdateSetRequest};
use crate::error::ApiError;

fn validate_entries(entries: &[CreateSetEntry]) -> Result<(), ApiError> {
    for entry in entries {
        validation::validate_set_entry_fields(&entry.item_id, &entry.item_title)?;
    }
    Ok(())
}

pub async fn list_sets(conn: &Connection, user_id: &str) -> Result<Vec<Set>, ApiError> {
    db::sets::list_sets(conn, user_id).await
}

pub async fn get_set(conn: &Connection, id: &str, user_id: &str) -> Result<Set, ApiError> {
    db::sets::get_set(conn, id, user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Set not found: {id}")))
}

pub async fn create_set(
    conn: &Connection,
    user_id: &str,
    input: &CreateSetRequest,
) -> Result<Set, ApiError> {
    validation::validate_set_name(&input.name)?;
    validation::validate_entries_not_empty(&input.entries, "Set")?;
    validate_entries(&input.entries)?;
    db::sets::insert_set(conn, user_id, input).await
}

pub async fn update_set(
    conn: &Connection,
    id: &str,
    user_id: &str,
    input: &UpdateSetRequest,
) -> Result<Set, ApiError> {
    validation::validate_set_name(&input.name)?;
    validation::validate_entries_not_empty(&input.entries, "Set")?;
    validate_entries(&input.entries)?;
    db::sets::update_set(conn, id, user_id, input)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Set not found: {id}")))
}

pub async fn delete_set(conn: &Connection, id: &str, user_id: &str) -> Result<(), ApiError> {
    let deleted = db::sets::delete_set(conn, id, user_id).await?;
    if !deleted {
        return Err(ApiError::NotFound(format!("Set not found: {id}")));
    }
    Ok(())
}
