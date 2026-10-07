//! Connect round rules (feature 010, US1).
//!
//! Ten item **names** (left) and their ten **values** (right, shuffled). On a turn a
//! team picks one name and one value into a middle box and confirms; a matching pair
//! scores 10, a mismatch costs one heart. Everything is a plain function over plain
//! data so the rules are unit tested without a database. The turn always passes.

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::errors::{ApiError, ApiResult};
use crate::modules::scoring::{self, ScoreEvent, ScoringOutcome, TeamStanding};

/// Pairs in a Connect round.
pub const PAIRS: usize = 10;
/// Points for a correct pair.
pub const POINTS_CORRECT: i32 = 10;
/// Hearts lost for a wrong pair.
pub const HEARTS_LOST: i32 = 1;

/// A name in the left column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectName {
    pub item_id: String,
    pub name: String,
    pub icon_name: Option<String>,
}

/// A value in the right column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectValue {
    pub item_id: String,
    pub display: String,
}

/// The pair staged in the middle box; either side may be unset.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectPending {
    pub name_item_id: Option<String>,
    pub value_item_id: Option<String>,
}

/// The whole Connect board.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectBoard {
    pub names: Vec<ConnectName>,
    pub values: Vec<ConnectValue>,
    pub matched: Vec<String>,
    pub pending: ConnectPending,
}

/// What confirming a pair did.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmOutcome {
    pub correct: bool,
    pub board: ConnectBoard,
    pub teams: Vec<TeamStanding>,
    pub events: Vec<ScoreEvent>,
    pub next_team_id: Option<String>,
    pub round_over: bool,
}

/// One drawn pair: the item id, its name, its icon, and its value's display text.
pub struct ConnectPair {
    pub item_id: String,
    pub name: String,
    pub icon_name: Option<String>,
    pub display: String,
}

/// Draw ten pairs at random from `pairs`, with both columns shuffled (feature 010).
pub fn sample_connect_round(
    pairs: &[ConnectPair],
    rng: &mut impl rand::Rng,
) -> ApiResult<ConnectBoard> {
    if pairs.len() < PAIRS {
        return Err(ApiError::not_ready(format!(
            "A Connect round needs at least {PAIRS} items with values."
        )));
    }

    let mut indices: Vec<usize> = (0..pairs.len()).collect();
    for index in (1..indices.len()).rev() {
        let pick = rng.random_range(0..=index);
        indices.swap(index, pick);
    }
    let chosen: Vec<&ConnectPair> = indices.iter().take(PAIRS).map(|i| &pairs[*i]).collect();

    let mut names: Vec<ConnectName> = chosen
        .iter()
        .map(|pair| ConnectName {
            item_id: pair.item_id.clone(),
            name: pair.name.clone(),
            icon_name: pair.icon_name.clone(),
        })
        .collect();
    let mut values: Vec<ConnectValue> = chosen
        .iter()
        .map(|pair| ConnectValue {
            item_id: pair.item_id.clone(),
            display: pair.display.clone(),
        })
        .collect();

    // Both columns are shown in a random order so neither is sweepable (feature 010).
    shuffle(&mut names, rng);
    shuffle(&mut values, rng);

    Ok(ConnectBoard {
        names,
        values,
        matched: Vec::new(),
        pending: ConnectPending::default(),
    })
}

fn shuffle<T>(items: &mut [T], rng: &mut impl rand::Rng) {
    for index in (1..items.len()).rev() {
        let pick = rng.random_range(0..=index);
        items.swap(index, pick);
    }
}

impl ConnectBoard {
    fn is_matched(&self, item_id: &str) -> bool {
        self.matched.iter().any(|id| id == item_id)
    }

    /// Stage a name in the middle box. A matched name cannot be picked again.
    pub fn pick_name(&self, item_id: &str) -> ApiResult<ConnectBoard> {
        if !self.names.iter().any(|name| name.item_id == item_id) {
            return Err(ApiError::not_found("That name is not on the board."));
        }
        if self.is_matched(item_id) {
            return Err(ApiError::not_ready("That pair is already matched."));
        }
        let mut next = self.clone();
        next.pending.name_item_id = Some(item_id.to_string());
        Ok(next)
    }

    /// Stage a value in the middle box. A matched value cannot be picked again.
    pub fn pick_value(&self, item_id: &str) -> ApiResult<ConnectBoard> {
        if !self.values.iter().any(|value| value.item_id == item_id) {
            return Err(ApiError::not_found("That value is not on the board."));
        }
        if self.is_matched(item_id) {
            return Err(ApiError::not_ready("That pair is already matched."));
        }
        let mut next = self.clone();
        next.pending.value_item_id = Some(item_id.to_string());
        Ok(next)
    }
}

