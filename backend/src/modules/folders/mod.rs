use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

use crate::errors::{ApiError, ApiResult};

pub const DEFAULT_FOLDER_ID: &str = "default";
pub const MAX_FOLDER_NAME_CHARS: usize = 60;
pub const FOLDER_ID_PREFIX: &str = "fld-";
pub const FOLDER_ID_BODY_CHARS: usize = 8;

/// How many ids to try before giving up on a collision.
const ID_ATTEMPTS: usize = 5;

/// What `GET /api/folders` reports for one folder.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderView {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    /// The caller's own state for this folder. With no credentials presented every
    /// protected folder reads as locked, so the answer never reveals whether a
    /// locked folder holds anything (FR-047).
    pub locked: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFolderRequest {
    pub name: String,
    pub password: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockFolderRequest {
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameFolderRequest {
    pub name: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteFolderRequest {
    #[serde(default)]
    pub confirm_password: String,
}

/// A folder id: `fld-` and eight lowercase alphanumerics.
pub fn generate_folder_id() -> String {
    use rand::Rng;
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    let body: String = (0..FOLDER_ID_BODY_CHARS)
        .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
        .collect();
    format!("{FOLDER_ID_PREFIX}{body}")
}

pub fn is_valid_folder_id(id: &str) -> bool {
    match id.strip_prefix(FOLDER_ID_PREFIX) {
        Some(body) => {
            body.len() == FOLDER_ID_BODY_CHARS
                && body
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        }
        None => false,
    }
}

/// Trimmed, 1-60 characters (FR-046). Returns the name to store.
pub fn validate_folder_name(name: &str) -> ApiResult<String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(ApiError::bad_request("Folder name cannot be empty."));
    }
    let len = trimmed.chars().count();
    if len > MAX_FOLDER_NAME_CHARS {
        return Err(ApiError::bad_request(format!(
            "Folder name cannot exceed {MAX_FOLDER_NAME_CHARS} characters."
        )));
    }
    Ok(trimmed.to_string())
}

/// Non-empty for every folder except the built-in one, which needs no password
/// at all (FR-046, FR-050). Returns the password to store.
pub fn validate_folder_password(password: &str, folder_id: &str) -> ApiResult<String> {
    if folder_id == DEFAULT_FOLDER_ID {
        return Ok(String::new());
    }
    // Whitespace is not a password: an all-blank one would read as locked while
    // being typeable by accident.
    if password.trim().is_empty() {
        return Err(ApiError::bad_request("Folder password cannot be empty."));
    }
    Ok(password.to_string())
}

/// The stored password for `folder_id`, or `not_found` when no such folder
/// exists. Never returns the password itself to callers.
fn stored_password(conn: &Connection, folder_id: &str) -> ApiResult<String> {
    conn.query_row(
        "SELECT password FROM folder WHERE id = ?1",
        params![folder_id],
        |row| row.get::<_, String>(0),
    )
    .optional()?
    .ok_or_else(|| ApiError::not_found(format!("No folder with id '{folder_id}'.")))
}

/// The default folder is the one folder that needs no password; everything else
/// must present its own.
pub fn password_matches(conn: &Connection, folder_id: &str, attempt: &str) -> ApiResult<bool> {
    let stored = stored_password(conn, folder_id)?;
    if stored.is_empty() {
        return Ok(true);
    }
    Ok(stored == attempt)
}

/// Create a protected folder and return its id.
pub fn create_folder(conn: &Connection, request: &CreateFolderRequest) -> ApiResult<String> {
    let name = validate_folder_name(&request.name)?;
    // The id decides whether a password is required, so it is drawn first.
    let mut id = String::new();
    for _ in 0..ID_ATTEMPTS {
        let candidate = generate_folder_id();
        let taken: Option<i64> = conn
            .query_row(
                "SELECT 1 FROM folder WHERE id = ?1",
                params![candidate],
                |row| row.get(0),
            )
            .optional()?;
        if taken.is_none() {
            id = candidate;
            break;
        }
    }
    if id.is_empty() {
        return Err(ApiError::conflict(
            "Could not allocate a free folder id; try again.",
        ));
    }
    let password = validate_folder_password(&request.password, &id)?;
    if name_exists(conn, &name)? {
        return Err(ApiError::conflict(format!(
            "A folder named '{name}' already exists."
        )));
    }
    conn.execute(
        "INSERT INTO folder (id, name, password, is_default, created_at)
         VALUES (?1, ?2, ?3, 0, ?4)",
        params![id, name, password, now_millis()],
    )?;
    Ok(id)
}

fn name_exists(conn: &Connection, name: &str) -> ApiResult<bool> {
    let found: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM folder WHERE name = ?1 AND is_default = 0",
            params![name],
            |row| row.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}

/// Every folder, default first then the rest alphabetically. `locked` is true for
/// every protected folder, because no credentials travel with the request.
pub fn list_folders(conn: &Connection) -> ApiResult<Vec<FolderView>> {
    let mut statement = conn.prepare(
        "SELECT id, name, is_default FROM folder
         ORDER BY is_default DESC, name COLLATE NOCASE ASC",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(FolderView {
            id: row.get(0)?,
            name: row.get(1)?,
            is_default: row.get::<_, i64>(2)? != 0,
            locked: row.get::<_, i64>(2)? == 0,
        })
    })?;
    let folders = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)?;
    Ok(folders)
}

