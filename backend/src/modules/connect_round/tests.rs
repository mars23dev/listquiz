//! Connect round rules, tested without a database (feature 010, US1).

use rand::SeedableRng;
use rand::rngs::SmallRng;

use super::*;
use crate::errors::ErrorCode;
use crate::modules::scoring::{MAX_HEARTS, TeamStanding};

fn rng(seed: u64) -> SmallRng {
    SmallRng::seed_from_u64(seed)
}

fn pairs(count: usize) -> Vec<ConnectPair> {
    (1..=count)
        .map(|n| ConnectPair {
            item_id: format!("item_{n}"),
            name: format!("Name {n}"),
            icon_name: Some("BookOpen".to_string()),
            display: n.to_string(),
        })
        .collect()
}

fn teams() -> Vec<TeamStanding> {
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

fn board() -> ConnectBoard {
    sample_connect_round(&pairs(12), &mut rng(1)).expect("a board of ten pairs")
}

/// The value item id that matches `name_item_id`.
fn partner(board: &ConnectBoard, name_item_id: &str) -> String {
    board
        .values
        .iter()
        .find(|value| value.item_id == name_item_id)
        .expect("a value for that name")
        .item_id
        .clone()
}

#[test]
fn a_board_has_ten_names_and_ten_values() {
    let board = board();
    assert_eq!(board.names.len(), PAIRS);
    assert_eq!(board.values.len(), PAIRS);
    assert!(board.matched.is_empty());
    assert_eq!(board.pending, ConnectPending::default());

    let mut names: Vec<&str> = board.names.iter().map(|n| n.item_id.as_str()).collect();
    let mut values: Vec<&str> = board.values.iter().map(|v| v.item_id.as_str()).collect();
    names.sort_unstable();
    values.sort_unstable();
    assert_eq!(names, values, "the two columns hold the same items");
}

#[test]
fn the_value_column_follows_the_list_order_and_names_are_shuffled() {
    let board = board();

    // The pairs were built in ascending list order, so list order is 1, 2, 3, ...
    let value_ids: Vec<u32> = board
        .values
        .iter()
        .map(|v| v.item_id.trim_start_matches("item_").parse().unwrap())
        .collect();
    let mut list_order = value_ids.clone();
    list_order.sort_unstable();
    assert_eq!(
        value_ids, list_order,
        "the value column keeps the list's order"
    );

    // The names are shuffled, so they do not follow the list order.
    let name_ids: Vec<u32> = board
        .names
        .iter()
        .map(|n| n.item_id.trim_start_matches("item_").parse().unwrap())
        .collect();
    assert_ne!(
        name_ids, list_order,
        "the name column is shuffled, not in list order"
    );
}

#[test]
fn a_list_under_ten_items_is_refused() {
    let error = sample_connect_round(&pairs(9), &mut rng(1)).expect_err("too few");
    assert_eq!(error.code, ErrorCode::NotReady);
}

#[test]
fn picking_stages_a_name_and_a_value() {
    let board = board();
    let name = board.names[0].item_id.clone();
    let value = board.values[0].item_id.clone();

    let with_name = board.pick_name(&name).expect("a name");
    assert_eq!(
        with_name.pending.name_item_id.as_deref(),
        Some(name.as_str())
    );

    let with_value = board.pick_value(&value).expect("a value");
    assert_eq!(
        with_value.pending.value_item_id.as_deref(),
        Some(value.as_str())
    );
}

#[test]
fn a_matching_pair_scores_ten_and_is_matched() {
    let board = board();
    let name = board.names[0].item_id.clone();
    let staged = board
        .pick_name(&name)
        .and_then(|b| b.pick_value(&partner(&board, &name)))
        .expect("staged");

    let outcome = confirm(&staged, &teams(), "team_1").expect("confirmed");

    assert!(outcome.correct);
    assert!(outcome.board.matched.contains(&name), "the pair is matched");
    assert_eq!(
        outcome.board.pending,
        ConnectPending::default(),
        "the box clears"
    );
    assert_eq!(
        outcome
            .teams
            .iter()
            .find(|t| t.id == "team_1")
            .unwrap()
            .score,
        POINTS_CORRECT
    );
    assert_eq!(
        outcome.next_team_id.as_deref(),
        Some("team_2"),
        "the turn passes"
    );
}

#[test]
fn a_mismatch_costs_one_heart_and_leaves_the_pair() {
    let board = board();
    let name = board.names[0].item_id.clone();
    let wrong_value = board
        .values
        .iter()
        .map(|v| v.item_id.clone())
        .find(|id| *id != name)
        .unwrap();
    let staged = board
        .pick_name(&name)
        .and_then(|b| b.pick_value(&wrong_value))
        .expect("staged");

    let outcome = confirm(&staged, &teams(), "team_1").expect("confirmed");

    assert!(!outcome.correct);
    assert!(outcome.board.matched.is_empty(), "the pair stays unmatched");
    assert_eq!(outcome.board.pending, ConnectPending::default());
    let team = outcome.teams.iter().find(|t| t.id == "team_1").unwrap();
    assert_eq!(team.hearts, MAX_HEARTS - 1, "one heart lost");
    assert_eq!(team.score, 0, "no points");
    assert_eq!(outcome.next_team_id.as_deref(), Some("team_2"));
}

#[test]
fn confirming_without_both_sides_is_refused() {
    let board = board();
    let name = board.names[0].item_id.clone();
    let only_name = board.pick_name(&name).expect("a name");
    assert!(confirm(&only_name, &teams(), "team_1").is_err());
}

#[test]
fn a_matched_id_cannot_be_picked_again() {
    let board = board();
    let name = board.names[0].item_id.clone();
    let mut matched = board.clone();
    matched.matched.push(name.clone());

    assert!(matched.pick_name(&name).is_err());
    assert!(matched.pick_value(&name).is_err());
}

#[test]
fn the_round_ends_when_all_pairs_match_or_no_team_has_hearts() {
    let mut full = board();
    full.matched = full.names.iter().map(|n| n.item_id.clone()).collect();
    assert!(round_is_over(&full, &teams()));

    let mut spent = teams();
    spent.iter_mut().for_each(|team| team.hearts = 0);
    assert!(round_is_over(&board(), &spent));

    assert!(!round_is_over(&board(), &teams()));
}
