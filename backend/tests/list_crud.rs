//! List CRUD, export, and import over HTTP (T117).

mod common;

use axum::Router;
use axum::http::StatusCode;
use serde_json::{Value, json};

use common::{call, call_text, error_code, password_request, seeded_shared_db, test_app};

const PASSWORD: &str = "pw";

fn list_body(title: &str) -> Value {
    let items: Vec<Value> = (0..11)
        .map(|index| {
            json!({
                "name": format!("Item {index}"),
                "iconName": "Tag",
                "position": index,
                "values": { "year": { "display": (1900 + index).to_string(), "numeric": 1900 + index } }
            })
        })
        .collect();
    json!({
        "title": title,
        "description": "A test list",
        "orderDirection": "low_to_high",
        "columns": [{ "id": "year", "name": "Year" }],
        "items": items,
    })
}

async fn create_folder(app: &Router) -> String {
    let (status, body) = call(
        app,
        common::json_request(
            "POST",
            "/api/folders",
            Some(&json!({ "name": "Facts", "password": PASSWORD })),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "folder created: {body}");
    body["id"].as_str().expect("a folder id").to_string()
}

#[tokio::test]
async fn a_list_round_trips_through_create_read_update_delete() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);
    let folder_id = create_folder(&app).await;

    let (status, created) = call(
        &app,
        password_request(
            "POST",
            &format!("/api/folders/{folder_id}/lists"),
            Some(&list_body("Facts list")),
            PASSWORD,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "list created: {created}");
    let list_id = created["id"].as_str().expect("a list id").to_string();
    assert_eq!(created["itemCount"], json!(11));
    assert_eq!(created["items"].as_array().expect("items").len(), 11);

    let (status, read) = call(
        &app,
        password_request("GET", &format!("/api/lists/{list_id}"), None, PASSWORD),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(read["folderId"], json!(folder_id));
    assert_eq!(read["items"][0]["values"]["year"]["display"], json!("1900"));

    let mut updated = list_body("Facts list (edited)");
    updated["items"][0]["position"] = json!(3);
    let (status, after) = call(
        &app,
        password_request(
            "PUT",
            &format!("/api/lists/{list_id}"),
            Some(&updated),
            PASSWORD,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "updated: {after}");
    assert_eq!(after["title"], json!("Facts list (edited)"));
    let moved = after["items"]
        .as_array()
        .expect("items")
        .iter()
        .find(|item| item["name"] == json!("Item 0"))
        .expect("the moved item");
    assert_eq!(moved["position"], json!(3));

    let (status, _) = call(
        &app,
        password_request("DELETE", &format!("/api/lists/{list_id}"), None, PASSWORD),
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (status, _) = call(
        &app,
        password_request("GET", &format!("/api/lists/{list_id}"), None, PASSWORD),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_protected_list_needs_the_folder_password() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);
    let folder_id = create_folder(&app).await;

    let (_, created) = call(
        &app,
        password_request(
            "POST",
            &format!("/api/folders/{folder_id}/lists"),
            Some(&list_body("Secret")),
            PASSWORD,
        ),
    )
    .await;
    let list_id = created["id"].as_str().expect("a list id").to_string();

    let (status, body) = call(
        &app,
        password_request("GET", &format!("/api/lists/{list_id}"), None, "wrong"),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error_code(&body), "unauthorized");
}

#[tokio::test]
async fn the_default_folder_rejects_list_writes() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);

    let (status, body) = call(
        &app,
        common::json_request(
            "POST",
            "/api/folders/default/lists",
            Some(&list_body("Nope")),
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error_code(&body), "conflict");
}

#[tokio::test]
async fn export_returns_csv_and_import_returns_the_list_and_report() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);
    let folder_id = create_folder(&app).await;

    let (_, created) = call(
        &app,
        password_request(
            "POST",
            &format!("/api/folders/{folder_id}/lists"),
            Some(&list_body("Exportable")),
            PASSWORD,
        ),
    )
    .await;
    let list_id = created["id"].as_str().expect("a list id").to_string();

    let (status, csv, content_type) = call_text(
        &app,
        password_request(
            "GET",
            &format!("/api/lists/{list_id}/export.csv"),
            None,
            PASSWORD,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!(content_type.unwrap_or_default().starts_with("text/csv"));
    assert!(csv.starts_with("Icon,Name,Year"));

    let mut body = String::from("Icon,Name,Year\n");
    for index in 0..11 {
        body.push_str(&format!("Tag,Imported {index},{}\n", 2000 + index));
    }
    let (status, imported) = call(
        &app,
        password_request(
            "POST",
            &format!("/api/folders/{folder_id}/lists/import"),
            Some(&json!({ "title": "Imported", "orderDirection": "low_to_high", "csv": body })),
            PASSWORD,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED, "imported: {imported}");
    assert_eq!(imported["list"]["itemCount"], json!(11));
    assert_eq!(imported["report"]["rowCount"], json!(11));
    assert_eq!(
        imported["report"]["defaultedRows"]
            .as_array()
            .expect("rows")
            .len(),
        0
    );
}

#[tokio::test]
async fn a_ten_row_import_is_rejected_naming_the_range() {
    let (server_db, _side) = seeded_shared_db();
    let app = test_app(server_db);
    let folder_id = create_folder(&app).await;

    let mut body = String::from("Icon,Name,Year\n");
    for index in 0..10 {
        body.push_str(&format!("Tag,Item {index},{}\n", 2000 + index));
    }
    let (status, answer) = call(
        &app,
        password_request(
            "POST",
            &format!("/api/folders/{folder_id}/lists/import"),
            Some(&json!({ "title": "Too small", "orderDirection": "low_to_high", "csv": body })),
            PASSWORD,
        ),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(error_code(&answer), "too_small");
}
