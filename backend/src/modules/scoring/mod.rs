use serde::{Deserialize, Serialize};

pub const MAX_HEARTS: i32 = 3;
pub const MIN_HEARTS: i32 = 0;

pub const REASON_CORRECT_PLACEMENT: &str = "correct_placement";
/// A placement on the wrong side of the bar: no points, one heart gone (FR-030).
pub const REASON_INCORRECT_PLACEMENT: &str = "incorrect_placement";
pub const REASON_TRIPMINE_HIT: &str = "tripmine_hit";
pub const REASON_SEARCHED_ITEM_FOUND: &str = "searched_item_found";
pub const REASON_SAFE_PICK: &str = "safe_pick";
pub const REASON_FINAL_ITEM: &str = "final_item";
pub const REASON_ZERO_HEARTS: &str = "zero_hearts";
pub const REASON_GAME_MASTER: &str = "game_master";
pub const REASON_ROUND_RESET: &str = "round_reset";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    PointsAwarded,
    PointsRemoved,
    HeartLost,
    HeartRestored,
    ManualAdjustment,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreEvent {
    pub team_id: String,
    pub kind: EventKind,
    /// Signed, matching the team table's delta.
    pub amount: i32,
    pub reason: String,
}

/// A team's row as the scoring rules see it: plain data, no database handle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamStanding {
    pub id: String,
    pub score: i32,
    pub hearts: i32,
    pub turn_index: i32,
}

/// Outcome of a scoring step: the new standings plus the events to append to the
/// append-only `score_event` table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoringOutcome {
    pub teams: Vec<TeamStanding>,
    pub events: Vec<ScoreEvent>,
}

/// Add or remove points. A positive delta is `points_awarded`, a negative delta is
/// `points_removed`; scores accumulate and are never reset by this module.
pub fn apply_points(
    teams: &[TeamStanding],
    team_id: &str,
    delta: i32,
    reason: &str,
) -> ScoringOutcome {
    let kind = if delta < 0 {
        EventKind::PointsRemoved
    } else {
        EventKind::PointsAwarded
    };
    mutate_score(teams, team_id, delta, kind, reason)
}

/// Change hearts by `delta`, clamped to `0..=3` (FR-016, FR-030).
pub fn apply_hearts(
    teams: &[TeamStanding],
    team_id: &str,
    delta: i32,
    reason: &str,
) -> ScoringOutcome {
    let kind = if delta < 0 {
        EventKind::HeartLost
    } else {
        EventKind::HeartRestored
    };
    mutate_hearts(teams, team_id, delta, kind, reason)
}

/// The gamemaster's manual score or heart change (FR-023).
pub fn manual_adjustment(
    teams: &[TeamStanding],
    team_id: &str,
    delta: i32,
    is_hearts: bool,
) -> ScoringOutcome {
    if is_hearts {
        mutate_hearts(
            teams,
            team_id,
            delta,
            EventKind::ManualAdjustment,
            REASON_GAME_MASTER,
        )
    } else {
        mutate_score(
            teams,
            team_id,
            delta,
            EventKind::ManualAdjustment,
            REASON_GAME_MASTER,
        )
    }
}

fn mutate_score(
    teams: &[TeamStanding],
    team_id: &str,
    delta: i32,
    kind: EventKind,
    reason: &str,
) -> ScoringOutcome {
    let mut next = teams.to_vec();
    let mut event = None;
    for team in next.iter_mut() {
        if team.id == team_id {
            team.score += delta;
            event = Some(ScoreEvent {
                team_id: team_id.to_string(),
                kind,
                amount: delta,
                reason: reason.to_string(),
            });
            break;
        }
    }
    ScoringOutcome {
        teams: next,
        events: event.into_iter().collect(),
    }
}

fn mutate_hearts(
    teams: &[TeamStanding],
    team_id: &str,
    delta: i32,
    kind: EventKind,
    reason: &str,
) -> ScoringOutcome {
    let mut next = teams.to_vec();
    let mut event = None;
    for team in next.iter_mut() {
        if team.id == team_id {
            let before = team.hearts;
            team.hearts = (before + delta).clamp(MIN_HEARTS, MAX_HEARTS);
            let applied = team.hearts - before;
            if applied != 0 {
                event = Some(ScoreEvent {
                    team_id: team_id.to_string(),
                    kind,
                    amount: applied,
                    reason: reason.to_string(),
                });
            }
            break;
        }
    }
    ScoringOutcome {
        teams: next,
        events: event.into_iter().collect(),
    }
}

/// Next team in rotation that still holds hearts. A team at 0 hearts is skipped for
/// the rest of the round (FR-018, FR-019). Returns `None` when no team has hearts,
/// which ends the round.
pub fn advance_turn(teams: &[TeamStanding], current_team_id: &str) -> Option<String> {
    if teams.is_empty() {
        return None;
    }
    let mut ordered: Vec<&TeamStanding> = teams.iter().collect();
    ordered.sort_by_key(|t| t.turn_index);

    let start = ordered
        .iter()
        .position(|t| t.id == current_team_id)
        .map(|i| (i + 1) % ordered.len())
        .unwrap_or(0);

    for offset in 0..ordered.len() {
        let candidate = ordered[(start + offset) % ordered.len()];
        if candidate.hearts > MIN_HEARTS {
            return Some(candidate.id.clone());
        }
    }
    None
}

/// True when `requested` is the team on turn. Only the active team acts; the
/// gamemaster acts on that team's behalf by naming it, so one check serves both
/// views (FR-044).
pub fn is_active_turn(active_team_id: Option<&str>, requested: &str) -> bool {
    matches!(active_team_id, Some(active) if active == requested)
}

/// The team holding the first turn of round `round_number`, rotating one team per
/// round (FR-015).
pub fn starting_team_for_round(round_number: i32, team_count: i32) -> String {
    if team_count <= 0 {
        return "team_1".to_string();
    }
    let index = ((round_number - 1).rem_euclid(team_count)) + 1;
    format!("team_{index}")
}

/// Hearts return to 3 at the start of every round; scores are left alone (FR-016,
/// FR-017).
pub fn reset_hearts_for_round(teams: &[TeamStanding]) -> ScoringOutcome {
    let mut next = teams.to_vec();
    let mut events = Vec::new();
    for team in next.iter_mut() {
        if team.hearts != MAX_HEARTS {
            let delta = MAX_HEARTS - team.hearts;
            team.hearts = MAX_HEARTS;
            events.push(ScoreEvent {
                team_id: team.id.clone(),
                kind: EventKind::HeartRestored,
                amount: delta,
                reason: REASON_ROUND_RESET.to_string(),
            });
        }
    }
    ScoringOutcome {
        teams: next,
        events,
    }
}

#[cfg(test)]
mod tests;
