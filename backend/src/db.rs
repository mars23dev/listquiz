use std::path::{Path, PathBuf};

use rusqlite::Connection;

/// The project-local SQLite file. Constitution v2.1.1 requires all state to stay
/// inside the repository, never under `%APPDATA%`.
pub fn default_db_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("listquiz.db")
}

/// Schema DDL, one statement per table, from `data-model.md`. Idempotent, so it is
/// safe to run on every open and on the in-memory database the tests build.
pub const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS folder (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    password    TEXT NOT NULL,
    is_default  INTEGER NOT NULL DEFAULT 0,
    created_at  INTEGER NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_folder_name_non_default
    ON folder(name) WHERE is_default = 0;

CREATE TABLE IF NOT EXISTS quiz_list (
    id              TEXT PRIMARY KEY,
    folder_id       TEXT NOT NULL REFERENCES folder(id) ON DELETE CASCADE,
    title           TEXT NOT NULL,
    description     TEXT,
    order_direction TEXT NOT NULL,
    item_count      INTEGER NOT NULL,
    is_default      INTEGER NOT NULL DEFAULT 0,
    created_at      INTEGER NOT NULL,
    updated_at      INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_list_folder ON quiz_list(folder_id);

CREATE TABLE IF NOT EXISTS list_item_column (
    list_id  TEXT NOT NULL REFERENCES quiz_list(id) ON DELETE CASCADE,
    id       TEXT NOT NULL,
    name     TEXT NOT NULL,
    position INTEGER NOT NULL,
    PRIMARY KEY (list_id, id)
);

CREATE TABLE IF NOT EXISTS item (
    id         TEXT PRIMARY KEY,
    list_id    TEXT NOT NULL REFERENCES quiz_list(id) ON DELETE CASCADE,
    name       TEXT NOT NULL,
    position   INTEGER NOT NULL,
    icon_name  TEXT,
    icon_image BLOB
);
CREATE INDEX IF NOT EXISTS idx_item_list ON item(list_id, position);

CREATE TABLE IF NOT EXISTS item_value (
    item_id       TEXT NOT NULL REFERENCES item(id) ON DELETE CASCADE,
    column_id     TEXT NOT NULL,
    display       TEXT NOT NULL,
    numeric       REAL NOT NULL,
    was_defaulted INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (item_id, column_id)
);

CREATE TABLE IF NOT EXISTS game_session (
    id             TEXT PRIMARY KEY,
    gm_password    TEXT NOT NULL,
    team_count     INTEGER NOT NULL,
    round_count    INTEGER NOT NULL,
    current_round  INTEGER NOT NULL DEFAULT 0,
    status         TEXT NOT NULL,
    muted          INTEGER NOT NULL DEFAULT 0,
    reveal_values  INTEGER NOT NULL DEFAULT 0,
    active_team_id TEXT,
    last_message   TEXT,
    revision       INTEGER NOT NULL DEFAULT 0,
    created_at     INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS team (
    id          TEXT NOT NULL,
    session_id  TEXT NOT NULL REFERENCES game_session(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    color       TEXT NOT NULL,
    primary_hex TEXT NOT NULL,
    accent_glow TEXT NOT NULL,
    score       INTEGER NOT NULL DEFAULT 0,
    hearts      INTEGER NOT NULL DEFAULT 3,
    turn_index  INTEGER NOT NULL,
    PRIMARY KEY (session_id, id)
);

CREATE TABLE IF NOT EXISTS round_config (
    session_id    TEXT NOT NULL REFERENCES game_session(id) ON DELETE CASCADE,
    round_number  INTEGER NOT NULL,
    quiz_type     TEXT NOT NULL,
    tripmine_side TEXT,
    tripmine_slots INTEGER,
    list_id       TEXT NOT NULL,
    column_id     TEXT NOT NULL,
    PRIMARY KEY (session_id, round_number)
);

CREATE TABLE IF NOT EXISTS order_state (
    session_id     TEXT PRIMARY KEY REFERENCES game_session(id) ON DELETE CASCADE,
    anchor_item_id TEXT NOT NULL,
    anchor_value   REAL NOT NULL,
    pool           TEXT NOT NULL,
    left_ids       TEXT NOT NULL,
    right_ids      TEXT NOT NULL,
    pending        TEXT
);

CREATE TABLE IF NOT EXISTS tripmine_state (
    session_id    TEXT PRIMARY KEY REFERENCES game_session(id) ON DELETE CASCADE,
    side          TEXT NOT NULL,
    searched_rank INTEGER NOT NULL,
    items         TEXT NOT NULL,
    hit           INTEGER NOT NULL DEFAULT 0,
    hit_by        TEXT,
    completed     INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS connect_state (
    session_id TEXT PRIMARY KEY REFERENCES game_session(id) ON DELETE CASCADE,
    names      TEXT NOT NULL,
    values_    TEXT NOT NULL,
    matched    TEXT NOT NULL,
    pending    TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS score_event (
    id           TEXT PRIMARY KEY,
    session_id   TEXT NOT NULL REFERENCES game_session(id) ON DELETE CASCADE,
    round_number INTEGER NOT NULL,
    team_id      TEXT NOT NULL,
    kind         TEXT NOT NULL,
    amount       INTEGER NOT NULL,
    reason       TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_score_event_session ON score_event(session_id, round_number);
"#;

/// Apply the schema. Safe to call repeatedly.
pub fn apply_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)?;
    add_order_state_placed_by(conn)?;
    add_turn_timer_columns(conn)?;
    add_tripmine_slots_column(conn)?;
    Ok(())
}

/// Additive migration (feature 015): the Tripmine item count (10 or 20). Guarded so
/// an existing database upgrades in place and the step is safe on every open.
fn add_tripmine_slots_column(conn: &Connection) -> rusqlite::Result<()> {
    if !column_exists(conn, "round_config", "tripmine_slots")? {
        conn.execute_batch("ALTER TABLE round_config ADD COLUMN tripmine_slots INTEGER")?;
    }
    Ok(())
}

/// Additive migration (feature 005): the gamemaster turn timer's end time and
/// duration. Both are `NULL` together (no timer). Guarded so an existing database
/// upgrades in place and the step is safe to run on every open.
fn add_turn_timer_columns(conn: &Connection) -> rusqlite::Result<()> {
    if !column_exists(conn, "game_session", "timer_ends_at")? {
        conn.execute_batch("ALTER TABLE game_session ADD COLUMN timer_ends_at INTEGER")?;
    }
    if !column_exists(conn, "game_session", "timer_total_seconds")? {
        conn.execute_batch("ALTER TABLE game_session ADD COLUMN timer_total_seconds INTEGER")?;
    }
    Ok(())
}

/// Additive migration (feature 004): `CREATE TABLE IF NOT EXISTS` never adds a
/// column to an existing database, so a store created before this feature must be
/// upgraded in place. The guard makes the step safe to run on every open.
fn add_order_state_placed_by(conn: &Connection) -> rusqlite::Result<()> {
    if column_exists(conn, "order_state", "placed_by")? {
        return Ok(());
    }
    conn.execute_batch("ALTER TABLE order_state ADD COLUMN placed_by TEXT")?;
    Ok(())
}

/// True when `table` has a column named `column`.
fn column_exists(conn: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    let mut statement = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        let name: String = row.get(1)?;
        if name == column {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Open the project-local database, creating its folder if absent.
pub fn open_db(path: &Path) -> rusqlite::Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|_| rusqlite::Error::InvalidPath(parent.to_path_buf()))?;
    }
    let conn = Connection::open(path)?;
    // WAL keeps a player view reading while the gamemaster writes.
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    apply_schema(&conn)?;
    Ok(conn)
}

/// The built-in folder is seeded, never created through the API (FR-050).
pub fn seed_default_folder(conn: &Connection) -> rusqlite::Result<()> {
    let now = now_ms();
    conn.execute(
        "INSERT OR IGNORE INTO folder (id, name, password, is_default, created_at)
         VALUES ('default', 'Default Lists', '', 1, ?1)",
        [now],
    )?;
    Ok(())
}

pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Insert a session in the `setup` (lobby) status. Session-scoped tests use this
/// instead of repeating the required columns, so a schema change lands in one place.
#[cfg(test)]
pub(crate) fn insert_test_session(conn: &Connection, id: &str, revision: i64) {
    conn.execute(
        "INSERT INTO game_session
            (id, gm_password, team_count, round_count, current_round, status, revision, created_at)
         VALUES (?1, 'letmein', 2, 3, 1, 'setup', ?2, 0)",
        rusqlite::params![id, revision],
    )
    .expect("insert test session");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_test_session_starts_in_setup_at_the_given_revision() {
        let conn = Connection::open_in_memory().unwrap();
        apply_schema(&conn).unwrap();

        insert_test_session(&conn, "s1", 7);

        let (status, revision): (String, i64) = conn
            .query_row(
                "SELECT status, revision FROM game_session WHERE id = 's1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(status, "setup");
        assert_eq!(revision, 7);
    }

    #[test]
    fn a_fresh_database_gets_the_placed_by_column() {
        let conn = Connection::open_in_memory().unwrap();
        apply_schema(&conn).unwrap();

        assert!(column_exists(&conn, "order_state", "placed_by").unwrap());
    }

    #[test]
    fn an_old_order_state_is_upgraded_in_place_without_losing_rows() {
        let conn = Connection::open_in_memory().unwrap();
        // The pre-feature table, with a row already on the bar.
        conn.execute_batch(
            "CREATE TABLE order_state (
                 session_id     TEXT PRIMARY KEY,
                 anchor_item_id TEXT NOT NULL,
                 anchor_value   REAL NOT NULL,
                 pool           TEXT NOT NULL,
                 left_ids       TEXT NOT NULL,
                 right_ids      TEXT NOT NULL,
                 pending        TEXT
             );
             INSERT INTO order_state
                 (session_id, anchor_item_id, anchor_value, pool, left_ids, right_ids, pending)
             VALUES ('s1', 'item_a', 5.0, '[]', '[\"item_b\"]', '[]', NULL);",
        )
        .unwrap();

        apply_schema(&conn).unwrap();
        // Running it again must be a no-op, not an error.
        apply_schema(&conn).unwrap();

        assert!(column_exists(&conn, "order_state", "placed_by").unwrap());
        let (left, placed_by): (String, Option<String>) = conn
            .query_row(
                "SELECT left_ids, placed_by FROM order_state WHERE session_id = 's1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(left, "[\"item_b\"]");
        assert_eq!(placed_by, None);
    }

    #[test]
    fn a_fresh_database_gets_the_timer_columns() {
        let conn = Connection::open_in_memory().unwrap();
        apply_schema(&conn).unwrap();

        assert!(column_exists(&conn, "game_session", "timer_ends_at").unwrap());
        assert!(column_exists(&conn, "game_session", "timer_total_seconds").unwrap());
    }

    #[test]
    fn an_old_game_session_is_upgraded_in_place_without_losing_rows() {
        let conn = Connection::open_in_memory().unwrap();
        // The pre-feature table, with a row already present.
        conn.execute_batch(
            "CREATE TABLE game_session (
                 id             TEXT PRIMARY KEY,
                 gm_password    TEXT NOT NULL,
                 team_count     INTEGER NOT NULL,
                 round_count    INTEGER NOT NULL,
                 current_round  INTEGER NOT NULL DEFAULT 0,
                 status         TEXT NOT NULL,
                 muted          INTEGER NOT NULL DEFAULT 0,
                 reveal_values  INTEGER NOT NULL DEFAULT 0,
                 active_team_id TEXT,
                 last_message   TEXT,
                 revision       INTEGER NOT NULL DEFAULT 0,
                 created_at     INTEGER NOT NULL
             );
             INSERT INTO game_session
                 (id, gm_password, team_count, round_count, current_round, status, revision, created_at)
             VALUES ('s1', 'letmein', 2, 3, 1, 'setup', 4, 0);",
        )
        .unwrap();

        apply_schema(&conn).unwrap();
        // Running it again must be a no-op, not an error.
        apply_schema(&conn).unwrap();

        let (status, ends_at, total): (String, Option<i64>, Option<i64>) = conn
            .query_row(
                "SELECT status, timer_ends_at, timer_total_seconds
                 FROM game_session WHERE id = 's1'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(status, "setup");
        assert_eq!(ends_at, None);
        assert_eq!(total, None);
    }
}
