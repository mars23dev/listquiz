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
            numeric: n as f64,
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
fn the_value_column_is_ordered_by_value_and_names_are_shuffled() {
    let board = board();

    // The values read low-to-high (the pairs are numbered 1..=12).
    let values: Vec<u32> = board
        .values
        .iter()
        .map(|v| v.display.parse().unwrap())
        .collect();
    let mut ordered = values.clone();
    ordered.sort_unstable();
    assert_eq!(values, ordered, "the value column is ordered by value");

    // The names are shuffled, so they are not in value order.
    let names: Vec<u32> = board
        .names
        .iter()
        .map(|n| n.name.trim_start_matches("Name ").parse().unwrap())
        .collect();
    assert_ne!(names, ordered, "the name column is shuffled");
}

#[test]
fn the_value_column_is_ordered_by_value_not_list_position() {
    let pairs: Vec<ConnectPair> = (1..=12)
        .rev()
        .map(|n| ConnectPair {
            item_id: format!("item_{n}"),
            name: format!("Name {n}"),
            icon_name: None,
            display: n.to_string(),
            numeric: n as f64,
        })
        .collect();
    let board = sample_connect_round(&pairs, &mut rng(3)).expect("a board");

    let values: Vec<u32> = board
        .values
        .iter()
        .map(|v| v.display.parse().unwrap())
        .collect();
    let mut ordered = values.clone();
    ordered.sort_unstable();
    assert_eq!(
        values, ordered,
        "the value column is ascending regardless of list position"
    );
}

#[test]
fn every_shipped_list_column_reads_ascending_in_connect() {
    use crate::modules::default_lists::DEFAULT_LISTS;

    for list in DEFAULT_LISTS.iter() {
        for column in list.columns.iter() {
            let pairs: Vec<ConnectPair> = list
                .items
                .iter()
                .map(|item| {
                    let (display, numeric) = item
                        .values
                        .iter()
                        .find(|(c, _, _)| *c == column.id)
                        .map(|(_, display, numeric)| (display.to_string(), *numeric))
                        .expect("every shipped item has this column");
                    ConnectPair {
                        item_id: item.id.to_string(),
                        name: item.name.to_string(),
                        icon_name: Some(item.icon_name.to_string()),
                        display,
                        numeric,
                    }
                })
                .collect();

            let board = sample_connect_round(&pairs, &mut rng(7)).expect("a board");
            let values: Vec<f64> = board
                .values
                .iter()
                .map(|value| {
                    pairs
                        .iter()
                        .find(|pair| pair.item_id == value.item_id)
                        .expect("a pair for the value")
                        .numeric
                })
                .collect();

            for window in values.windows(2) {
                assert!(
                    window[0] <= window[1],
                    "{} / {} is not ascending in Connect: {} then {}",
                    list.id,
                    column.id,
                    window[0],
                    window[1]
                );
            }
        }
    }
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
