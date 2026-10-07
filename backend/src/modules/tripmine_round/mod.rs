//! Tripmine round rules (US4, T079-T092).
//!
//! The ten items, their ranks, and the mined end are fixed the moment the round
//! starts (FR-036a). Everything a pick does is a plain function over the stored
//! grid, so the four outcomes can be unit tested without a database:
//!
//! * the searched position, +10, search advances to the next unrevealed position
//!   in the search direction (FR-040);
//! * a wrong, non-mined position, +5, searched position untouched (FR-041);
//! * the mine, -10 for the picker and +10 for every other team, everything
//!   revealed, round over (FR-042);
//! * the valid position when only two are left, +20 instead of 10, everything
//!   revealed, round over (FR-043).

use rand::Rng;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

use crate::errors::{ApiError, ApiResult};
use crate::modules::scoring::{self, ScoreEvent, ScoringOutcome, TeamStanding};
use crate::modules::sessions::TripmineSide;
use crate::modules::sessions::round_state::{self, ListItemValue};

/// Items in a Tripmine grid (FR-035).
pub const TRIPMINE_SLOTS: usize = 10;
/// Points for the item at the searched position (FR-040).
pub const POINTS_SEARCHED: i32 = 10;
/// Points for a wrong, non-mined pick (FR-041).
pub const POINTS_SAFE: i32 = 5;
/// Points lost by the team that sets off the mine (FR-042).
pub const POINTS_TRIPMINE_PENALTY: i32 = -10;
/// Points every other team gains when the mine goes off (FR-042).
pub const POINTS_TRIPMINE_OTHERS: i32 = 10;
/// Points for the valid item with exactly two left (FR-043).
pub const POINTS_FINAL: i32 = 20;

/// One item of the grid, with the answer key that never reaches a player view.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TripmineSlot {
    pub item: ListItemValue,
    /// Position in the sorted ranking, 1 (lowest) to 10 (highest) (FR-037).
    pub true_rank: i32,
    pub is_tripmine: bool,
    pub revealed: bool,
    pub revealed_by: Option<String>,
    pub points_awarded: Option<i32>,
}

/// The whole Tripmine round as the rules see it.
#[derive(Debug, Clone, PartialEq)]
pub struct TripmineRound {
    pub side: TripmineSide,
    pub searched_rank: i32,
    pub items: Vec<TripmineSlot>,
    pub hit: bool,
    pub hit_by: Option<String>,
    pub completed: bool,
}

impl TripmineRound {
    pub fn find(&self, item_id: &str) -> Option<&TripmineSlot> {
        self.items.iter().find(|slot| slot.item.item_id == item_id)
    }

    pub fn unrevealed(&self) -> usize {
        self.items.iter().filter(|slot| !slot.revealed).count()
    }
}

/// What one pick changed.
#[derive(Debug, Clone, PartialEq)]
pub struct PickOutcome {
    pub round: TripmineRound,
    pub teams: Vec<TeamStanding>,
    pub events: Vec<ScoreEvent>,
    /// The team on turn afterwards, or `None` once the round is over.
    pub next_team_id: Option<String>,
}

/// The rank the search starts from: `1` when hunting the highest item, `10` when
/// hunting the lowest (FR-037).
pub fn starting_search_rank(side: TripmineSide) -> i32 {
    match side {
        TripmineSide::Highest => 1,
        TripmineSide::Lowest => TRIPMINE_SLOTS as i32,
        // Random has no fixed start; `start_tripmine_round` picks a random target.
        TripmineSide::Random => 1,
    }
}

/// A random unrevealed, non-mine rank — the Random side's searched target. `None`
/// only when nothing but the mine is left.
fn random_search_rank(items: &[TripmineSlot], rng: &mut impl Rng) -> Option<i32> {
    let candidates: Vec<i32> = items
        .iter()
        .filter(|slot| !slot.revealed && !slot.is_tripmine)
        .map(|slot| slot.true_rank)
        .collect();
    if candidates.is_empty() {
        None
    } else {
        Some(candidates[rng.random_range(0..candidates.len())])
    }
}

