mod common;

use axum::Router;
use axum::http::StatusCode;
use rusqlite::Connection;
use serde_json::{Value, json};

use common::{call, json_request, seeded_shared_db, test_app};

fn with_password(
    method: &str,
    path: &str,
    password: &str,
    body: Option<&Value>,
) -> axum::http::Request<axum::body::Body> {
    let mut builder = axum::http::Request::builder()
        .method(method)
        .uri(path)
        .header("X-Quiz-Password", password);
    if body.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }
    match body {
        Some(value) => builder
            .body(axum::body::Body::from(value.to_string()))
            .unwrap(),
        None => builder.body(axum::body::Body::empty()).unwrap(),
    }
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
    assert_eq!(status, StatusCode::CREATED, "{body}");
    body["id"].as_str().unwrap().to_string()
}

fn plant_list(conn: &Connection, folder_id: &str, list_id: &str, title: &str) {
    conn.execute(
        "INSERT INTO quiz_list
            (id, folder_id, title, description, order_direction, item_count, is_default, created_at, updated_at)
         VALUES (?1, ?2, ?3, '', 'low_to_high', 0, 0, 0, 0)",
        rusqlite::params![list_id, folder_id, title],
    )
    .expect("plant list");
}

fn list_titles(conn: &Connection, folder_id: &str) -> Vec<String> {
    let mut statement = conn
        .prepare("SELECT title FROM quiz_list WHERE folder_id = ?1 ORDER BY title")
        .expect("statement");
    let rows = statement
        .query_map(rusqlite::params![folder_id], |row| row.get::<_, String>(0))
        .expect("query");
    rows.map(|row| row.expect("row")).collect()
}

#[tokio::test]
async fn deleting_a_folder_needs_the_password_re_entered_as_confirmation() {
    let (server_db, side) = seeded_shared_db();
    let app = test_app(server_db);
    let folder_id = create_folder(&app, "Family facts", "hunter2").await;
    plant_list(&side, &folder_id, "lst-recipes", "Grandma's recipes");
    plant_list(&side, &folder_id, "lst-trips", "Trips we took");

    // A wrong confirmation is refused and the folder keeps its lists.
    let (status, answer) = call(
        &app,
        with_password(
            "POST",
            &format!("/api/folders/{folder_id}/delete"),
            "hunter2",
            Some(&json!({ "confirmPassword": "not-it" })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{answer}");
    assert_eq!(answer["error"]["code"], json!("unauthorized"));
    assert_eq!(list_titles(&side, &folder_id).len(), 2);

    // The header alone is not enough: the body must carry the same password.
    let (status, _) = call(
        &app,
        with_password(
            "POST",
            &format!("/api/folders/{folder_id}/delete"),
            "hunter2",
            Some(&json!({})),
        ),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "a missing confirmation is refused"
    );
    assert_eq!(list_titles(&side, &folder_id).len(), 2);
}

#[tokio::test]
async fn deleting_a_folder_removes_its_lists_and_leaves_everything_else() {
    let (server_db, side) = seeded_shared_db();
    let app = test_app(server_db);
    let doomed = create_folder(&app, "Family facts", "hunter2").await;
    let keeper = create_folder(&app, "Work trivia", "letmein").await;
    plant_list(&side, &doomed, "lst-recipes", "Grandma's recipes");
    plant_list(&side, &doomed, "lst-trips", "Trips we took");
    plant_list(&side, &keeper, "lst-quarterly", "Quarterly numbers");

    // The confirmation prompt has to name the list count (FR-046c).
    let (status, count) = call(
        &app,
        with_password(
            "GET",
            &format!("/api/folders/{doomed}/lists"),
            "hunter2",
            None,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{count}");
    assert_eq!(count["lists"].as_array().unwrap().len(), 2);

    let (status, _answer) = call(
        &app,
        with_password(
            "POST",
            &format!("/api/folders/{doomed}/delete"),
            "hunter2",
            Some(&json!({ "confirmPassword": "hunter2" })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    // Its lists are gone...
    assert!(list_titles(&side, &doomed).is_empty());
    // ...the folder itself is gone...
    let remaining: i64 = side
        .query_row(
            "SELECT COUNT(*) FROM folder WHERE id = ?1",
            rusqlite::params![doomed],
            |row| row.get(0),
        )
        .expect("count");
    assert_eq!(remaining, 0);
    // ...the sibling folder and its list survive...
    assert_eq!(list_titles(&side, &keeper), vec!["Quarterly numbers"]);
    // ...and so does the default folder with all three seeded lists (FR-046d).
    assert_eq!(list_titles(&side, "default").len(), 3);

    // A second delete cannot find the folder any more.
    let (status, _) = call(
        &app,
        with_password(
            "POST",
            &format!("/api/folders/{doomed}/delete"),
            "hunter2",
            Some(&json!({ "confirmPassword": "hunter2" })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_deleted_folders_items_are_cascaded_away() {
    let (server_db, side) = seeded_shared_db();
    let app = test_app(server_db);
    let folder_id = create_folder(&app, "Family facts", "hunter2").await;
    plant_list(&side, &folder_id, "lst-recipes", "Grandma's recipes");
    side.execute(
        "INSERT INTO item (id, list_id, name, position, icon_name)
         VALUES ('itm-1', 'lst-recipes', 'Apple pie', 0, NULL)",
        [],
    )
    .expect("plant item");

    let (status, _) = call(
        &app,
        with_password(
            "POST",
            &format!("/api/folders/{folder_id}/delete"),
            "hunter2",
            Some(&json!({ "confirmPassword": "hunter2" })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let items: i64 = side
        .query_row(
            "SELECT COUNT(*) FROM item WHERE list_id = 'lst-recipes'",
            [],
            |row| row.get(0),
        )
        .expect("count");
    assert_eq!(items, 0, "FR-046d: the cascade must reach items too");
}
