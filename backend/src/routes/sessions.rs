//! Session endpoints (US1, T037).
//!
//! Every handler follows the same shape: validate, write in one transaction, bump
//! the revision, then answer. The revision bump is what makes a live client
//! refetch, so it happens after the write and before the response.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::app_state::{AppState, SessionLinks, UserRole, build_links};
use crate::errors::{ApiError, ApiResult};
use crate::modules::order_round;
use crate::modules::sessions::{self, CreateSessionRequest, round_state};
use crate::modules::tripmine_round;
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
#[serde(rename_all = "camelCase")]
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
    #[serde(default)]
    pub muted: Option<bool>,
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
    headers: HeaderMap,
    Json(request): Json<ActionRequest>,
) -> ApiResult<Json<ActionResponse>> {
    state.require_session(&session_id)?;
    let password = headers
        .get("x-quiz-password")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    let matches =
        state.with_db(|conn| sessions::gm_password_matches(conn, &session_id, password))?;
    if !matches {
        return Err(ApiError::unauthorized());
    }

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
        // FR-024: one flag, honoured by `redact`, for both quiz types.
        "reveal_values" => {
            state.with_db(|conn| order_round::set_reveal_values(conn, session_id, true))?;
            state.bump_revision(session_id)
        }
        action if is_order_action(action) => {
            state.with_db(|conn| apply_order_action(conn, session_id, request))?;
            state.bump_revision(session_id)
        }
        "pick_tripitem" => {
            state.with_db(|conn| apply_tripmine_action(conn, session_id, request))?;
            state.bump_revision(session_id)
        }
        // FR-024b: one flag, honoured by every view's audio.
        "set_muted" => {
            let muted = request
                .muted
                .ok_or_else(|| ApiError::bad_request("muted is required."))?;
            state.with_db(|conn| {
                conn.execute(
                    "UPDATE game_session SET muted = ?2 WHERE id = ?1",
                    rusqlite::params![session_id, i64::from(muted)],
                )
                .map_err(ApiError::from)?;
                Ok::<(), ApiError>(())
            })?;
            state.bump_revision(session_id)
        }
        other => Err(ApiError::bad_request(format!("Unknown action '{other}'."))),
    }
}

fn is_order_action(action: &str) -> bool {
    matches!(
        action,
        "select_item" | "choose_slot" | "cancel_selection" | "return_to_pool"
    )
}

fn require_field<'a>(value: Option<&'a str>, name: &str) -> ApiResult<&'a str> {
    match value.map(str::trim).filter(|v| !v.is_empty()) {
        Some(value) => Ok(value),
        None => Err(ApiError::bad_request(format!("{name} is required."))),
    }
}

/// The board of the running Order round, or an explanation of why there is none.
fn load_required_board(
    conn: &rusqlite::Connection,
    session_id: &str,
) -> ApiResult<order_round::OrderBoard> {
    order_round::load_board(conn, session_id)?
        .ok_or_else(|| ApiError::not_ready("There is no Order round running in this session yet."))
}

/// The team named by the request, which has to be the one on turn: a team cannot
/// place while another team is playing (FR-018).
fn require_team_on_turn(
    conn: &rusqlite::Connection,
    session_id: &str,
    team_id: Option<&str>,
) -> ApiResult<String> {
    let requested = require_field(team_id, "teamId")?.to_string();
    let active: Option<String> = conn
        .query_row(
            "SELECT active_team_id FROM game_session WHERE id = ?1",
            [session_id],
            |row| row.get(0),
        )
        .optional()?;
    let active = active.ok_or_else(|| ApiError::not_ready("No team is on turn."))?;
    if !crate::modules::scoring::is_active_turn(Some(&active), &requested) {
        return Err(ApiError::not_ready(format!(
            "It is {active}'s turn, not {requested}'s."
        )));
    }
    Ok(requested)
}

