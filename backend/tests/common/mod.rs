//! Test-only helpers shared by the integration tests in this directory.

use rusqlite::Connection;

/// A fresh in-memory database with the schema applied. Nothing is written to disk,
/// so tests never touch `backend/data/listquiz.db`.
pub fn test_db() -> Connection {
    let conn = Connection::open_in_memory().expect("in-memory database");
    listquiz::db::apply_schema(&conn).expect("schema");
    conn
}

/// A test database that also holds the default folder and its three seeded lists.
pub fn seeded_db() -> Connection {
    let conn = test_db();
    listquiz::db::seed_default_folder(&conn).expect("default folder");
    listquiz::modules::default_lists::seed_default_lists(&conn, "default").expect("default lists");
    conn
}