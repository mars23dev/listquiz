mod common;

use serde_json::json;

use common::{
    call, error_code, get_state, json_request, seeded_shared_db, test_app, valid_session_body,
};

#[tokio::test]
async fn a_player_sees_no_values_until_the_gamemaster_reveals_them() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);
    let (status, created) = call(
        &app,
        json_request("POST", "/api/sessions", Some(&valid_session_body())),
    )
    .await;
    assert_eq!(status, 201);
    let session_id = created["sessionId"].as_str().unwrap().to_string();

    call(
        &app,
        json_request(
            "POST",
            &format!("/api/sessions/{session_id}/actions"),
            Some(&json!({ "action": "next_round" })),
        ),
    )
    .await;

    let (_, player) = get_state(&app, &session_id, "p1").await;
    let (_, host) = get_state(&app, &session_id, "gamemaster").await;

    assert_eq!(player["order"]["left"][0]["value"], json!(null));
    assert_eq!(player["order"]["left"][0]["revealed"], json!(false));
    assert!(host["order"]["left"][0]["value"].is_string());
    assert_eq!(host["order"]["left"][0]["revealed"], json!(true));

    // FR-026: the anchor is public by design, in every view.
    assert!(player["order"]["anchor"]["value"].is_string());
    assert_eq!(
        player["order"]["anchor"]["value"],
        host["order"]["anchor"]["value"]
    );
}

#[tokio::test]
async fn an_item_slot_beyond_the_team_count_is_read_only() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);
    let (status, created) = call(
        &app,
        json_request("POST", "/api/sessions", Some(&valid_session_body())),
    )
    .await;
    assert_eq!(status, 201);
    let session_id = created["sessionId"].as_str().unwrap().to_string();

    let (_, spectator) = get_state(&app, &session_id, "view").await;
    let (_, player_four) = get_state(&app, &session_id, "p4").await;

    let links = &spectator["links"]["players"];
    assert_eq!(links[0]["hasTeam"], json!(true));
    assert_eq!(links[1]["hasTeam"], json!(true));
    assert_eq!(links[2]["hasTeam"], json!(false));
    assert_eq!(links[3]["hasTeam"], json!(false));

    // No team sits behind p3 or p4, so neither view may report an active team.
    assert_eq!(player_four["teams"].as_array().unwrap().len(), 2);
    assert_eq!(
        player_four["teams"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|team| team["onTurn"] == json!(true))
            .count(),
        0
    );
}

#[tokio::test]
async fn a_state_request_for_an_unknown_role_is_not_found() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);
    let (_status, created) = call(
        &app,
        json_request("POST", "/api/sessions", Some(&valid_session_body())),
    )
    .await;
    let session_id = created["sessionId"].as_str().unwrap().to_string();

    let (status, body) = call(
        &app,
        json_request(
            "GET",
            &format!("/api/sessions/{session_id}/state?role=admin"),
            None,
        ),
    )
    .await;

    assert_eq!(status, 404);
    assert_eq!(error_code(&body), "not_found");
}

#[tokio::test]
async fn the_round_count_bounds_are_exactly_one_to_ten() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);
    let base = valid_session_body();

    for count in [1, 10] {
        let mut body = base.clone();
        body["roundCount"] = json!(count);
        body["rounds"] = json!(Vec::<serde_json::Value>::new());
        for index in 1..=count {
            body["rounds"].as_array_mut().unwrap().push(json!({
                "roundNumber": index,
                "quizType": "ORDER",
                "listId": "default_world_history",
                "columnId": "year"
            }));
        }
        let (status, _) = call(&app, json_request("POST", "/api/sessions", Some(&body))).await;
        assert_eq!(status, 201, "{count} rounds must be accepted");
    }
}
