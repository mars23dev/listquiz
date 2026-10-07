//! Turn timer actions over HTTP (feature 005, US2).

mod common;

use axum::Router;
use axum::http::StatusCode;
use rusqlite::Connection;
use serde_json::{Value, json};
use std::time::Duration;

use common::{call, error_code, gm_action_request, json_request, test_app};

async fn started_order_session() -> (Router, String) {
    let conn: Connection = common::seeded_db();
    let app = test_app(conn);

    let (status, created) = call(
        &app,
        json_request("POST", "/api/sessions", Some(&common::valid_session_body())),
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

async fn started_tripmine_session() -> (Router, String) {
    let conn: Connection = common::seeded_db();
    let app = test_app(conn);

    let body = json!({
        "gmPassword": "letmein",
        "teamCount": 2,
        "teams": [{ "name": "Team Cyan Pulsar" }, { "name": "Team Ruby Spark" }],
        "roundCount": 2,
        "rounds": [
            { "roundNumber": 1, "quizType": "TRIPMINE",
              "listId": "default_world_history", "columnId": "year", "tripmineSide": "highest" },
            { "roundNumber": 2, "quizType": "ORDER",
              "listId": "default_world_history", "columnId": "year" }
        ]
    });
    let (status, created) = call(&app, json_request("POST", "/api/sessions", Some(&body))).await;
    assert_eq!(status, StatusCode::CREATED, "session created: {created}");
    let session_id = created["sessionId"].as_str().expect("an id").to_string();

    let (status, _) = gm(&app, &session_id, &json!({ "action": "next_round" })).await;
    assert_eq!(status, StatusCode::OK, "round 1 started");
    (app, session_id)
}

async fn gm(app: &Router, session_id: &str, body: &Value) -> (StatusCode, Value) {
    call(
        app,
        gm_action_request(&format!("/api/sessions/{session_id}/actions"), body),
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

fn on_turn(body: &Value) -> String {
    body["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["onTurn"] == json!(true))
        .expect("a team on turn")["id"]
        .as_str()
        .expect("an id")
        .to_string()
}

fn hearts_of(body: &Value, team_id: &str) -> i64 {
    body["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["id"] == json!(team_id))
        .expect("the team")["hearts"]
        .as_i64()
        .expect("a number")
}

#[tokio::test]
async fn the_gamemaster_starts_and_stops_a_timer() {
    let (app, session_id) = started_order_session().await;

    let (status, body) = gm(
        &app,
        &session_id,
        &json!({ "action": "start_timer", "seconds": 20 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "started: {body}");

    let running = state(&app, &session_id, "gamemaster").await;
    assert_eq!(running["timer"]["totalSeconds"], json!(20));
    assert!(running["timer"]["secondsLeft"].as_u64().unwrap() <= 20);

    let (status, body) = gm(&app, &session_id, &json!({ "action": "stop_timer" })).await;
    assert_eq!(status, StatusCode::OK, "stopped: {body}");

    let after = state(&app, &session_id, "gamemaster").await;
    assert!(after["timer"].is_null(), "the timer is gone");
}

#[tokio::test]
async fn a_player_cannot_start_a_timer() {
    let (app, session_id) = started_order_session().await;
    let team = on_turn(&state(&app, &session_id, "gamemaster").await);

    let (status, body) = call(
        &app,
        json_request(
            "POST",
            &format!("/api/sessions/{session_id}/actions"),
            Some(&json!({ "action": "start_timer", "seconds": 5, "role": "p1", "teamId": team })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&body), "bad_request");
}

#[tokio::test]
async fn a_time_out_before_the_end_is_refused() {
    let (app, session_id) = started_order_session().await;
    let team = on_turn(&state(&app, &session_id, "gamemaster").await);
    let (status, _) = gm(
        &app,
        &session_id,
        &json!({ "action": "start_timer", "seconds": 60 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = gm(
        &app,
        &session_id,
        &json!({ "action": "time_out", "teamId": team }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "not ready yet: {body}");
    assert_eq!(error_code(&body), "not_ready");
}

#[tokio::test]
async fn a_short_timer_ends_the_turn_on_its_own() {
    let (app, session_id) = started_order_session().await;
    let before = state(&app, &session_id, "gamemaster").await;
    let team = on_turn(&before);
    let hearts_before = hearts_of(&before, &team);

    let (status, _) = gm(
        &app,
        &session_id,
        &json!({ "action": "start_timer", "seconds": 1 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // No `time_out` is posted: the server's scheduled deadline ends the turn.
    tokio::time::sleep(Duration::from_millis(1_400)).await;

    let after = state(&app, &session_id, "gamemaster").await;
    assert_eq!(
        hearts_of(&after, &team),
        hearts_before - 1,
        "the scheduled timeout cost one heart"
    );
    assert!(after["timer"].is_null(), "the timer clears");
    assert_ne!(on_turn(&after), team, "the turn passed on its own");
}

#[tokio::test]
async fn ending_the_turn_another_way_clears_the_timer() {
    let (app, session_id) = started_order_session().await;
    let (status, _) = gm(
        &app,
        &session_id,
        &json!({ "action": "start_timer", "seconds": 30 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, _) = gm(&app, &session_id, &json!({ "action": "pass_turn" })).await;
    assert_eq!(status, StatusCode::OK);

    let after = state(&app, &session_id, "gamemaster").await;
    assert!(
        after["timer"].is_null(),
        "passing the turn clears the timer"
    );
}

#[tokio::test]
async fn a_tripmine_timeout_passes_the_turn_without_a_heart() {
    let (app, session_id) = started_tripmine_session().await;
    let before = state(&app, &session_id, "gamemaster").await;
    let team = on_turn(&before);
    let hearts_before = hearts_of(&before, &team);

    let (status, _) = gm(
        &app,
        &session_id,
        &json!({ "action": "start_timer", "seconds": 1 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // The server's scheduled deadline ends the turn; no `time_out` is posted.
    tokio::time::sleep(Duration::from_millis(1_400)).await;

    let after = state(&app, &session_id, "gamemaster").await;
    assert_eq!(
        hearts_of(&after, &team),
        hearts_before,
        "a Tripmine timeout costs no heart"
    );
    assert_ne!(on_turn(&after), team, "the turn passes on");
    assert!(after["timer"].is_null(), "the timer clears");
}
