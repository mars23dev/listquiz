//! Quick game over HTTP (feature 005, US5).

mod common;

use axum::Router;
use axum::http::StatusCode;
use rusqlite::Connection;
use serde_json::{Value, json};

use common::{call, error_code, json_request, test_app};

async fn quick(app: &Router, body: &Value) -> (StatusCode, Value) {
    call(
        &app.clone(),
        json_request("POST", "/api/sessions/quick", Some(body)),
    )
    .await
}

async fn state(app: &Router, session_id: &str, role: &str) -> Value {
    let (status, body) = call(
        app,
        json_request(
            "GET",
            &format!("/api/sessions/{session_id}/state?role={role}"),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "state for {role}: {body}");
    body
}

#[tokio::test]
async fn a_quick_order_game_starts_immediately_with_one_team() {
    let conn: Connection = common::seeded_db();
    let app = test_app(conn);

    let (status, created) = quick(&app, &json!({ "gameType": "ORDER" })).await;
    assert_eq!(status, StatusCode::CREATED, "created: {created}");
    let session_id = created["sessionId"].as_str().expect("an id").to_string();

    let body = state(&app, &session_id, "p1").await;
    assert_eq!(
        body["status"],
        json!("running"),
        "the round is already running"
    );
    assert_eq!(body["teamCount"], json!(1));
    assert_eq!(body["round"]["quizType"], json!("ORDER"));
    assert!(!body["order"].is_null(), "the board exists at once");
}

#[tokio::test]
async fn a_quick_tripmine_game_starts_immediately() {
    let conn: Connection = common::seeded_db();
    let app = test_app(conn);

    let (status, created) = quick(&app, &json!({ "gameType": "TRIPMINE" })).await;
    assert_eq!(status, StatusCode::CREATED, "created: {created}");
    let session_id = created["sessionId"].as_str().expect("an id").to_string();

    let body = state(&app, &session_id, "p1").await;
    assert_eq!(body["status"], json!("running"));
    assert_eq!(body["round"]["quizType"], json!("TRIPMINE"));
    assert!(!body["tripmine"].is_null(), "the grid exists at once");
}

#[tokio::test]
async fn an_unknown_quick_game_type_is_rejected() {
    let conn: Connection = common::seeded_db();
    let app = test_app(conn);

    let (status, body) = quick(&app, &json!({ "gameType": "SNAKES" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&body), "bad_request");
}
