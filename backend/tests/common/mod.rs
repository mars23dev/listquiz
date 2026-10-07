#![allow(dead_code)]
//! Test-only helpers shared by the integration tests in this directory.

use axum::Router;
use rusqlite::Connection;
use std::path::PathBuf;

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

/// A shared in-memory database, handed out as two connections. A test that needs
/// to see what the server wrote (or to break the session behind its back) keeps one
/// side and gives the other to the router.
pub fn shared_db() -> (Connection, Connection) {
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let name = format!(
        "file:lq-test-{}-{}?mode=memory&cache=shared",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    );
    let server = Connection::open(&name).expect("shared in-memory database");
    listquiz::db::apply_schema(&server).expect("schema");
    let test_side = Connection::open(&name).expect("second connection");
    (server, test_side)
}

/// `shared_db` with the default folder and its three seeded lists.
pub fn seeded_shared_db() -> (Connection, Connection) {
    let (server, test_side) = shared_db();
    listquiz::db::seed_default_folder(&server).expect("default folder");
    listquiz::modules::default_lists::seed_default_lists(&server, "default")
        .expect("default lists");
    (server, test_side)
}

/// The application router over an in-memory database. The `dist_dir` points at
/// nothing, so any accidental frontend fallback surfaces as a test failure
/// instead of quietly serving a stale build.
pub fn test_app(conn: Connection) -> Router {
    listquiz::routes::api::router(listquiz::app_state::AppState::new(
        conn,
        PathBuf::from("no-frontend-here"),
    ))
}

/// A seeded list with a usable value column, so session tests build rounds from
/// real data instead of inventing ids.
pub fn default_list_with_column(conn: &Connection) -> (String, String) {
    let (list_id, column_id): (String, String) = conn
        .query_row(
            "SELECT l.id, c.id FROM quiz_list l
             JOIN list_item_column c ON c.list_id = l.id
             WHERE l.id = 'default_world_history' AND c.id <> 'name'
             ORDER BY c.position
             LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("seeded history list and its value column");
    (list_id, column_id)
}

/// The body of a `POST /api/sessions` request that the tests share, so a single
/// field change does not touch every test.
pub fn valid_session_body() -> serde_json::Value {
    serde_json::json!({
        "gmPassword": "letmein",
        "teamCount": 2,
        "teams": [{ "name": "Team Cyan Pulsar" }, { "name": "Team Ruby Spark" }],
        "roundCount": 2,
        "rounds": [
            { "roundNumber": 1, "quizType": "ORDER",    "listId": "default_world_history", "columnId": "year" },
            { "roundNumber": 2, "quizType": "TRIPMINE", "listId": "default_world_history", "columnId": "year",
              "tripmineSide": "highest" }
        ]
    })
}

/// Call the API and read the JSON answer. One helper for every integration test
/// so status codes and error shapes are read the same way everywhere.
pub async fn call(
    app: &Router,
    request: axum::http::Request<axum::body::Body>,
) -> (axum::http::StatusCode, serde_json::Value) {
    use tower::ServiceExt;

    let response = app.clone().oneshot(request).await.expect("response");
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let parsed = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, parsed)
}

pub fn json_request(
    method: &str,
    path: &str,
    body: Option<&serde_json::Value>,
) -> axum::http::Request<axum::body::Body> {
    use axum::body::Body;

    let mut builder = axum::http::Request::builder().method(method).uri(path);
    if let Some(body) = body {
        builder = builder.header("Content-Type", "application/json");
        return builder.body(Body::from(body.to_string())).unwrap();
    }
    builder.body(Body::empty()).unwrap()
}

/// An authenticated session action request. Test sessions use `letmein` from
/// `valid_session_body`, matching the REST contract's `X-Quiz-Password` header.
pub fn gm_action_request(
    path: &str,
    body: &serde_json::Value,
) -> axum::http::Request<axum::body::Body> {
    let mut request = json_request("POST", path, Some(body));
    request.headers_mut().insert(
        "X-Quiz-Password",
        axum::http::HeaderValue::from_static("letmein"),
    );
    request
}

/// A request that carries a folder/list password in the `X-Quiz-Password` header.
pub fn password_request(
    method: &str,
    path: &str,
    body: Option<&serde_json::Value>,
    password: &str,
) -> axum::http::Request<axum::body::Body> {
    let mut request = json_request(method, path, body);
    request.headers_mut().insert(
        "X-Quiz-Password",
        axum::http::HeaderValue::from_str(password).expect("a valid header value"),
    );
    request
}

/// `GET /api/sessions/{id}/state?role=<role>`, the redacted view a role sees.
pub async fn get_state(
    app: &Router,
    session_id: &str,
    role: &str,
) -> (axum::http::StatusCode, serde_json::Value) {
    call(
        app,
        json_request(
            "GET",
            &format!("/api/sessions/{session_id}/state?role={role}"),
            None,
        ),
    )
    .await
}

/// The error code from a `{"error": {...}}` body.
pub fn error_code(body: &serde_json::Value) -> String {
    body["error"]["code"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

/// Call the API and read a non-JSON body (CSV export) with its content type.
pub async fn call_text(
    app: &Router,
    request: axum::http::Request<axum::body::Body>,
) -> (axum::http::StatusCode, String, Option<String>) {
    use tower::ServiceExt;

    let response = app.clone().oneshot(request).await.expect("response");
    let status = response.status();
    let content_type = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    (
        status,
        String::from_utf8_lossy(&bytes).to_string(),
        content_type,
    )
}
