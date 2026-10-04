use axum::Json;
use axum::extract::{Path, State};
use axum::http::HeaderMap;

use crate::app_state::AppState;
use crate::errors::{ApiError, ApiResult};
use crate::modules::default_lists::{self, ListSummaries};
use crate::modules::folders;
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

pub fn mount(api: axum::Router<AppState>) -> axum::Router<AppState> {
    api.route("/default-lists", axum::routing::get(default_lists))
        .route(
            "/folders/{folder_id}/lists",
            axum::routing::get(folder_lists),
        )
}
