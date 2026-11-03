//! Connect actions over HTTP (feature 010, US1).

mod common;

use axum::Router;
use axum::http::StatusCode;
use rusqlite::Connection;
use serde_json::{Value, json};

use common::{call, gm_action_request, json_request, test_app};

fn body() -> Value {
    json!({
        "gmPassword": "letmein",
        "teamCount": 2,
        "teams": [{ "name": "Team Cyan Pulsar" }, { "name": "Team Ruby Spark" }],
        "roundCount": 1,
        "rounds": [
            { "roundNumber": 1, "quizType": "CONNECT",
              "listId": "default_world_history", "columnId": "year" }
        ]
    })
}

async fn started_connect_session() -> (Router, String) {
    let conn: Connection = common::seeded_db();
    let app = test_app(conn);
    let (status, created) = call(&app, json_request("POST", "/api/sessions", Some(&body()))).await;
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

fn first_name(body: &Value) -> String {
    body["connect"]["names"][0]["itemId"]
        .as_str()
        .expect("a name")
        .to_string()
}

/// The value item id matching `name_item_id`.
fn partner_value(body: &Value, name_item_id: &str) -> String {
    body["connect"]["values"]
        .as_array()
        .expect("values")
        .iter()
        .find(|value| value["itemId"] == json!(name_item_id))
        .expect("the matching value")["itemId"]
        .as_str()
        .expect("an id")
        .to_string()
}

fn a_different_value(body: &Value, name_item_id: &str) -> String {
    body["connect"]["values"]
        .as_array()
        .expect("values")
        .iter()
        .map(|value| value["itemId"].as_str().expect("an id").to_string())
        .find(|id| id != name_item_id)
        .expect("a different value")
}

#[tokio::test]
async fn the_connect_board_is_public_to_a_player() {
    let (app, session_id) = started_connect_session().await;
    let body = state(&app, &session_id, "p1").await;

    assert_eq!(body["round"]["quizType"], json!("CONNECT"));
    assert_eq!(
        body["connect"]["names"].as_array().expect("names").len(),
        10
    );
    assert_eq!(
        body["connect"]["values"].as_array().expect("values").len(),
        10
    );
    assert!(
        body["connect"]["matched"]
            .as_array()
            .expect("matched")
            .is_empty()
    );
}

#[tokio::test]
async fn a_matching_pair_scores_ten_and_passes_the_turn() {
    let (app, session_id) = started_connect_session().await;
    let before = state(&app, &session_id, "gamemaster").await;
    let team = on_turn(&before);
    let name = first_name(&before);
    let value = partner_value(&before, &name);

    assert_eq!(
        gm(
            &app,
            &session_id,
            &json!({ "action": "pick_name", "itemId": name, "teamId": team })
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        gm(
            &app,
            &session_id,
            &json!({ "action": "pick_value", "itemId": value, "teamId": team })
        )
        .await
        .0,
        StatusCode::OK
    );
    let (status, out) = gm(
        &app,
        &session_id,
        &json!({ "action": "confirm_connect", "teamId": team }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "confirmed: {out}");

    let after = state(&app, &session_id, "gamemaster").await;
    assert!(
        after["connect"]["matched"]
            .as_array()
            .expect("matched")
            .contains(&json!(name)),
        "the pair is matched"
    );
    let score = after["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|t| t["id"] == json!(team))
        .expect("team")["score"]
        .as_i64()
        .unwrap();
    assert_eq!(score, 10, "a correct pair scores ten");
    assert_ne!(on_turn(&after), team, "the turn passes");
}

#[tokio::test]
async fn a_mismatch_costs_a_heart_and_passes_the_turn() {
    let (app, session_id) = started_connect_session().await;
    let before = state(&app, &session_id, "gamemaster").await;
    let team = on_turn(&before);
    let hearts_before = hearts_of(&before, &team);
    let name = first_name(&before);
    let wrong = a_different_value(&before, &name);

    assert_eq!(
        gm(
            &app,
            &session_id,
            &json!({ "action": "pick_name", "itemId": name, "teamId": team })
        )
        .await
        .0,
        StatusCode::OK
    );
    assert_eq!(
        gm(
            &app,
            &session_id,
            &json!({ "action": "pick_value", "itemId": wrong, "teamId": team })
        )
        .await
        .0,
        StatusCode::OK
    );
    let (status, out) = gm(
        &app,
        &session_id,
        &json!({ "action": "confirm_connect", "teamId": team }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "confirmed: {out}");

    let after = state(&app, &session_id, "gamemaster").await;
    assert_eq!(
        hearts_of(&after, &team),
        hearts_before - 1,
        "one heart lost"
    );
    assert!(
        after["connect"]["matched"]
            .as_array()
            .expect("matched")
            .is_empty(),
        "the pair stays unmatched"
    );
    assert_ne!(on_turn(&after), team, "the turn passes");
}
