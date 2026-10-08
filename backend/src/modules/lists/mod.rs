//! List rules and CRUD (US6, T115-T116, T120-T121).
//!
//! A list is a title, a direction, one or more value columns, and 11-100 items in
//! the gamemaster's own order (FR-052 to FR-056). Values live per column so a round
//! reads exactly one of them (FR-054).

use std::collections::HashMap;

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::errors::{ApiError, ApiResult};
use crate::modules::{csv_import, default_lists};

pub const LIST_ID_PREFIX: &str = "lst-";
pub const ITEM_ID_PREFIX: &str = "itm-";
const ID_BODY_CHARS: usize = 8;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListInput {
    /// Supplied when duplicating, so the copy can reuse a new id chosen by the client.
    #[serde(default)]
    pub id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub columns: Vec<ColumnInput>,
    #[serde(default)]
    pub items: Vec<ItemInput>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnInput {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemInput {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub icon_name: Option<String>,
    #[serde(default)]
    pub icon_image: Option<String>,
    #[serde(default)]
    pub position: Option<i64>,
    #[serde(default)]
    pub values: HashMap<String, ValueInput>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueInput {
    pub display: String,
    #[serde(default)]
    pub numeric: Option<f64>,
    #[serde(default)]
    pub was_defaulted: Option<bool>,
}

/// A column ready to store.
#[derive(Debug, Clone, PartialEq)]
pub struct NewColumn {
    pub id: String,
    pub name: String,
}

/// A value ready to store.
#[derive(Debug, Clone, PartialEq)]
pub struct NewValue {
    pub column_id: String,
    pub display: String,
    pub numeric: f64,
    pub was_defaulted: bool,
}

/// An item ready to store, with its position exactly as the gamemaster set it.
#[derive(Debug, Clone, PartialEq)]
pub struct NewItem {
    pub id: String,
    pub name: String,
    pub icon_name: Option<String>,
    pub icon_image: Option<String>,
    pub position: i64,
    pub values: Vec<NewValue>,
}

/// A validated list, independent of transport.
#[derive(Debug, Clone, PartialEq)]
pub struct NewList {
    pub id: String,
    pub title: String,
    pub description: String,
    pub columns: Vec<NewColumn>,
    pub items: Vec<NewItem>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ColumnView {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueView {
    pub display: String,
    pub numeric: f64,
    pub was_defaulted: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemView {
    pub id: String,
    pub name: String,
    pub icon_name: Option<String>,
    pub icon_image: Option<String>,
    pub position: i64,
    pub values: HashMap<String, ValueView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListDetail {
    pub id: String,
    pub folder_id: String,
    pub title: String,
    pub description: String,
    pub columns: Vec<ColumnView>,
    pub item_count: i64,
    pub is_default: bool,
    pub items: Vec<ItemView>,
}

fn random_id(prefix: &str) -> String {
    use rand::Rng;
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    let body: String = (0..ID_BODY_CHARS)
        .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
        .collect();
    format!("{prefix}{body}")
}

fn valid_id(id: &str, prefix: &str) -> bool {
    match id.strip_prefix(prefix) {
        Some(body) => {
            body.len() == ID_BODY_CHARS
                && body
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        }
        None => false,
    }
}

/// Turn a `ListInput` into a validated `NewList` (T120).
pub fn build_new(input: &ListInput) -> ApiResult<NewList> {
    default_lists::validate_list_title(&input.title)?;

    if input.columns.is_empty() {
        return Err(ApiError::bad_request(
            "A list needs at least one value column.",
        ));
    }
    let mut columns = Vec::with_capacity(input.columns.len());
    let mut column_ids: Vec<String> = Vec::new();
    for column in &input.columns {
        default_lists::validate_column_name(&column.name)?;
        let id = match &column.id {
            Some(id) if !id.trim().is_empty() => id.trim().to_string(),
            _ => csv_import::slugify(&column.name),
        };
        if column_ids.contains(&id) {
            return Err(ApiError::bad_request(format!(
                "Column id '{id}' is used twice in one list."
            )));
        }
        column_ids.push(id.clone());
        columns.push(NewColumn {
            id,
            name: column.name.trim().to_string(),
        });
    }

    default_lists::validate_list_size(input.items.len())?;

    let mut items = Vec::with_capacity(input.items.len());
    let mut item_ids: Vec<String> = Vec::new();
    for (index, item) in input.items.iter().enumerate() {
        let name = item.name.trim();
        if name.is_empty() {
            return Err(ApiError::bad_request("Every item needs a name."));
        }
        // FR-056: an item carries an icon name or an uploaded image, never both.
        if item.icon_name.is_some() && item.icon_image.is_some() {
            return Err(ApiError::bad_request(
                "An item cannot have both an icon name and an icon image.",
            ));
        }
        let id = match &item.id {
            Some(id) if valid_id(id, ITEM_ID_PREFIX) => id.clone(),
            _ => {
                let mut candidate = random_id(ITEM_ID_PREFIX);
                while item_ids.contains(&candidate) {
                    candidate = random_id(ITEM_ID_PREFIX);
                }
                candidate
            }
        };
        item_ids.push(id.clone());

        let position = item.position.unwrap_or(index as i64);
        let mut values = Vec::new();
        for column in &columns {
            let Some(value) = item.values.get(&column.id) else {
                continue;
            };
            let (numeric, was_defaulted) = match value.numeric {
                Some(numeric) => (numeric, value.was_defaulted.unwrap_or(false)),
                None => match csv_import::parse_number(&value.display) {
                    Some(numeric) => (numeric, false),
                    None => ((position as f64 + 1.0) * 10.0, true),
                },
            };
            values.push(NewValue {
                column_id: column.id.clone(),
                display: value.display.clone(),
                numeric,
                was_defaulted,
            });
        }

        items.push(NewItem {
            id,
            name: name.to_string(),
            icon_name: item.icon_name.clone(),
            icon_image: item.icon_image.clone(),
            position,
            values,
        });
    }

    let id = match &input.id {
        Some(id) if valid_id(id, LIST_ID_PREFIX) => id.clone(),
        _ => random_id(LIST_ID_PREFIX),
    };

    Ok(NewList {
        id,
        title: input.title.trim().to_string(),
        description: input.description.clone().unwrap_or_default(),
        columns,
        items,
    })
}

/// Build a `NewList` from a parsed CSV (T118).
pub fn build_from_import(parsed: &csv_import::ParsedList) -> ApiResult<NewList> {
    let columns: Vec<ColumnInput> = parsed
        .columns
        .iter()
        .map(|column| ColumnInput {
            id: Some(column.id.clone()),
            name: column.name.clone(),
        })
        .collect();
    let items: Vec<ItemInput> = parsed
        .items
        .iter()
        .map(|item| {
            let values = item
                .values
                .iter()
                .map(|value| {
                    (
                        value.column_id.clone(),
                        ValueInput {
                            display: value.display.clone(),
                            numeric: Some(value.numeric),
                            was_defaulted: Some(value.was_defaulted),
                        },
                    )
                })
                .collect();
            ItemInput {
                id: None,
                name: item.name.clone(),
                icon_name: item.icon_name.clone(),
                icon_image: None,
                position: Some(item.position),
                values,
            }
        })
        .collect();
    build_new(&ListInput {
        id: None,
        title: parsed.title.clone(),
        description: None,
        columns,
        items,
    })
}

/// The folder's `is_default` flag, or `not_found` when the folder is gone.
fn folder_is_default(conn: &Connection, folder_id: &str) -> ApiResult<bool> {
    conn.query_row(
        "SELECT is_default FROM folder WHERE id = ?1",
        params![folder_id],
        |row| row.get::<_, i64>(0),
    )
    .optional()?
    .map(|flag| flag != 0)
    .ok_or_else(|| ApiError::not_found(format!("No folder with id '{folder_id}'.")))
}

fn reject_default_list(conn: &Connection, list_id: &str) -> ApiResult<()> {
    let is_default: Option<i64> = conn
        .query_row(
            "SELECT is_default FROM quiz_list WHERE id = ?1",
            params![list_id],
            |row| row.get(0),
        )
        .optional()?;
    match is_default {
        None => Err(ApiError::not_found(format!("No list with id '{list_id}'."))),
        Some(flag) if flag != 0 => Err(ApiError::conflict("The default folder is read-only.")),
        Some(_) => Ok(()),
    }
}

fn persist(conn: &Connection, folder_id: &str, list: &NewList, is_default: bool) -> ApiResult<()> {
    let now = crate::db::now_ms();
    conn.execute(
        "INSERT INTO quiz_list
            (id, folder_id, title, description, item_count, is_default, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)",
        params![
            list.id,
            folder_id,
            list.title,
            list.description,
            list.items.len() as i64,
            i64::from(is_default),
            now,
        ],
    )?;
    for (position, column) in list.columns.iter().enumerate() {
        conn.execute(
            "INSERT INTO list_item_column (list_id, id, name, position) VALUES (?1, ?2, ?3, ?4)",
            params![list.id, column.id, column.name, position as i64],
        )?;
    }
    for item in &list.items {
        conn.execute(
            "INSERT INTO item (id, list_id, name, position, icon_name, icon_image)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                item.id,
                list.id,
                item.name,
                item.position,
                item.icon_name,
                item.icon_image,
            ],
        )?;
        for value in &item.values {
            conn.execute(
                "INSERT INTO item_value (item_id, column_id, display, numeric, was_defaulted)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    item.id,
                    value.column_id,
                    value.display,
                    value.numeric,
                    i64::from(value.was_defaulted),
                ],
            )?;
        }
    }
    Ok(())
}

/// Create a list in a non-default folder (T121).
pub fn create(conn: &Connection, folder_id: &str, input: &ListInput) -> ApiResult<ListDetail> {
    if folder_is_default(conn, folder_id)? {
        return Err(ApiError::conflict("The default folder is read-only."));
    }
    let list = build_new(input)?;
    let tx = conn.unchecked_transaction()?;
    persist(&tx, folder_id, &list, false)?;
    tx.commit()?;
    read(conn, &list.id)
}

/// Create a list from a CSV import (T121).
pub fn create_from_import(
    conn: &Connection,
    folder_id: &str,
    parsed: &csv_import::ParsedList,
) -> ApiResult<ListDetail> {
    if folder_is_default(conn, folder_id)? {
        return Err(ApiError::conflict("The default folder is read-only."));
    }
    let list = build_from_import(parsed)?;
    let tx = conn.unchecked_transaction()?;
    persist(&tx, folder_id, &list, false)?;
    tx.commit()?;
    read(conn, &list.id)
}

/// Replace a list's editable fields; the folder and id never change (T121).
pub fn update(conn: &Connection, list_id: &str, input: &ListInput) -> ApiResult<ListDetail> {
    reject_default_list(conn, list_id)?;
    let folder_id: String = conn.query_row(
        "SELECT folder_id FROM quiz_list WHERE id = ?1",
        params![list_id],
        |row| row.get(0),
    )?;
    let mut list = build_new(input)?;
    // The stored id wins: an update never renames the row it targets.
    list.id = list_id.to_string();

    let tx = conn.unchecked_transaction()?;
    // The cascade removes this list's columns, items, and values in one step.
    tx.execute("DELETE FROM quiz_list WHERE id = ?1", params![list_id])?;
    persist(&tx, &folder_id, &list, false)?;
    tx.commit()?;
    read(conn, list_id)
}

/// Delete a list from a non-default folder (T121).
pub fn delete(conn: &Connection, list_id: &str) -> ApiResult<()> {
    reject_default_list(conn, list_id)?;
    conn.execute("DELETE FROM quiz_list WHERE id = ?1", params![list_id])?;
    Ok(())
}

/// Read a list with its columns, items, and values (T121).
pub fn read(conn: &Connection, list_id: &str) -> ApiResult<ListDetail> {
    let row: Option<(String, String, Option<String>, i64, i64)> = conn
        .query_row(
            "SELECT folder_id, title, description, item_count, is_default
             FROM quiz_list WHERE id = ?1",
            params![list_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .optional()?;
    let Some((folder_id, title, description, item_count, is_default)) = row else {
        return Err(ApiError::not_found(format!("No list with id '{list_id}'.")));
    };

    let columns = read_columns(conn, list_id)?;
    let items = read_items(conn, list_id)?;

    Ok(ListDetail {
        id: list_id.to_string(),
        folder_id,
        title,
        description: description.unwrap_or_default(),
        columns,
        item_count,
        is_default: is_default != 0,
        items,
    })
}

fn read_columns(conn: &Connection, list_id: &str) -> ApiResult<Vec<ColumnView>> {
    let mut statement = conn.prepare(
        "SELECT id, name FROM list_item_column WHERE list_id = ?1 ORDER BY position ASC",
    )?;
    let rows = statement.query_map(params![list_id], |row| {
        Ok(ColumnView {
            id: row.get(0)?,
            name: row.get(1)?,
        })
    })?;
    Ok(rows.collect::<Result<Vec<_>, _>>()?)
}

fn read_items(conn: &Connection, list_id: &str) -> ApiResult<Vec<ItemView>> {
    let mut statement = conn.prepare(
        "SELECT id, name, icon_name, icon_image, position FROM item WHERE list_id = ?1 ORDER BY position ASC",
    )?;
    let base = statement
        .query_map(params![list_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut items = Vec::with_capacity(base.len());
    for (id, name, icon_name, icon_image, position) in base {
        let mut values = HashMap::new();
        let mut value_statement = conn.prepare(
            "SELECT column_id, display, numeric, was_defaulted FROM item_value WHERE item_id = ?1",
        )?;
        let rows = value_statement.query_map(params![id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                ValueView {
                    display: row.get(1)?,
                    numeric: row.get(2)?,
                    was_defaulted: row.get::<_, i64>(3)? != 0,
                },
            ))
        })?;
        for row in rows {
            let (column_id, value) = row?;
            values.insert(column_id, value);
        }
        items.push(ItemView {
            id,
            name,
            icon_name,
            icon_image,
            position,
            values,
        });
    }
    Ok(items)
}

#[cfg(test)]
mod tests;
