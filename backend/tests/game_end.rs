//! Finishing the game when the last round ends (feature 007).

mod common;

use axum::Router;
use axum::http::StatusCode;
use rusqlite::Connection;
use serde_json::{Value, json};

use common::{call, gm_action_request, json_request, test_app};

fn body(rounds: i32, quiz_type: &str) -> Value {
    let mut list = Vec::new();
    for n in 1..=rounds {
        list.push(json!({
            "roundNumber": n,
            "quizType": quiz_type,
            "listId": "default_world_history",
            "columnId": "year",
            "tripmineSide": "highest"
        }));
    }
    json!({
        "gmPassword": "letmein",
        "teamCount": 2,
        "teams": [{ "name": "Team Cyan Pulsar" }, { "name": "Team Ruby Spark" }],
        "roundCount": rounds,
        "rounds": list
    })
}

async fn create(app: &Router, rounds: i32) -> String {
    let (status, created) = call(
        app,
        json_request("POST", "/api/sessions", Some(&body(rounds, "TRIPMINE"))),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "session created: {created}");
    created["sessionId"].as_str().expect("an id").to_string()
}

async fn gm(app: &Router, session_id: &str, body: &Value) -> (StatusCode, Value) {
    call(
        app,
        gm_action_request(&format!("/api/sessions/{session_id}/actions"), body),
    )
    .await
}

async fn state(app: &Router, session_id: &str) -> Value {
    let (status, body) = call(
        app,
        json_request(
            "GET",
            &format!("/api/sessions/{session_id}/state?role=gamemaster"),
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "state: {body}");
    body
}

/// Pick the mine so the round ends at once; returns the mine's id.
async fn hit_the_mine(app: &Router, session_id: &str) -> Value {
    let before = state(app, session_id).await;
    let team = before["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["onTurn"] == json!(true))
        .expect("a team on turn")["id"]
        .as_str()
        .expect("an id")
        .to_string();
    let mine = before["tripmine"]["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|item| item["isTripmine"] == json!(true))
        .expect("the mine")["itemId"]
        .as_str()
        .expect("an id")
        .to_string();

    let (status, body) = gm(
        app,
        session_id,
        &json!({ "action": "pick_tripitem", "itemId": mine, "teamId": team }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "picked the mine: {body}");
    state(app, session_id).await
}

#[tokio::test]
async fn the_last_round_ending_finishes_the_game_with_no_extra_action() {
    let conn: Connection = common::seeded_db();
    let app = test_app(conn);
    let session_id = create(&app, 1).await;

    let (status, _) = gm(&app, &session_id, &json!({ "action": "next_round" })).await;
    assert_eq!(status, StatusCode::OK, "round 1 started");

    let after = hit_the_mine(&app, &session_id).await;
    assert_eq!(after["tripmine"]["completed"], json!(true));
    assert_eq!(
        after["status"],
        json!("finished"),
        "the only (last) round ending finishes the game"
    );
    assert_eq!(after["revealValues"], json!(true));
}

#[tokio::test]
async fn a_non_last_round_ending_leaves_the_game_running() {
    let conn: Connection = common::seeded_db();
    let app = test_app(conn);
    let session_id = create(&app, 2).await;

    let (status, _) = gm(&app, &session_id, &json!({ "action": "next_round" })).await;
    assert_eq!(status, StatusCode::OK);

    let after = hit_the_mine(&app, &session_id).await;
    assert_eq!(after["tripmine"]["completed"], json!(true));
    assert_eq!(
        after["status"],
        json!("running"),
        "round 1 of 2 is not the last round"
    );
}
