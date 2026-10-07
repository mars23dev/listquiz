//! Order round rules, tested without a database (T059-T065, T069).

use rand::SeedableRng;
use rand::rngs::SmallRng;

use super::*;
use crate::errors::ErrorCode;
use crate::modules::scoring::{MAX_HEARTS, TeamStanding};

/// A list of `count` items with ascending values `1..=count`.
fn items(count: usize) -> Vec<ListItemValue> {
    (1..=count)
        .map(|value| ListItemValue {
            item_id: format!("item_{value}"),
            name: format!("Item {value}"),
            display: value.to_string(),
            numeric: value as f64,
            icon_name: None,
        })
        .collect()
}

fn rng(seed: u64) -> SmallRng {
    SmallRng::seed_from_u64(seed)
}

fn board_with(anchor_value: f64, left: &[f64], right: &[f64], pool: &[f64]) -> OrderBoard {
    let make = |value: f64, tag: &str| ListItemValue {
        item_id: format!("{tag}_{value}"),
        name: format!("Item {value}"),
        display: value.to_string(),
        numeric: value,
        icon_name: None,
    };
    OrderBoard {
        anchor: make(anchor_value, "anchor"),
        pool: pool.iter().map(|v| make(*v, "pool")).collect(),
        left: left.iter().map(|v| make(*v, "left")).collect(),
        right: right.iter().map(|v| make(*v, "right")).collect(),
        pending: None,
        placed_by: std::collections::BTreeMap::new(),
    }
}

fn two_teams() -> Vec<TeamStanding> {
    vec![
        TeamStanding {
            id: "team_1".to_string(),
            score: 0,
            hearts: MAX_HEARTS,
            turn_index: 0,
        },
        TeamStanding {
            id: "team_2".to_string(),
            score: 0,
            hearts: MAX_HEARTS,
            turn_index: 1,
        },
    ]
}

/// A board with one pool item already selected by `team_1`.
fn with_pending(
    board: &OrderBoard,
    item_id: &str,
    side: BarSide,
    slot_index: i32,
    team_id: &str,
) -> OrderBoard {
    let mut next = board.clone();
    next.pending = Some(PendingPlacement {
        item_id: item_id.to_string(),
        name: board
            .find(item_id)
            .map(|item| item.name.clone())
            .unwrap_or_default(),
        side,
        slot_index,
        team_id: team_id.to_string(),
    });
    next
}

// T059 ---------------------------------------------------------------------

#[test]
fn sampling_draws_five_below_the_anchor_and_five_above() {
    let sorted = items(21);
    let setup = sample_order_round(&sorted, &mut rng(7)).expect("a round");

    let anchor_value = setup.anchor.numeric;
    assert_eq!(anchor_value, 11.0, "the anchor is the middle item");
    assert_eq!(setup.pool.len(), POOL_SIZE);

    let below = setup
        .pool
        .iter()
        .filter(|item| item.numeric < anchor_value)
        .count();
    let above = setup
        .pool
        .iter()
        .filter(|item| item.numeric > anchor_value)
        .count();
    assert_eq!(below, POOL_PER_SIDE, "five from below the anchor");
    assert_eq!(above, POOL_PER_SIDE, "five from above the anchor");
}

#[test]
fn the_anchor_is_never_one_of_the_candidates() {
    let setup = sample_order_round(&items(21), &mut rng(11)).expect("a round");
    assert!(
        !setup
            .pool
            .iter()
            .any(|item| item.item_id == setup.anchor.item_id),
        "the anchor must not be in the pool"
    );
}

#[test]
fn a_highest_first_list_still_puts_the_lower_candidates_on_the_left() {
    // Below the anchor in a highest-first list means *higher* values, so the sides
    // swap. The bar judges sides by value, never by list order (FR-025).
    let mut descending = items(21);
    descending.reverse();
    let setup = sample_order_round(&descending, &mut rng(3)).expect("a round");

    let anchor_value = setup.anchor.numeric;
    assert_eq!(anchor_value, 11.0);
    assert_eq!(setup.pool.len(), POOL_SIZE);
    assert!(
        setup
            .pool
            .iter()
            .all(|item| item.item_id != setup.anchor.item_id),
        "the anchor is never a candidate"
    );
}

