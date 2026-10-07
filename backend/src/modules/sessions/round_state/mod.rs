//! Building and starting a round (US1 `next_round`, groundwork for US3/US4).
//!
//! The Order anchor is the median item of the whole list, never a hand-picked one
//! (FR-026): the round then has five items on each side of it, which is what makes
//! `FR-028`'s three slots (left, between-two, right) possible.

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::errors::{ApiError, ApiResult};
use crate::modules::scoring;
use crate::modules::sessions::{QuizType, TripmineSide};
use crate::modules::tripmine_round;

/// One item of a list, as the round builder sees it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListItemValue {
    pub item_id: String,
    pub name: String,
    pub display: String,
    pub numeric: f64,
    pub icon_name: Option<String>,
}

/// Every item of `list_id` with its value in `column_id`, sorted by value.
pub fn load_ordered_items(
    conn: &Connection,
    list_id: &str,
    column_id: &str,
    low_to_high: bool,
) -> ApiResult<Vec<ListItemValue>> {
    let mut items = Vec::new();
    let mut statement = conn
        .prepare(
            "SELECT i.id, i.name, v.display, v.numeric, i.icon_name
             FROM item i
             JOIN item_value v ON v.item_id = i.id AND v.column_id = ?2
             WHERE i.list_id = ?1
             ORDER BY v.numeric ASC, i.position ASC",
        )
        .map_err(ApiError::from)?;
    let rows = statement
        .query_map(params![list_id, column_id], |row| {
            Ok(ListItemValue {
                item_id: row.get(0)?,
                name: row.get(1)?,
                display: row.get(2)?,
                numeric: row.get(3)?,
                icon_name: row.get(4)?,
            })
        })
        .map_err(ApiError::from)?;
    for row in rows {
        items.push(row.map_err(ApiError::from)?);
    }
    if !low_to_high {
        items.reverse();
    }
    Ok(items)
}

/// The round the gamemaster is asking to start.
#[derive(Debug, Clone, PartialEq)]
pub struct RoundToStart {
    pub round_number: i32,
    pub quiz_type: QuizType,
    pub tripmine_side: Option<TripmineSide>,
    pub tripmine_slots: Option<i32>,
    pub list_id: String,
    pub column_id: String,
    pub list_title: String,
    pub column_name: String,
    pub low_to_high: bool,
}

/// Read the round configuration, or explain why it cannot run.
pub fn load_round(
    conn: &Connection,
    session_id: &str,
    round_number: i32,
) -> ApiResult<RoundToStart> {
    let row = conn
        .query_row(
            "SELECT rc.quiz_type, rc.tripmine_side, rc.tripmine_slots, rc.list_id, rc.column_id,
                    l.title, l.order_direction, c.name
             FROM round_config rc
             JOIN quiz_list l ON l.id = rc.list_id
             JOIN list_item_column c ON c.list_id = rc.list_id AND c.id = rc.column_id
             WHERE rc.session_id = ?1 AND rc.round_number = ?2",
            params![session_id, round_number],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<i32>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            },
        )
        .optional()?;

    let (quiz_type_raw, side_raw, slots, list_id, column_id, list_title, direction, column_name) =
        row.ok_or_else(|| {
            ApiError::not_ready(format!(
                "Round {round_number} is not ready: its list or column no longer exists."
            ))
        })?;

    let quiz_type = QuizType::parse(&quiz_type_raw).ok_or_else(|| {
        ApiError::not_ready(format!(
            "Round {round_number} is not ready: its quiz type is unusable."
        ))
    })?;

    Ok(RoundToStart {
        round_number,
        quiz_type,
        tripmine_side: side_raw.as_deref().and_then(TripmineSide::parse),
        tripmine_slots: slots,
        list_id,
        column_id,
        list_title,
        column_name,
        low_to_high: direction == "low_to_high",
    })
}

/// The team whose turn it is in `round_number`: rotation by round number, skipping
/// teams that are out of hearts (FR-018, FR-019).
pub fn starting_team_id(
    conn: &Connection,
    session_id: &str,
    round_number: i32,
) -> ApiResult<String> {
    let mut statement = conn
        .prepare("SELECT id, hearts FROM team WHERE session_id = ?1 ORDER BY turn_index")
        .map_err(ApiError::from)?;
    let teams: Vec<(String, i32)> = statement
        .query_map(params![session_id], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(ApiError::from)?
        .collect::<Result<_, _>>()
        .map_err(ApiError::from)?;
    if teams.is_empty() {
        return Err(ApiError::conflict("The session has no teams."));
    }

    let start = scoring::starting_team_for_round(round_number, teams.len() as i32);
    let index = start
        .trim_start_matches("team_")
        .parse::<usize>()
        .unwrap_or(1)
        - 1;
    let candidate = teams[index.min(teams.len() - 1)].0.clone();
    Ok(candidate)
}

/// Reset every team's hearts for a new round (FR-023).
pub fn reset_hearts(conn: &Connection, session_id: &str) -> ApiResult<()> {
    conn.execute(
        "UPDATE team SET hearts = ?2 WHERE session_id = ?1",
        params![session_id, scoring::MAX_HEARTS],
    )
    .map_err(ApiError::from)?;
    Ok(())
}

/// Drop any state left from an earlier round, so a replay starts clean.
pub fn clear_round_state(conn: &Connection, session_id: &str) -> ApiResult<()> {
    for table in ["order_state", "tripmine_state", "connect_state"] {
        conn.execute(
            &format!("DELETE FROM {table} WHERE session_id = ?1"),
            params![session_id],
        )
        .map_err(ApiError::from)?;
    }
    Ok(())
}

/// Write the fresh state for `round` and make it the session's current round.
pub fn store_round_start(
    conn: &Connection,
    session_id: &str,
    round: &RoundToStart,
    starting_team: &str,
) -> ApiResult<()> {
    match round.quiz_type {
        QuizType::Order => {
            // US3 replaced the old "five items pre-placed on each side" layout: the
            // bar starts with only the anchor and the ten candidates are sampled at
            // random around it (FR-026, FR-027).
            let items =
                load_ordered_items(conn, &round.list_id, &round.column_id, round.low_to_high)?;
            let setup = crate::modules::order_round::sample_order_round(&items, &mut rand::rng())?;
            let board = setup.into_board();
            crate::modules::order_round::save_board(conn, session_id, &board)?;
        }
        QuizType::Tripmine => {
            let side = round.tripmine_side.unwrap_or(TripmineSide::Highest);
            let slots = round.tripmine_slots.unwrap_or(10).max(1) as usize;
            let tripmine = tripmine_round::start_tripmine_round(
                conn,
                &round.list_id,
                &round.column_id,
                side,
                slots,
                &mut rand::rng(),
            )?;
            tripmine_round::save_round(conn, session_id, &tripmine)?;
        }
        QuizType::Connect => {
            let board = crate::modules::connect_round::start_connect_round(
                conn,
                &round.list_id,
                &round.column_id,
                &mut rand::rng(),
            )?;
            crate::modules::connect_round::save_board(conn, session_id, &board)?;
        }
    }

    conn.execute(
        "UPDATE game_session
         SET current_round = ?2, status = 'running', active_team_id = ?3
         WHERE id = ?1",
        params![session_id, round.round_number, starting_team],
    )
    .map_err(ApiError::from)?;
    Ok(())
}

#[cfg(test)]
mod tests;
