//! Quick game (feature 005, US5).
//!
//! A quick game is not a new kind of session: it is the ordinary session with fixed,
//! sensible choices — one team and one round of the chosen type on a default list
//! that fits. It reuses `create_session`, so it inherits every rule, the end dialog,
//! and the finished-session cleanup. The frontend only ever shows the player view.

use crate::errors::{ApiError, ApiResult};
use crate::modules::connect_round::PAIRS;
use crate::modules::default_lists::DefaultList;
use crate::modules::order_round::MIN_ITEMS_FOR_ORDER;
use crate::modules::sessions::{CreateSessionRequest, QuizType, RoundInput};
use crate::modules::teams::TeamColorInput;
use crate::modules::tripmine_round::TRIPMINE_SLOTS;

/// The colour of the single quick-game team.
const QUICK_TEAM_HEX: &str = "#06b6d4";

/// The items a game type needs on its list: Order needs an anchor plus ten candidates;
/// Tripmine needs its ten slots; Connect needs ten pairs.
fn required_items(game_type: QuizType) -> usize {
    match game_type {
        QuizType::Order => MIN_ITEMS_FOR_ORDER,
        QuizType::Tripmine => TRIPMINE_SLOTS,
        QuizType::Connect => PAIRS,
    }
}

/// Build the create-session request for a quick game of `game_type`, choosing the
/// first default list whose size fits. Errors when no list fits.
pub fn build_request(
    game_type_raw: &str,
    lists: &[DefaultList],
) -> ApiResult<CreateSessionRequest> {
    let game_type = QuizType::parse(game_type_raw)
        .ok_or_else(|| ApiError::bad_request("gameType must be ORDER, TRIPMINE, or CONNECT."))?;
    let needed = required_items(game_type);

    let list = lists
        .iter()
        .find(|list| !list.columns.is_empty() && list.items.len() >= needed)
        .ok_or_else(|| {
            ApiError::not_ready(format!(
                "No default list is large enough for a quick {game_type} game.",
                game_type = game_type.as_str()
            ))
        })?;

    Ok(CreateSessionRequest {
        gm_password: "quick".to_string(),
        team_count: Some(1),
        teams: vec![TeamColorInput {
            name: Some("Player".to_string()),
            primary_hex: QUICK_TEAM_HEX.to_string(),
        }],
        round_count: Some(1),
        rounds: vec![RoundInput {
            round_number: 1,
            quiz_type: game_type.as_str().to_string(),
            list_id: list.id.to_string(),
            column_id: list.columns[0].id.to_string(),
            tripmine_side: match game_type {
                QuizType::Tripmine => Some("highest".to_string()),
                QuizType::Order | QuizType::Connect => None,
            },
            tripmine_slots: None,
        }],
    })
}

#[cfg(test)]
mod tests;