#[test]
fn a_list_too_small_to_deal_ten_candidates_is_not_ready() {
    let error = sample_order_round(&items(MIN_ITEMS_FOR_ORDER - 1), &mut rng(1))
        .expect_err("ten candidates plus an anchor need eleven items");
    assert_eq!(error.code, ErrorCode::NotReady);
}

// T069 ---------------------------------------------------------------------

#[test]
fn the_smallest_possible_list_deals_every_item_but_the_anchor() {
    let sorted = items(11);
    let setup = sample_order_round(&sorted, &mut rng(5)).expect("a round");

    assert_eq!(setup.anchor.numeric, 6.0, "the middle of eleven items");
    assert_eq!(setup.pool.len(), POOL_SIZE);
    assert!(
        !setup
            .pool
            .iter()
            .any(|item| item.item_id == setup.anchor.item_id),
        "the anchor is not dealt"
    );
    let dealt: Vec<&str> = setup
        .pool
        .iter()
        .map(|item| item.item_id.as_str())
        .collect();
    let mut expected: Vec<&str> = sorted.iter().map(|i| i.item_id.as_str()).collect();
    expected.retain(|id| *id != setup.anchor.item_id);
    assert_eq!(dealt.len(), expected.len());
}

#[test]
fn an_even_list_deals_ten_of_the_twenty_around_its_anchor() {
    let sorted = items(20);
    let setup = sample_order_round(&sorted, &mut rng(9)).expect("a round");

    // Twenty items put the anchor at index 10, so ten sit below it and nine above:
    // the shorter side is topped up rather than dealt short.
    assert_eq!(setup.anchor.numeric, 11.0);
    assert_eq!(setup.pool.len(), POOL_SIZE);
    let below = setup
        .pool
        .iter()
        .filter(|item| item.numeric < setup.anchor.numeric)
        .count();
    assert_eq!(below, POOL_PER_SIDE);
    assert!(
        !setup
            .pool
            .iter()
            .any(|item| item.item_id == setup.anchor.item_id),
        "the anchor is not dealt"
    );
}

#[test]
fn a_seeded_generator_deals_the_same_round_twice() {
    let sorted = items(20);
    let first = sample_order_round(&sorted, &mut rng(42)).expect("a round");
    let second = sample_order_round(&sorted, &mut rng(42)).expect("a round");
    assert_eq!(first, second, "the same seed deals the same round");
}

// T062, T070 ---------------------------------------------------------------

#[test]
fn only_left_between_and_right_slots_are_offered() {
    // Three items on the left give four slots: 0 is left of the first, 1 and 2 are
    // between two items, and 3 is right of the last (FR-028).
    let board = board_with(50.0, &[10.0, 20.0, 30.0], &[], &[5.0]);
    let pending = with_pending(&board, "pool_5", BarSide::Left, 0, "team_1");

    for slot in 0..=3 {
        assert!(
            validate_placement(&pending, BarSide::Left, slot).is_ok(),
            "slot {slot} is a real position on the left"
        );
    }
}

#[test]
fn a_slot_outside_the_side_is_rejected() {
    let board = board_with(50.0, &[10.0, 20.0, 30.0], &[], &[5.0]);
    let pending = with_pending(&board, "pool_5", BarSide::Left, 0, "team_1");

    for slot in [-1, 4, 99] {
        let error = validate_placement(&pending, BarSide::Left, slot)
            .expect_err("that slot is not on the left side");
        assert_eq!(
            error.code,
            ErrorCode::BadRequest,
            "slot {slot} is out of range"
        );
    }
}

#[test]
fn a_full_side_offers_no_more_slots() {
    let board = board_with(50.0, &[10.0, 20.0, 30.0, 40.0, 45.0], &[], &[5.0]);
    let pending = with_pending(&board, "pool_5", BarSide::Left, 0, "team_1");
    let error = validate_placement(&pending, BarSide::Left, 5)
        .expect_err("the left side already holds its five items");
    assert_eq!(error.code, ErrorCode::BadRequest);
}

