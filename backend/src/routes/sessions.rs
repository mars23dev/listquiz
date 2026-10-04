//! Session endpoints (US1, T037).
//!
//! Every handler follows the same shape: validate, write in one transaction, bump
//! the revision, then answer. The revision bump is what makes a live client
//! refetch, so it happens after the write and before the response.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::app_state::{AppState, SessionLinks, UserRole, build_links};
use crate::errors::{ApiError, ApiResult};
use crate::modules::sessions::{self, CreateSessionRequest, round_state};
use crate::state_reader;

/// The `POST /api/sessions` answer: the new id plus its five role links.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionResponse {
    pub session_id: String,
    pub links: SessionLinks,
}

/// `POST /api/sessions` — create a session and hand back its five role links.
pub async fn create_session(
    State(state): State<AppState>,
    Json(request): Json<CreateSessionRequest>,
) -> ApiResult<(StatusCode, Json<CreateSessionResponse>)> {
    let (session_id, team_count) = state.with_db(|conn| {
        let session_id = sessions::create_session(conn, &request)?;
        let team_count = conn.query_row(
            "SELECT team_count FROM game_session WHERE id = ?1",
            [&session_id],
            |row| row.get::<_, i32>(0),
        )?;
        Ok::<_, ApiError>((session_id, team_count))
    })?;

    Ok((
        StatusCode::CREATED,
        Json(CreateSessionResponse {
            session_id: session_id.clone(),
            links: build_links(&session_id, team_count),
        }),
    ))
}

/// `POST /api/sessions/{id}/unlock` — the gamemaster password gate.
#[derive(Debug, Deserialize)]
pub struct UnlockRequest {
    #[serde(default)]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct UnlockResponse {
    pub ok: bool,
}

pub async fn unlock_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(request): Json<UnlockRequest>,
) -> ApiResult<Json<UnlockResponse>> {
    state.require_session(&session_id)?;
    let matches = state
        .with_db(|conn| sessions::gm_password_matches(conn, &session_id, &request.password))?;
    if !matches {
        return Err(ApiError::unauthorized());
    }
    Ok(Json(UnlockResponse { ok: true }))
}

/// `GET /api/sessions/{id}/state?role=...` — the role-redacted session state.
/// An unknown role is a `not_found`, not a `bad_request`: the contract treats both
/// an unknown role and an unknown session the same way (FR-013).
pub async fn session_state(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<crate::app_state::SessionState>> {
    let role = params.get("role").map(String::as_str).unwrap_or_default();
    let role = UserRole::parse(role)
        .ok_or_else(|| ApiError::not_found(format!("Unknown role '{role}'.")))?;

    let snapshot = state.with_db(|conn| state_reader::read_state(conn, &session_id, role))?;
    Ok(Json(snapshot))
}

/// `POST /api/sessions/{id}/actions` — the gamemaster's action channel.
#[derive(Debug, Deserialize)]
pub struct ActionRequest {
    pub action: String,
    #[serde(default)]
    pub team_id: Option<String>,
    #[serde(default)]
    pub item_id: Option<String>,
    #[serde(default)]
    pub value: Option<f64>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default)]
    pub slot_index: Option<i32>,
    #[serde(default)]
    pub quantity: Option<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResponse {
    pub ok: bool,
    pub revision: i64,
}

pub async fn session_action(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
    Json(request): Json<ActionRequest>,
) -> ApiResult<Json<ActionResponse>> {
    state.require_session(&session_id)?;
    let revision = apply_action(&state, &session_id, &request)?;
    Ok(Json(ActionResponse { ok: true, revision }))
}

/// Run one action and bump the revision if it changed anything.
fn apply_action(state: &AppState, session_id: &str, request: &ActionRequest) -> ApiResult<i64> {
    match request.action.as_str() {
        "next_round" => {
            state.with_db(|conn| start_next_round(conn, session_id))?;
            state.bump_revision(session_id)
        }
        other => Err(ApiError::bad_request(format!("Unknown action '{other}'."))),
    }
}

/// Move the session on to the next configured round. A round whose list or
/// column has gone is reported as `not_ready` rather than half-started (T033).
fn start_next_round(conn: &rusqlite::Connection, session_id: &str) -> ApiResult<()> {
    let current_round: i32 = conn.query_row(
        "SELECT current_round FROM game_session WHERE id = ?1",
        [session_id],
        |row| row.get(0),
    )?;
    let target = current_round + 1;

    let round = round_state::load_round(conn, session_id, target)?;
    let starting_team = round_state::starting_team_id(conn, session_id, target)?;

    let tx = conn.unchecked_transaction()?;
    round_state::clear_round_state(&tx, session_id)?;
    round_state::reset_hearts(&tx, session_id)?;
    round_state::store_round_start(&tx, session_id, &round, &starting_team)?;
    tx.commit()?;
    Ok(())
}

/// Mount the session routes onto the `/api` sub-router.
pub fn mount(api: Router<AppState>) -> Router<AppState> {
    api.route("/sessions", post(create_session))
        .route("/sessions/{session_id}/unlock", post(unlock_session))
        .route("/sessions/{session_id}/state", get(session_state))
        .route("/sessions/{session_id}/actions", post(session_action))
}
