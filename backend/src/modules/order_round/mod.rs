//! Order round rules (US3, T059-T072).
//!
//! Everything that decides a placement is a plain function over plain data, so the
//! scoring rules can be unit tested without a database. The three invariants the
//! whole module exists to keep:
//!
//! * the bar holds lower values to the anchor's left and higher values to its
//!   right (FR-025), and each side is capped at five items;
//! * the ten candidates are drawn at random, five from each side of the anchor,
//!   and never include the anchor itself (FR-026, FR-027);
//! * the bar is stored in placement order and is never re-sorted by value
//!   (FR-029), so a correct placement that breaks the ascending run is a mistake.

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::errors::{ApiError, ApiResult};
use crate::modules::scoring::{self, ScoreEvent, ScoringOutcome, TeamStanding};
use crate::modules::sessions::round_state::{self, ListItemValue};

/// Candidates drawn from each side of the anchor (FR-027).
pub const POOL_PER_SIDE: usize = 5;
/// Candidates in an Order round: five below the anchor, five above (FR-027).
pub const POOL_SIZE: usize = POOL_PER_SIDE * 2;
/// Items each side of the bar can hold (FR-025).
pub const BAR_SLOTS_PER_SIDE: usize = 5;
/// The smallest list an Order round can be built from: ten candidates and an anchor.
pub const MIN_ITEMS_FOR_ORDER: usize = POOL_SIZE + 1;
/// Points for a correct placement (FR-029).
pub const POINTS_PER_PLACEMENT: i32 = 10;
/// Hearts lost for a wrong placement (FR-030).
pub const HEARTS_LOST_PER_MISTAKE: i32 = 1;
/// `slot_index` meaning "an item is selected but no slot has been chosen yet".
pub const SLOT_UNCHOSEN: i32 = -1;

/// Which half of the bar a placement targets. Left holds the lower values and right
/// the higher ones, whatever order the list itself is sorted in (FR-025).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BarSide {
    Left,
    Right,
}

impl BarSide {
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "left" => Some(BarSide::Left),
            "right" => Some(BarSide::Right),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            BarSide::Left => "left",
            BarSide::Right => "right",
        }
    }

    pub fn other(self) -> Self {
        match self {
            BarSide::Left => BarSide::Right,
            BarSide::Right => BarSide::Left,
        }
    }
}

/// The item a team has picked but not yet dropped onto the bar (FR-028). Stored as
/// the `order_state.pending` JSON column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingPlacement {
    pub item_id: String,
    /// Carried so a view can label the pending choice without a second lookup; the
    /// state reader's own `PendingPlacement` expects this field.
    pub name: String,
    /// Pre-filled with the item's own side so the view can highlight it; `choose_slot`
    /// is what settles where it actually goes.
    pub side: BarSide,
    /// `SLOT_UNCHOSEN` until the team picks a slot.
    pub slot_index: i32,
    pub team_id: String,
}

/// The whole Order board as the rules see it.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderBoard {
    pub anchor: ListItemValue,
    pub pool: Vec<ListItemValue>,
    pub left: Vec<ListItemValue>,
    pub right: Vec<ListItemValue>,
    pub pending: Option<PendingPlacement>,
    /// Which team placed each item on a side (feature 004, US2). Pool and anchor
    /// items never appear here. Missing on an older database, hence an empty map.
    pub placed_by: BTreeMap<String, String>,
}

/// A freshly sampled round: the anchor plus the ten candidates, before any placement.
#[derive(Debug, Clone, PartialEq)]
pub struct OrderSetup {
    pub anchor: ListItemValue,
    pub pool: Vec<ListItemValue>,
}

impl OrderSetup {
    /// The starting board: anchor in the centre, both sides empty, nothing pending.
    pub fn into_board(self) -> OrderBoard {
        OrderBoard {
            anchor: self.anchor,
            pool: self.pool,
            left: Vec::new(),
            right: Vec::new(),
            pending: None,
            placed_by: BTreeMap::new(),
        }
    }
}

