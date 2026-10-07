//! Cross-view sync over HTTP (T096-T100).
//!
//! One gamemaster acts; every role's next state read must reflect it, and a player
//! read must never grow a host control or an unrevealed value.

mod common;

use axum::Router;
use axum::http::StatusCode;
use serde_json::{Value, json};

use common::{
    call, error_code, get_state, gm_action_request, json_request, seeded_shared_db, test_app,
    valid_session_body,
};

/// A session with its order round already started.
async fn started_order_session() -> (Router, String) {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);

    let (status, created) = call(
        &app,
        json_request("POST", "/api/sessions", Some(&valid_session_body())),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "session created: {created}");
    let session_id = created["sessionId"]
        .as_str()
        .expect("a session id")
        .to_string();

    let (status, _) = call(
        &app,
        gm_action_request(
            &format!("/api/sessions/{session_id}/actions"),
            &json!({ "action": "next_round" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "round 1 started");
    (app, session_id)
}

async fn exact(app: &Router, session_id: &str, body: &Value) -> (StatusCode, Value) {
    call(
        app,
        gm_action_request(&format!("/api/sessions/{session_id}/actions"), body),
    )
    .await
}

// T096 ---------------------------------------------------------------------

#[tokio::test]
async fn a_gamemaster_action_is_visible_on_the_next_player_read() {
    let (app, session_id) = started_order_session().await;

    let (_, before) = get_state(&app, &session_id, "p1").await;
    assert_eq!(before["revealValues"], json!(false));

    let (status, _) = exact(&app, &session_id, &json!({ "action": "reveal_values" })).await;
    assert_eq!(status, StatusCode::OK);

    let (_, after) = get_state(&app, &session_id, "p1").await;
    assert_eq!(
        after["revealValues"],
        json!(true),
        "the player's next read carries the host's change"
    );
}

// T097 ---------------------------------------------------------------------

#[tokio::test]
async fn a_player_read_has_no_controls_and_no_unrevealed_values() {
    let (app, session_id) = started_order_session().await;

    let (_, player) = get_state(&app, &session_id, "p1").await;

    assert!(
        player.get("controls").is_none(),
        "a player view never carries a host control list"
    );
    for item in player["order"]["pool"].as_array().expect("pool") {
        assert!(
            item["value"].is_null(),
            "an unrevealed value must stay null for a player"
        );
    }
}

// T098 ---------------------------------------------------------------------

#[tokio::test]
async fn an_unknown_session_id_is_not_found_on_a_state_read() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);

    let (status, body) = call(
        &app,
        json_request("GET", "/api/sessions/lq-does-not-exist/state?role=p1", None),
    )
    .await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(error_code(&body), "not_found");
}

// T099 ---------------------------------------------------------------------

#[tokio::test]
async fn the_revision_grows_after_every_accepted_action() {
    let (app, session_id) = started_order_session().await;

    let (_, first) = get_state(&app, &session_id, "gamemaster").await;
    let before = first["revision"].as_i64().expect("a revision");

    let (status, _) = exact(&app, &session_id, &json!({ "action": "reveal_values" })).await;
    assert_eq!(status, StatusCode::OK);

    let (_, second) = get_state(&app, &session_id, "gamemaster").await;
    let after = second["revision"].as_i64().expect("a revision");
    assert!(after > before, "the revision must move forward");
}

// T100 ---------------------------------------------------------------------

#[tokio::test]
async fn set_muted_applies_to_every_role_state() {
    let (app, session_id) = started_order_session().await;

    let (status, _) = exact(
        &app,
        &session_id,
        &json!({ "action": "set_muted", "muted": true }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    for role in ["gamemaster", "p1", "p2", "view"] {
        let (_, state) = get_state(&app, &session_id, role).await;
        assert_eq!(state["muted"], json!(true), "{role} sees the mute");
    }
}
