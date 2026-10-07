//! Quick-game request builder, tested without a database (feature 005, US5).

use super::*;
use crate::errors::ErrorCode;
use crate::modules::default_lists::{DEFAULT_LISTS, DefaultList};

#[test]
fn an_order_quick_game_is_one_team_and_one_round() {
    let request = build_request("ORDER", &DEFAULT_LISTS).expect("a fitting list");

    assert_eq!(request.effective_team_count(), 1);
    assert_eq!(request.effective_round_count(), 1);
    assert_eq!(request.rounds[0].quiz_type, "ORDER");
    assert_eq!(request.rounds[0].list_id, DEFAULT_LISTS[0].id);
    assert_eq!(request.rounds[0].tripmine_side, None);
    assert!(!request.rounds[0].column_id.is_empty());
}

#[test]
fn a_tripmine_quick_game_picks_a_side() {
    let request = build_request("TRIPMINE", &DEFAULT_LISTS).expect("a fitting list");

    assert_eq!(request.rounds[0].quiz_type, "TRIPMINE");
    assert_eq!(request.rounds[0].tripmine_side.as_deref(), Some("highest"));
}

#[test]
fn an_unknown_game_type_is_rejected() {
    let error = build_request("SNAKES", &DEFAULT_LISTS).expect_err("not a game type");
    assert_eq!(error.code, ErrorCode::BadRequest);
}

#[test]
fn no_fitting_list_is_an_error() {
    let too_small = [DefaultList {
        id: "tiny",
        title: "Tiny",
        description: "",
        order_direction: "low_to_high",
        columns: &[],
        items: &[],
    }];

    let error = build_request("ORDER", &too_small).expect_err("nothing fits");
    assert_eq!(error.code, ErrorCode::NotReady);
}
