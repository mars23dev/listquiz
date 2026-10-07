//! Tripmine round actions over HTTP (T087 and the FR-039 to FR-044 outcomes).

mod common;

use axum::Router;
use axum::http::StatusCode;
use rusqlite::Connection;
use serde_json::{Value, json};

use common::{
    call, error_code, get_state, gm_action_request, json_request, seeded_shared_db, test_app,
    valid_session_body,
};

/// A session sitting on its Tripmine round (round 2 of the shared body), with the
/// test-side database connection so a test can read the hidden answer key.
async fn started_tripmine_session() -> (Router, Connection, String) {
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

    // Round 1 is Order; advancing twice lands on round 2, the Tripmine round.
    for round in 1..=2 {
        let (status, body) = call(
            &app,
            gm_action_request(
                &format!("/api/sessions/{session_id}/actions"),
                &json!({ "action": "next_round" }),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "round {round} started: {body}");
    }
    (app, side, session_id)
}

async fn action(app: &Router, session_id: &str, body: &Value) -> (StatusCode, Value) {
    call(
        app,
        gm_action_request(&format!("/api/sessions/{session_id}/actions"), body),
    )
    .await
}

/// The stored grid, including the answer key the views never see before a reveal.
fn stored_items(side: &Connection, session_id: &str) -> Vec<Value> {
    let raw: String = side
        .query_row(
            "SELECT items FROM tripmine_state WHERE session_id = ?1",
            [session_id],
            |row| row.get(0),
        )
        .expect("the stored grid");
    serde_json::from_str(&raw).expect("grid json")
}

fn item_at_rank(items: &[Value], rank: i32) -> String {
    items
        .iter()
        .find(|slot| slot["true_rank"] == json!(rank))
        .expect("a slot at that rank")["item"]["item_id"]
        .as_str()
        .expect("an item id")
        .to_string()
}

fn searched_rank(side: &Connection, session_id: &str) -> i32 {
    side.query_row(
        "SELECT searched_rank FROM tripmine_state WHERE session_id = ?1",
        [session_id],
        |row| row.get(0),
    )
    .expect("the searched rank")
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

// T087 ---------------------------------------------------------------------

#[tokio::test]
async fn a_player_never_sees_the_mine_before_it_is_revealed() {
    let (app, side, session_id) = started_tripmine_session().await;
    let items = stored_items(&side, &session_id);
    let mine_id = items
        .iter()
        .find(|slot| slot["is_tripmine"] == json!(true))
        .expect("a mine")["item"]["item_id"]
        .as_str()
        .expect("an id")
        .to_string();

    let (_, player) = get_state(&app, &session_id, "p1").await;
    for item in player["tripmine"]["items"].as_array().expect("items") {
        if item["revealed"] == json!(false) {
            assert_eq!(
                item["isTripmine"],
                json!(false),
                "an unrevealed item must not carry the answer"
            );
            assert_eq!(
                item["trueRank"],
                json!(null),
                "an unrevealed item must not reveal its position either"
            );
            assert!(item["value"].is_null());
        }
    }

    let mine = player["tripmine"]["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|item| item["itemId"] == json!(mine_id))
        .expect("the mine row");
    assert_eq!(mine["isTripmine"], json!(false), "the mine stays hidden");

    // The gamemaster does see the answer key.
    let (_, host) = get_state(&app, &session_id, "gamemaster").await;
    let host_mine = host["tripmine"]["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|item| item["itemId"] == json!(mine_id))
        .expect("the mine row");
    assert_eq!(host_mine["isTripmine"], json!(true));
}

#[tokio::test]
async fn the_searched_pick_scores_ten_and_advances_the_search() {
    let (app, side, session_id) = started_tripmine_session().await;
    let items = stored_items(&side, &session_id);
    let searched = searched_rank(&side, &session_id);
    let target = item_at_rank(&items, searched);

    let host = get_state(&app, &session_id, "gamemaster").await.1;
    let team = active_team(&host);
    let (status, body) = action(
        &app,
        &session_id,
        &json!({ "action": "pick_tripitem", "itemId": target, "teamId": team }),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "pick accepted: {body}");

    let after = get_state(&app, &session_id, "gamemaster").await.1;
    assert_eq!(team_score(&after, &team), 10);
    assert_eq!(after["tripmine"]["searchedRank"], json!(searched + 1));

    let (_, player) = get_state(&app, &session_id, "p1").await;
    let picked = player["tripmine"]["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|item| item["itemId"] == json!(target))
        .expect("the pick");
    assert_eq!(picked["revealed"], json!(true));
    assert_eq!(picked["pointsAwarded"], json!(10));
    assert!(
        picked["value"].is_string(),
        "the position's value is now public"
    );
}

#[tokio::test]
async fn a_wrong_pick_scores_five_and_leaves_the_search_alone() {
    let (app, side, session_id) = started_tripmine_session().await;
    let items = stored_items(&side, &session_id);
    let searched = searched_rank(&side, &session_id);
    // Rank 5 is neither the searched rank (1) nor the mine (10).
    let wrong = item_at_rank(&items, 5);

    let host = get_state(&app, &session_id, "gamemaster").await.1;
    let team = active_team(&host);
    let (status, _) = action(
        &app,
        &session_id,
        &json!({ "action": "pick_tripitem", "itemId": wrong, "teamId": team }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let after = get_state(&app, &session_id, "gamemaster").await.1;
    assert_eq!(team_score(&after, &team), 5);
    assert_eq!(after["tripmine"]["searchedRank"], json!(searched));
}

#[tokio::test]
async fn a_pick_from_a_team_not_on_turn_is_refused() {
    let (app, side, session_id) = started_tripmine_session().await;
    let items = stored_items(&side, &session_id);
    let target = item_at_rank(&items, 1);

    let host = get_state(&app, &session_id, "gamemaster").await.1;
    let active = active_team(&host);
    let other = host["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["id"] != json!(active))
        .expect("another team")["id"]
        .as_str()
        .expect("an id")
        .to_string();

    let (status, body) = action(
        &app,
        &session_id,
        &json!({ "action": "pick_tripitem", "itemId": target, "teamId": other }),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error_code(&body), "not_ready");
}

#[tokio::test]
async fn the_tripmine_side_cannot_change_mid_round() {
    let (app, side, session_id) = started_tripmine_session().await;
    let items = stored_items(&side, &session_id);
    let target = item_at_rank(&items, 1);

    let host = get_state(&app, &session_id, "gamemaster").await.1;
    let team = active_team(&host);
    let (status, body) = action(
        &app,
        &session_id,
        &json!({
            "action": "pick_tripitem",
            "itemId": target,
            "teamId": team,
            "side": "lowest"
        }),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error_code(&body), "conflict");
}

#[tokio::test]
async fn setting_off_the_mine_pays_the_other_teams_and_ends_the_round() {
    let (app, side, session_id) = started_tripmine_session().await;
    let items = stored_items(&side, &session_id);
    let mine = item_at_rank(&items, 10);

    let host = get_state(&app, &session_id, "gamemaster").await.1;
    let picker = active_team(&host);
    let other = host["teams"]
        .as_array()
        .expect("teams")
        .iter()
        .find(|team| team["id"] != json!(picker))
        .expect("another team")["id"]
        .as_str()
        .expect("an id")
        .to_string();

    let (status, _) = action(
        &app,
        &session_id,
        &json!({ "action": "pick_tripitem", "itemId": mine, "teamId": picker }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let after = get_state(&app, &session_id, "gamemaster").await.1;
    assert_eq!(team_score(&after, &picker), -10);
    assert_eq!(team_score(&after, &other), 10);
    assert_eq!(after["tripmine"]["hit"], json!(true));
    assert_eq!(after["tripmine"]["completed"], json!(true));
    assert!(
        after["tripmine"]["items"]
            .as_array()
            .expect("items")
            .iter()
            .all(|item| item["revealed"] == json!(true)),
        "every position is revealed (FR-042)"
    );
}

#[tokio::test]
async fn the_tripmine_target_value_is_shown_to_every_role() {
    let (app, side, session_id) = started_tripmine_session().await;
    let items = stored_items(&side, &session_id);
    let mine_value = items
        .iter()
        .find(|slot| slot["is_tripmine"] == json!(true))
        .expect("the mine")["item"]["display"]
        .as_str()
        .expect("a value")
        .to_string();

    for role in ["gamemaster", "p1", "view"] {
        let (_, state) = get_state(&app, &session_id, role).await;
        assert_eq!(
            state["tripmine"]["targetValue"],
            json!(mine_value),
            "the {role} view shows the mined value"
        );
    }

    // The hint reveals one value, not the cells.
    let (_, player) = get_state(&app, &session_id, "p1").await;
    assert!(
        player["tripmine"]["items"]
            .as_array()
            .expect("items")
            .iter()
            .all(|item| item["value"].is_null()),
        "no unpicked cell shows a value"
    );
}