#[test]
fn a_slot_cannot_be_judged_before_it_is_chosen() {
    let board = board_with(50.0, &[], &[], &[5.0]);
    let pending = with_pending(&board, "pool_5", BarSide::Left, SLOT_UNCHOSEN, "team_1");
    let error =
        validate_placement(&pending, BarSide::Left, 0).expect_err("no slot has been chosen");
    assert_eq!(error.code, ErrorCode::NotReady);
}

#[test]
fn nothing_can_be_placed_before_something_is_selected() {
    let board = board_with(50.0, &[], &[], &[5.0]);
    let error = validate_placement(&board, BarSide::Left, 0).expect_err("nothing is selected");
    assert_eq!(error.code, ErrorCode::NotReady);
}

// T060, T061, T070 ---------------------------------------------------------

#[test]
fn a_correct_left_placement_awards_exactly_ten_points() {
    let board = board_with(50.0, &[20.0, 30.0], &[], &[10.0, 40.0]);
    let pending = with_pending(&board, "pool_10", BarSide::Left, 0, "team_1");

    assert!(validate_placement(&pending, BarSide::Left, 0).expect("judged"));

    let outcome = confirm_placement(&pending, &two_teams()).expect("confirmed");
    assert!(outcome.correct);
    let team = outcome
        .teams
        .iter()
        .find(|t| t.id == "team_1")
        .expect("team_1 is still standing");
    assert_eq!(team.score, 10, "exactly ten points (FR-029)");
    assert_eq!(
        team.hearts, MAX_HEARTS,
        "a correct placement costs no heart"
    );
}

#[test]
fn a_correct_right_placement_awards_exactly_ten_points() {
    let board = board_with(50.0, &[], &[60.0, 70.0], &[80.0, 90.0]);
    // 80 goes after 70: slot 0 would break the run and be a mistake.
    let pending = with_pending(&board, "pool_80", BarSide::Right, 2, "team_1");

    assert!(validate_placement(&pending, BarSide::Right, 2).expect("judged"));

    let outcome = confirm_placement(&pending, &two_teams()).expect("confirmed");
    assert!(outcome.correct);
    let team = outcome
        .teams
        .iter()
        .find(|t| t.id == "team_1")
        .expect("team_1 is still standing");
    assert_eq!(team.score, 10, "exactly ten points (FR-029)");
    assert_eq!(team.hearts, MAX_HEARTS);
}

#[test]
fn a_wrong_side_placement_awards_nothing_and_costs_exactly_one_heart() {
    let board = board_with(50.0, &[20.0, 30.0], &[60.0, 70.0], &[80.0]);
    // 80 belongs on the right, so putting it on the left is wrong.
    let pending = with_pending(&board, "pool_80", BarSide::Left, 0, "team_1");

    assert!(
        !validate_placement(&pending, BarSide::Left, 0).expect("judged"),
        "an item above the anchor does not belong on the left"
    );

    let outcome = confirm_placement(&pending, &two_teams()).expect("confirmed");
    assert!(!outcome.correct);
    let team = outcome
        .teams
        .iter()
        .find(|t| t.id == "team_1")
        .expect("team_1 is still standing");
    assert_eq!(team.score, 0, "a wrong placement awards no points (FR-030)");
    assert_eq!(team.hearts, MAX_HEARTS - 1, "exactly one heart (FR-030)");
}

#[test]
fn a_wrong_placement_leaves_the_item_in_the_pool() {
    let board = board_with(50.0, &[20.0], &[], &[80.0]);
    let pending = with_pending(&board, "pool_80", BarSide::Left, 0, "team_1");
    let outcome = confirm_placement(&pending, &two_teams()).expect("confirmed");

    assert!(
        outcome.board.pool.iter().any(|i| i.item_id == "pool_80"),
        "the item stays in the pool for another try (FR-031)"
    );
    assert!(
        outcome.board.left.iter().all(|i| i.item_id != "pool_80"),
        "and never joins the bar"
    );
    assert!(outcome.board.pending.is_none(), "the selection is spent");
}

