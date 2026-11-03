use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

use crate::app_state::AppState;
use crate::errors::{ApiError, ApiResult};
use crate::modules::csv_import;
use crate::modules::default_lists::{self, ListSummaries};
use crate::modules::folders;
use crate::modules::lists::{self, ListDetail, ListInput};
use crate::routes::folders::header_password;

/// `GET /api/default-lists` — the read-only default folder needs no password
/// (FR-050), which is what lets the setup wizard offer lists before any folder
/// has been created.
async fn default_lists(State(state): State<AppState>) -> ApiResult<Json<ListSummaries>> {
    let summaries =
        state.with_db(|conn| default_lists::list_summaries(conn, folders::DEFAULT_FOLDER_ID))?;
    Ok(Json(summaries))
}

/// `GET /api/folders/{folderId}/lists`. A protected folder must present its
/// password, and a wrong one is refused before any title is read, so a locked
/// folder never leaks a list name (FR-047).
async fn folder_lists(
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Json<ListSummaries>> {
    let attempt = header_password(&headers);
    let summaries = state.with_db(|conn| {
        if !folders::password_matches(conn, &folder_id, &attempt)? {
            return Err(ApiError::unauthorized());
        }
        default_lists::list_summaries(conn, &folder_id)
    })?;
    Ok(Json(summaries))
}

/// A list's folder password gate, shared by read, write, and export.
fn require_list_password(
    conn: &rusqlite::Connection,
    list_id: &str,
    headers: &HeaderMap,
) -> ApiResult<()> {
    let folder_id = lists::folder_id(conn, list_id)?;
    if !folders::password_matches(conn, &folder_id, &header_password(headers))? {
        return Err(ApiError::unauthorized());
    }
    Ok(())
}

async fn read_list(
    State(state): State<AppState>,
    Path(list_id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<Json<ListDetail>> {
    let detail = state.with_db(|conn| {
        require_list_password(conn, &list_id, &headers)?;
        lists::read(conn, &list_id)
    })?;
    Ok(Json(detail))
}

async fn create_list(
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
    headers: HeaderMap,
    Json(input): Json<ListInput>,
) -> ApiResult<(StatusCode, Json<ListDetail>)> {
    let detail = state.with_db(|conn| {
        if !folders::password_matches(conn, &folder_id, &header_password(&headers))? {
            return Err(ApiError::unauthorized());
        }
        lists::create(conn, &folder_id, &input)
    })?;
    Ok((StatusCode::CREATED, Json(detail)))
}

async fn update_list(
    State(state): State<AppState>,
    Path(list_id): Path<String>,
    headers: HeaderMap,
    Json(input): Json<ListInput>,
) -> ApiResult<Json<ListDetail>> {
    let detail = state.with_db(|conn| {
        require_list_password(conn, &list_id, &headers)?;
        lists::update(conn, &list_id, &input)
    })?;
    Ok(Json(detail))
}

async fn delete_list(
    State(state): State<AppState>,
    Path(list_id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<StatusCode> {
    state.with_db(|conn| {
        require_list_password(conn, &list_id, &headers)?;
        lists::delete(conn, &list_id)
    })?;
    Ok(StatusCode::NO_CONTENT)
}

/// `GET /api/lists/{id}/export.csv` (FR-058).
async fn export_list(
    State(state): State<AppState>,
    Path(list_id): Path<String>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    let csv = state.with_db(|conn| {
        require_list_password(conn, &list_id, &headers)?;
        let detail = lists::read(conn, &list_id)?;
        let column_names: Vec<String> = detail.columns.iter().map(|c| c.name.clone()).collect();
        let items: Vec<csv_import::ExportItem> = detail
            .items
            .iter()
            .map(|item| csv_import::ExportItem {
                icon_name: item.icon_name.clone(),
                name: item.name.clone(),
                values: detail
                    .columns
                    .iter()
                    .map(|column| {
                        item.values
                            .get(&column.id)
                            .map(|value| value.display.clone())
                            .unwrap_or_default()
                    })
                    .collect(),
            })
            .collect();
        csv_import::export_csv(&column_names, &items)
    })?;

    Ok((
        [
            (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{list_id}.csv\""),
            ),
        ],
        csv,
    ))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportRequest {
    title: String,
    csv: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImportResponse {
    list: ListDetail,
    report: csv_import::ImportReport,
}

/// `POST /api/folders/{id}/lists/import` (FR-057, FR-059).
async fn import_list(
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<ImportRequest>,
) -> ApiResult<(StatusCode, Json<ImportResponse>)> {
    let (detail, report) = state.with_db(|conn| {
        if !folders::password_matches(conn, &folder_id, &header_password(&headers))? {
            return Err(ApiError::unauthorized());
        }
        let parsed = csv_import::parse_csv(&request.title, &request.csv)?;
        let detail = lists::create_from_import(conn, &folder_id, &parsed)?;
        Ok::<_, ApiError>((detail, parsed.report))
    })?;
    Ok((
        StatusCode::CREATED,
        Json(ImportResponse {
            list: detail,
            report,
        }),
    ))
}

pub fn mount(api: axum::Router<AppState>) -> axum::Router<AppState> {
    api.route("/default-lists", axum::routing::get(default_lists))
        .route(
            "/folders/{folder_id}/lists",
            axum::routing::get(folder_lists).post(create_list),
        )
        .route(
            "/folders/{folder_id}/lists/import",
            axum::routing::post(import_list),
        )
        .route(
            "/lists/{list_id}",
            axum::routing::get(read_list)
                .put(update_list)
                .delete(delete_list),
        )
        .route(
            "/lists/{list_id}/export.csv",
            axum::routing::get(export_list),
        )
}