/// The next unrevealed position in the search direction, scanning upward from
/// `from_rank` for a highest mine and downward for a lowest one (FR-040).
///
/// Positions behind the search are wrapped to rather than skipped: a team can pick
/// out of order, and the search must still point at a position that is really there.
/// `None` only when nothing is left to reveal.
pub fn next_searched_rank(
    side: TripmineSide,
    items: &[TripmineSlot],
    from_rank: i32,
) -> Option<i32> {
    let mut unrevealed: Vec<i32> = items
        .iter()
        .filter(|slot| !slot.revealed)
        .map(|slot| slot.true_rank)
        .collect();
    unrevealed.sort_unstable();

    match side {
        TripmineSide::Highest => unrevealed
            .iter()
            .copied()
            .filter(|rank| *rank > from_rank)
            .min()
            .or_else(|| unrevealed.first().copied()),
        TripmineSide::Lowest => unrevealed
            .iter()
            .copied()
            .filter(|rank| *rank < from_rank)
            .max()
            .or_else(|| unrevealed.last().copied()),
        // Random: any unrevealed, non-mine rank.
        TripmineSide::Random => {
            let candidates: Vec<i32> = items
                .iter()
                .filter(|slot| !slot.revealed && !slot.is_tripmine)
                .map(|slot| slot.true_rank)
                .collect();
            if candidates.is_empty() {
                None
            } else {
                Some(candidates[rand::rng().random_range(0..candidates.len())])
            }
        }
    }
}

/// Draw ten items at random and rank them 1 (lowest) to 10 (highest), marking the
/// mined end (FR-035, FR-036, FR-037).
pub fn sample_tripmine_grid(
    items: &[ListItemValue],
    side: TripmineSide,
    rng: &mut impl rand::Rng,
) -> ApiResult<Vec<TripmineSlot>> {
    if items.len() < TRIPMINE_SLOTS {
        return Err(ApiError::not_ready(format!(
            "A Tripmine round needs at least {TRIPMINE_SLOTS} items with values."
        )));
    }

    let mut indices: Vec<usize> = (0..items.len()).collect();
    for index in (1..indices.len()).rev() {
        let pick = rng.random_range(0..=index);
        indices.swap(index, pick);
    }
    let mut chosen: Vec<ListItemValue> = indices
        .into_iter()
        .take(TRIPMINE_SLOTS)
        .map(|index| items[index].clone())
        .collect();
    chosen.sort_by(|a, b| {
        a.numeric
            .partial_cmp(&b.numeric)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mined_rank = match side {
        TripmineSide::Highest => TRIPMINE_SLOTS as i32,
        TripmineSide::Lowest => 1,
        // Hard: the mine is any rank.
        TripmineSide::Random => rng.random_range(1..=TRIPMINE_SLOTS as i32),
    };

    Ok(chosen
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let true_rank = index as i32 + 1;
            TripmineSlot {
                item,
                true_rank,
                is_tripmine: true_rank == mined_rank,
                revealed: false,
                revealed_by: None,
                points_awarded: None,
            }
        })
        .collect())
}

/// Shuffle the grid's display order in place. Each slot keeps its `true_rank` and
/// `is_tripmine`; only the array sequence changes, and no rule reads by index, so
/// every outcome is unaffected (FR-009, FR-010).
pub fn shuffle_grid(grid: &mut [TripmineSlot], rng: &mut impl rand::Rng) {
    for index in (1..grid.len()).rev() {
        let pick = rng.random_range(0..=index);
        grid.swap(index, pick);
    }
}

