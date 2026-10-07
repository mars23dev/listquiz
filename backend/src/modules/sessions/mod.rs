//! Session creation rules (US1, T034-T036).
//!
//! Validation lives here as plain functions over plain data so the rules can be
//! unit tested without a database; the insert itself is a single transaction that
//! writes `game_session`, `team`, and `round_config` together.

pub mod round_state;

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::errors::{ApiError, ApiResult};
use crate::modules::order_round;
use crate::modules::scoring;
use crate::modules::teams::{self, NewTeam, TeamColorInput};
use crate::modules::turn_timer;

pub const MIN_ROUNDS: i32 = 1;
pub const MAX_ROUNDS: i32 = 10;
pub const SESSION_ID_PREFIX: &str = "lq-";
pub const SESSION_ID_BODY_CHARS: usize = 8;

/// The player roles that always get a link, whether or not a team backs them.
pub const PLAYER_ROLES: [&str; 4] = ["p1", "p2", "p3", "p4"];

/// One round as the setup screen sends it (contracts/rest-api.md).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoundInput {
    #[serde(default)]
    pub round_number: i32,
    pub quiz_type: String,
    pub list_id: String,
    pub column_id: String,
    #[serde(default)]
    pub tripmine_side: Option<String>,
}

/// The `POST /api/sessions` body.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionRequest {
    #[serde(default)]
    pub gm_password: String,
    #[serde(default)]
    pub team_count: Option<i32>,
    #[serde(default)]
    pub teams: Vec<TeamColorInput>,
    #[serde(default)]
    pub round_count: Option<i32>,
    #[serde(default)]
    pub rounds: Vec<RoundInput>,
}

impl CreateSessionRequest {
    /// The effective team count: the explicit field, else the length of `teams`.
    pub fn effective_team_count(&self) -> i32 {
        self.team_count.unwrap_or(self.teams.len() as i32)
    }

    /// The effective round count: the explicit field, else the length of `rounds`.
    pub fn effective_round_count(&self) -> i32 {
        self.round_count.unwrap_or(self.rounds.len() as i32)
    }
}

/// One of the two round types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum QuizType {
    Order,
    Tripmine,
}

impl QuizType {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_uppercase().as_str() {
            "ORDER" => Some(QuizType::Order),
            "TRIPMINE" => Some(QuizType::Tripmine),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            QuizType::Order => "ORDER",
            QuizType::Tripmine => "TRIPMINE",
        }
    }
}

/// Which end of the ranking Tripmine hunts for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TripmineSide {
    Highest,
    Lowest,
    /// Hard: the mine is a random item and the search target is a random remaining item.
    Random,
}

impl TripmineSide {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "highest" => Some(TripmineSide::Highest),
            "lowest" => Some(TripmineSide::Lowest),
            "random" => Some(TripmineSide::Random),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            TripmineSide::Highest => "highest",
            TripmineSide::Lowest => "lowest",
            TripmineSide::Random => "random",
        }
    }
}

/// A round that passed validation, with its type and side resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidRound {
    pub round_number: i32,
    pub quiz_type: QuizType,
    pub tripmine_side: Option<TripmineSide>,
    pub list_id: String,
    pub column_id: String,
}

/// A session id: `lq-` and eight lowercase alphanumerics (FR-005).
pub fn generate_session_id() -> String {
    use rand::Rng;
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    let body: String = (0..SESSION_ID_BODY_CHARS)
        .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
        .collect();
    format!("{SESSION_ID_PREFIX}{body}")
}

pub fn is_valid_session_id(id: &str) -> bool {
    match id.strip_prefix(SESSION_ID_PREFIX) {
        Some(body) => {
            body.len() == SESSION_ID_BODY_CHARS
                && body
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        }
        None => false,
    }
}

