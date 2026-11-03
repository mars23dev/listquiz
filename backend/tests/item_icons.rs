//! Item icons must reach every game view (feature 016).

mod common;

use axum::Router;
use axum::http::StatusCode;
use serde_json::{Value, json};

use common::{
    call, get_state, gm_action_request, json_request, seeded_shared_db, test_app,
    valid_session_body,
};

async fn create_session(app: &Router, body: &Value) -> String {
    let (status, created) = call(app, json_request("POST", "/api/sessions", Some(body))).await;
    assert_eq!(status, StatusCode::CREATED, "session created: {created}");
    created["sessionId"]
        .as_str()
        .expect("a session id")
        .to_string()
}

async fn start_round(app: &Router, session_id: &str) {
    let (status, body) = call(
        app,
        gm_action_request(
            &format!("/api/sessions/{session_id}/actions"),
            &json!({ "action": "next_round" }),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "round started: {body}");
}

#[tokio::test]
async fn order_items_carry_the_list_icon() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);
    let session_id = create_session(&app, &valid_session_body()).await;
    start_round(&app, &session_id).await;

    let (_, state) = get_state(&app, &session_id, "p1").await;
    let pool = state["order"]["pool"].as_array().expect("a pool");
    assert!(!pool.is_empty(), "the pool has items");
    for item in pool {
        assert!(
            item["iconName"].is_string(),
            "order item {} is missing its icon: {item}",
            item["name"]
        );
    }
    assert!(
        state["order"]["anchor"]["iconName"].is_string(),
        "the anchor is missing its icon: {}",
        state["order"]["anchor"]
    );
}

#[tokio::test]
async fn tripmine_items_carry_the_list_icon() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);
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
    start_round(&app, &session_id).await;

    let (_, state) = get_state(&app, &session_id, "p1").await;
    let items = state["tripmine"]["items"]
        .as_array()
        .expect("tripmine items");
    assert!(!items.is_empty(), "the grid has items");
    for item in items {
        assert!(
            item["iconName"].is_string(),
            "tripmine item {} is missing its icon: {item}",
            item["name"]
        );
    }
}