impl OrderBoard {
    pub fn side(&self, side: BarSide) -> &[ListItemValue] {
        match side {
            BarSide::Left => &self.left,
            BarSide::Right => &self.right,
        }
    }

    pub fn side_mut(&mut self, side: BarSide) -> &mut Vec<ListItemValue> {
        match side {
            BarSide::Left => &mut self.left,
            BarSide::Right => &mut self.right,
        }
    }

    pub fn find(&self, item_id: &str) -> Option<&ListItemValue> {
        self.pool
            .iter()
            .chain(self.left.iter())
            .chain(self.right.iter())
            .chain(std::iter::once(&self.anchor))
            .find(|item| item.item_id == item_id)
    }

    /// The slots a side currently offers: `0..=len`, which is left of the first item,
    /// between two items, and right of the last one (FR-028).
    fn check_slot(&self, side: BarSide, slot_index: i32) -> ApiResult<usize> {
        let items = self.side(side);
        if items.len() >= BAR_SLOTS_PER_SIDE {
            return Err(ApiError::bad_request(format!(
                "The {} side of the bar already holds {BAR_SLOTS_PER_SIDE} items.",
                side.as_str()
            )));
        }
        if slot_index < 0 || slot_index > items.len() as i32 {
            return Err(ApiError::bad_request(format!(
                "Slot {slot_index} is not on the {} side. The slots there are 0 to {}.",
                side.as_str(),
                items.len()
            )));
        }
        Ok(slot_index as usize)
    }

    /// Pick a pool item for `team_id` (FR-028). Any previous selection is dropped,
    /// because a team places one item at a time.
    pub fn select_item(&self, item_id: &str, team_id: &str) -> ApiResult<OrderBoard> {
        let item = self
            .pool
            .iter()
            .find(|candidate| candidate.item_id == item_id)
            .ok_or_else(|| {
                ApiError::not_ready(format!("Item '{item_id}' is not in the pool any more."))
            })?;
        let mut next = self.clone();
        next.pending = Some(PendingPlacement {
            item_id: item_id.to_string(),
            name: item.name.clone(),
            side: side_for_value(item, &self.anchor),
            slot_index: SLOT_UNCHOSEN,
            team_id: team_id.to_string(),
        });
        Ok(next)
    }

    /// Settle where the selected item goes, without scoring it yet.
    pub fn choose_slot(&self, side: BarSide, slot_index: i32) -> ApiResult<OrderBoard> {
        let mut next = self.clone();
        let index = self.check_slot(side, slot_index)?;
        let pending = next.pending.as_mut().ok_or_else(|| {
            ApiError::not_ready("No item has been selected, so there is no slot to choose.")
        })?;
        pending.side = side;
        pending.slot_index = index as i32;
        Ok(next)
    }

    /// Put the selected item back with no scoring and no heart change (FR-033).
    pub fn cancel_selection(&self) -> ApiResult<OrderBoard> {
        if self.pending.is_none() {
            return Err(ApiError::not_ready("There is no selection to cancel."));
        }
        let mut next = self.clone();
        next.pending = None;
        Ok(next)
    }
}

/// The side an item belongs on, judged by its value against the anchor (FR-025). An
/// item equal to the anchor satisfies both sides, which is what makes FR-034 work.
fn side_for_value(item: &ListItemValue, anchor: &ListItemValue) -> BarSide {
    if item.numeric <= anchor.numeric {
        BarSide::Left
    } else {
        BarSide::Right
    }
}

