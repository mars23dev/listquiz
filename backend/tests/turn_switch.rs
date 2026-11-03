//! The gamemaster turn switch must not put an eliminated team on turn (feature 013).

mod common;

use axum::Router;
use axum::http::StatusCode;
use serde_json::{Value, json};

use common::{
    call, gm_action_request, json_request, seeded_shared_db, test_app, valid_session_body,
};

async fn create_session(app: &Router, body: &Value) -> String {
    let (status, created) = call(app, json_request("POST", "/api/sessions", Some(body))).await;
    assert_eq!(status, StatusCode::CREATED, "session created: {created}");
    created["sessionId"]
        .as_str()
        .expect("a session id")
        .to_string()
}

async fn action(app: &Router, session_id: &str, body: &Value) -> (StatusCode, Value) {
    call(
        app,
        gm_action_request(&format!("/api/sessions/{session_id}/actions"), body),
    )
    .await
}

async fn state(app: &Router, session_id: &str) -> Value {
    call(
        app,
        json_request(
            "GET",
            &format!("/api/sessions/{session_id}/state?role=gamemaster"),
            None,
        ),
    )
    .await
    .1
}

fn on_turn(state: &Value) -> String {
    state["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["onTurn"] == json!(true))
        .expect("a team on turn")["id"]
        .as_str()
        .expect("an id")
        .to_string()
}

#[tokio::test]
async fn the_turn_switch_refuses_a_team_with_no_hearts_in_order() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);
    let session_id = create_session(&app, &valid_session_body()).await;

    // Start round 1 (ORDER).
    let (status, body) = action(&app, &session_id, &json!({ "action": "next_round" })).await;
    assert_eq!(status, StatusCode::OK, "round 1 started: {body}");

    let active_before = on_turn(&state(&app, &session_id).await);

    // Take team_2 out of the round.
    let (status, body) = action(
        &app,
        &session_id,
        &json!({ "action": "adjust_hearts", "teamId": "team_2", "delta": -3 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "team_2 at zero hearts: {body}");

    // A switch to the eliminated team is refused and the turn stays put.
    let (status, body) = action(
        &app,
        &session_id,
        &json!({ "action": "set_turn", "teamId": "team_2" }),
    )
    .await;
    assert_ne!(
        status,
        StatusCode::OK,
        "an eliminated team must not be put on turn: {body}"
    );
    assert_eq!(
        on_turn(&state(&app, &session_id).await),
        active_before,
        "the active team is unchanged"
    );

    // A switch to a team that still has hearts works as before.
    let (status, body) = action(
        &app,
        &session_id,
        &json!({ "action": "set_turn", "teamId": "team_1" }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "a team with hearts switches: {body}"
    );
    assert_eq!(on_turn(&state(&app, &session_id).await), "team_1");
}

#[tokio::test]
async fn the_turn_switch_is_unrestricted_in_tripmine() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);

    // A single Tripmine round; Tripmine does not use hearts.
    let body = json!({
        "gmPassword": "letmein",
        "teamCount": 2,
        "teams": [{ "name": "Team Cyan Pulsar" }, { "name": "Team Ruby Spark" }],
        "roundCount": 1,
        "rounds": [
            { "roundNumber": 1, "quizType": "TRIPMINE", "listId": "default_world_history",
              "columnId": "year", "tripmineSide": "highest" }
        ]
    });
    let session_id = create_session(&app, &body).await;

    let (status, body) = action(&app, &session_id, &json!({ "action": "next_round" })).await;
    assert_eq!(status, StatusCode::OK, "tripmine round started: {body}");

    let (status, _) = action(
        &app,
        &session_id,
        &json!({ "action": "adjust_hearts", "teamId": "team_2", "delta": -3 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = action(
        &app,
        &session_id,
        &json!({ "action": "set_turn", "teamId": "team_2" }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "tripmine has no hearts, so every team is switchable: {body}"
    );
    assert_eq!(on_turn(&state(&app, &session_id).await), "team_2");
}
