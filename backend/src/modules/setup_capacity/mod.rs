//! How many valued items a round needs before a session may be created.
//!
//! The thresholds live in the mode modules; this module only combines them and
//! compares them with the selected column's value count. The rule is a plain
//! function over plain data so it can be tested without a database.

use rusqlite::{Connection, params};

use crate::errors::{ApiError, ApiResult};
use crate::modules::sessions::{QuizType, ValidRound};
use crate::modules::{connect_round, order_round, tripmine_round};

/// The smallest number of valued items `quiz_type` needs for the selected variant.
pub fn required_valued_items(quiz_type: QuizType, tripmine_slots: Option<i32>) -> ApiResult<usize> {
    match quiz_type {
        QuizType::Order => Ok(order_round::MIN_ITEMS_FOR_ORDER),
        QuizType::Connect => Ok(connect_round::PAIRS),
        QuizType::Tripmine => match tripmine_slots {
            None | Some(10) => Ok(tripmine_round::TRIPMINE_SLOTS),
            Some(20) => Ok(tripmine_round::TRIPMINE_SLOTS_LARGE),
            Some(other) => Err(ApiError::bad_request(format!(
                "tripmineSlots must be 10 or 20; got {other}."
            ))),
        },
    }
}

/// The human name used in a shortfall message.
fn mode_label(quiz_type: QuizType, tripmine_slots: Option<i32>) -> String {
    match quiz_type {
        QuizType::Order => "Order".to_string(),
        QuizType::Connect => "Connect".to_string(),
        QuizType::Tripmine => {
            let slots = tripmine_slots.unwrap_or(10);
            format!("Tripmine with {slots} items")
        }
    }
}

/// Refuse `available` valued items when the round needs more.
pub fn validate_capacity(
    position: i32,
    quiz_type: QuizType,
    tripmine_slots: Option<i32>,
    available: usize,
) -> ApiResult<()> {
    let required = required_valued_items(quiz_type, tripmine_slots)?;
    if available < required {
        return Err(ApiError::bad_request(format!(
            "Round {position}: {} needs at least {required} items with values in the selected column; the list has {available}.",
            mode_label(quiz_type, tripmine_slots)
        )));
    }
    Ok(())
}

/// Count the items of `list_id` that have a value in `column_id`.
pub fn valued_item_count(conn: &Connection, list_id: &str, column_id: &str) -> ApiResult<usize> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*)
         FROM item i
         JOIN item_value v ON v.item_id = i.id AND v.column_id = ?2
         WHERE i.list_id = ?1",
        params![list_id, column_id],
        |row| row.get(0),
    )?;
    Ok(count as usize)
}

/// Refuse a round whose selected column cannot supply enough valued items.
pub fn validate_round_capacity(conn: &Connection, round: &ValidRound) -> ApiResult<()> {
    let available = valued_item_count(conn, &round.list_id, &round.column_id)?;
    validate_capacity(
        round.round_number,
        round.quiz_type,
        round.tripmine_slots,
        available,
    )
}

#[cfg(test)]
mod tests;
