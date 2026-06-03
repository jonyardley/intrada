use libsql::Connection;

use intrada_core::domain::item::Item;
use intrada_core::domain::types::{CreateItem, UpdateItem};
use intrada_core::validation;

use crate::db;
use crate::error::ApiError;

pub async fn list_items(conn: &Connection, user_id: &str) -> Result<Vec<Item>, ApiError> {
    db::items::list_items(conn, user_id).await
}

pub async fn get_item(conn: &Connection, id: &str, user_id: &str) -> Result<Item, ApiError> {
    db::items::get_item(conn, id, user_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Item not found: {id}")))
}

pub async fn create_item(
    conn: &Connection,
    user_id: &str,
    input: &CreateItem,
) -> Result<Item, ApiError> {
    let input = validation::normalize_create_item(input.clone());
    validation::validate_create_item(&input)?;
    db::items::insert_item(conn, user_id, &input).await
}

pub async fn update_item(
    conn: &Connection,
    id: &str,
    user_id: &str,
    input: &UpdateItem,
) -> Result<Item, ApiError> {
    let input = validation::normalize_update_item(input.clone());
    validation::validate_update_item(&input)?;
    db::items::update_item(conn, id, user_id, &input)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Item not found: {id}")))
}

pub async fn delete_item(conn: &Connection, id: &str, user_id: &str) -> Result<(), ApiError> {
    let deleted = db::items::delete_item(conn, id, user_id).await?;
    if !deleted {
        return Err(ApiError::NotFound(format!("Item not found: {id}")));
    }
    Ok(())
}