/// Validate the whole request without touching the database.
pub fn validate_request(
    request: &CreateSessionRequest,
) -> ApiResult<(i32, i32, Vec<NewTeam>, Vec<ValidRound>)> {
    if request.gm_password.trim().is_empty() {
        return Err(ApiError::bad_request("A gamemaster password is required."));
    }

    let team_count = request.effective_team_count();
    teams::validate_team_count(team_count)?;
    let built_teams = teams::build_teams(team_count, &request.teams)?;

    let round_count = request.effective_round_count();
    if !(MIN_ROUNDS..=MAX_ROUNDS).contains(&round_count) {
        return Err(ApiError::bad_request(format!(
            "Round count must be between {MIN_ROUNDS} and {MAX_ROUNDS}."
        )));
    }
    if request.rounds.len() as i32 != round_count {
        return Err(ApiError::bad_request(format!(
            "roundCount is {round_count} but {} rounds were sent.",
            request.rounds.len()
        )));
    }

    let rounds = request
        .rounds
        .iter()
        .enumerate()
        .map(|(index, round)| validate_round(round, index as i32 + 1))
        .collect::<ApiResult<Vec<_>>>()?;

    Ok((team_count, round_count, built_teams, rounds))
}

/// Validate one round: type, side, and both ids must be present (FR-010).
pub fn validate_round(round: &RoundInput, position: i32) -> ApiResult<ValidRound> {
    let quiz_type = QuizType::parse(&round.quiz_type).ok_or_else(|| {
        ApiError::bad_request(format!(
            "Round {position}: quizType must be ORDER or TRIPMINE."
        ))
    })?;

    let tripmine_side = match (quiz_type, round.tripmine_side.as_deref()) {
        (QuizType::Tripmine, Some(raw)) => Some(TripmineSide::parse(raw).ok_or_else(|| {
            ApiError::bad_request(format!(
                "Round {position}: tripmineSide must be highest or lowest."
            ))
        })?),
        (QuizType::Tripmine, None) => {
            return Err(ApiError::bad_request(format!(
                "Round {position}: a TRIPMINE round needs a tripmineSide."
            )));
        }
        (QuizType::Order, Some(_)) => {
            return Err(ApiError::bad_request(format!(
                "Round {position}: an ORDER round must not carry a tripmineSide."
            )));
        }
        (QuizType::Order, None) => None,
    };

    if round.list_id.trim().is_empty() {
        return Err(ApiError::bad_request(format!(
            "Round {position}: listId is required."
        )));
    }
    if round.column_id.trim().is_empty() {
        return Err(ApiError::bad_request(format!(
            "Round {position}: columnId is required."
        )));
    }

    Ok(ValidRound {
        round_number: if round.round_number > 0 {
            round.round_number
        } else {
            position
        },
        quiz_type,
        tripmine_side,
        list_id: round.list_id.clone(),
        column_id: round.column_id.clone(),
    })
}

/// Check that a round points at a list that exists and a column that belongs to
/// it. A column id from another list is a `not_found`, not a `bad_request`,
/// because both ids are well-formed (FR-010, contracts/rest-api.md).
pub fn validate_round_references(conn: &Connection, round: &ValidRound) -> ApiResult<()> {
    let list_exists: Option<String> = conn
        .query_row(
            "SELECT id FROM quiz_list WHERE id = ?1",
            params![round.list_id],
            |row| row.get(0),
        )
        .optional()?;
    if list_exists.is_none() {
        return Err(ApiError::not_found(format!(
            "No list with id '{}'.",
            round.list_id
        )));
    }

    let column_exists: Option<String> = conn
        .query_row(
            "SELECT id FROM list_item_column WHERE list_id = ?1 AND id = ?2",
            params![round.list_id, round.column_id],
            |row| row.get(0),
        )
        .optional()?;
    if column_exists.is_none() {
        return Err(ApiError::not_found(format!(
            "No column '{}' on list '{}'.",
            round.column_id, round.list_id
        )));
    }
    Ok(())
}