#[test]
fn a_correct_placement_removes_the_item_from_the_pool() {
    let board = board_with(50.0, &[20.0], &[], &[10.0]);
    let pending = with_pending(&board, "pool_10", BarSide::Left, 0, "team_1");
    let outcome = confirm_placement(&pending, &two_teams()).expect("confirmed");

    assert!(outcome.board.pool.iter().all(|i| i.item_id != "pool_10"));
    assert_eq!(outcome.board.left.len(), 2);
}

#[test]
fn a_placement_that_breaks_the_run_is_wrong() {
    // Left holds 20 then 30; dropping 40 between them breaks the ascending run, and
    // 40 is above the anchor besides.
    let board = board_with(50.0, &[20.0, 30.0], &[], &[40.0]);
    let pending = with_pending(&board, "pool_40", BarSide::Left, 1, "team_1");
    assert!(!validate_placement(&pending, BarSide::Left, 1).expect("judged"));
}

#[test]
fn a_placement_that_keeps_the_run_is_correct() {
    let board = board_with(50.0, &[20.0, 30.0], &[], &[25.0]);
    let pending = with_pending(&board, "pool_25", BarSide::Left, 1, "team_1");
    assert!(validate_placement(&pending, BarSide::Left, 1).expect("judged"));

    let outcome = confirm_placement(&pending, &two_teams()).expect("confirmed");
    assert!(outcome.correct);
    let placed: Vec<f64> = outcome.board.left.iter().map(|i| i.numeric).collect();
    assert_eq!(placed, vec![20.0, 25.0, 30.0]);
}

// T063, FR-034 -------------------------------------------------------------

#[test]
fn an_item_equal_to_its_neighbour_is_correct_on_that_side() {
    // 30 ties with the 30 already on the bar: equal values are correctly placed
    // (FR-034).
    let board = board_with(50.0, &[20.0, 30.0], &[], &[30.0]);
    let pending = with_pending(&board, "pool_30", BarSide::Left, 2, "team_1");
    assert!(
        validate_placement(&pending, BarSide::Left, 2).expect("judged"),
        "a tie with the neighbour is not a mistake"
    );
}

#[test]
fn an_item_equal_to_the_anchor_fits_wherever_the_run_stays_intact() {
    let board = board_with(50.0, &[20.0], &[60.0], &[50.0]);
    // 50 ties the anchor, so it is as low as the anchor and as high as it: it may go
    // on either side, but only where it does not break the ascending run.
    let on_the_left = with_pending(&board, "pool_50", BarSide::Left, 1, "team_1");
    assert!(
        validate_placement(&on_the_left, BarSide::Left, 1).expect("judged"),
        "50 fits after 20 and before the anchor"
    );

    let on_the_right = with_pending(&board, "pool_50", BarSide::Right, 0, "team_1");
    assert!(
        validate_placement(&on_the_right, BarSide::Right, 0).expect("judged"),
        "50 fits before 60 on the right too"
    );

    let after_the_run = with_pending(&board, "pool_50", BarSide::Right, 1, "team_1");
    assert!(
        !validate_placement(&after_the_run, BarSide::Right, 1).expect("judged"),
        "but not after 60, which would put a lower value above a higher one"
    );
}

// T064, FR-029 -------------------------------------------------------------

#[test]
fn a_correct_placement_lands_exactly_where_it_was_dropped() {
    let board = board_with(50.0, &[10.0, 20.0, 30.0], &[60.0, 70.0], &[15.0]);
    let pending = with_pending(&board, "pool_15", BarSide::Left, 1, "team_1");
    let outcome = confirm_placement(&pending, &two_teams()).expect("confirmed");

    assert!(outcome.correct);
    let placed: Vec<f64> = outcome.board.left.iter().map(|i| i.numeric).collect();
    assert_eq!(
        placed,
        vec![10.0, 15.0, 20.0, 30.0],
        "the item joins the bar at the slot it was dropped on"
    );
    assert_eq!(
        outcome
            .board
            .right
            .iter()
            .map(|i| i.numeric)
            .collect::<Vec<_>>(),
        vec![60.0, 70.0],
        "the other side is untouched"
    );
}

