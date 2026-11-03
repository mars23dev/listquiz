//! Session creation over HTTP (US1, T028-T031).
//!
//! These tests were written before the handler existed and failed with 404s; the
//! rules they pin down come straight from spec.md FR-005 to FR-010.

mod common;

use axum::Router;
use serde_json::{Value, json};

use common::{call, error_code, get_state, json_request, seeded_db, test_app, valid_session_body};

async fn create(app: &Router, body: Value) -> (axum::http::StatusCode, Value) {
    call(app, json_request("POST", "/api/sessions", Some(&body))).await
}

#[tokio::test]
async fn an_empty_gamemaster_password_is_refused() {
    let app = test_app(seeded_db());
    let mut body = valid_session_body();
    body["gmPassword"] = json!("");

    let (status, response) = create(&app, body).await;

    assert_eq!(status, 400);
    assert_eq!(error_code(&response), "bad_request");
}

#[tokio::test]
async fn a_missing_gamemaster_password_is_refused() {
    let app = test_app(seeded_db());
    let mut body = valid_session_body();
    body.as_object_mut().unwrap().remove("gmPassword");

    let (status, response) = create(&app, body).await;

    assert_eq!(status, 400);
    assert_eq!(error_code(&response), "bad_request");
}

#[tokio::test]
async fn zero_or_eleven_rounds_are_refused() {
    for round_count in [0, 11] {
        let app = test_app(seeded_db());
        let mut body = valid_session_body();
        body["roundCount"] = json!(round_count);

        let (status, response) = create(&app, body).await;

        assert_eq!(status, 400, "{round_count} rounds should be refused");
        assert_eq!(error_code(&response), "bad_request");
    }
}

#[tokio::test]
async fn a_round_count_that_disagrees_with_the_round_list_is_refused() {
    let app = test_app(seeded_db());
    let mut body = valid_session_body();
    body["roundCount"] = json!(3);

    let (status, response) = create(&app, body).await;

    assert_eq!(status, 400);
    assert_eq!(error_code(&response), "bad_request");
}

#[tokio::test]
async fn zero_or_five_teams_are_refused() {
    for team_count in [0, 5] {
        let app = test_app(seeded_db());
        let mut body = valid_session_body();
        body["teamCount"] = json!(team_count);

        let (status, response) = create(&app, body).await;

        assert_eq!(status, 400, "{team_count} teams should be refused");
        assert_eq!(error_code(&response), "bad_request");
    }
}

#[tokio::test]
async fn an_unknown_list_is_refused() {
    let app = test_app(seeded_db());
    let mut body = valid_session_body();
    body["rounds"][0]["listId"] = json!("lst-does-not-exist");

    let (status, response) = create(&app, body).await;

    assert_eq!(status, 404);
    assert_eq!(error_code(&response), "not_found");
}

#[tokio::test]
async fn a_column_from_another_list_is_refused() {
    let app = test_app(seeded_db());
    let mut body = valid_session_body();
    // `height_m` belongs to the statues list, not to the history list.
    body["rounds"][0]["columnId"] = json!("height_m");

    let (status, response) = create(&app, body).await;

    assert_eq!(status, 404);
    assert_eq!(error_code(&response), "not_found");
}

#[tokio::test]
async fn a_tripmine_round_without_a_side_is_refused() {
    let app = test_app(seeded_db());
    let mut body = valid_session_body();
    body["rounds"][1]
        .as_object_mut()
        .unwrap()
        .remove("tripmineSide");

    let (status, response) = create(&app, body).await;

    assert_eq!(status, 400);
    assert_eq!(error_code(&response), "bad_request");
}

#[tokio::test]
async fn an_order_round_carrying_a_tripmine_side_is_refused() {
    let app = test_app(seeded_db());
    let mut body = valid_session_body();
    body["rounds"][0]["tripmineSide"] = json!("highest");

    let (status, response) = create(&app, body).await;

    assert_eq!(status, 400);
    assert_eq!(error_code(&response), "bad_request");
}

#[tokio::test]
async fn a_created_session_returns_exactly_five_role_links() {
    let app = test_app(seeded_db());

    let (status, response) = create(&app, valid_session_body()).await;

    assert_eq!(status, 201);
    let session_id = response["sessionId"].as_str().expect("sessionId");
    assert!(session_id.starts_with("lq-"), "unexpected id: {session_id}");
    assert_eq!(session_id.len(), "lq-".len() + 8);

    let links = &response["links"];
    assert_eq!(links["gamemaster"], format!("/{session_id}/gamemaster"));
    assert_eq!(links["spectator"], format!("/{session_id}/view"));

    let players = links["players"].as_array().expect("players array");
    assert_eq!(players.len(), 4, "P1 through P4 are always linked");
    for (index, role) in ["p1", "p2", "p3", "p4"].iter().enumerate() {
        assert_eq!(players[index]["role"], *role);
        assert_eq!(players[index]["path"], format!("/{session_id}/{role}"));
    }

    // Two configured teams fill P1 and P2; the spare slots still resolve but say
    // they have no team, so the setup screen can label them.
    assert_eq!(players[0]["hasTeam"], json!(true));
    assert_eq!(players[1]["hasTeam"], json!(true));
    assert_eq!(players[2]["hasTeam"], json!(false));
    assert_eq!(players[3]["hasTeam"], json!(false));
}

#[tokio::test]
async fn both_teams_start_on_zero_points_and_three_hearts() {
    let app = test_app(seeded_db());

    let (status, response) = create(&app, valid_session_body()).await;
    assert_eq!(status, 201);
    let session_id = response["sessionId"].as_str().unwrap().to_string();

    let (status, state) = get_state(&app, &session_id, "gamemaster").await;

    assert_eq!(status, 200);
    assert_eq!(state["sessionId"], json!(session_id));
    let teams = state["teams"].as_array().expect("teams");
    assert_eq!(teams.len(), 2);
    for team in teams {
        assert_eq!(team["score"], json!(0));
        assert_eq!(team["hearts"], json!(3));
    }
    assert_eq!(state["currentRound"], json!(0), "no round has started yet");
}
