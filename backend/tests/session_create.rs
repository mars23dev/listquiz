//! Session creation over HTTP (US1, T028-T031).
//!
//! These tests were written before the handler existed and failed with 404s; the
//! rules they pin down come straight from spec.md FR-005 to FR-010.

mod common;

use axum::Router;
use rusqlite::Connection;
use serde_json::{Value, json};

use common::{
    call, error_code, get_state, json_request, seeded_db, seeded_shared_db, test_app,
    valid_session_body,
};

const FOLDER_PASSWORD: &str = "hunter2";

async fn create(app: &Router, body: Value) -> (axum::http::StatusCode, Value) {
    call(app, json_request("POST", "/api/sessions", Some(&body))).await
}

async fn create_folder(app: &Router, name: &str, password: &str) -> String {
    let (status, body) = call(
        app,
        json_request(
            "POST",
            "/api/folders",
            Some(&json!({ "name": name, "password": password })),
        ),
    )
    .await;
    assert_eq!(status, 201, "folder created: {body}");
    body["id"].as_str().expect("a folder id").to_string()
}

fn plant_items_with_values(conn: &Connection, list_id: &str, column_id: &str, count: usize) {
    for index in 0..count {
        let item_id = format!("{list_id}-itm-{index}");
        conn.execute(
            "INSERT INTO item (id, list_id, name, position, icon_name)
             VALUES (?1, ?2, ?3, ?4, NULL)",
            rusqlite::params![item_id, list_id, format!("Item {index}"), index as i64],
        )
        .expect("plant item");
        conn.execute(
            "INSERT INTO item_value (item_id, column_id, display, numeric, was_defaulted)
             VALUES (?1, ?2, ?3, ?4, 0)",
            rusqlite::params![item_id, column_id, index.to_string(), index as f64],
        )
        .expect("plant value");
    }
}

fn plant_custom_list(conn: &Connection, folder_id: &str, list_id: &str, column_id: &str) {
    plant_list(conn, folder_id, list_id, column_id, 11);
}

fn plant_list(conn: &Connection, folder_id: &str, list_id: &str, column_id: &str, count: usize) {
    conn.execute(
        "INSERT INTO quiz_list
            (id, folder_id, title, description, item_count, is_default, created_at, updated_at)
         VALUES (?1, ?2, 'Test', '', ?3, 0, 0, 0)",
        rusqlite::params![list_id, folder_id, count as i64],
    )
    .expect("plant list");
    conn.execute(
        "INSERT INTO list_item_column (list_id, id, name, position)
         VALUES (?1, ?2, 'Value', 0)",
        rusqlite::params![list_id, column_id],
    )
    .expect("plant column");
    plant_items_with_values(conn, list_id, column_id, count);
}

fn one_round_body(
    list_id: &str,
    column_id: &str,
    quiz_type: &str,
    tripmine_slots: Option<i32>,
) -> Value {
    let mut round = json!({
        "roundNumber": 1,
        "quizType": quiz_type,
        "listId": list_id,
        "columnId": column_id
    });
    if quiz_type == "TRIPMINE" {
        round["tripmineSide"] = json!("highest");
        if let Some(slots) = tripmine_slots {
            round["tripmineSlots"] = json!(slots);
        }
    }
    json!({
        "gmPassword": "letmein",
        "teamCount": 1,
        "teams": [{ "name": "Player" }],
        "roundCount": 1,
        "rounds": [round]
    })
}

fn folder_passwords(entries: &[(&str, &str)]) -> Value {
    let mut map = serde_json::Map::new();
    for (folder_id, password) in entries {
        map.insert((*folder_id).to_string(), json!(*password));
    }
    Value::Object(map)
}