#[test]
fn the_bar_is_never_reordered_behind_the_teams_back() {
    // A correct placement always leaves the side ascending, so re-sorting by value
    // would be a no-op here; what must never happen is the bar being rebuilt from
    // values instead of kept in the order it was built. Removing an item is the case
    // that would expose it, so that is what this checks (FR-029).
    let board = board_with(50.0, &[10.0, 20.0, 30.0], &[], &[25.0]);
    let pending = with_pending(&board, "pool_25", BarSide::Left, 2, "team_1");
    let placed = confirm_placement(&pending, &two_teams()).expect("confirmed");
    assert!(placed.correct);

    let trimmed = return_to_pool(&placed.board, "left_20").expect("returned");
    let remaining: Vec<f64> = trimmed.left.iter().map(|i| i.numeric).collect();
    assert_eq!(
        remaining,
        vec![10.0, 25.0, 30.0],
        "the survivors keep their order and their spacing"
    );
}

#[test]
fn return_to_pool_takes_an_item_off_the_bar() {
    let board = board_with(50.0, &[10.0, 20.0], &[], &[30.0]);
    let next = return_to_pool(&board, "left_20").expect("returned");

    assert!(next.left.iter().all(|i| i.item_id != "left_20"));
    assert!(
        next.pool.iter().any(|i| i.item_id == "left_20"),
        "the item is back in the pool (FR-033)"
    );
    assert_eq!(next.left.len(), 1);
}

#[test]
fn return_to_pool_refuses_an_item_that_is_not_on_the_bar() {
    let board = board_with(50.0, &[10.0], &[], &[30.0]);
    let error = return_to_pool(&board, "pool_30").expect_err("it is in the pool already");
    assert_eq!(error.code, ErrorCode::NotReady);
}

// US3 (feature 004) --------------------------------------------------------

fn value(item_id: &str, numeric: f64) -> ListItemValue {
    ListItemValue {
        item_id: item_id.to_string(),
        name: item_id.to_string(),
        display: numeric.to_string(),
        numeric,
        icon_name: None,
    }
}

#[test]
fn reveal_arrangement_puts_lower_values_left_and_higher_right() {
    let anchor = value("anchor", 5.0);
    let items = vec![
        value("a", 3.0),
        value("b", 7.0),
        value("c", 1.0),
        value("d", 9.0),
        value("e", 5.0),
    ];

    let (left, right) = arrange_revealed_order(&anchor, &items);

    assert_eq!(
        left.iter().map(|i| i.item_id.as_str()).collect::<Vec<_>>(),
        ["c", "a", "e"],
        "values <= the anchor, ascending"
    );
    assert_eq!(
        right.iter().map(|i| i.item_id.as_str()).collect::<Vec<_>>(),
        ["b", "d"],
        "values > the anchor, ascending"
    );
}

#[test]
fn reveal_arrangement_keeps_input_order_for_ties() {
    let anchor = value("anchor", 5.0);
    let items = vec![value("first", 4.0), value("second", 4.0)];

    let (left, _) = arrange_revealed_order(&anchor, &items);

    assert_eq!(
        left.iter().map(|i| i.item_id.as_str()).collect::<Vec<_>>(),
        ["first", "second"],
        "a stable sort keeps ties in the order they were handed in"
    );
}

#[test]
fn reveal_arrangement_caps_each_side_at_the_bar_limit() {
    let anchor = value("anchor", 100.0);
    let items: Vec<ListItemValue> = (0..7).map(|n| value(&format!("i{n}"), n as f64)).collect();

    let (left, right) = arrange_revealed_order(&anchor, &items);

    assert_eq!(
        left.len(),
        BAR_SLOTS_PER_SIDE,
        "no side may overflow the bar"
    );
    assert!(right.is_empty());
}