/// Draw the anchor and the ten candidates (FR-026, FR-027).
///
/// `sorted` must already be in the list's own direction, because "the middle of the
/// list in the list's chosen direction" is what fixes the anchor (FR-026). The
/// candidates are then drawn from whichever side of that anchor they sit on.
pub fn sample_order_round(
    sorted: &[ListItemValue],
    rng: &mut impl rand::Rng,
) -> ApiResult<OrderSetup> {
    if sorted.len() < MIN_ITEMS_FOR_ORDER {
        return Err(ApiError::not_ready(format!(
            "An Order round needs at least {MIN_ITEMS_FOR_ORDER} items with values."
        )));
    }

    let anchor_index = sorted.len() / 2;
    let anchor = sorted[anchor_index].clone();

    let mut below: Vec<usize> = (0..anchor_index).collect();
    let mut above: Vec<usize> = (anchor_index + 1..sorted.len()).collect();
    shuffle(&mut below, rng);
    shuffle(&mut above, rng);

    // Five from each side. When the anchor is not quite centred one side can be
    // short, and that side is topped up from the other so the round still deals ten
    // candidates instead of quietly dealing fewer.
    let (mut below_taken, mut above_taken) = (0usize, 0usize);
    let mut picked: Vec<ListItemValue> = Vec::with_capacity(POOL_SIZE);
    for _ in 0..POOL_PER_SIDE {
        if below_taken < below.len() {
            picked.push(sorted[below[below_taken]].clone());
            below_taken += 1;
        } else if above_taken < above.len() {
            picked.push(sorted[above[above_taken]].clone());
            above_taken += 1;
        }
    }
    for _ in 0..POOL_PER_SIDE {
        if above_taken < above.len() {
            picked.push(sorted[above[above_taken]].clone());
            above_taken += 1;
        } else if below_taken < below.len() {
            picked.push(sorted[below[below_taken]].clone());
            below_taken += 1;
        }
    }

    picked.retain(|item| item.item_id != anchor.item_id);
    // The pool is shown in random order, not value order (FR-027).
    let mut pool = picked;
    shuffle(&mut pool, rng);

    Ok(OrderSetup { anchor, pool })
}

/// Load a list's values and sample a round from them (T068).
pub fn start_order_round(
    conn: &Connection,
    list_id: &str,
    column_id: &str,
    low_to_high: bool,
    rng: &mut impl rand::Rng,
) -> ApiResult<OrderSetup> {
    let items = round_state::load_ordered_items(conn, list_id, column_id, low_to_high)?;
    sample_order_round(&items, rng)
}

/// Fisher-Yates, so a seeded generator makes a whole round reproducible.
fn shuffle<T>(items: &mut [T], rng: &mut impl rand::Rng) {
    for index in (1..items.len()).rev() {
        let pick = rng.random_range(0..=index);
        items.swap(index, pick);
    }
}

/// Check a proposed placement and report whether it is correct.
///
/// `Ok(true)` is a correct placement, `Ok(false)` a wrong one that costs a heart
/// (FR-030). An out-of-range slot is an error rather than a wrong placement, because
/// it names a position the bar does not offer (FR-028).
pub fn validate_placement(board: &OrderBoard, side: BarSide, slot_index: i32) -> ApiResult<bool> {
    let pending = board
        .pending
        .as_ref()
        .ok_or_else(|| ApiError::not_ready("No item has been selected yet."))?;
    if pending.slot_index == SLOT_UNCHOSEN {
        return Err(ApiError::not_ready(
            "Choose a slot before the placement can be judged.",
        ));
    }

    let index = board.check_slot(side, slot_index)?;
    let item = board.find(&pending.item_id).ok_or_else(|| {
        ApiError::not_ready(format!(
            "The selected item '{}' is no longer on the board.",
            pending.item_id
        ))
    })?;

    let mut candidate = board.side(side).to_vec();
    candidate.insert(index, item.clone());
    Ok(side_stays_ordered(side, &board.anchor, &candidate))
}

/// The bar invariant: each side ascends towards the anchor, ties are allowed
/// (FR-034), and left holds nothing above the anchor while right holds nothing below.
fn side_stays_ordered(side: BarSide, anchor: &ListItemValue, candidate: &[ListItemValue]) -> bool {
    if candidate
        .windows(2)
        .any(|pair| pair[1].numeric < pair[0].numeric)
    {
        return false;
    }
    match side {
        BarSide::Left => candidate.iter().all(|item| item.numeric <= anchor.numeric),
        BarSide::Right => candidate.iter().all(|item| item.numeric >= anchor.numeric),
    }
}

