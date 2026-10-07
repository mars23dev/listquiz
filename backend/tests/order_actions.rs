//! Order round actions over HTTP (T066, T067).

mod common;

use axum::Router;
use axum::http::StatusCode;
use rusqlite::Connection;
use serde_json::{Value, json};

use common::{call, error_code, gm_action_request, json_request, test_app};

/// A running session with round 1 (Order) already started.
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

async fn action(app: &Router, session_id: &str, body: &Value) -> (StatusCode, Value) {
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

/// The pool item whose value is furthest from the anchor, so a test can pick a
/// placement it knows the answer to.
fn pool_item_named(order: &Value, predicate: impl Fn(&Value) -> bool) -> &Value {
    order["pool"]
        .as_array()
        .expect("a pool")
        .iter()
        .find(|item| predicate(item))
        .expect("a matching pool item")
}

// T066 ---------------------------------------------------------------------

#[tokio::test]
async fn session_actions_require_the_gamemaster_password_header() {
    let (app, session_id) = started_order_session().await;

    let (status, body) = call(
        &app,
        json_request(
            "POST",
            &format!("/api/sessions/{session_id}/actions"),
            Some(&json!({ "action": "reveal_values" })),
        ),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&body), "unauthorized");

    let mut wrong_password = json_request(
        "POST",
        &format!("/api/sessions/{session_id}/actions"),
        Some(&json!({ "action": "reveal_values" })),
    );
    wrong_password.headers_mut().insert(
        "X-Quiz-Password",
        axum::http::HeaderValue::from_static("wrong"),
    );
    let (status, body) = call(&app, wrong_password).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&body), "unauthorized");
}

#[tokio::test]
async fn revealing_values_unmasks_them_for_the_player_roles() {
    let (app, session_id) = started_order_session().await;

    let before = state(&app, &session_id, "p1").await;
    assert_eq!(before["revealValues"], json!(false));
    let hidden = pool_item_named(&before["order"], |item| item["value"].is_null()).clone();
    let hidden_id = hidden["itemId"].clone();
    assert!(
        hidden["value"].is_null(),
        "a player never sees a value before the reveal (FR-024)"
    );

    let (status, _) = action(&app, &session_id, &json!({ "action": "reveal_values" })).await;
    assert_eq!(status, StatusCode::OK, "reveal accepted");

    let after = state(&app, &session_id, "p1").await;
    assert_eq!(
        after["revealValues"],
        json!(true),
        "the session carries the flag the views read (FR-024)"
    );
    let on_bar = after["order"]["left"]
        .as_array()
        .expect("left")
        .iter()
        .chain(after["order"]["right"].as_array().expect("right").iter())
        .find(|item| item["itemId"] == hidden_id)
        .expect("the revealed item is arranged onto the bar (US3)");
    assert!(
        !on_bar["value"].is_null(),
        "the very item that was hidden now shows its value to a player"
    );
}

#[tokio::test]
async fn the_reveal_also_covers_the_order_bar_and_the_anchor() {
    let (app, session_id) = started_order_session().await;

    let before = state(&app, &session_id, "p1").await;
    assert!(
        before["order"]["anchor"]["value"].is_string(),
        "the anchor always shows its value (FR-026)"
    );
    assert!(
        before["order"]["pool"]
            .as_array()
            .expect("pool")
            .iter()
            .all(|item| item["value"].is_null()),
        "pool values start hidden"
    );

    let (status, _) = action(&app, &session_id, &json!({ "action": "reveal_values" })).await;
    assert_eq!(status, StatusCode::OK);

    let after = state(&app, &session_id, "p1").await;
    assert!(
        after["order"]["left"]
            .as_array()
            .expect("left")
            .iter()
            .chain(after["order"]["right"].as_array().expect("right").iter())
            .all(|item| item["value"].is_string()),
        "every revealed value is shown on the arranged bar (US3)"
    );
}

#[tokio::test]
async fn the_reveal_flag_survives_for_every_role_at_once() {
    let (app, session_id) = started_order_session().await;
    let (status, _) = action(&app, &session_id, &json!({ "action": "reveal_values" })).await;
    assert_eq!(status, StatusCode::OK);

    for role in ["gamemaster", "p1", "p2", "view"] {
        let body = state(&app, &session_id, role).await;
        assert_eq!(body["revealValues"], json!(true), "{role} sees the flag");
    }
}