/// Create the session and write its teams and rounds in one transaction.
/// Returns the new session id.
pub fn create_session(conn: &Connection, request: &CreateSessionRequest) -> ApiResult<String> {
    let (team_count, round_count, built_teams, rounds) = validate_request(request)?;
    for round in &rounds {
        validate_round_references(conn, round)?;
    }

    let now = crate::db::now_ms();
    let tx = conn.unchecked_transaction()?;

    // Retry on the (vanishingly unlikely) id collision rather than failing the
    // gamemaster for something they cannot act on.
    let mut session_id = String::new();
    for _ in 0..5 {
        let candidate = generate_session_id();
        let taken: Option<i64> = tx
            .query_row(
                "SELECT 1 FROM game_session WHERE id = ?1",
                params![candidate],
                |row| row.get(0),
            )
            .optional()?;
        if taken.is_none() {
            tx.execute(
                "INSERT INTO game_session
                    (id, gm_password, team_count, round_count, current_round, status, revision, created_at)
                 VALUES (?1, ?2, ?3, ?4, 0, 'setup', 0, ?5)",
                params![
                    candidate,
                    request.gm_password,
                    team_count,
                    round_count,
                    now
                ],
            )?;
            session_id = candidate;
            break;
        }
    }
    if session_id.is_empty() {
        return Err(ApiError::conflict(
            "Could not allocate a free session id; please try again.",
        ));
    }

    for team in &built_teams {
        tx.execute(
            "INSERT INTO team
                (id, session_id, name, color, primary_hex, accent_glow, score, hearts, turn_index)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                team.id,
                session_id,
                team.name,
                team.color_name,
                team.primary_hex,
                team.accent_glow,
                team.score,
                team.hearts,
                team.turn_index
            ],
        )?;
    }

    for round in &rounds {
        tx.execute(
            "INSERT INTO round_config
                (session_id, round_number, quiz_type, tripmine_side, list_id, column_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                session_id,
                round.round_number,
                round.quiz_type.as_str(),
                round.tripmine_side.map(|side| side.as_str()),
                round.list_id,
                round.column_id
            ],
        )?;
    }

    tx.commit()?;
    Ok(session_id)
}

/// What advancing the session did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoundAdvance {
    /// True when the last round was crossed and the game finished instead.
    pub finished: bool,
    /// The round now in play, or the last round when the game just finished.
    pub round_number: i32,
}