/// What confirming a placement did.
#[derive(Debug, Clone, PartialEq)]
pub struct PlacementOutcome {
    pub correct: bool,
    pub board: OrderBoard,
    pub teams: Vec<TeamStanding>,
    pub events: Vec<ScoreEvent>,
    /// The team on turn afterwards, or `None` when the round is over (FR-018).
    pub next_team_id: Option<String>,
    pub round_over: bool,
}

/// Score the pending placement and hand the turn on (FR-029, FR-030).
///
/// A correct placement joins the bar and takes 10 points; a wrong one leaves the item
/// in the pool for another attempt (FR-031) and takes one heart. Either way the turn
/// passes (FR-018).
pub fn confirm_placement(
    board: &OrderBoard,
    teams: &[TeamStanding],
) -> ApiResult<PlacementOutcome> {
    let pending = board
        .pending
        .clone()
        .ok_or_else(|| ApiError::not_ready("There is no placement waiting to be confirmed."))?;

    // Recomputed here rather than trusted from the caller, so the score can never
    // disagree with the board.
    let correct = validate_placement(board, pending.side, pending.slot_index)?;
    let item = board.find(&pending.item_id).cloned().ok_or_else(|| {
        ApiError::not_ready(format!(
            "The selected item '{}' is no longer on the board.",
            pending.item_id
        ))
    })?;

    let mut next = board.clone();
    let outcome: ScoringOutcome = if correct {
        next.side_mut(pending.side)
            .insert(pending.slot_index as usize, item.clone());
        next.pool
            .retain(|candidate| candidate.item_id != item.item_id);
        next.placed_by
            .insert(item.item_id.clone(), pending.team_id.clone());
        scoring::apply_points(
            teams,
            &pending.team_id,
            POINTS_PER_PLACEMENT,
            scoring::REASON_CORRECT_PLACEMENT,
        )
    } else {
        scoring::apply_hearts(
            teams,
            &pending.team_id,
            -HEARTS_LOST_PER_MISTAKE,
            scoring::REASON_INCORRECT_PLACEMENT,
        )
    };
    next.pending = None;

    let next_team_id = scoring::advance_turn(&outcome.teams, &pending.team_id);
    let round_over = round_is_over(&next, &outcome.teams);

    Ok(PlacementOutcome {
        correct,
        board: next,
        teams: outcome.teams,
        events: outcome.events,
        next_team_id,
        round_over,
    })
}

/// Sort every item into its correct bar position, for the reveal (feature 004, US3).
///
/// Left holds the values `<=` the anchor and right the values `>` the anchor, each
/// ascending, which is exactly the bar invariant `side_stays_ordered` checks. The
/// sort is stable on the value, so ties keep the order they were handed in. Pure and
/// presentational: it never mutates the stored board, so the reveal scores nothing.
pub fn arrange_revealed_order(
    anchor: &ListItemValue,
    items: &[ListItemValue],
) -> (Vec<ListItemValue>, Vec<ListItemValue>) {
    let mut left: Vec<ListItemValue> = items
        .iter()
        .filter(|item| item.numeric <= anchor.numeric)
        .cloned()
        .collect();
    let mut right: Vec<ListItemValue> = items
        .iter()
        .filter(|item| item.numeric > anchor.numeric)
        .cloned()
        .collect();
    left.sort_by(|a, b| a.numeric.total_cmp(&b.numeric));
    right.sort_by(|a, b| a.numeric.total_cmp(&b.numeric));
    left.truncate(BAR_SLOTS_PER_SIDE);
    right.truncate(BAR_SLOTS_PER_SIDE);
    (left, right)
}

/// Take a placed item off the bar and put it back in the pool (FR-033). No scoring and
/// no turn change: only the gamemaster may do this, to undo a misdrop.
pub fn return_to_pool(board: &OrderBoard, item_id: &str) -> ApiResult<OrderBoard> {
    let mut next = board.clone();
    for side in [BarSide::Left, BarSide::Right] {
        if let Some(index) = next
            .side(side)
            .iter()
            .position(|item| item.item_id == item_id)
        {
            let item = next.side_mut(side).remove(index);
            next.pool.push(item);
            next.placed_by.remove(item_id);
            return Ok(next);
        }
    }
    Err(ApiError::not_ready(format!(
        "Item '{item_id}' is not on the bar, so it cannot be returned to the pool."
    )))
}

