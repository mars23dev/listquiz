//! Starting a round (US1, T033).
//!
//! A round whose list has disappeared cannot start. The gamemaster is told
//! `not_ready` rather than being dropped onto a half-built board.

mod common;

use axum::Router;
use serde_json::{Value, json};

use common::{
    call, error_code, get_state, json_request, seeded_shared_db, test_app, valid_session_body,
};

async fn create(app: &Router, body: Value) -> (u16, Value) {
    let (status, response) = call(app, json_request("POST", "/api/sessions", Some(&body))).await;
    (status.as_u16(), response)
}

async fn create_default_session(app: &Router) -> String {
    let (status, response) = create(app, valid_session_body()).await;
    assert_eq!(status, 201);
    response["sessionId"].as_str().unwrap().to_string()
}

async fn act(app: &Router, session_id: &str, action: Value) -> (u16, Value) {
    let (status, response) = call(
        app,
        json_request(
            "POST",
            &format!("/api/sessions/{session_id}/actions"),
            Some(&action),
        ),
    )
    .await;
    (status.as_u16(), response)
}

#[tokio::test]
async fn a_ready_round_starts_when_the_gamemaster_advances() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);
    let session_id = create_default_session(&app).await;

    let (status, _) = act(&app, &session_id, json!({ "action": "next_round" })).await;

    assert_eq!(status, 200);
    let (_, state) = get_state(&app, &session_id, "gamemaster").await;
    assert_eq!(state["currentRound"], json!(1));
    assert_eq!(state["round"]["roundNumber"], json!(1));
}

#[tokio::test]
async fn a_round_whose_list_is_gone_cannot_start() {
    let (server_db, side) = seeded_shared_db();
    let app = test_app(server_db);
    let session_id = create_default_session(&app).await;

    // The gamemaster deletes the list behind the session's back; the round row
    // survives because it holds a plain list id, and the round is now unplayable.
    side.execute(
        "DELETE FROM quiz_list WHERE id = 'default_world_history'",
        [],
    )
    .expect("delete the referenced list");

    let (status, body) = act(&app, &session_id, json!({ "action": "next_round" })).await;

    assert_eq!(status, 409);
    assert_eq!(error_code(&body), "not_ready");
}

#[tokio::test]
async fn acting_on_an_unknown_session_is_not_found() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);

    let (status, body) = act(&app, "lq-nothinghere", json!({ "action": "next_round" })).await;

    assert_eq!(status, 404);
    assert_eq!(error_code(&body), "not_found");
}
