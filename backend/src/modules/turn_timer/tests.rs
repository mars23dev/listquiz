//! Turn timer rules, tested without a database (feature 005, US2).

use super::*;
use crate::modules::scoring::{MAX_HEARTS, TeamStanding};

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

#[test]
fn start_rejects_a_zero_duration() {
    assert!(start(1_000, 0).is_err());
}

#[test]
fn start_ends_after_the_chosen_seconds() {
    let timer = start(1_000, 20).expect("a positive duration");
    assert_eq!(timer.total_seconds, 20);
    assert_eq!(timer.ends_at, 1_000 + 20_000);
}

#[test]
fn remaining_rounds_up_and_never_goes_negative() {
    assert_eq!(remaining(10_000, 8_500), 2, "1.5s left rounds up to 2");
    assert_eq!(remaining(10_000, 9_001), 1);
    assert_eq!(remaining(10_000, 10_000), 0);
    assert_eq!(remaining(10_000, 12_000), 0, "past the end stays at zero");
}

#[test]
fn is_expired_only_at_or_after_the_end() {
    assert!(!is_expired(10_000, 9_999));
    assert!(is_expired(10_000, 10_000));
    assert!(is_expired(10_000, 10_001));
}

#[test]
fn a_lose_heart_expiry_costs_exactly_one_heart_and_passes_the_turn() {
    let (after, _events, next) = expire(&teams(), "team_1", TimeoutRule::LoseHeart);
    let team_1 = after.iter().find(|t| t.id == "team_1").unwrap();
    assert_eq!(team_1.hearts, MAX_HEARTS - 1, "one heart lost");
    assert_eq!(next.as_deref(), Some("team_2"), "the turn passes on");
}

#[test]
fn a_pass_only_expiry_changes_no_heart_and_passes_the_turn() {
    let (after, events, next) = expire(&teams(), "team_1", TimeoutRule::PassOnly);
    assert_eq!(after, teams(), "no heart or score change");
    assert!(events.is_empty(), "a pass-only timeout emits no events");
    assert_eq!(next.as_deref(), Some("team_2"), "the turn passes on");
}

#[test]
fn a_lose_heart_expiry_never_takes_a_team_below_zero_hearts() {
    let mut standings = teams();
    standings[0].hearts = 0;
    let (after, _events, _) = expire(&standings, "team_1", TimeoutRule::LoseHeart);
    let team_1 = after.iter().find(|t| t.id == "team_1").unwrap();
    assert_eq!(team_1.hearts, 0);
}

#[test]
fn an_expiry_with_no_hearts_left_ends_the_round() {
    let mut standings = teams();
    standings[0].hearts = 0;
    standings[1].hearts = 0;
    let (_, _events, next) = expire(&standings, "team_1", TimeoutRule::LoseHeart);
    assert_eq!(next, None, "nobody is left to play");
}
