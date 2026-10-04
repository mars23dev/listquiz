use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use serde::Serialize;

use crate::app_state::AppState;
use crate::errors::ApiResult;
use crate::modules::folders::{
    self, CreateFolderRequest, DeleteFolderRequest, FolderView, RenameFolderRequest,
    UnlockFolderRequest,
};

/// The header every protected-folder write carries its password in.
const PASSWORD_HEADER: &str = "x-quiz-password";

pub fn header_password(headers: &HeaderMap) -> String {
    headers
        .get(PASSWORD_HEADER)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default()
        .to_string()
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreatedFolder {
    id: String,
    name: String,
    is_default: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UnlockedFolder {
    name: String,
}

async fn create_folder(
    State(state): State<AppState>,
    Json(request): Json<CreateFolderRequest>,
) -> ApiResult<(StatusCode, Json<CreatedFolder>)> {
    let created = state.with_db(|conn| -> ApiResult<CreatedFolder> {
        let id = folders::create_folder(conn, &request)?;
        Ok(CreatedFolder {
            id,
            name: request.name.trim().to_string(),
            is_default: false,
        })
    })?;
    Ok((StatusCode::CREATED, Json(created)))
}

async fn list_folders(State(state): State<AppState>) -> ApiResult<Json<Vec<FolderView>>> {
    let folders = state.with_db(folders::list_folders)?;
    Ok(Json(folders))
}

async fn unlock_folder(
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
    Json(request): Json<UnlockFolderRequest>,
) -> ApiResult<Json<UnlockedFolder>> {
    let name = state.with_db(|conn| folders::unlock(conn, &folder_id, &request))?;
    Ok(Json(UnlockedFolder { name }))
}

async fn rename_folder(
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<RenameFolderRequest>,
) -> ApiResult<Json<FolderView>> {
    let attempt = header_password(&headers);
    Ok(Json(state.with_db(|conn| {
        folders::rename(conn, &folder_id, &attempt, &request)
    })?))
}

async fn delete_folder(
    State(state): State<AppState>,
    Path(folder_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<DeleteFolderRequest>,
) -> ApiResult<StatusCode> {
    let attempt = header_password(&headers);
    state.with_db(|conn| folders::delete(conn, &folder_id, &attempt, &request))?;
    Ok(StatusCode::NO_CONTENT)
}

pub fn mount(api: axum::Router<AppState>) -> axum::Router<AppState> {
    api.route(
        "/folders",
        axum::routing::get(list_folders).post(create_folder),
    )
    .route(
        "/folders/{folder_id}/unlock",
        axum::routing::post(unlock_folder),
    )
    .route(
        "/folders/{folder_id}/rename",
        axum::routing::post(rename_folder),
    )
    .route(
        "/folders/{folder_id}/delete",
        axum::routing::post(delete_folder),
    )
}
