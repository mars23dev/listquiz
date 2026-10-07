//! Deleting a finished session over HTTP (feature 005, US4).

mod common;

use axum::Router;
use axum::http::StatusCode;
use rusqlite::Connection;
use serde_json::{Value, json};

use common::{call, error_code, gm_action_request, json_request, test_app};

fn one_round_body() -> Value {
    json!({
        "gmPassword": "letmein",
        "teamCount": 2,
        "teams": [{ "name": "Team Cyan Pulsar" }, { "name": "Team Ruby Spark" }],
        "roundCount": 1,
        "rounds": [
            { "roundNumber": 1, "quizType": "ORDER",
              "listId": "default_world_history", "columnId": "year" }
        ]
    })
}

async fn create(app: &Router) -> String {
    let (status, created) = call(
        app,
        json_request("POST", "/api/sessions", Some(&one_round_body())),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "session created: {created}");
    created["sessionId"].as_str().expect("an id").to_string()
}

async fn next_round(app: &Router, session_id: &str) {
    let (status, body) = call(
        app,
        gm_action_request(
            &format!("/api/sessions/{session_id}/actions"),
            &json!({ "action": "next_round" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "next_round: {body}");
}

async fn delete(app: &Router, session_id: &str) -> StatusCode {
    call(
        app,
        json_request("DELETE", &format!("/api/sessions/{session_id}"), None),
    )
    .await
    .0
}

#[tokio::test]
async fn a_running_session_cannot_be_deleted() {
    let conn: Connection = common::seeded_db();
    let app = test_app(conn);
    let session_id = create(&app).await;
    next_round(&app, &session_id).await;

    let (status, body) = call(
        &app,
        json_request("DELETE", &format!("/api/sessions/{session_id}"), None),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "still running: {body}");
    assert_eq!(error_code(&body), "not_ready");
}

#[tokio::test]
async fn a_finished_session_is_deleted_and_gone() {
    let conn: Connection = common::seeded_db();
    let app = test_app(conn);
    let session_id = create(&app).await;
    next_round(&app, &session_id).await; // start round 1
    next_round(&app, &session_id).await; // finish the only round

    assert_eq!(delete(&app, &session_id).await, StatusCode::NO_CONTENT);

    let (status, _) = call(
        &app,
        json_request(
            "GET",
            &format!("/api/sessions/{session_id}/state?role=gamemaster"),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "the session is gone");

    // Idempotent.
    assert_eq!(delete(&app, &session_id).await, StatusCode::NO_CONTENT);
}
