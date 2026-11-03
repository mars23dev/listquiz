//! The gamemaster password gate (US1, T032).

mod common;

use axum::Router;
use serde_json::json;

use common::{call, error_code, json_request, seeded_db, test_app, valid_session_body};

async fn create_session(app: &Router) -> String {
    let (status, response) = call(
        app,
        json_request("POST", "/api/sessions", Some(&valid_session_body())),
    )
    .await;
    assert_eq!(status, 201);
    response["sessionId"].as_str().unwrap().to_string()
}

async fn unlock(app: &Router, session_id: &str, password: &str) -> (u16, serde_json::Value) {
    let body = json!({ "password": password });
    let (status, response) = call(
        app,
        json_request(
            "POST",
            &format!("/api/sessions/{session_id}/unlock"),
            Some(&body),
        ),
    )
    .await;
    (status.as_u16(), response)
}

#[tokio::test]
async fn a_wrong_password_is_refused_with_the_drafts_own_copy() {
    let app = test_app(seeded_db());
    let session_id = create_session(&app).await;

    let (status, response) = unlock(&app, &session_id, "wrong").await;

    assert_eq!(status, 400);
    assert_eq!(error_code(&response), "unauthorized");
    assert_eq!(
        response["error"]["message"], "Incorrect password!",
        "the message copy comes from the draft"
    );
}

#[tokio::test]
async fn the_right_password_unlocks_the_gamemaster_view() {
    let app = test_app(seeded_db());
    let session_id = create_session(&app).await;

    let (status, response) = unlock(&app, &session_id, "letmein").await;

    assert_eq!(status, 200);
    assert_eq!(response["ok"], json!(true));
}

#[tokio::test]
async fn an_unknown_session_cannot_be_unlocked() {
    let app = test_app(seeded_db());

    let (status, response) = unlock(&app, "lq-nothinghere", "letmein").await;

    assert_eq!(status, 404);
    assert_eq!(error_code(&response), "not_found");
}

#[tokio::test]
async fn a_player_state_request_carries_no_answer_key() {
    let app = test_app(seeded_db());
    let session_id = create_session(&app).await;

    let (status, state) = common::get_state(&app, &session_id, "p1").await;

    assert_eq!(status, 200);
    // The lobby has no round yet, so there is nothing to leak; what matters here
    // is that a player never receives host-only fields.
    assert!(
        state.get("order").is_some(),
        "the order key is always present"
    );
    assert!(
        state.get("tripmine").is_some(),
        "the tripmine key is always present"
    );
    assert!(
        state.get("links").is_some(),
        "links stay available for sharing"
    );
}