/// Confirm the staged pair (feature 010). A matching pair scores 10 and is matched; a
/// mismatch costs one heart. Either way the turn passes.
pub fn confirm(
    board: &ConnectBoard,
    teams: &[TeamStanding],
    team_id: &str,
) -> ApiResult<ConfirmOutcome> {
    let name_id = board
        .pending
        .name_item_id
        .clone()
        .ok_or_else(|| ApiError::not_ready("Pick a name before confirming."))?;
    let value_id = board
        .pending
        .value_item_id
        .clone()
        .ok_or_else(|| ApiError::not_ready("Pick a value before confirming."))?;

    let correct = name_id == value_id;
    let mut next = board.clone();
    next.pending = ConnectPending::default();

    let outcome: ScoringOutcome = if correct {
        next.matched.push(name_id);
        scoring::apply_points(
            teams,
            team_id,
            POINTS_CORRECT,
            scoring::REASON_CORRECT_PLACEMENT,
        )
    } else {
        scoring::apply_hearts(
            teams,
            team_id,
            -HEARTS_LOST,
            scoring::REASON_INCORRECT_PLACEMENT,
        )
    };

    let next_team_id = scoring::advance_turn(&outcome.teams, team_id);
    let round_over = round_is_over(&next, &outcome.teams);
    Ok(ConfirmOutcome {
        correct,
        board: next,
        teams: outcome.teams,
        events: outcome.events,
        next_team_id,
        round_over,
    })
}

/// The round is over when every pair is matched or no team holds a heart (feature 010).
pub fn round_is_over(board: &ConnectBoard, teams: &[TeamStanding]) -> bool {
    board.matched.len() >= board.names.len()
        || !teams.iter().any(|team| team.hearts > scoring::MIN_HEARTS)
}

// --- storage --------------------------------------------------------------

/// Read every item of a list with its value in `column_id`, for sampling.
pub fn load_pairs(
    conn: &Connection,
    list_id: &str,
    column_id: &str,
) -> ApiResult<Vec<ConnectPair>> {
    let mut statement = conn
        .prepare(
            "SELECT i.id, i.name, i.icon_name, v.display
             FROM item i
             JOIN item_value v ON v.item_id = i.id AND v.column_id = ?2
             WHERE i.list_id = ?1
             ORDER BY i.position",
        )
        .map_err(ApiError::from)?;
    let rows = statement
        .query_map(params![list_id, column_id], |row| {
            Ok(ConnectPair {
                item_id: row.get(0)?,
                name: row.get(1)?,
                icon_name: row.get(2)?,
                display: row.get(3)?,
            })
        })
        .map_err(ApiError::from)?;
    let mut pairs = Vec::new();
    for row in rows {
        pairs.push(row.map_err(ApiError::from)?);
    }
    Ok(pairs)
}

/// Sample a Connect round from a list and column.
pub fn start_connect_round(
    conn: &Connection,
    list_id: &str,
    column_id: &str,
    rng: &mut impl rand::Rng,
) -> ApiResult<ConnectBoard> {
    let pairs = load_pairs(conn, list_id, column_id)?;
    sample_connect_round(&pairs, rng)
}

/// Read the stored board back.
pub fn load_board(conn: &Connection, session_id: &str) -> ApiResult<Option<ConnectBoard>> {
    let row: Option<(String, String, String, String)> = conn
        .query_row(
            "SELECT names, values_, matched, pending FROM connect_state WHERE session_id = ?1",
            params![session_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .optional()?;
    let Some((names, values, matched, pending)) = row else {
        return Ok(None);
    };
    Ok(Some(ConnectBoard {
        names: serde_json::from_str(&names).unwrap_or_default(),
        values: serde_json::from_str(&values).unwrap_or_default(),
        matched: serde_json::from_str(&matched).unwrap_or_default(),
        pending: serde_json::from_str(&pending).unwrap_or_default(),
    }))
}

/// Write the board back.
pub fn save_board(conn: &Connection, session_id: &str, board: &ConnectBoard) -> ApiResult<()> {
    let names = serde_json::to_string(&board.names).map_err(ApiError::from)?;
    let values = serde_json::to_string(&board.values).map_err(ApiError::from)?;
    let matched = serde_json::to_string(&board.matched).map_err(ApiError::from)?;
    let pending = serde_json::to_string(&board.pending).map_err(ApiError::from)?;
    conn.execute(
        "INSERT OR REPLACE INTO connect_state (session_id, names, values_, matched, pending)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![session_id, names, values, matched, pending],
    )
    .map_err(ApiError::from)?;
    Ok(())
}

#[cfg(test)]
mod tests;