fn custom_session_body(list_id: &str, column_id: &str, passwords: Value) -> Value {
    json!({
        "gmPassword": "letmein",
        "teamCount": 2,
        "teams": [{ "name": "Team Cyan Pulsar" }, { "name": "Team Ruby Spark" }],
        "roundCount": 1,
        "rounds": [
            { "roundNumber": 1, "quizType": "ORDER", "listId": list_id, "columnId": column_id }
        ],
        "folderPasswords": passwords
    })
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

#[tokio::test]
async fn a_custom_list_session_needs_and_accepts_its_folder_password() {
    let (server_db, side) = seeded_shared_db();
    let app = test_app(server_db);
    let folder_id = create_folder(&app, "Family facts", FOLDER_PASSWORD).await;
    plant_custom_list(&side, &folder_id, "lst-test", "value");

    let (status, response) = create(
        &app,
        custom_session_body(
            "lst-test",
            "value",
            folder_passwords(&[(&folder_id, FOLDER_PASSWORD)]),
        ),
    )
    .await;

    assert_eq!(status, 201, "custom-list session created: {response}");
    let session_id = response["sessionId"].as_str().expect("a session id");
    let (list_id, column_id): (String, String) = side
        .query_row(
            "SELECT list_id, column_id FROM round_config WHERE session_id = ?1",
            [session_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .expect("the configured round");
    assert_eq!(list_id, "lst-test");
    assert_eq!(column_id, "value");
}

#[tokio::test]
async fn a_locked_custom_list_is_refused_without_proof() {
    let (server_db, side) = seeded_shared_db();
    let app = test_app(server_db);
    let folder_id = create_folder(&app, "Family facts", FOLDER_PASSWORD).await;
    plant_custom_list(&side, &folder_id, "lst-test", "value");

    let mut missing = custom_session_body("lst-test", "value", json!({}));
    missing.as_object_mut().unwrap().remove("folderPasswords");
    let (status, answer) = create(&app, missing).await;
    assert_eq!(status, 400, "missing proof: {answer}");
    assert_eq!(error_code(&answer), "unauthorized");
    assert_eq!(answer["error"]["message"], json!("Incorrect password!"));

    let (status, answer) = create(
        &app,
        custom_session_body(
            "lst-test",
            "value",
            folder_passwords(&[(&folder_id, "wrong")]),
        ),
    )
    .await;
    assert_eq!(status, 400, "wrong proof: {answer}");
    assert_eq!(error_code(&answer), "unauthorized");

    let sessions: i64 = side
        .query_row("SELECT COUNT(*) FROM game_session", [], |row| row.get(0))
        .expect("session count");
    assert_eq!(sessions, 0, "a refused request writes no session");
}

#[tokio::test]
async fn two_unlocked_folders_can_supply_two_rounds_but_one_cannot_vouch_for_the_other() {
    let (server_db, side) = seeded_shared_db();
    let app = test_app(server_db);
    let family = create_folder(&app, "Family facts", "family-pw").await;
    let work = create_folder(&app, "Work trivia", "work-pw").await;
    plant_custom_list(&side, &family, "lst-family", "value");
    plant_custom_list(&side, &work, "lst-work", "value");

    let body = json!({
        "gmPassword": "letmein",
        "teamCount": 2,
        "teams": [{ "name": "Team Cyan Pulsar" }, { "name": "Team Ruby Spark" }],
        "roundCount": 2,
        "rounds": [
            { "roundNumber": 1, "quizType": "ORDER", "listId": "lst-family", "columnId": "value" },
            { "roundNumber": 2, "quizType": "ORDER", "listId": "lst-work", "columnId": "value" }
        ],
        "folderPasswords": folder_passwords(&[
            (&family, "family-pw"),
            (&work, "work-pw")
        ])
    });
    let (status, response) = create(&app, body).await;
    assert_eq!(status, 201, "both folders unlocked: {response}");

    let partial = json!({
        "gmPassword": "letmein",
        "teamCount": 2,
        "teams": [{ "name": "Team Cyan Pulsar" }, { "name": "Team Ruby Spark" }],
        "roundCount": 2,
        "rounds": [
            { "roundNumber": 1, "quizType": "ORDER", "listId": "lst-family", "columnId": "value" },
            { "roundNumber": 2, "quizType": "ORDER", "listId": "lst-work", "columnId": "value" }
        ],
        "folderPasswords": folder_passwords(&[(&family, "family-pw")])
    });
    let (status, answer) = create(&app, partial).await;
    assert_eq!(status, 400, "one password cannot unlock both: {answer}");
    assert_eq!(error_code(&answer), "unauthorized");

    let sessions: i64 = side
        .query_row("SELECT COUNT(*) FROM game_session", [], |row| row.get(0))
        .expect("session count");
    assert_eq!(sessions, 1, "only the fully proven request was written");
}

#[tokio::test]
async fn capacity_minimums_are_enforced_for_every_mode() {
    let cases = [
        ("ORDER", 10, 11, None),
        ("CONNECT", 9, 10, None),
        ("TRIPMINE", 9, 10, Some(10)),
        ("TRIPMINE", 19, 20, Some(20)),
    ];

    for (mode, short_count, exact_count, slots) in cases {
        let short_id = format!("lst-short-{mode}-{short_count}");
        let conn = seeded_db();
        plant_list(&conn, "default", &short_id, "value", short_count);
        let app = test_app(conn);

        let (status, body) = create(&app, one_round_body(&short_id, "value", mode, slots)).await;
        assert_eq!(status, 400, "{mode} short: {body}");
        assert_eq!(error_code(&body), "bad_request");
        let message = body["error"]["message"].as_str().unwrap_or_default();
        assert!(
            message.contains(&format!("at least {exact_count}")),
            "{mode} message: {message}"
        );

        let exact_id = format!("lst-exact-{mode}-{exact_count}");
        let conn = seeded_db();
        plant_list(&conn, "default", &exact_id, "value", exact_count);
        let app = test_app(conn);

        let (status, body) = create(&app, one_round_body(&exact_id, "value", mode, slots)).await;
        assert_eq!(status, 201, "{mode} exact minimum: {body}");
    }
}

#[tokio::test]
async fn a_short_round_writes_no_session_teams_or_rounds() {
    let (server_db, side) = seeded_shared_db();
    plant_list(&server_db, "default", "lst-short", "value", 19);
    let app = test_app(server_db);

    let (status, body) = create(
        &app,
        one_round_body("lst-short", "value", "TRIPMINE", Some(20)),
    )
    .await;

    assert_eq!(status, 400, "{body}");
    for table in ["game_session", "team", "round_config"] {
        let count: i64 = side
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .expect("count");
        assert_eq!(count, 0, "{table} must stay empty");
    }
}

#[tokio::test]
async fn one_short_round_refuses_the_whole_multi_round_session() {
    let (server_db, side) = seeded_shared_db();
    plant_list(&server_db, "default", "lst-short", "value", 9);
    let app = test_app(server_db);
    let mut body = valid_session_body();
    body["rounds"][1]["listId"] = json!("lst-short");
    body["rounds"][1]["columnId"] = json!("value");
    body["rounds"][1]["tripmineSlots"] = json!(10);

    let (status, answer) = create(&app, body).await;

    assert_eq!(status, 400, "{answer}");
    assert_eq!(error_code(&answer), "bad_request");
    for table in ["game_session", "team", "round_config"] {
        let count: i64 = side
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                row.get(0)
            })
            .expect("count");
        assert_eq!(count, 0, "{table} must stay empty");
    }
}

#[tokio::test]
async fn items_without_a_value_in_the_selected_column_do_not_count() {
    let (server_db, side) = seeded_shared_db();
    plant_list(&server_db, "default", "lst-missing", "value", 20);
    side.execute(
        "DELETE FROM item_value WHERE item_id = 'lst-missing-itm-0'",
        [],
    )
    .expect("remove one value");
    let app = test_app(server_db);

    let (status, body) = create(
        &app,
        one_round_body("lst-missing", "value", "TRIPMINE", Some(20)),
    )
    .await;

    assert_eq!(status, 400, "{body}");
    let message = body["error"]["message"].as_str().unwrap_or_default();
    assert!(message.contains("list has 19"), "{message}");
}