/// The round ends when every candidate is placed, or when no team holds a heart
/// (FR-032).
pub fn round_is_over(board: &OrderBoard, teams: &[TeamStanding]) -> bool {
    if !teams.iter().any(|team| team.hearts > scoring::MIN_HEARTS) {
        return true;
    }
    board.pending.is_none() && board.pool.is_empty()
}

// ---------------------------------------------------------------------------
// Persistence (T072)
// ---------------------------------------------------------------------------

/// Every team's standing for the session, in turn order.
pub fn load_teams(conn: &Connection, session_id: &str) -> ApiResult<Vec<TeamStanding>> {
    let mut statement = conn
        .prepare("SELECT id, score, hearts, turn_index FROM team WHERE session_id = ?1 ORDER BY turn_index")
        .map_err(ApiError::from)?;
    let teams = statement
        .query_map(params![session_id], |row| {
            Ok(TeamStanding {
                id: row.get(0)?,
                score: row.get(1)?,
                hearts: row.get(2)?,
                turn_index: row.get(3)?,
            })
        })
        .map_err(ApiError::from)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)?;
    Ok(teams)
}

/// Write the standings back. Scores accumulate across rounds and are never reset
/// here (FR-017).
pub fn save_teams(conn: &Connection, session_id: &str, teams: &[TeamStanding]) -> ApiResult<()> {
    for team in teams {
        conn.execute(
            "UPDATE team SET score = ?3, hearts = ?4 WHERE session_id = ?1 AND id = ?2",
            params![session_id, team.id, team.score, team.hearts],
        )
        .map_err(ApiError::from)?;
    }
    Ok(())
}

/// Append to the audit trail, which is what makes the scorecard checkable (SC-005).
pub fn append_events(
    conn: &Connection,
    session_id: &str,
    round_number: i32,
    events: &[ScoreEvent],
) -> ApiResult<()> {
    for event in events {
        conn.execute(
            "INSERT INTO score_event (id, session_id, round_number, team_id, kind, amount, reason)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                event_id(),
                session_id,
                round_number,
                event.team_id,
                event_kind(event.kind),
                event.amount,
                event.reason
            ],
        )
        .map_err(ApiError::from)?;
    }
    Ok(())
}

/// The column the current round is being played on, which is the column every Order
/// value is read from.
fn current_column_id(conn: &Connection, session_id: &str) -> ApiResult<String> {
    let column_id: Option<String> = conn
        .query_row(
            "SELECT rc.column_id FROM round_config rc
             JOIN game_session g ON g.id = rc.session_id AND g.current_round = rc.round_number
             WHERE rc.session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )
        .optional()?;
    Ok(column_id.unwrap_or_default())
}

/// One item with the value it holds in `column_id`, or `None` when the item is gone
/// (a list deleted mid-round must not take the round down with it).
fn load_item(
    conn: &Connection,
    column_id: &str,
    item_id: &str,
    fallback_numeric: Option<f64>,
) -> ApiResult<Option<ListItemValue>> {
    let row = conn
        .query_row(
            "SELECT i.name, v.numeric, v.display
             FROM item i
             LEFT JOIN item_value v ON v.item_id = i.id AND v.column_id = ?2
             WHERE i.id = ?1",
            params![item_id, column_id],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                    row.get::<_, Option<f64>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                ))
            },
        )
        .optional()?;
    let Some((name, numeric, display)) = row else {
        return Ok(None);
    };
    let numeric = numeric.or(fallback_numeric).unwrap_or_default();
    Ok(Some(ListItemValue {
        item_id: item_id.to_string(),
        name,
        display: display.unwrap_or_else(|| crate::state_reader::format_value(numeric)),
        numeric,
    }))
}

fn load_ids(conn: &Connection, column_id: &str, raw: &str) -> ApiResult<Vec<ListItemValue>> {
    let ids: Vec<String> = serde_json::from_str(raw).unwrap_or_default();
    let mut items = Vec::with_capacity(ids.len());
    for id in ids {
        if let Some(item) = load_item(conn, column_id, &id, None)? {
            items.push(item);
        }
    }
    Ok(items)
}

