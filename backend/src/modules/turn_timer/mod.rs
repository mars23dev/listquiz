//! The gamemaster's turn timer (feature 005, US2).
//!
//! The timer is a single countdown attached to the current turn. The server stores
//! when it ends; a client counts down locally and reports the timeout, and the server
//! judges that report here. A timeout is a failed turn: the on-turn team loses exactly
//! one heart and the turn passes to the next team with hearts, reusing the scoring
//! module so there is one definition of a bad turn.

use crate::errors::{ApiError, ApiResult};
use crate::modules::scoring::{self, ScoreEvent, TeamStanding};

/// A running timer: when it ends (epoch ms) and the duration that was chosen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TurnTimer {
    pub ends_at: i64,
    pub total_seconds: u32,
}

/// Start a timer of `seconds` from `now`. The duration must be positive.
pub fn start(now: i64, seconds: u32) -> ApiResult<TurnTimer> {
    if seconds == 0 {
        return Err(ApiError::bad_request(
            "The timer must be at least one second.",
        ));
    }
    Ok(TurnTimer {
        ends_at: now + (seconds as i64) * 1000,
        total_seconds: seconds,
    })
}

/// Seconds left until `ends_at`, rounded up and never negative.
pub fn remaining(ends_at: i64, now: i64) -> u32 {
    let millis = ends_at - now;
    if millis <= 0 {
        return 0;
    }
    ((millis + 999) / 1000) as u32
}

/// True once `now` has reached the timer's end.
pub fn is_expired(ends_at: i64, now: i64) -> bool {
    now >= ends_at
}

/// What a timeout does to the on-turn team. Hearts are an Order mechanic, so a
/// Tripmine timeout only ends the turn (feature 006).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutRule {
    /// Order: the on-turn team loses one heart.
    LoseHeart,
    /// Tripmine: the turn simply passes.
    PassOnly,
}

/// Apply a timeout: the turn passes to the next team with hearts, and the on-turn
/// team loses one heart only under `LoseHeart` (never below zero). Returns the new
/// standings, the score events to append, and the next team, or `None` when no team
/// has hearts left (round over).
pub fn expire(
    teams: &[TeamStanding],
    active_team_id: &str,
    rule: TimeoutRule,
) -> (Vec<TeamStanding>, Vec<ScoreEvent>, Option<String>) {
    match rule {
        TimeoutRule::LoseHeart => {
            let outcome =
                scoring::apply_hearts(teams, active_team_id, -1, scoring::REASON_TIME_OUT);
            let next = scoring::advance_turn(&outcome.teams, active_team_id);
            (outcome.teams, outcome.events, next)
        }
        TimeoutRule::PassOnly => {
            let next = scoring::advance_turn(teams, active_team_id);
            (teams.to_vec(), Vec::new(), next)
        }
    }
}

#[cfg(test)]
mod tests;