/// One Order action. `choose_slot` is the placement itself: the contract's action list
/// has no separate confirm step, so choosing the slot scores it (FR-029, FR-030).
fn apply_order_action(
    conn: &rusqlite::Connection,
    session_id: &str,
    request: &ActionRequest,
) -> ApiResult<()> {
    match request.action.as_str() {
        "select_item" => {
            let item_id = require_field(request.item_id.as_deref(), "itemId")?;
            let board = load_required_board(conn, session_id)?;
            let team_id = require_team_on_turn(conn, session_id, request.team_id.as_deref())?;
            let next = board.select_item(item_id, &team_id)?;
            order_round::save_board(conn, session_id, &next)
        }
        "choose_slot" => {
            let side_raw = require_field(request.side.as_deref(), "side")?;
            let side = order_round::BarSide::parse(side_raw)
                .ok_or_else(|| ApiError::bad_request("side must be left or right."))?;
            let slot_index = request
                .slot_index
                .ok_or_else(|| ApiError::bad_request("slotIndex is required."))?;

            let board = load_required_board(conn, session_id)?;
            require_team_on_turn(conn, session_id, request.team_id.as_deref())?;
            let chosen = board.choose_slot(side, slot_index)?;

            let teams = order_round::load_teams(conn, session_id)?;
            let outcome = order_round::confirm_placement(&chosen, &teams)?;
            let round_number: i32 = conn.query_row(
                "SELECT current_round FROM game_session WHERE id = ?1",
                [session_id],
                |row| row.get(0),
            )?;

            let tx = conn.unchecked_transaction()?;
            order_round::save_board(&tx, session_id, &outcome.board)?;
            order_round::save_teams(&tx, session_id, &outcome.teams)?;
            order_round::append_events(&tx, session_id, round_number, &outcome.events)?;
            // The turn passes on after every scored action (FR-018); a round that has
            // ended leaves nobody on turn (FR-032).
            tx.execute(
                "UPDATE game_session SET active_team_id = ?2, last_message = ?3 WHERE id = ?1",
                rusqlite::params![
                    session_id,
                    outcome.next_team_id,
                    if outcome.round_over {
                        Some(format!("Round {round_number} is over."))
                    } else {
                        None
                    }
                ],
            )?;
            tx.commit()?;
            Ok(())
        }
        "cancel_selection" => {
            let board = load_required_board(conn, session_id)?;
            require_team_on_turn(conn, session_id, request.team_id.as_deref())?;
            let next = board.cancel_selection()?;
            order_round::save_board(conn, session_id, &next)
        }
        // FR-033: only the gamemaster undoes a placement, so this one checks no turn.
        "return_to_pool" => {
            let item_id = require_field(request.item_id.as_deref(), "itemId")?;
            let board = load_required_board(conn, session_id)?;
            let next = order_round::return_to_pool(&board, item_id)?;
            order_round::save_board(conn, session_id, &next)
        }
        other => Err(ApiError::bad_request(format!("Unknown action '{other}'."))),
    }
}

/// One Tripmine pick (FR-039 to FR-044). The side is fixed at round start, so a
/// request that tries to change it is refused before any scoring happens (FR-036).
fn apply_tripmine_action(
    conn: &rusqlite::Connection,
    session_id: &str,
    request: &ActionRequest,
) -> ApiResult<()> {
    let item_id = require_field(request.item_id.as_deref(), "itemId")?;
    let round = tripmine_round::load_round(conn, session_id)?.ok_or_else(|| {
        ApiError::not_ready("There is no Tripmine round running in this session yet.")
    })?;
    tripmine_round::ensure_side_unchanged(&round, request.side.as_deref())?;

    // Only the team on turn picks; the gamemaster acts on that team's behalf by
    // naming the active team, and any other team id is refused (FR-044).
    let team_id = require_team_on_turn(conn, session_id, request.team_id.as_deref())?;
    let teams = order_round::load_teams(conn, session_id)?;
    let outcome = tripmine_round::pick_item(&round, item_id, &team_id, &teams)?;

    let round_number: i32 = conn.query_row(
        "SELECT current_round FROM game_session WHERE id = ?1",
        [session_id],
        |row| row.get(0),
    )?;

    let tx = conn.unchecked_transaction()?;
    tripmine_round::save_round(&tx, session_id, &outcome.round)?;
    order_round::save_teams(&tx, session_id, &outcome.teams)?;
    order_round::append_events(&tx, session_id, round_number, &outcome.events)?;
    tx.execute(
        "UPDATE game_session SET active_team_id = ?2, last_message = ?3 WHERE id = ?1",
        rusqlite::params![
            session_id,
            outcome.next_team_id,
            if outcome.round.completed {
                Some(format!("Round {round_number} is over."))
            } else {
                None
            }
        ],
    )?;
    tx.commit()?;
    Ok(())
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