/// Confirm a password without changing anything. The answer is only ever the
/// folder name, so unlocking cannot become a channel for list data.
pub fn unlock(
    conn: &Connection,
    folder_id: &str,
    request: &UnlockFolderRequest,
) -> ApiResult<String> {
    if !password_matches(conn, folder_id, &request.password)? {
        return Err(ApiError::unauthorized());
    }
    let name: String = conn
        .query_row(
            "SELECT name FROM folder WHERE id = ?1",
            params![folder_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| ApiError::not_found(format!("No folder with id '{folder_id}'.")))?;
    Ok(name)
}

/// Rename a folder. The default folder and any taken name both conflict (FR-046b).
pub fn rename(
    conn: &Connection,
    folder_id: &str,
    attempt: &str,
    request: &RenameFolderRequest,
) -> ApiResult<FolderView> {
    let is_default = reject_default_write(conn, folder_id)?;
    if !password_matches(conn, folder_id, attempt)? {
        return Err(ApiError::unauthorized());
    }
    let name = validate_folder_name(&request.name)?;
    if name_exists(conn, &name)? {
        return Err(ApiError::conflict(format!(
            "A folder named '{name}' already exists."
        )));
    }
    conn.execute(
        "UPDATE folder SET name = ?2 WHERE id = ?1",
        params![folder_id, name],
    )?;
    Ok(FolderView {
        id: folder_id.to_string(),
        name,
        is_default,
        locked: false,
    })
}

/// How many lists a folder holds, so the UI can name the cost of a delete
/// before accepting it (FR-046c).
pub fn list_count(conn: &Connection, folder_id: &str) -> ApiResult<usize> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM quiz_list WHERE folder_id = ?1",
        params![folder_id],
        |row| row.get(0),
    )?;
    Ok(count as usize)
}

/// Delete a folder and, through the schema's cascade, only the lists inside it
/// (FR-046d). The password has to be re-entered as confirmation.
pub fn delete(
    conn: &Connection,
    folder_id: &str,
    attempt: &str,
    request: &DeleteFolderRequest,
) -> ApiResult<()> {
    reject_default_write(conn, folder_id)?;
    if !password_matches(conn, folder_id, attempt)? {
        return Err(ApiError::unauthorized());
    }
    // The confirmation is checked against the stored password, not the header:
    // both must agree before anything is removed.
    if request.confirm_password != stored_password(conn, folder_id)? {
        return Err(ApiError::unauthorized());
    }
    conn.execute("DELETE FROM folder WHERE id = ?1", params![folder_id])?;
    Ok(())
}

/// The default folder accepts no writes at all (FR-049, FR-050). Returns
/// `is_default` for the caller's convenience.
fn reject_default_write(conn: &Connection, folder_id: &str) -> ApiResult<bool> {
    let is_default: Option<i64> = conn
        .query_row(
            "SELECT is_default FROM folder WHERE id = ?1",
            params![folder_id],
            |row| row.get(0),
        )
        .optional()?;
    match is_default {
        None => Err(ApiError::not_found(format!(
            "No folder with id '{folder_id}'."
        ))),
        Some(flag) if flag != 0 => Err(ApiError::conflict("The default folder is read-only.")),
        Some(_) => Ok(false),
    }
}

fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests;
