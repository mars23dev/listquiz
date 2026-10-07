//! Session endpoints (US1, T037).
//!
//! Every handler follows the same shape: validate, write in one transaction, bump
//! the revision, then answer. The revision bump is what makes a live client
//! refetch, so it happens after the write and before the response.

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::app_state::{AppState, SessionLinks, UserRole, build_links};
use crate::errors::{ApiError, ApiResult};
use crate::modules::connect_round;
use crate::modules::default_lists;
use crate::modules::order_round;
use crate::modules::quick_game;
use crate::modules::scoring;
use crate::modules::sessions::{self, CreateSessionRequest};
use crate::modules::tripmine_round;
use crate::modules::turn_timer;
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

/// `POST /api/sessions/{id}/actions` — the session action channel.
///
/// The gamemaster sends no `role` (or `"gamemaster"`) and the password header. A
/// player role sends its own role and no password, and may act only for the team
/// on turn (FR-005, contracts/actions-amendment.md).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionRequest {
    pub action: String,
    #[serde(default)]
    pub role: Option<String>,
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
    #[serde(default)]
    pub delta: Option<i32>,
    /// The turn timer's duration in seconds (feature 005, US2).
    #[serde(default)]
    pub seconds: Option<u32>,
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

    let role = request
        .role
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_else(|| "gamemaster".to_string());

    if role == "gamemaster" {
        let password = headers
            .get("x-quiz-password")
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        let matches =
            state.with_db(|conn| sessions::gm_password_matches(conn, &session_id, password))?;
        if !matches {
            return Err(ApiError::unauthorized());
        }
    } else {
        // A player role may only run an in-round action for its own team on turn.
        if !is_player_action(&request.action) {
            return Err(ApiError::bad_request(
                "This action is available to the gamemaster only.",
            ));
        }
        let requested_team = require_field(request.team_id.as_deref(), "teamId")?;
        state.with_db(|conn| sessions::player_may_act(conn, &session_id, &role, requested_team))?;
    }

    let revision = apply_action(&state, &session_id, &request)?;
    Ok(Json(ActionResponse { ok: true, revision }))
}

/// The in-round actions a player role may post for its own team (FR-001).
fn is_player_action(action: &str) -> bool {
    matches!(
        action,
        "select_item"
            | "choose_slot"
            | "cancel_selection"
            | "pick_tripitem"
            | "time_out"
            | "pick_name"
            | "pick_value"
            | "confirm_connect"
    )
}

