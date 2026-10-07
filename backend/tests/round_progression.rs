//! Scores carry across rounds and the game finishes (T132-T133).

mod common;

use axum::Router;
use axum::http::StatusCode;
use serde_json::{Value, json};

use common::{
    call, error_code, get_state, gm_action_request, json_request, seeded_shared_db, test_app,
    valid_session_body,
};

async fn create_session(app: &Router) -> String {
    let (status, created) = call(
        app,
        json_request("POST", "/api/sessions", Some(&valid_session_body())),
    )
    .await;
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

fn team_score(state: &Value, team_id: &str) -> i64 {
    state["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["id"] == json!(team_id))
        .expect("the team")["score"]
        .as_i64()
        .expect("a score")
}

// T132 ---------------------------------------------------------------------

#[tokio::test]
async fn a_round_two_opening_score_equals_the_round_one_closing_score() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);
    let session_id = create_session(&app).await;

    // Round 1.
    let (status, _) = action(&app, &session_id, &json!({ "action": "next_round" })).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = action(
        &app,
        &session_id,
        &json!({ "action": "adjust_score", "teamId": "team_1", "delta": 30 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let closing = get_state(&app, &session_id, "gamemaster").await.1;
    assert_eq!(team_score(&closing, "team_1"), 30);

    // Round 2 opens on the same total.
    let (status, _) = action(&app, &session_id, &json!({ "action": "next_round" })).await;
    assert_eq!(status, StatusCode::OK);
    let opening = get_state(&app, &session_id, "gamemaster").await.1;
    assert_eq!(opening["currentRound"], json!(2));
    assert_eq!(team_score(&opening, "team_1"), 30);
}

// T133 ---------------------------------------------------------------------

#[tokio::test]
async fn next_round_past_the_last_round_finishes_then_is_refused() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);
    let session_id = create_session(&app).await;

    // The shared body has two rounds.
    assert_eq!(
        action(&app, &session_id, &json!({ "action": "next_round" }))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        action(&app, &session_id, &json!({ "action": "next_round" }))
            .await
            .0,
        StatusCode::OK
    );

    // Crossing the last round finishes the game.
    let (status, _) = action(&app, &session_id, &json!({ "action": "next_round" })).await;
    assert_eq!(status, StatusCode::OK);
    let finished = get_state(&app, &session_id, "gamemaster").await.1;
    assert_eq!(finished["status"], json!("finished"));
    assert_eq!(finished["revealValues"], json!(true));

    // A further attempt is refused.
    let (status, body) = action(&app, &session_id, &json!({ "action": "next_round" })).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error_code(&body), "not_ready");
}

#[tokio::test]
async fn pass_turn_and_adjust_hearts_change_the_board() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);
    let session_id = create_session(&app).await;
    assert_eq!(
        action(&app, &session_id, &json!({ "action": "next_round" }))
            .await
            .0,
        StatusCode::OK
    );

    let before = get_state(&app, &session_id, "gamemaster").await.1;
    let on_turn = before["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["onTurn"] == json!(true))
        .expect("a team on turn")["id"]
        .as_str()
        .expect("an id")
        .to_string();

    let (status, _) = action(&app, &session_id, &json!({ "action": "pass_turn" })).await;
    assert_eq!(status, StatusCode::OK);
    let after = get_state(&app, &session_id, "gamemaster").await.1;
    assert_ne!(
        after["teams"]
            .as_array()
            .expect("teams")
            .iter()
            .find(|team| team["onTurn"] == json!(true))
            .expect("a team on turn")["id"],
        json!(on_turn)
    );

    let (status, _) = action(
        &app,
        &session_id,
        &json!({ "action": "adjust_hearts", "teamId": "team_1", "delta": -1 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let adjusted = get_state(&app, &session_id, "gamemaster").await.1;
    let hearts = adjusted["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["id"] == json!("team_1"))
        .expect("team_1")["hearts"]
        .as_i64();
    assert_eq!(hearts, Some(2));
}