/// The `order_state` columns a board is rebuilt from.
type OrderStateRow = (
    String,
    f64,
    String,
    String,
    String,
    Option<String>,
    Option<String>,
);

/// Read the stored board back with its values, so a placement can be judged.
pub fn load_board(conn: &Connection, session_id: &str) -> ApiResult<Option<OrderBoard>> {
    let row: Option<OrderStateRow> = conn
        .query_row(
            "SELECT anchor_item_id, anchor_value, pool, left_ids, right_ids, pending, placed_by
             FROM order_state WHERE session_id = ?1",
            params![session_id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                ))
            },
        )
        .optional()?;
    let Some((anchor_item_id, anchor_value, pool, left, right, pending, placed_by)) = row else {
        return Ok(None);
    };

    let column_id = current_column_id(conn, session_id)?;
    let anchor = load_item(conn, &column_id, &anchor_item_id, Some(anchor_value))?.unwrap_or(
        ListItemValue {
            item_id: anchor_item_id.clone(),
            name: String::new(),
            display: crate::state_reader::format_value(anchor_value),
            numeric: anchor_value,
        },
    );

    Ok(Some(OrderBoard {
        anchor,
        pool: load_ids(conn, &column_id, &pool)?,
        left: load_ids(conn, &column_id, &left)?,
        right: load_ids(conn, &column_id, &right)?,
        pending: pending
            .as_deref()
            .and_then(|raw| serde_json::from_str(raw).ok()),
        placed_by: placed_by
            .as_deref()
            .and_then(|raw| serde_json::from_str(raw).ok())
            .unwrap_or_default(),
    }))
}

/// Write the board back to the four JSON columns, in placement order and never
/// re-sorted by value (T072, FR-029).
pub fn save_board(conn: &Connection, session_id: &str, board: &OrderBoard) -> ApiResult<()> {
    let ids = |items: &[ListItemValue]| -> ApiResult<String> {
        serde_json::to_string(
            &items
                .iter()
                .map(|item| item.item_id.clone())
                .collect::<Vec<String>>(),
        )
        .map_err(ApiError::from)
    };
    let pending = match &board.pending {
        Some(pending) => serde_json::to_string(pending).map_err(ApiError::from)?,
        None => "null".to_string(),
    };
    let placed_by = serde_json::to_string(&board.placed_by).map_err(ApiError::from)?;

    conn.execute(
        "INSERT OR REPLACE INTO order_state
            (session_id, anchor_item_id, anchor_value, pool, left_ids, right_ids, pending, placed_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            session_id,
            board.anchor.item_id,
            board.anchor.numeric,
            ids(&board.pool)?,
            ids(&board.left)?,
            ids(&board.right)?,
            pending,
            placed_by
        ],
    )
    .map_err(ApiError::from)?;
    Ok(())
}

/// Reveal every value in every view (FR-024). One flag, honoured by `redact`.
pub fn set_reveal_values(conn: &Connection, session_id: &str, revealed: bool) -> ApiResult<()> {
    conn.execute(
        "UPDATE game_session SET reveal_values = ?2 WHERE id = ?1",
        params![session_id, i64::from(revealed)],
    )
    .map_err(ApiError::from)?;
    Ok(())
}

fn event_kind(kind: scoring::EventKind) -> &'static str {
    match kind {
        scoring::EventKind::PointsAwarded => "points_awarded",
        scoring::EventKind::PointsRemoved => "points_removed",
        scoring::EventKind::HeartLost => "heart_lost",
        scoring::EventKind::HeartRestored => "heart_restored",
        scoring::EventKind::ManualAdjustment => "manual_adjustment",
    }
}

fn event_id() -> String {
    use rand::Rng;
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::rng();
    let body: String = (0..12)
        .map(|_| ALPHABET[rng.random_range(0..ALPHABET.len())] as char)
        .collect();
    format!("ev-{body}")
}

#[cfg(test)]
mod tests;