// T065, FR-032 -------------------------------------------------------------

#[test]
fn the_round_ends_once_every_item_is_placed() {
    let board = board_with(50.0, &[10.0, 20.0], &[60.0], &[]);
    assert!(round_is_over(&board, &two_teams()));

    let still_pooling = board_with(50.0, &[10.0, 20.0], &[60.0], &[30.0]);
    assert!(
        !round_is_over(&still_pooling, &two_teams()),
        "an unplaced item keeps the round open"
    );
}

#[test]
fn the_round_ends_when_no_team_has_hearts() {
    let board = board_with(50.0, &[], &[], &[10.0, 20.0]);
    let exhausted = vec![
        TeamStanding {
            id: "team_1".to_string(),
            score: 0,
            hearts: 0,
            turn_index: 0,
        },
        TeamStanding {
            id: "team_2".to_string(),
            score: 0,
            hearts: 0,
            turn_index: 1,
        },
    ];
    assert!(round_is_over(&board, &exhausted));
    assert!(!round_is_over(&board, &two_teams()));
}

#[test]
fn a_selection_still_open_keeps_the_round_going() {
    let board = board_with(50.0, &[], &[], &[]);
    let pending = with_pending(&board, "anchor_50", BarSide::Left, 0, "team_1");
    assert!(!round_is_over(&pending, &two_teams()));
}

// FR-018 -------------------------------------------------------------------

#[test]
fn confirming_hands_the_turn_to_the_next_team_with_hearts() {
    let board = board_with(50.0, &[], &[], &[10.0]);
    let pending = with_pending(&board, "pool_10", BarSide::Left, 0, "team_1");
    let outcome = confirm_placement(&pending, &two_teams()).expect("confirmed");
    assert_eq!(outcome.next_team_id.as_deref(), Some("team_2"));
}

#[test]
fn the_turn_comes_back_round_when_only_one_team_still_has_hearts() {
    let board = board_with(50.0, &[], &[], &[10.0, 20.0]);
    let pending = with_pending(&board, "pool_10", BarSide::Left, 0, "team_1");
    let teams = vec![
        TeamStanding {
            id: "team_1".to_string(),
            score: 0,
            hearts: MAX_HEARTS,
            turn_index: 0,
        },
        TeamStanding {
            id: "team_2".to_string(),
            score: 0,
            hearts: 0,
            turn_index: 1,
        },
    ];
    let outcome = confirm_placement(&pending, &teams).expect("confirmed");
    assert_eq!(
        outcome.next_team_id.as_deref(),
        Some("team_1"),
        "team_2 is skipped for the rest of the round, so the turn wraps to team_1 (FR-019)"
    );
    assert!(
        !outcome.round_over,
        "and the round carries on while a team still has hearts"
    );
}

#[test]
fn a_turn_out_of_hearts_leaves_nobody_to_hand_it_to() {
    let board = board_with(50.0, &[], &[], &[10.0]);
    let pending = with_pending(&board, "pool_10", BarSide::Left, 0, "team_1");
    // The placing team drops its last heart, so nobody is left holding the turn.
    let teams = vec![
        TeamStanding {
            id: "team_1".to_string(),
            score: 0,
            hearts: 1,
            turn_index: 0,
        },
        TeamStanding {
            id: "team_2".to_string(),
            score: 0,
            hearts: 0,
            turn_index: 1,
        },
    ];
    let wrong = with_pending(
        &board_with(50.0, &[], &[], &[80.0]),
        "pool_80",
        BarSide::Left,
        0,
        "team_1",
    );
    let outcome = confirm_placement(&wrong, &teams).expect("confirmed");
    assert!(
        !outcome.correct,
        "80 does not belong on the left of a 50 anchor"
    );
    assert_eq!(outcome.next_team_id, None, "no team has hearts left");
    assert!(outcome.round_over, "so the round is over (FR-032)");
    let _ = pending;
}

// FR-028 -------------------------------------------------------------------

