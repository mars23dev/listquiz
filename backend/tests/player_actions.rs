//! Player-role actions (US1, FR-001 to FR-005): the team on turn may act from its
//! own role, without the gamemaster password; every other player role and the
//! spectator role are refused; the gamemaster path is unchanged.

mod common;

use axum::Router;
use axum::http::StatusCode;
use rusqlite::Connection;
use serde_json::{Value, json};

use common::{
    call, error_code, get_state, gm_action_request, json_request, seeded_shared_db, test_app,
    valid_session_body,
};

async fn started_session() -> (Router, Connection, String) {
    let (server_db, side) = seeded_shared_db();
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
    (app, side, session_id)
}

async fn gm(app: &Router, session_id: &str, body: &Value) -> (StatusCode, Value) {
    call(
        app,
        gm_action_request(&format!("/api/sessions/{session_id}/actions"), body),
    )
    .await
}

/// A player post carries the role in the body and no password header.
async fn player(app: &Router, session_id: &str, body: &Value) -> (StatusCode, Value) {
    call(
        app,
        json_request(
            "POST",
            &format!("/api/sessions/{session_id}/actions"),
            Some(body),
        ),
    )
    .await
}

fn active_team(state: &Value) -> String {
    state["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["onTurn"] == json!(true))
        .expect("a team on turn")["id"]
        .as_str()
        .expect("a team id")
        .to_string()
}

fn first_pool_item(state: &Value) -> String {
    state["order"]["pool"][0]["itemId"]
        .as_str()
        .expect("a pool item")
        .to_string()
}

#[tokio::test]
async fn the_team_on_turn_may_act_from_its_own_player_role() {
    let (app, _side, session_id) = started_session().await;
    assert_eq!(
        gm(&app, &session_id, &json!({ "action": "next_round" }))
            .await
            .0,
        StatusCode::OK
    );

    let host = get_state(&app, &session_id, "gamemaster").await.1;
    assert_eq!(active_team(&host), "team_1");
    let item = first_pool_item(&host);

    // p1 owns team_1, which is on turn: accepted with no password.
    let (status, body) = player(
        &app,
        &session_id,
        &json!({ "action": "select_item", "role": "p1", "itemId": item, "teamId": "team_1" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "p1 selects: {body}");

    // Then it places (wrong side is fine here: the action is still accepted and the
    // turn passes to team_2).
    let side = get_state(&app, &session_id, "gamemaster").await.1["order"]["pending"]["side"]
        .as_str()
        .expect("pending side")
        .to_string();
    let (status, body) = player(
        &app,
        &session_id,
        &json!({ "action": "choose_slot", "role": "p1", "side": side, "slotIndex": 0, "teamId": "team_1" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "p1 places: {body}");

    let after = get_state(&app, &session_id, "gamemaster").await.1;
    assert_eq!(active_team(&after), "team_2", "the turn passed");
}

#[tokio::test]
async fn other_roles_and_views_are_refused() {
    let (app, _side, session_id) = started_session().await;
    assert_eq!(
        gm(&app, &session_id, &json!({ "action": "next_round" }))
            .await
            .0,
        StatusCode::OK
    );
    let item = first_pool_item(&get_state(&app, &session_id, "gamemaster").await.1);

    // p2 does not own the active team (team_1).
    let (status, body) = player(
        &app,
        &session_id,
        &json!({ "action": "select_item", "role": "p2", "itemId": item, "teamId": "team_2" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error_code(&body), "not_ready");

    // p1 names team_2, which it does not own.
    let (status, body) = player(
        &app,
        &session_id,
        &json!({ "action": "select_item", "role": "p1", "itemId": item, "teamId": "team_2" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error_code(&body), "not_ready");

    // The spectator role never acts.
    let (status, body) = player(
        &app,
        &session_id,
        &json!({ "action": "select_item", "role": "view", "itemId": item, "teamId": "team_1" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error_code(&body), "not_ready");

    // A player role cannot reach a host-only action.
    let (status, body) = player(
        &app,
        &session_id,
        &json!({ "action": "return_to_pool", "role": "p1", "itemId": item }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&body), "bad_request");
}

#[tokio::test]
async fn the_gamemaster_path_still_requires_the_password() {
    let (app, _side, session_id) = started_session().await;

    // No role, no password: refused.
    let (status, body) = call(
        &app,
        json_request(
            "POST",
            &format!("/api/sessions/{session_id}/actions"),
            Some(&json!({ "action": "next_round" })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&body), "unauthorized");

    // With the password it works.
    assert_eq!(
        gm(&app, &session_id, &json!({ "action": "next_round" }))
            .await
            .0,
        StatusCode::OK
    );
}

#[tokio::test]
async fn a_player_picks_a_tripmine_cell_when_its_team_is_on_turn() {
    let (app, side, session_id) = started_session().await;
    assert_eq!(
        gm(&app, &session_id, &json!({ "action": "next_round" }))
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        gm(&app, &session_id, &json!({ "action": "next_round" }))
            .await
            .0,
        StatusCode::OK
    );

    // Round 2 is Tripmine; the starting team is team_2, owned by p2.
    let raw: String = side
        .query_row(
            "SELECT items FROM tripmine_state WHERE session_id = ?1",
            [&session_id],
            |row| row.get(0),
        )
        .expect("the grid");
    let items: Vec<Value> = serde_json::from_str(&raw).expect("grid json");
    let target = items
        .iter()
        .find(|slot| slot["true_rank"] == json!(1))
        .expect("the searched rank")["item"]["item_id"]
        .as_str()
        .expect("an id")
        .to_string();

    let (status, body) = player(
        &app,
        &session_id,
        &json!({ "action": "pick_tripitem", "role": "p2", "itemId": target, "teamId": "team_2" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "p2 picks: {body}");
}