/// Run one action and bump the revision if it changed anything.
fn apply_action(state: &AppState, session_id: &str, request: &ActionRequest) -> ApiResult<i64> {
    match request.action.as_str() {
        "next_round" => {
            state.with_db(|conn| sessions::advance_round(conn, session_id))?;
            state.with_db(|conn| sessions::clear_timer(conn, session_id))?;
            state.bump_revision(session_id)
        }
        // FR-024: one flag, honoured by `redact`, for both quiz types.
        "reveal_values" => {
            state.with_db(|conn| order_round::set_reveal_values(conn, session_id, true))?;
            state.bump_revision(session_id)
        }
        // feature 005, US2: the gamemaster's turn timer; feature 008: schedule the timeout.
        "start_timer" => {
            let seconds = request
                .seconds
                .ok_or_else(|| ApiError::bad_request("seconds is required."))?;
            state.with_db(|conn| sessions::start_timer(conn, session_id, seconds))?;
            let scheduled = state.with_db(|conn| sessions::load_timer(conn, session_id));
            if let Ok(Some((ends_at, _))) = scheduled {
                schedule_turn_timeout(state.clone(), session_id.to_string(), ends_at);
            }
            state.bump_revision(session_id)
        }
        "stop_timer" => {
            state.with_db(|conn| sessions::clear_timer(conn, session_id))?;
            state.bump_revision(session_id)
        }
        "time_out" => {
            state.with_db(|conn| apply_time_out(conn, session_id, request))?;
            state.bump_revision(session_id)
        }
        action if is_order_action(action) => {
            state.with_db(|conn| apply_order_action(conn, session_id, request))?;
            state.bump_revision(session_id)
        }
        action if is_connect_action(action) => {
            state.with_db(|conn| apply_connect_action(conn, session_id, request))?;
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
        // FR-018: pass the turn without scoring.
        "pass_turn" => {
            state.with_db(|conn| apply_pass_turn(conn, session_id))?;
            state.with_db(|conn| sessions::clear_timer(conn, session_id))?;
            state.bump_revision(session_id)
        }
        // FR-064: the gamemaster simulating a turn.
        "set_turn" => {
            state.with_db(|conn| apply_set_turn(conn, session_id, request))?;
            state.with_db(|conn| sessions::clear_timer(conn, session_id))?;
            state.bump_revision(session_id)
        }
        // FR-023: manual score and heart changes, recorded as `game_master` events.
        "adjust_score" => {
            state.with_db(|conn| apply_manual(conn, session_id, request, false))?;
            state.bump_revision(session_id)
        }
        "adjust_hearts" => {
            state.with_db(|conn| apply_manual(conn, session_id, request, true))?;
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

/// The Connect actions (feature 010, US1).
fn is_connect_action(action: &str) -> bool {
    matches!(action, "pick_name" | "pick_value" | "confirm_connect")
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
            sessions::clear_timer(conn, session_id)?;
            if outcome.round_over {
                sessions::finish_if_last_round(conn, session_id)?;
            }
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

/// The running Connect board, or why there is none.
fn load_required_connect(
    conn: &rusqlite::Connection,
    session_id: &str,
) -> ApiResult<connect_round::ConnectBoard> {
    connect_round::load_board(conn, session_id)?.ok_or_else(|| {
        ApiError::not_ready("There is no Connect round running in this session yet.")
    })
}

/// One Connect action (feature 010, US1). Confirming scores and hands the turn on.
fn apply_connect_action(
    conn: &rusqlite::Connection,
    session_id: &str,
    request: &ActionRequest,
) -> ApiResult<()> {
    match request.action.as_str() {
        "pick_name" => {
            let item_id = require_field(request.item_id.as_deref(), "itemId")?;
            let board = load_required_connect(conn, session_id)?;
            require_team_on_turn(conn, session_id, request.team_id.as_deref())?;
            let next = board.pick_name(item_id)?;
            connect_round::save_board(conn, session_id, &next)
        }
        "pick_value" => {
            let item_id = require_field(request.item_id.as_deref(), "itemId")?;
            let board = load_required_connect(conn, session_id)?;
            require_team_on_turn(conn, session_id, request.team_id.as_deref())?;
            let next = board.pick_value(item_id)?;
            connect_round::save_board(conn, session_id, &next)
        }
        "confirm_connect" => {
            let board = load_required_connect(conn, session_id)?;
            let team_id = require_team_on_turn(conn, session_id, request.team_id.as_deref())?;
            let teams = order_round::load_teams(conn, session_id)?;
            let outcome = connect_round::confirm(&board, &teams, &team_id)?;
            let round_number = current_round_number(conn, session_id)?;

            let tx = conn.unchecked_transaction()?;
            connect_round::save_board(&tx, session_id, &outcome.board)?;
            order_round::save_teams(&tx, session_id, &outcome.teams)?;
            order_round::append_events(&tx, session_id, round_number, &outcome.events)?;
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
            sessions::clear_timer(conn, session_id)?;
            if outcome.round_over {
                sessions::finish_if_last_round(conn, session_id)?;
            }
            Ok(())
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
    sessions::clear_timer(conn, session_id)?;
    if outcome.round.completed {
        sessions::finish_if_last_round(conn, session_id)?;
    }
    Ok(())
}

/// Move the session on to the next configured round. A round whose list or
/// column has gone is reported as `not_ready` rather than half-started (T033).
fn current_round_number(conn: &rusqlite::Connection, session_id: &str) -> ApiResult<i32> {
    Ok(conn.query_row(
        "SELECT current_round FROM game_session WHERE id = ?1",
        [session_id],
        |row| row.get(0),
    )?)
}

/// The manual timeout action (feature 005, US2; feature 008): accepted only when a
/// timer is running and has expired, and only from the gamemaster or the on-turn team.
/// The consequence itself lives in `sessions::apply_turn_timeout`; normally the
/// scheduled deadline applies it, so a client never has to.
fn apply_time_out(
    conn: &rusqlite::Connection,
    session_id: &str,
    request: &ActionRequest,
) -> ApiResult<()> {
    let (ends_at, _total) = sessions::load_timer(conn, session_id)?
        .ok_or_else(|| ApiError::not_ready("No timer is running."))?;
    if !turn_timer::is_expired(ends_at, crate::db::now_ms()) {
        return Err(ApiError::not_ready("The timer has not run out yet."));
    }
    require_team_on_turn(conn, session_id, request.team_id.as_deref())?;
    sessions::apply_turn_timeout(conn, session_id)?;
    Ok(())
}

/// Schedule a timeout for a started timer (feature 008). The task wakes at the end
/// time and applies the timeout only if that same timer is still stored, so a
/// stopped, replaced, ended, or deleted timer is a no-op. This is what ends the turn
/// with no client involvement (a background tab cannot be trusted to post).
fn schedule_turn_timeout(state: AppState, session_id: String, ends_at: i64) {
    tokio::spawn(async move {
        let delay = (ends_at - crate::db::now_ms()).max(0) as u64;
        tokio::time::sleep(std::time::Duration::from_millis(delay + 50)).await;

        let stored = state.with_db(|conn| sessions::load_timer(conn, &session_id));
        let still_current = matches!(stored, Ok(Some((stored_ends, _))) if stored_ends == ends_at);
        if !still_current {
            return;
        }

        let applied = state
            .with_db(|conn| sessions::apply_turn_timeout(conn, &session_id))
            .unwrap_or(false);
        if applied {
            let _ = state.bump_revision(&session_id);
        }
    });
}

/// Pass the turn to the next team with hearts, scoring nothing (FR-018).
fn apply_pass_turn(conn: &rusqlite::Connection, session_id: &str) -> ApiResult<()> {
    let active: Option<String> = conn
        .query_row(
            "SELECT active_team_id FROM game_session WHERE id = ?1",
            [session_id],
            |row| row.get(0),
        )
        .optional()?;
    let active = active.ok_or_else(|| ApiError::not_ready("No team is on turn."))?;
    let teams = order_round::load_teams(conn, session_id)?;
    let next = scoring::advance_turn(&teams, &active);
    conn.execute(
        "UPDATE game_session SET active_team_id = ?2 WHERE id = ?1",
        rusqlite::params![session_id, next],
    )?;
    Ok(())
}

/// Put a chosen team on turn (FR-064). In a round that uses hearts (Order, Connect)
/// a team with zero hearts is out and cannot be put back on turn (feature 013).
fn apply_set_turn(
    conn: &rusqlite::Connection,
    session_id: &str,
    request: &ActionRequest,
) -> ApiResult<()> {
    let team_id = require_field(request.team_id.as_deref(), "teamId")?;
    let exists: Option<i64> = conn
        .query_row(
            "SELECT 1 FROM team WHERE session_id = ?1 AND id = ?2",
            rusqlite::params![session_id, team_id],
            |row| row.get(0),
        )
        .optional()?;
    if exists.is_none() {
        return Err(ApiError::not_found(format!(
            "No team '{team_id}' in this session."
        )));
    }

    // The current round's quiz type decides whether hearts apply (Tripmine has none).
    let quiz_type: Option<String> = conn
        .query_row(
            "SELECT quiz_type FROM round_config
             WHERE session_id = ?1 AND round_number = (
                 SELECT current_round FROM game_session WHERE id = ?1
             )",
            rusqlite::params![session_id],
            |row| row.get(0),
        )
        .optional()?;
    if matches!(quiz_type.as_deref(), Some("ORDER") | Some("CONNECT")) {
        let teams = order_round::load_teams(conn, session_id)?;
        let hearts = teams
            .iter()
            .find(|team| team.id == team_id)
            .map(|team| team.hearts)
            .unwrap_or(scoring::MIN_HEARTS);
        if hearts <= scoring::MIN_HEARTS {
            return Err(ApiError::not_ready(format!(
                "Team '{team_id}' is out of hearts and cannot be put on turn."
            )));
        }
    }

    conn.execute(
        "UPDATE game_session SET active_team_id = ?2 WHERE id = ?1",
        rusqlite::params![session_id, team_id],
    )?;
    Ok(())
}

/// A manual score or heart adjustment, clamped by the scoring module (FR-023).
fn apply_manual(
    conn: &rusqlite::Connection,
    session_id: &str,
    request: &ActionRequest,
    is_hearts: bool,
) -> ApiResult<()> {
    let team_id = require_field(request.team_id.as_deref(), "teamId")?;
    let delta = request
        .delta
        .ok_or_else(|| ApiError::bad_request("delta is required."))?;
    let teams = order_round::load_teams(conn, session_id)?;
    let outcome = scoring::manual_adjustment(&teams, team_id, delta, is_hearts);
    let round = current_round_number(conn, session_id)?;

    let tx = conn.unchecked_transaction()?;
    order_round::save_teams(&tx, session_id, &outcome.teams)?;
    order_round::append_events(&tx, session_id, round, &outcome.events)?;
    tx.commit()?;
    Ok(())
}

/// The `POST /api/sessions/quick` body (feature 005, US5).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickSessionRequest {
    pub game_type: String,
}

/// Start a one-team quick game of the chosen type immediately (feature 005, US5).
pub async fn create_quick_session(
    State(state): State<AppState>,
    Json(request): Json<QuickSessionRequest>,
) -> ApiResult<(StatusCode, Json<CreateSessionResponse>)> {
    let session_id = state.with_db(|conn| {
        let body = quick_game::build_request(&request.game_type, &default_lists::DEFAULT_LISTS)?;
        let session_id = sessions::create_session(conn, &body)?;
        // Start the single round at once so the game is playable immediately.
        sessions::advance_round(conn, &session_id)?;
        Ok::<String, ApiError>(session_id)
    })?;
    let links = build_links(&session_id, 1);
    Ok((
        StatusCode::CREATED,
        Json(CreateSessionResponse { session_id, links }),
    ))
}

/// Delete a finished session (feature 005, US4). Idempotent: deleting an already
/// missing session succeeds. Refused while the game is still running or between
/// rounds, so a live game cannot be wiped.
pub async fn delete_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> ApiResult<StatusCode> {
    state.with_db(|conn| sessions::delete_session(conn, &session_id))?;
    Ok(StatusCode::NO_CONTENT)
}

/// Mount the session routes onto the `/api` sub-router.
pub fn mount(api: Router<AppState>) -> Router<AppState> {
    api.route("/sessions", post(create_session))
        .route("/sessions/quick", post(create_quick_session))
        .route("/sessions/{session_id}/unlock", post(unlock_session))
        .route("/sessions/{session_id}/state", get(session_state))
        .route("/sessions/{session_id}/actions", post(session_action))
        .route("/sessions/{session_id}", delete(delete_session))
}
