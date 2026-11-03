//! Tripmine round rules, tested without a database (T079-T086).

use rand::SeedableRng;
use rand::rngs::SmallRng;

use super::*;
use crate::errors::ErrorCode;
use crate::modules::scoring::{self, MAX_HEARTS, TeamStanding};

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

/// A sampled round over `items(20)`, ready to pick from.
fn round(side: TripmineSide) -> TripmineRound {
    let grid = sample_tripmine_grid(&items(20), side, TRIPMINE_SLOTS, &mut rng(11))
        .expect("a grid of ten");
    TripmineRound {
        side,
        searched_rank: starting_search_rank(side, TRIPMINE_SLOTS),
        items: grid,
        hit: false,
        hit_by: None,
        completed: false,
    }
}

fn slot_at(round: &TripmineRound, rank: i32) -> TripmineSlot {
    round
        .items
        .iter()
        .find(|slot| slot.true_rank == rank)
        .expect("a slot at that rank")
        .clone()
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

// T079 ---------------------------------------------------------------------

#[test]
fn a_grid_holds_ten_ranked_items_with_one_mine_at_the_mined_end() {
    for (side, mined_rank) in [
        (TripmineSide::Highest, TRIPMINE_SLOTS as i32),
        (TripmineSide::Lowest, 1),
    ] {
        let grid =
            sample_tripmine_grid(&items(20), side, TRIPMINE_SLOTS, &mut rng(3)).expect("a grid");

        assert_eq!(grid.len(), TRIPMINE_SLOTS);
        let mut ranks: Vec<i32> = grid.iter().map(|slot| slot.true_rank).collect();
        ranks.sort_unstable();
        assert_eq!(ranks, (1..=TRIPMINE_SLOTS as i32).collect::<Vec<_>>());

        let mines: Vec<&TripmineSlot> = grid.iter().filter(|slot| slot.is_tripmine).collect();
        assert_eq!(mines.len(), 1, "exactly one item is the mine");
        assert_eq!(mines[0].true_rank, mined_rank);
    }
}

#[test]
fn a_twenty_slot_grid_ranks_twenty_items_with_the_mine_at_the_chosen_end() {
    for (side, mined_rank) in [(TripmineSide::Highest, 20), (TripmineSide::Lowest, 1)] {
        let grid = sample_tripmine_grid(&items(20), side, 20, &mut rng(5)).expect("a grid");

        assert_eq!(grid.len(), 20);
        let mut ranks: Vec<i32> = grid.iter().map(|slot| slot.true_rank).collect();
        ranks.sort_unstable();
        assert_eq!(ranks, (1..=20).collect::<Vec<_>>());

        let mines: Vec<&TripmineSlot> = grid.iter().filter(|slot| slot.is_tripmine).collect();
        assert_eq!(mines.len(), 1, "exactly one item is the mine");
        assert_eq!(mines[0].true_rank, mined_rank);
    }

    let random =
        sample_tripmine_grid(&items(20), TripmineSide::Random, 20, &mut rng(9)).expect("a grid");
    let mine_rank = random
        .iter()
        .find(|slot| slot.is_tripmine)
        .expect("a mine")
        .true_rank;
    assert!((1..=20).contains(&mine_rank));
}

// T080 ---------------------------------------------------------------------
#[test]
fn the_search_starts_at_the_far_end_from_the_mine() {
    assert_eq!(
        starting_search_rank(TripmineSide::Highest, TRIPMINE_SLOTS),
        1
    );
    assert_eq!(
        starting_search_rank(TripmineSide::Lowest, TRIPMINE_SLOTS),
        10
    );

    assert_eq!(round(TripmineSide::Highest).searched_rank, 1);
    assert_eq!(round(TripmineSide::Lowest).searched_rank, 10);
}

// T081 ---------------------------------------------------------------------

#[test]
fn picking_the_searched_rank_awards_ten_and_advances_the_search() {
    let round = round(TripmineSide::Highest);
    let target = slot_at(&round, round.searched_rank);

    let outcome = pick_item(&round, &target.item.item_id, "team_1", &two_teams()).expect("a pick");

    assert_eq!(outcome.teams[0].score, POINTS_SEARCHED);
    let picked = outcome.round.find(&target.item.item_id).expect("the pick");
    assert!(picked.revealed);
    assert_eq!(picked.points_awarded, Some(POINTS_SEARCHED));
    assert_eq!(
        outcome.round.searched_rank, 2,
        "the search advances upward to the next unrevealed rank"
    );
    assert!(!outcome.round.completed);
    assert_eq!(outcome.next_team_id.as_deref(), Some("team_2"));
}

// T082 ---------------------------------------------------------------------

#[test]
fn a_wrong_pick_awards_five_and_leaves_the_search_alone() {
    let round = round(TripmineSide::Highest);
    let searched = round.searched_rank;
    // Rank 5 is neither the searched rank (1) nor the mine (10).
    let wrong = slot_at(&round, 5);

    let outcome = pick_item(&round, &wrong.item.item_id, "team_1", &two_teams()).expect("a pick");

    assert_eq!(outcome.teams[0].score, POINTS_SAFE);
    assert_eq!(outcome.round.searched_rank, searched);
    let revealed: Vec<i32> = outcome
        .round
        .items
        .iter()
        .filter(|slot| slot.revealed)
        .map(|slot| slot.true_rank)
        .collect();
    assert_eq!(revealed, vec![5], "only the picked position is revealed");
    assert_eq!(
        outcome
            .round
            .find(&wrong.item.item_id)
            .expect("the pick")
            .points_awarded,
        Some(POINTS_SAFE)
    );
}

// T083 ---------------------------------------------------------------------

#[test]
fn setting_off_the_mine_costs_the_picker_and_pays_everyone_else() {
    let round = round(TripmineSide::Highest);
    let mine = round
        .items
        .iter()
        .find(|slot| slot.is_tripmine)
        .expect("a mine")
        .clone();

    let outcome = pick_item(&round, &mine.item.item_id, "team_1", &two_teams()).expect("a pick");

    assert_eq!(outcome.teams[0].score, POINTS_TRIPMINE_PENALTY);
    assert_eq!(outcome.teams[1].score, POINTS_TRIPMINE_OTHERS);
    assert!(outcome.round.hit);
    assert_eq!(outcome.round.hit_by.as_deref(), Some("team_1"));
    assert!(outcome.round.completed);
    assert!(outcome.round.items.iter().all(|slot| slot.revealed));
    assert!(outcome.next_team_id.is_none(), "the round is over");

    let reasons: Vec<&str> = outcome
        .events
        .iter()
        .map(|event| event.reason.as_str())
        .collect();
    assert!(
        reasons
            .iter()
            .all(|reason| *reason == scoring::REASON_TRIPMINE_HIT)
    );
    assert!(
        outcome
            .events
            .iter()
            .any(|event| event.amount == POINTS_TRIPMINE_PENALTY)
    );
    assert!(
        outcome
            .events
            .iter()
            .any(|event| event.amount == POINTS_TRIPMINE_OTHERS)
    );
}

// T084 ---------------------------------------------------------------------

#[test]
fn the_valid_item_with_two_left_is_worth_twenty_and_ends_the_round() {
    let mut round = round(TripmineSide::Highest);
    // Leave only the searched rank (1) and the mine (10) unrevealed.
    for slot in round.items.iter_mut() {
        if slot.true_rank != round.searched_rank && !slot.is_tripmine {
            slot.revealed = true;
        }
    }
    assert_eq!(round.unrevealed(), 2);

    let valid = slot_at(&round, round.searched_rank);
    let outcome = pick_item(&round, &valid.item.item_id, "team_1", &two_teams()).expect("a pick");

    assert_eq!(outcome.teams[0].score, POINTS_FINAL);
    assert!(outcome.round.completed);
    assert!(outcome.round.items.iter().all(|slot| slot.revealed));
    assert!(outcome.next_team_id.is_none());
    assert_eq!(
        outcome.events.first().map(|event| event.reason.as_str()),
        Some(scoring::REASON_FINAL_ITEM)
    );
}

// T085 ---------------------------------------------------------------------

#[test]
fn only_the_active_team_may_pick() {
    assert!(scoring::is_active_turn(Some("team_1"), "team_1"));
    // The gamemaster acts on the active team's behalf by naming it, so the check is
    // the same one players would pass.
    assert!(scoring::is_active_turn(Some("team_1"), "team_1"));
    assert!(!scoring::is_active_turn(Some("team_1"), "team_2"));
    assert!(!scoring::is_active_turn(None, "team_1"));
}

// T086 ---------------------------------------------------------------------

#[test]
fn the_side_is_fixed_once_the_round_has_started() {
    let round = round(TripmineSide::Highest);

    assert!(ensure_side_unchanged(&round, None).is_ok());
    assert!(ensure_side_unchanged(&round, Some("highest")).is_ok());

    let changed = ensure_side_unchanged(&round, Some("lowest")).expect_err("a real change");
    assert_eq!(changed.code, ErrorCode::Conflict);

    let nonsense = ensure_side_unchanged(&round, Some("sideways")).expect_err("not a side");
    assert_eq!(nonsense.code, ErrorCode::BadRequest);
}

// T010 ---------------------------------------------------------------------

#[test]
fn the_grid_is_shuffled_out_of_rank_order_without_changing_the_answer_key() {
    let base = sample_tripmine_grid(
        &items(10),
        TripmineSide::Highest,
        TRIPMINE_SLOTS,
        &mut rng(1),
    )
    .expect("a grid");
    let ascending: Vec<i32> = base.iter().map(|slot| slot.true_rank).collect();
    assert_eq!(
        ascending,
        (1..=TRIPMINE_SLOTS as i32).collect::<Vec<_>>(),
        "sampling ranks the grid ascending before display"
    );

    let order = |grid: &[TripmineSlot]| grid.iter().map(|slot| slot.true_rank).collect::<Vec<_>>();

    let mut first = base.clone();
    shuffle_grid(&mut first, &mut rng(2));
    let mut second = base.clone();
    shuffle_grid(&mut second, &mut rng(3));

    assert_ne!(
        order(&first),
        order(&second),
        "different seeds reorder differently"
    );
    assert_ne!(
        order(&first),
        ascending,
        "the display order is not the rank order"
    );

    // Reordering touches nothing but the array sequence.
    let mut ranks = order(&first);
    ranks.sort_unstable();
    assert_eq!(ranks, (1..=TRIPMINE_SLOTS as i32).collect::<Vec<_>>());
    assert_eq!(first.iter().filter(|slot| slot.is_tripmine).count(), 1);
    let mine = first
        .iter()
        .find(|slot| slot.is_tripmine)
        .expect("the mine");
    assert_eq!(mine.true_rank, TRIPMINE_SLOTS as i32);
}

// Random (Hard) side (feature 009) -----------------------------------------

#[test]
fn random_side_parses_and_round_trips() {
    assert_eq!(TripmineSide::parse("random"), Some(TripmineSide::Random));
    assert_eq!(TripmineSide::Random.as_str(), "random");
}

#[test]
fn a_random_grid_has_one_mine_at_any_rank() {
    let mut mined_ranks = std::collections::BTreeSet::new();
    for seed in 1..=30 {
        let grid = sample_tripmine_grid(
            &items(20),
            TripmineSide::Random,
            TRIPMINE_SLOTS,
            &mut rng(seed),
        )
        .expect("a grid");
        let mut ranks: Vec<i32> = grid.iter().map(|slot| slot.true_rank).collect();
        ranks.sort_unstable();
        assert_eq!(
            ranks,
            (1..=TRIPMINE_SLOTS as i32).collect::<Vec<_>>(),
            "ranked ascending"
        );

        let mines: Vec<&TripmineSlot> = grid.iter().filter(|slot| slot.is_tripmine).collect();
        assert_eq!(mines.len(), 1, "exactly one mine");
        assert!((1..=TRIPMINE_SLOTS as i32).contains(&mines[0].true_rank));
        mined_ranks.insert(mines[0].true_rank);
    }
    assert!(
        mined_ranks.len() > 1,
        "the mine is not always the same rank"
    );
}

#[test]
fn a_random_search_target_is_unrevealed_and_never_the_mine() {
    let grid = sample_tripmine_grid(
        &items(20),
        TripmineSide::Random,
        TRIPMINE_SLOTS,
        &mut rng(7),
    )
    .expect("grid");
    let mine = grid.iter().find(|slot| slot.is_tripmine).unwrap().true_rank;

    for _ in 0..20 {
        let rank = next_searched_rank(TripmineSide::Random, &grid, 0).expect("a target");
        assert_ne!(rank, mine, "the target is never the mine");
        let slot = grid.iter().find(|slot| slot.true_rank == rank).unwrap();
        assert!(!slot.revealed, "the target is unrevealed");
    }
}

#[test]
fn a_random_search_with_only_the_mine_left_has_no_target() {
    let grid = vec![
        TripmineSlot {
            item: items(1)[0].clone(),
            true_rank: 1,
            is_tripmine: true,
            revealed: false,
            revealed_by: None,
            points_awarded: None,
        },
        TripmineSlot {
            item: items(2)[1].clone(),
            true_rank: 2,
            is_tripmine: false,
            revealed: false,
            revealed_by: None,
            points_awarded: None,
        },
    ];

    assert_eq!(
        next_searched_rank(TripmineSide::Random, &grid, 0),
        Some(2),
        "the only remaining item is the target"
    );

    let only_mine = vec![grid[0].clone()];
    assert_eq!(
        next_searched_rank(TripmineSide::Random, &only_mine, 0),
        None
    );
}