/// Load a list's values and sample a Tripmine grid from them (T088).
pub fn start_tripmine_round(
    conn: &Connection,
    list_id: &str,
    column_id: &str,
    side: TripmineSide,
    rng: &mut impl rand::Rng,
) -> ApiResult<TripmineRound> {
    // `true_rank` is always 1 (lowest) to 10 (highest) whatever order the list is
    // kept in, so the grid is always sampled and ranked ascending (FR-037).
    let items = round_state::load_ordered_items(conn, list_id, column_id, true)?;
    let mut grid = sample_tripmine_grid(&items, side, rng)?;
    // The displayed order is shuffled so the ten items are not sweepable in rank
    // order (FR-009).
    shuffle_grid(&mut grid, rng);
    let searched_rank = match side {
        TripmineSide::Random => random_search_rank(&grid, rng).unwrap_or(1),
        _ => starting_search_rank(side),
    };
    Ok(TripmineRound {
        side,
        searched_rank,
        items: grid,
        hit: false,
        hit_by: None,
        completed: false,
    })
}

/// The side is fixed while the round runs, so a request that tries to change it is
/// refused (FR-036). A request that repeats the current side is harmless.
pub fn ensure_side_unchanged(round: &TripmineRound, requested: Option<&str>) -> ApiResult<()> {
    let Some(raw) = requested else {
        return Ok(());
    };
    let parsed = TripmineSide::parse(raw)
        .ok_or_else(|| ApiError::bad_request("side must be highest, lowest, or random."))?;
    if parsed == round.side {
        return Ok(());
    }
    Err(ApiError::conflict(
        "The tripmine side is fixed once the round has started.",
    ))
}

/// Score one pick (FR-039 to FR-043). The picker has already been checked to be the
/// team on turn by the caller.
pub fn pick_item(
    round: &TripmineRound,
    item_id: &str,
    picker_team_id: &str,
    teams: &[TeamStanding],
) -> ApiResult<PickOutcome> {
    if round.completed {
        return Err(ApiError::not_ready("This Tripmine round is already over."));
    }
    let slot = round.find(item_id).ok_or_else(|| {
        ApiError::not_found(format!("Item '{item_id}' is not in this Tripmine grid."))
    })?;
    if slot.revealed {
        return Err(ApiError::not_ready(format!(
            "'{}' has already been revealed.",
            slot.item.name
        )));
    }

    let picked = slot.clone();
    let mut next = round.clone();
    let unrevealed_before = round.unrevealed();

    if picked.is_tripmine {
        // FR-042: the picker loses 10, every other team gains 10, the whole grid is
        // revealed, and the round ends.
        let mut scoring = scoring::apply_points(
            teams,
            picker_team_id,
            POINTS_TRIPMINE_PENALTY,
            scoring::REASON_TRIPMINE_HIT,
        );
        for team in teams.iter().filter(|team| team.id != picker_team_id) {
            let next = scoring::apply_points(
                &scoring.teams,
                &team.id,
                POINTS_TRIPMINE_OTHERS,
                scoring::REASON_TRIPMINE_HIT,
            );
            scoring = merge(scoring, next);
        }
        for item in next.items.iter_mut() {
            item.revealed = true;
            if item.item.item_id == picked.item.item_id {
                item.revealed_by = Some(picker_team_id.to_string());
                item.points_awarded = Some(POINTS_TRIPMINE_PENALTY);
            }
        }
        next.hit = true;
        next.hit_by = Some(picker_team_id.to_string());
        next.completed = true;
        return Ok(PickOutcome {
            round: next,
            teams: scoring.teams,
            events: scoring.events,
            next_team_id: None,
        });
    }

    let is_searched = picked.true_rank == round.searched_rank;
    let is_final_two = unrevealed_before == 2;

    if is_searched && is_final_two {
        // FR-043: the valid item with exactly two left is worth 20 and ends the
        // round with every position revealed.
        let scoring = scoring::apply_points(
            teams,
            picker_team_id,
            POINTS_FINAL,
            scoring::REASON_FINAL_ITEM,
        );
        for item in next.items.iter_mut() {
            item.revealed = true;
            if item.item.item_id == picked.item.item_id {
                item.revealed_by = Some(picker_team_id.to_string());
                item.points_awarded = Some(POINTS_FINAL);
            }
        }
        next.completed = true;
        return Ok(PickOutcome {
            round: next,
            teams: scoring.teams,
            events: scoring.events,
            next_team_id: None,
        });
    }

    if is_searched {
        // FR-040: +10 and the search advances to the next unrevealed position.
        let scoring = scoring::apply_points(
            teams,
            picker_team_id,
            POINTS_SEARCHED,
            scoring::REASON_SEARCHED_ITEM_FOUND,
        );
        reveal_only(
            &mut next,
            &picked.item.item_id,
            picker_team_id,
            POINTS_SEARCHED,
        );
        next.searched_rank = next_searched_rank(next.side, &next.items, picked.true_rank)
            .unwrap_or(next.searched_rank);
        next.completed = next.unrevealed() == 0;
        let next_team_id = if next.completed {
            None
        } else {
            scoring::advance_turn(&scoring.teams, picker_team_id)
        };
        return Ok(PickOutcome {
            round: next,
            teams: scoring.teams,
            events: scoring.events,
            next_team_id,
        });
    }

    // FR-041: a wrong, non-mined pick is +5, reveals only that position, and leaves
    // the searched position alone.
    let scoring = scoring::apply_points(
        teams,
        picker_team_id,
        POINTS_SAFE,
        scoring::REASON_SAFE_PICK,
    );
    reveal_only(&mut next, &picked.item.item_id, picker_team_id, POINTS_SAFE);
    next.completed = next.unrevealed() == 0;
    let next_team_id = if next.completed {
        None
    } else {
        scoring::advance_turn(&scoring.teams, picker_team_id)
    };
    Ok(PickOutcome {
        round: next,
        teams: scoring.teams,
        events: scoring.events,
        next_team_id,
    })
}