// T067 ---------------------------------------------------------------------

#[tokio::test]
async fn selecting_an_item_out_of_turn_is_refused() {
    let (app, session_id) = started_order_session().await;
    let before = state(&app, &session_id, "gamemaster").await;
    let on_turn = before["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["onTurn"] == json!(true))
        .expect("a team on turn")["id"]
        .as_str()
        .expect("an id")
        .to_string();
    let other = before["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["id"].as_str() != Some(on_turn.as_str()))
        .expect("another team")["id"]
        .as_str()
        .expect("an id")
        .to_string();

    let item = pool_item_named(&before["order"], |_| true)["itemId"]
        .as_str()
        .expect("an item")
        .to_string();

    let (status, body) = action(
        &app,
        &session_id,
        &json!({ "action": "select_item", "itemId": item, "teamId": other }),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT, "wrong turn: {body}");
    assert_eq!(error_code(&body), "not_ready", "FR-018, FR-019");

    let after = state(&app, &session_id, "gamemaster").await;
    assert!(
        after["order"]["pending"].is_null(),
        "the item was not taken out of turn"
    );
}

#[tokio::test]
async fn choosing_a_slot_out_of_turn_is_refused() {
    let (app, session_id) = started_order_session().await;
    let before = state(&app, &session_id, "gamemaster").await;
    let on_turn = before["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["onTurn"] == json!(true))
        .expect("a team on turn")["id"]
        .as_str()
        .expect("an id")
        .to_string();
    let other = before["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["id"].as_str() != Some(on_turn.as_str()))
        .expect("another team")["id"]
        .as_str()
        .expect("an id")
        .to_string();

    // Select legitimately first, then try to place from the wrong team.
    let item = pool_item_named(&before["order"], |_| true)["itemId"]
        .as_str()
        .expect("an item")
        .to_string();
    let (status, _) = action(
        &app,
        &session_id,
        &json!({ "action": "select_item", "itemId": item, "teamId": on_turn }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = action(
        &app,
        &session_id,
        &json!({ "action": "choose_slot", "side": "left", "slotIndex": 0, "teamId": other }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "wrong turn: {body}");
    assert_eq!(error_code(&body), "not_ready");

    let after = state(&app, &session_id, "gamemaster").await;
    assert!(
        after["order"]["left"].as_array().expect("left").is_empty(),
        "the bar is untouched (FR-028)"
    );
    assert_eq!(
        after["teams"]
            .as_array()
            .expect("teams")
            .iter()
            .find(|team| team["id"] == json!(on_turn))
            .expect("the team")["score"],
        json!(0),
        "and nothing was scored"
    );
}

// The placement path the two refusals above protect.

#[tokio::test]
async fn a_correct_placement_scores_ten_points_and_hands_the_turn_on() {
    let (app, session_id) = started_order_session().await;
    let revealed = state(&app, &session_id, "gamemaster").await;
    let on_turn = revealed["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["onTurn"] == json!(true))
        .expect("a team on turn")["id"]
        .as_str()
        .expect("an id")
        .to_string();

    // The gamemaster sees values without revealing, so the pool stays pickable
    // (revealing would arrange the whole bar, US3).
    let board = state(&app, &session_id, "gamemaster").await;

    // The pool item with the lowest value belongs on the left; the bar is empty, so
    // slot 0 is the only position it can be right at.
    let item = board["order"]["pool"]
        .as_array()
        .expect("pool")
        .iter()
        .min_by(|a, b| {
            let left = a["value"]
                .as_str()
                .unwrap_or("")
                .parse::<f64>()
                .unwrap_or(f64::MAX);
            let right = b["value"]
                .as_str()
                .unwrap_or("")
                .parse::<f64>()
                .unwrap_or(f64::MAX);
            left.partial_cmp(&right)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("a pool item")["itemId"]
        .as_str()
        .expect("an id")
        .to_string();

    let (status, body) = action(
        &app,
        &session_id,
        &json!({ "action": "select_item", "itemId": item, "teamId": on_turn }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "selected: {body}");

    let pending = state(&app, &session_id, "gamemaster").await["order"]["pending"].clone();
    assert_eq!(
        pending["itemId"],
        json!(item),
        "the choice is visible to the board"
    );
    assert_eq!(
        pending["side"],
        json!("left"),
        "the lowest item belongs on the left"
    );

    let (status, body) = action(
        &app,
        &session_id,
        &json!({ "action": "choose_slot", "side": "left", "slotIndex": 0, "teamId": on_turn }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "placed: {body}");

    let after = state(&app, &session_id, "gamemaster").await;
    let team = after["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["id"] == json!(on_turn))
        .expect("the team");
    assert_eq!(team["score"], json!(10), "exactly ten points (FR-029)");
    assert_eq!(
        team["hearts"],
        json!(3),
        "a correct placement costs no heart"
    );
    assert_eq!(team["onTurn"], json!(false), "the turn has passed (FR-018)");

    assert_eq!(
        after["order"]["left"].as_array().expect("left").len(),
        1,
        "the item is on the bar"
    );
    assert!(
        after["order"]["pool"]
            .as_array()
            .expect("pool")
            .iter()
            .all(|entry| entry["itemId"] != json!(item)),
        "and gone from the pool"
    );
    assert!(
        after["order"]["pending"].is_null(),
        "the selection is spent"
    );
    assert_eq!(
        after["order"]["left"][0]["placedBy"],
        json!(on_turn),
        "the placing team is recorded on the bar item (US2)"
    );
    assert!(
        after["order"]["pool"]
            .as_array()
            .expect("pool")
            .iter()
            .all(|entry| entry["placedBy"].is_null()),
        "a pool item carries no placing team"
    );
}

#[tokio::test]
async fn a_wrong_placement_costs_a_heart_and_leaves_the_item_in_the_pool() {
    let (app, session_id) = started_order_session().await;
    // The gamemaster sees values without revealing, so the pool stays pickable.
    let board = state(&app, &session_id, "gamemaster").await;
    let on_turn = board["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["onTurn"] == json!(true))
        .expect("a team on turn")["id"]
        .as_str()
        .expect("an id")
        .to_string();

    // The highest pool item cannot belong on an empty left side.
    let item = board["order"]["pool"]
        .as_array()
        .expect("pool")
        .iter()
        .max_by(|a, b| {
            let left = a["value"]
                .as_str()
                .unwrap_or("")
                .parse::<f64>()
                .unwrap_or(f64::MIN);
            let right = b["value"]
                .as_str()
                .unwrap_or("")
                .parse::<f64>()
                .unwrap_or(f64::MIN);
            left.partial_cmp(&right)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("a pool item")["itemId"]
        .as_str()
        .expect("an id")
        .to_string();

    let (status, _) = action(
        &app,
        &session_id,
        &json!({ "action": "select_item", "itemId": item, "teamId": on_turn }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let (status, body) = action(
        &app,
        &session_id,
        &json!({ "action": "choose_slot", "side": "left", "slotIndex": 0, "teamId": on_turn }),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "the mistake is accepted and scored: {body}"
    );

    let after = state(&app, &session_id, "gamemaster").await;
    let team = after["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["id"] == json!(on_turn))
        .expect("the team");
    assert_eq!(
        team["score"],
        json!(0),
        "no points for a wrong side (FR-030)"
    );
    assert_eq!(team["hearts"], json!(2), "exactly one heart gone (FR-030)");
    assert_eq!(
        team["onTurn"],
        json!(false),
        "the turn still passes (FR-030)"
    );

    assert!(
        after["order"]["pool"]
            .as_array()
            .expect("pool")
            .iter()
            .any(|entry| entry["itemId"] == json!(item)),
        "the item stays in the pool for another try (FR-031)"
    );
    assert!(
        after["order"]["left"].as_array().expect("left").is_empty(),
        "and never reaches the bar"
    );
}

#[tokio::test]
async fn the_gamemaster_can_return_a_placed_item_to_the_pool() {
    let (app, session_id) = started_order_session().await;
    // The gamemaster sees values without revealing, so the pool stays pickable
    // (revealing would arrange the whole bar, US3).
    let board = state(&app, &session_id, "gamemaster").await;
    let on_turn = board["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["onTurn"] == json!(true))
        .expect("a team on turn")["id"]
        .as_str()
        .expect("an id")
        .to_string();

    let item = board["order"]["pool"][0]["itemId"]
        .as_str()
        .expect("an id")
        .to_string();
    let (status, _) = action(
        &app,
        &session_id,
        &json!({ "action": "select_item", "itemId": item, "teamId": on_turn }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let side = state(&app, &session_id, "gamemaster").await["order"]["pending"]["side"]
        .as_str()
        .expect("the pending side")
        .to_string();
    let (status, body) = action(
        &app,
        &session_id,
        &json!({ "action": "choose_slot", "side": side, "slotIndex": 0, "teamId": on_turn }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "placed correctly: {body}");

    let placed = state(&app, &session_id, "gamemaster").await;
    assert_eq!(placed["order"][&side].as_array().expect("side").len(), 1);
    assert_eq!(
        placed["order"][&side][0]["placedBy"],
        json!(on_turn),
        "the placing team is recorded before the return (US2)"
    );

    let (status, body) = action(
        &app,
        &session_id,
        &json!({ "action": "return_to_pool", "itemId": item }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "returned: {body}");

    let after = state(&app, &session_id, "gamemaster").await;
    assert!(
        after["order"][&side].as_array().expect("side").is_empty(),
        "the item is off the bar (FR-033)"
    );
    assert!(
        after["order"]["pool"]
            .as_array()
            .expect("pool")
            .iter()
            .any(|entry| entry["itemId"] == json!(item)),
        "and back in the pool"
    );
    assert!(
        after["order"]["pool"]
            .as_array()
            .expect("pool")
            .iter()
            .filter(|entry| entry["itemId"] == json!(item))
            .all(|entry| entry["placedBy"].is_null()),
        "the returned item's placing team is cleared (US2)"
    );
    let team = after["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["id"] == json!(on_turn))
        .expect("the team");
    assert_eq!(
        team["score"],
        json!(10),
        "returning an item does not undo the score it already earned"
    );
}

#[tokio::test]
async fn cancelling_a_selection_scores_nothing() {
    let (app, session_id) = started_order_session().await;
    let board = state(&app, &session_id, "gamemaster").await;
    let on_turn = board["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["onTurn"] == json!(true))
        .expect("a team on turn")["id"]
        .as_str()
        .expect("an id")
        .to_string();
    let pool_before = board["order"]["pool"].as_array().expect("pool").len();
    let item = board["order"]["pool"][0]["itemId"]
        .as_str()
        .expect("an id")
        .to_string();

    let (status, _) = action(
        &app,
        &session_id,
        &json!({ "action": "select_item", "itemId": item, "teamId": on_turn }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(!state(&app, &session_id, "gamemaster").await["order"]["pending"].is_null());

    let (status, body) = action(
        &app,
        &session_id,
        &json!({ "action": "cancel_selection", "teamId": on_turn }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "cancelled: {body}");

    let after = state(&app, &session_id, "gamemaster").await;
    assert!(after["order"]["pending"].is_null(), "the selection is gone");
    assert_eq!(
        after["order"]["pool"].as_array().expect("pool").len(),
        pool_before,
        "the item never left the pool"
    );
    let team = after["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["id"] == json!(on_turn))
        .expect("the team");
    assert_eq!(team["score"], json!(0), "nothing scored");
    assert_eq!(team["hearts"], json!(3), "no heart lost");
    assert_eq!(
        team["onTurn"],
        json!(true),
        "and the turn stays where it was"
    );
}

#[tokio::test]
async fn a_player_never_sees_another_teams_pending_team_id() {
    let (app, session_id) = started_order_session().await;
    let board = state(&app, &session_id, "gamemaster").await;
    let on_turn = board["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["onTurn"] == json!(true))
        .expect("a team on turn")["id"]
        .as_str()
        .expect("an id")
        .to_string();
    let item = board["order"]["pool"][0]["itemId"]
        .as_str()
        .expect("an id")
        .to_string();
    let (status, _) = action(
        &app,
        &session_id,
        &json!({ "action": "select_item", "itemId": item, "teamId": on_turn }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let gm = state(&app, &session_id, "gamemaster").await;
    assert_eq!(gm["order"]["pending"]["teamId"], json!(on_turn));

    let player = state(&app, &session_id, "p2").await;
    assert_eq!(
        player["order"]["pending"]["teamId"],
        json!(""),
        "another team's pending placement is not disclosed (contracts/realtime.md)"
    );
    assert_eq!(
        player["order"]["pending"]["itemId"],
        json!(item),
        "but the board itself is shared"
    );
}

#[tokio::test]
async fn an_order_action_before_a_round_has_started_is_not_ready() {
    let conn: Connection = common::seeded_db();
    let app = test_app(conn);
    let (status, created) = call(
        &app,
        json_request("POST", "/api/sessions", Some(&common::valid_session_body())),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let session_id = created["sessionId"].as_str().expect("an id").to_string();

    let (status, body) = action(
        &app,
        &session_id,
        &json!({ "action": "select_item", "itemId": "whatever", "teamId": "team_1" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{body}");
    assert_eq!(error_code(&body), "not_ready");
}

#[tokio::test]
async fn an_unknown_action_is_still_rejected() {
    let (app, session_id) = started_order_session().await;
    let (status, body) = action(&app, &session_id, &json!({ "action": "sort_the_pool" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(error_code(&body), "bad_request");
}

#[tokio::test]
async fn a_round_starts_with_only_the_anchor_and_ten_candidates() {
    let (app, session_id) = started_order_session().await;
    let board = state(&app, &session_id, "gamemaster").await;
    let order = &board["order"];

    assert!(
        order["left"].as_array().expect("left").is_empty(),
        "the bar starts empty on the left (FR-025, FR-027)"
    );
    assert!(
        order["right"].as_array().expect("right").is_empty(),
        "and empty on the right"
    );
    assert_eq!(
        order["pool"].as_array().expect("pool").len(),
        10,
        "ten candidates are dealt (FR-027)"
    );
    let anchor = order["anchor"]["itemId"].as_str().expect("an anchor id");
    assert!(
        !order["pool"]
            .as_array()
            .expect("pool")
            .iter()
            .any(|item| item["itemId"].as_str() == Some(anchor)),
        "the anchor is not among them (FR-026)"
    );
    assert!(order["pending"].is_null(), "nothing is selected yet");
}

#[tokio::test]
async fn a_chosen_slot_out_of_range_is_refused() {
    let (app, session_id) = started_order_session().await;
    let board = state(&app, &session_id, "gamemaster").await;
    let on_turn = board["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["onTurn"] == json!(true))
        .expect("a team on turn")["id"]
        .as_str()
        .expect("an id")
        .to_string();
    let item = board["order"]["pool"][0]["itemId"]
        .as_str()
        .expect("an id")
        .to_string();

    let (status, _) = action(
        &app,
        &session_id,
        &json!({ "action": "select_item", "itemId": item, "teamId": on_turn }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // The bar is empty, so slot 3 does not exist on either side (FR-028).
    let (status, body) = action(
        &app,
        &session_id,
        &json!({ "action": "choose_slot", "side": "left", "slotIndex": 3, "teamId": on_turn }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
    assert_eq!(error_code(&body), "bad_request");

    let after = state(&app, &session_id, "gamemaster").await;
    assert_eq!(
        after["teams"]
            .as_array()
            .expect("teams")
            .iter()
            .find(|team| team["id"] == json!(on_turn))
            .expect("the team")["score"],
        json!(0),
        "a refused slot scores nothing"
    );
}

#[tokio::test]
async fn revealing_arranges_the_bar_and_scores_nothing() {
    let (app, session_id) = started_order_session().await;
    let before = state(&app, &session_id, "gamemaster").await;
    let scores_before: Vec<Value> = before["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .map(|team| team["score"].clone())
        .collect();

    let (status, _) = action(&app, &session_id, &json!({ "action": "reveal_values" })).await;
    assert_eq!(status, StatusCode::OK);

    let after = state(&app, &session_id, "gamemaster").await;
    let order = &after["order"];
    assert!(
        order["pool"].as_array().expect("pool").is_empty(),
        "the reveal arranges every pool item onto the bar (US3)"
    );
    let left = order["left"].as_array().expect("left");
    let right = order["right"].as_array().expect("right");
    assert_eq!(
        left.len() + right.len(),
        10,
        "all ten candidates sit on the bar"
    );

    let scores_after: Vec<Value> = after["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .map(|team| team["score"].clone())
        .collect();
    assert_eq!(
        scores_before, scores_after,
        "revealing values awards no points (FR-012)"
    );
}