/// Move the session on: start the next configured round, or finish the game when
/// the last round is crossed (FR-023, FR-024, FR-015). Starting a round resets
/// hearts to three, rotates the first turn, and leaves every score alone.
pub fn advance_round(conn: &Connection, session_id: &str) -> ApiResult<RoundAdvance> {
    let (current_round, round_count, status): (i32, i32, String) = conn
        .query_row(
            "SELECT current_round, round_count, status FROM game_session WHERE id = ?1",
            params![session_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .ok_or_else(|| ApiError::not_found(format!("No session with id '{session_id}'.")))?;

    if status == "finished" {
        return Err(ApiError::not_ready("The game has already finished."));
    }

    if current_round >= round_count {
        // Crossing the last round finishes the game and unlocks every value (FR-024).
        conn.execute(
            "UPDATE game_session
             SET status = 'finished', reveal_values = 1, active_team_id = NULL, last_message = ?2
             WHERE id = ?1",
            params![session_id, "Game over"],
        )?;
        return Ok(RoundAdvance {
            finished: true,
            round_number: current_round,
        });
    }

    let target = current_round + 1;
    let round = round_state::load_round(conn, session_id, target)?;
    let starting_team = round_state::starting_team_id(conn, session_id, target)?;

    let tx = conn.unchecked_transaction()?;
    round_state::clear_round_state(&tx, session_id)?;
    round_state::reset_hearts(&tx, session_id)?;
    round_state::store_round_start(&tx, session_id, &round, &starting_team)?;
    tx.execute(
        "UPDATE game_session SET last_message = ?2 WHERE id = ?1",
        params![
            session_id,
            format!(
                "Round {target} Started: {} ({})",
                round.list_title,
                round.quiz_type.as_str()
            )
        ],
    )?;
    tx.commit()?;
    Ok(RoundAdvance {
        finished: false,
        round_number: target,
    })
}

/// Record a running turn timer (feature 005, US2). The duration must be positive.
pub fn start_timer(conn: &Connection, session_id: &str, seconds: u32) -> ApiResult<()> {
    let timer = crate::modules::turn_timer::start(crate::db::now_ms(), seconds)?;
    conn.execute(
        "UPDATE game_session SET timer_ends_at = ?2, timer_total_seconds = ?3 WHERE id = ?1",
        params![session_id, timer.ends_at, timer.total_seconds],
    )?;
    Ok(())
}

/// Clear the running timer (feature 005, US2). A no-op when none runs.
pub fn clear_timer(conn: &Connection, session_id: &str) -> ApiResult<()> {
    conn.execute(
        "UPDATE game_session SET timer_ends_at = NULL, timer_total_seconds = NULL WHERE id = ?1",
        params![session_id],
    )?;
    Ok(())
}

/// The running timer's end time and duration, if any.
pub fn load_timer(conn: &Connection, session_id: &str) -> ApiResult<Option<(i64, u32)>> {
    let row: Option<(Option<i64>, Option<i64>)> = conn
        .query_row(
            "SELECT timer_ends_at, timer_total_seconds FROM game_session WHERE id = ?1",
            params![session_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()?;
    Ok(match row {
        Some((Some(ends_at), Some(total))) => Some((ends_at, total.max(0) as u32)),
        _ => None,
    })
}

/// Delete a finished session and, through the schema's cascade, its teams, rounds,
/// round state, and score events (feature 005, US4). A missing session is success
/// (idempotent); a session that is not `finished` is refused so a live game cannot
/// be wiped. Quiz lists are never touched.
pub fn delete_session(conn: &Connection, session_id: &str) -> ApiResult<()> {
    let status: Option<String> = conn
        .query_row(
            "SELECT status FROM game_session WHERE id = ?1",
            params![session_id],
            |row| row.get(0),
        )
        .optional()?;
    match status.as_deref() {
        None => Ok(()),
        Some("finished") => {
            conn.execute(
                "DELETE FROM game_session WHERE id = ?1",
                params![session_id],
            )?;
            Ok(())
        }
        Some(_) => Err(ApiError::not_ready("Only a finished game can be deleted.")),
    }
}

/// Finish the game when its last configured round has ended (feature 007).
///
/// The game-end dialog keys off the `finished` status, and a quick game has no
/// gamemaster to press "Finish Game", so the last round's end finishes the game by
/// itself. Reuses the manual finish's outcome (reveal values, clear the active team).
/// A no-op when the session is already finished or the round is not the last.
pub fn finish_if_last_round(conn: &Connection, session_id: &str) -> ApiResult<bool> {
    let row: Option<(String, i32, i32)> = conn
        .query_row(
            "SELECT status, current_round, round_count FROM game_session WHERE id = ?1",
            params![session_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?;
    let Some((status, current_round, round_count)) = row else {
        return Ok(false);
    };
    if status == "finished" || current_round < round_count {
        return Ok(false);
    }
    conn.execute(
        "UPDATE game_session
         SET status = 'finished', reveal_values = 1, active_team_id = NULL, last_message = ?2
         WHERE id = ?1",
        params![session_id, "Game over"],
    )?;
    Ok(true)
}

/// Apply a running timer's timeout (feature 008). No timer → no-op returning `false`.
/// Otherwise: the on-turn team is penalised by the round's rule (Order loses one
/// heart, Tripmine just passes), the timer clears, a timeout message is set, and the
/// game finishes if it was the last round. This is the one rule the manual `time_out`
/// action and the scheduled deadline both use.
pub fn apply_turn_timeout(conn: &Connection, session_id: &str) -> ApiResult<bool> {
    if load_timer(conn, session_id)?.is_none() {
        return Ok(false);
    }
    let active: Option<String> = conn.query_row(
        "SELECT active_team_id FROM game_session WHERE id = ?1",
        params![session_id],
        |row| row.get(0),
    )?;
    let Some(active) = active else {
        return Ok(false);
    };

    let quiz_type: String = conn.query_row(
        "SELECT quiz_type FROM round_config
         WHERE session_id = ?1 AND round_number = (
             SELECT current_round FROM game_session WHERE id = ?1
         )",
        params![session_id],
        |row| row.get(0),
    )?;
    let rule = if quiz_type == "TRIPMINE" {
        turn_timer::TimeoutRule::PassOnly
    } else {
        turn_timer::TimeoutRule::LoseHeart
    };
    let message = if rule == turn_timer::TimeoutRule::LoseHeart {
        "Time ran out! -1 heart"
    } else {
        "Time ran out! Turn passed"
    };

    let round: i32 = conn.query_row(
        "SELECT current_round FROM game_session WHERE id = ?1",
        params![session_id],
        |row| row.get(0),
    )?;
    let teams = order_round::load_teams(conn, session_id)?;
    let (teams, events, next) = turn_timer::expire(&teams, &active, rule);
    let round_over = next.is_none();

    let tx = conn.unchecked_transaction()?;
    order_round::save_teams(&tx, session_id, &teams)?;
    order_round::append_events(&tx, session_id, round, &events)?;
    tx.execute(
        "UPDATE game_session
         SET active_team_id = ?2, timer_ends_at = NULL, timer_total_seconds = NULL, last_message = ?3
         WHERE id = ?1",
        params![session_id, next, Some(message.to_string())],
    )?;
    tx.commit()?;
    if round_over {
        finish_if_last_round(conn, session_id)?;
    }
    Ok(true)
}

/// The team a player role owns: `p1`..`p4` map to `team_1`..`team_4`; the/// gamemaster, spectator, and unknown roles own no team.
pub fn player_role_team(role: &str) -> Option<String> {
    let index: u32 = role.strip_prefix('p')?.parse().ok()?;
    (1..=4).contains(&index).then(|| format!("team_{index}"))
}

/// FR-005: a player role may act only for the team it owns while that team is on
/// turn. The gamemaster is authorized separately by password.
pub fn player_may_act(
    conn: &Connection,
    session_id: &str,
    role: &str,
    requested_team: &str,
) -> ApiResult<()> {
    let owned = player_role_team(role)
        .ok_or_else(|| ApiError::not_ready(format!("Role '{role}' cannot act in a round.")))?;
    if owned != requested_team {
        return Err(ApiError::not_ready(
            "A player can only act for their own team.",
        ));
    }
    let active: Option<String> = conn
        .query_row(
            "SELECT active_team_id FROM game_session WHERE id = ?1",
            params![session_id],
            |row| row.get(0),
        )
        .optional()?;
    if !scoring::is_active_turn(active.as_deref(), requested_team) {
        return Err(ApiError::not_ready("It is not this team's turn."));
    }
    Ok(())
}

/// True when the given password matches the session's gamemaster password.
pub fn gm_password_matches(conn: &Connection, session_id: &str, attempt: &str) -> ApiResult<bool> {
    let stored: String = conn
        .query_row(
            "SELECT gm_password FROM game_session WHERE id = ?1",
            params![session_id],
            |row| row.get(0),
        )
        .optional()?
        .ok_or_else(|| ApiError::not_found(format!("No session with id '{session_id}'.")))?;
    Ok(stored == attempt)
}

/// Every player's team index, or `None` when that slot has no team behind it.
pub fn player_team_id(
    conn: &Connection,
    session_id: &str,
    slot: usize,
) -> ApiResult<Option<String>> {
    let team_id: Option<String> = conn
        .query_row(
            "SELECT id FROM team WHERE session_id = ?1 ORDER BY turn_index LIMIT 1 OFFSET ?2",
            params![session_id, slot as i64],
            |row| row.get(0),
        )
        .optional()?;
    Ok(team_id)
}

#[cfg(test)]
mod tests;