#[test]
fn selecting_an_item_records_its_team_and_side() {
    let board = board_with(50.0, &[], &[], &[10.0, 80.0]);
    let next = board.select_item("pool_80", "team_1").expect("selected");
    let pending = next.pending.expect("a pending placement");

    assert_eq!(pending.item_id, "pool_80");
    assert_eq!(pending.team_id, "team_1");
    assert_eq!(pending.side, BarSide::Right, "80 is above the anchor");
    assert_eq!(pending.slot_index, SLOT_UNCHOSEN, "no slot yet");
}

#[test]
fn selecting_replaces_an_earlier_selection() {
    let board = board_with(50.0, &[], &[], &[10.0, 20.0]);
    let first = board.select_item("pool_10", "team_1").expect("selected");
    let second = first.select_item("pool_20", "team_1").expect("selected");
    assert_eq!(second.pending.expect("pending").item_id, "pool_20");
}

#[test]
fn an_item_that_is_not_in_the_pool_cannot_be_selected() {
    let board = board_with(50.0, &[10.0], &[], &[20.0]);
    let error = board
        .select_item("left_10", "team_1")
        .expect_err("it is already on the bar");
    assert_eq!(error.code, ErrorCode::NotReady);
}

#[test]
fn cancelling_puts_the_selection_away_without_scoring() {
    let board = board_with(50.0, &[], &[], &[10.0]);
    let pending = board.select_item("pool_10", "team_1").expect("selected");
    let next = pending.cancel_selection().expect("cancelled");

    assert!(next.pending.is_none());
    assert_eq!(next.pool.len(), 1, "the item never left the pool");
    assert!(next.left.is_empty(), "and never reached the bar");
}

#[test]
fn choosing_a_slot_settles_where_the_item_goes() {
    let board = board_with(50.0, &[20.0, 30.0], &[], &[10.0]);
    let pending = board
        .select_item("pool_10", "team_1")
        .expect("selected")
        .choose_slot(BarSide::Left, 0)
        .expect("slot chosen");
    let settled = pending.pending.expect("pending");

    assert_eq!(settled.side, BarSide::Left);
    assert_eq!(settled.slot_index, 0);
}

#[test]
fn a_slot_cannot_be_chosen_before_an_item_is_selected() {
    let board = board_with(50.0, &[], &[], &[10.0]);
    let error = board
        .choose_slot(BarSide::Left, 0)
        .expect_err("there is nothing to place");
    assert_eq!(error.code, ErrorCode::NotReady);
}

#[test]
fn an_out_of_range_slot_is_refused_when_it_is_chosen_too() {
    let board = board_with(50.0, &[20.0, 30.0], &[], &[10.0]);
    let error = board
        .select_item("pool_10", "team_1")
        .expect("selected")
        .choose_slot(BarSide::Left, 9)
        .expect_err("that slot does not exist");
    assert_eq!(error.code, ErrorCode::BadRequest);
}

#[test]
fn sides_name_themselves_for_storage_and_for_the_wire() {
    assert_eq!(BarSide::parse("LEFT"), Some(BarSide::Left));
    assert_eq!(BarSide::parse(" right "), Some(BarSide::Right));
    assert_eq!(BarSide::parse("middle"), None);
    assert_eq!(BarSide::Left.as_str(), "left");
    assert_eq!(BarSide::Left.other(), BarSide::Right);
}

#[test]
fn a_pending_placement_survives_the_json_the_state_reader_expects() {
    let pending = PendingPlacement {
        item_id: "item_1".to_string(),
        name: "Item 1".to_string(),
        side: BarSide::Left,
        slot_index: 2,
        team_id: "team_1".to_string(),
    };
    let json = serde_json::to_string(&pending).expect("serialised");
    assert_eq!(
        json,
        r#"{"itemId":"item_1","name":"Item 1","side":"left","slotIndex":2,"teamId":"team_1"}"#
    );
    let back: PendingPlacement = serde_json::from_str(&json).expect("parsed");
    assert_eq!(back, pending);
}