fn reveal_only(round: &mut TripmineRound, item_id: &str, picker_team_id: &str, points: i32) {
    for item in round.items.iter_mut() {
        if item.item.item_id == item_id {
            item.revealed = true;
            item.revealed_by = Some(picker_team_id.to_string());
            item.points_awarded = Some(points);
        }
    }
}

/// Fold a later scoring step into the running outcome.
fn merge(mut acc: ScoringOutcome, next: ScoringOutcome) -> ScoringOutcome {
    acc.teams = next.teams;
    acc.events.extend(next.events);
    acc
}

// ---------------------------------------------------------------------------
// Persistence (T091)
// ---------------------------------------------------------------------------

/// Read the stored Tripmine round, if this session is playing one.
pub fn load_round(conn: &Connection, session_id: &str) -> ApiResult<Option<TripmineRound>> {
    let row: Option<(String, i32, String, i64, Option<String>, i64)> = conn
        .query_row(
            "SELECT side, searched_rank, items, hit, hit_by, completed
             FROM tripmine_state WHERE session_id = ?1",
            params![session_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .optional()?;
    let Some((side, searched_rank, items, hit, hit_by, completed)) = row else {
        return Ok(None);
    };
    let items: Vec<TripmineSlot> = serde_json::from_str(&items).unwrap_or_default();
    Ok(Some(TripmineRound {
        side: TripmineSide::parse(&side).unwrap_or(TripmineSide::Highest),
        searched_rank,
        items,
        hit: hit != 0,
        hit_by,
        completed: completed != 0,
    }))
}

/// Write the round back. The ten items are only ever replaced by a new round
/// start (FR-036a), which is `store_round_start`'s job.
pub fn save_round(conn: &Connection, session_id: &str, round: &TripmineRound) -> ApiResult<()> {
    conn.execute(
        "INSERT OR REPLACE INTO tripmine_state
            (session_id, side, searched_rank, items, hit, hit_by, completed)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            session_id,
            round.side.as_str(),
            round.searched_rank,
            serde_json::to_string(&round.items).map_err(ApiError::from)?,
            i64::from(round.hit),
            round.hit_by,
            i64::from(round.completed),
        ],
    )
    .map_err(ApiError::from)?;
    Ok(())
}

#[cfg(test)]
mod tests;
