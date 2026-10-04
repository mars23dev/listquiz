mod common;

use axum::Router;
use axum::http::{Request, StatusCode};
use rusqlite::Connection;
use serde_json::{Value, json};

use common::{call, error_code, json_request, seeded_shared_db, test_app};

/// A password-carrying request, because rename and delete take theirs in the
/// `X-Quiz-Password` header.
fn with_password(
    method: &str,
    path: &str,
    password: &str,
    body: Option<&Value>,
) -> Request<axum::body::Body> {
    let mut builder = axum::http::Request::builder()
        .method(method)
        .uri(path)
        .header("X-Quiz-Password", password);
    if let Some(value) = body {
        builder = builder.header("Content-Type", "application/json");
        return builder
            .body(axum::body::Body::from(value.to_string()))
            .unwrap();
    }
    builder.body(axum::body::Body::empty()).unwrap()
}

/// Create a folder and return its id.
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

/// Put a list into `folder_id` directly, so folder tests do not depend on the
/// list-creation story landing first.
fn plant_list(conn: &Connection, folder_id: &str, list_id: &str, title: &str) {
    conn.execute(
        "INSERT INTO quiz_list
            (id, folder_id, title, description, order_direction, item_count, is_default, created_at, updated_at)
         VALUES (?1, ?2, ?3, '', 'low_to_high', 0, 0, 0, 0)",
        rusqlite::params![list_id, folder_id, title],
    )
    .expect("plant list");
}

#[tokio::test]
async fn creating_a_folder_without_a_name_or_password_is_rejected() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);

    for body in [
        json!({ "name": "", "password": "hunter2" }),
        json!({ "name": "   ", "password": "hunter2" }),
        json!({ "name": "Family facts", "password": "" }),
        json!({ "name": "Family facts", "password": "  " }),
    ] {
        let (status, answer) = call(&app, json_request("POST", "/api/folders", Some(&body))).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body} -> {answer}");
        assert_eq!(error_code(&answer), "bad_request");
    }
}

#[tokio::test]
async fn a_duplicate_folder_name_conflicts() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);

    let (status, _) = call(
        &app,
        json_request(
            "POST",
            "/api/folders",
            Some(&json!({ "name": "Family facts", "password": "hunter2" })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);

    // Names are compared after trimming.
    let (status, answer) = call(
        &app,
        json_request(
            "POST",
            "/api/folders",
            Some(&json!({ "name": "  Family facts  ", "password": "other" })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{answer}");
    assert_eq!(error_code(&answer), "conflict");
}

#[tokio::test]
async fn a_wrong_folder_password_reveals_nothing() {
    let (server_db, side) = seeded_shared_db();
    let app = test_app(server_db);
    let folder_id = create_folder(&app, "Family facts", "hunter2").await;
    plant_list(&side, &folder_id, "lst-secret", "Grandma's recipes");

    let (status, answer) = call(
        &app,
        json_request(
            "POST",
            &format!("/api/folders/{folder_id}/unlock"),
            Some(&json!({ "password": "wrong" })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST, "{answer}");
    assert_eq!(error_code(&answer), "unauthorized");
    assert_eq!(answer["error"]["message"], json!("Incorrect password!"));
    // Nothing about the folder leaks through the refusal.
    assert_eq!(answer.get("lists"), None);
    assert_eq!(answer.get("name"), None);

    let (status, _answer) = call(
        &app,
        with_password(
            "POST",
            &format!("/api/folders/{folder_id}/rename"),
            "wrong",
            Some(&json!({ "name": "Renamed" })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn the_default_folder_rejects_every_write() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);

    let (status, answer) = call(
        &app,
        json_request(
            "POST",
            "/api/folders/default/rename",
            Some(&json!({ "name": "Renamed" })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "rename: {answer}");
    assert_eq!(error_code(&answer), "conflict");

    let (status, _) = call(
        &app,
        json_request(
            "POST",
            "/api/folders/default/delete",
            Some(&json!({ "confirmPassword": "" })),
        ),
    )
    .await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "delete must refuse the default folder"
    );
}

#[tokio::test]
async fn listing_folders_marks_unlocked_by_nobody_ones_locked() {
    let (server_db, side) = seeded_shared_db();
    let app = test_app(server_db);
    let folder_id = create_folder(&app, "Family facts", "hunter2").await;
    plant_list(&side, &folder_id, "lst-secret", "Grandma's recipes");

    let (status, body) = call(&app, json_request("GET", "/api/folders", None)).await;
    assert_eq!(status, StatusCode::OK);

    let folders = body.as_array().expect("an array of folders");
    let default_folder = folders
        .iter()
        .find(|folder| folder["id"] == json!("default"))
        .expect("the seeded default folder");
    assert_eq!(default_folder["isDefault"], json!(true));
    assert_eq!(default_folder["locked"], json!(false));

    let protected = folders
        .iter()
        .find(|folder| folder["id"] == json!(folder_id.as_str()))
        .expect("the folder we created");
    assert_eq!(protected["locked"], json!(true));

    // No list name for the locked folder, anywhere in the payload.
    let serialized = body.to_string();
    assert!(
        !serialized.contains("Grandma"),
        "a locked folder must not leak list names: {serialized}"
    );
    assert!(!serialized.contains("lst-secret"));

    // Unlocking is what flips it, and it still returns only the folder name.
    let (status, unlocked) = call(
        &app,
        json_request(
            "POST",
            &format!("/api/folders/{folder_id}/unlock"),
            Some(&json!({ "password": "hunter2" })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{unlocked}");
    assert_eq!(unlocked["name"], json!("Family facts"));
    assert_eq!(unlocked.get("lists"), None);
}

#[tokio::test]
async fn renaming_to_an_existing_name_conflicts() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);
    let first = create_folder(&app, "Family facts", "hunter2").await;
    create_folder(&app, "Work trivia", "letmein").await;

    let (status, answer) = call(
        &app,
        with_password(
            "POST",
            &format!("/api/folders/{first}/rename"),
            "hunter2",
            Some(&json!({ "name": "Work trivia" })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT, "{answer}");
    assert_eq!(error_code(&answer), "conflict");

    // Renaming to its own current name is still a conflict-free no-op path, but a
    // rename to a free name works.
    let (status, renamed) = call(
        &app,
        with_password(
            "POST",
            &format!("/api/folders/{first}/rename"),
            "hunter2",
            Some(&json!({ "name": "Family & friends" })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{renamed}");
    assert_eq!(renamed["name"], json!("Family & friends"));
}

#[tokio::test]
async fn the_default_folder_is_read_only_and_seeded() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);

    let (status, body) = call(&app, json_request("GET", "/api/folders", None)).await;
    assert_eq!(status, StatusCode::OK);

    let default_folder = body
        .as_array()
        .unwrap()
        .iter()
        .find(|folder| folder["id"] == json!("default"))
        .expect("the seeded default folder");
    assert_eq!(default_folder["name"], json!("Default Lists"));

    // Unlocking the default folder needs no password at all (FR-050).
    let (status, unlocked) = call(
        &app,
        json_request(
            "POST",
            "/api/folders/default/unlock",
            Some(&json!({ "password": "" })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{unlocked}");
}
