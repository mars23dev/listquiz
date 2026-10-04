use super::*;

fn team(id: &str, score: i32, hearts: i32, turn_index: i32) -> TeamStanding {
    TeamStanding {
        id: id.to_string(),
        score,
        hearts,
        turn_index,
    }
}

#[test]
fn correct_placement_awards_ten_points() {
    let teams = vec![team("team_1", 0, 3, 0)];

    let outcome = apply_points(&teams, "team_1", 10, REASON_CORRECT_PLACEMENT);

    assert_eq!(outcome.teams[0].score, 10);
    assert_eq!(outcome.events.len(), 1);
    assert_eq!(outcome.events[0].kind, EventKind::PointsAwarded);
    assert_eq!(outcome.events[0].amount, 10);
    assert_eq!(outcome.events[0].reason, REASON_CORRECT_PLACEMENT);
}

#[test]
fn tripmine_hit_removes_ten_points() {
    let teams = vec![team("team_1", 40, 3, 0)];

    let outcome = apply_points(&teams, "team_1", -10, REASON_TRIPMINE_HIT);

    assert_eq!(outcome.teams[0].score, 30);
    assert_eq!(outcome.events[0].kind, EventKind::PointsRemoved);
    assert_eq!(outcome.events[0].amount, -10);
}

#[test]
fn scores_accumulate_across_rounds_and_are_never_reset() {
    let teams = vec![team("team_1", 60, 1, 0)];

    let outcome =
        reset_hearts_for_round(&apply_points(&teams, "team_1", 10, REASON_SAFE_PICK).teams);

    assert_eq!(outcome.teams[0].score, 70);
}

#[test]
fn hearts_clamp_at_zero_and_at_three() {
    let teams = vec![team("team_1", 0, 1, 0), team("team_2", 0, 3, 1)];

    let lost = apply_hearts(&teams, "team_1", -1, REASON_CORRECT_PLACEMENT);
    assert_eq!(lost.teams[0].hearts, 0);
    assert_eq!(lost.events[0].amount, -1);

    let gained = apply_hearts(&teams, "team_2", 5, REASON_GAME_MASTER);
    assert_eq!(gained.teams[1].hearts, 3);
    assert!(
        gained.events.is_empty(),
        "a change absorbed by the clamp emits no event"
    );
}

#[test]
fn a_large_negative_heart_change_stops_at_zero() {
    let teams = vec![team("team_1", 0, 2, 0)];

    let lost = apply_hearts(&teams, "team_1", -9, REASON_GAME_MASTER);

    assert_eq!(lost.teams[0].hearts, 0);
    assert_eq!(
        lost.events[0].amount, -2,
        "the event records what was applied"
    );
}

#[test]
fn wrong_placement_costs_exactly_one_heart() {
    let teams = vec![team("team_1", 0, 3, 0)];

    let outcome = apply_hearts(&teams, "team_1", -1, REASON_CORRECT_PLACEMENT);

    assert_eq!(outcome.teams[0].hearts, 2);
    assert_eq!(outcome.events[0].kind, EventKind::HeartLost);
}

#[test]
fn advance_turn_skips_a_team_with_zero_hearts() {
    let teams = vec![
        team("team_1", 0, 0, 0),
        team("team_2", 10, 3, 1),
        team("team_3", 20, 2, 2),
    ];

    assert_eq!(advance_turn(&teams, "team_1").as_deref(), Some("team_2"));
    assert_eq!(advance_turn(&teams, "team_2").as_deref(), Some("team_3"));
    assert_eq!(advance_turn(&teams, "team_3").as_deref(), Some("team_2"));
}

#[test]
fn advance_turn_returns_none_when_no_team_has_hearts() {
    let teams = vec![team("team_1", 0, 0, 0), team("team_2", 0, 0, 1)];

    assert_eq!(advance_turn(&teams, "team_1"), None);
}

#[test]
fn advance_turn_in_a_one_team_session_stays_on_team_one() {
    let teams = vec![team("team_1", 0, 3, 0)];

    assert_eq!(advance_turn(&teams, "team_1").as_deref(), Some("team_1"));
}

#[test]
fn starting_team_rotates_one_team_per_round() {
    assert_eq!(starting_team_for_round(1, 3), "team_1");
    assert_eq!(starting_team_for_round(2, 3), "team_2");
    assert_eq!(starting_team_for_round(3, 3), "team_3");
    assert_eq!(starting_team_for_round(4, 3), "team_1");
}

#[test]
fn starting_team_wraps_at_the_team_count() {
    assert_eq!(starting_team_for_round(4, 2), "team_2");
    assert_eq!(starting_team_for_round(5, 2), "team_1");
    assert_eq!(starting_team_for_round(7, 1), "team_1");
}

#[test]
fn the_first_turn_of_a_round_skips_a_team_with_zero_hearts() {
    let teams = vec![team("team_1", 0, 0, 0), team("team_2", 0, 3, 1)];

    let starter = starting_team_for_round(2, 2);
    assert_eq!(starter, "team_2");
    assert_eq!(advance_turn(&teams, "team_2").as_deref(), Some("team_2"));
}

#[test]
fn starting_a_round_resets_hearts_and_leaves_scores_untouched() {
    let teams = vec![team("team_1", 60, 0, 0), team("team_2", 40, 1, 1)];

    let outcome = reset_hearts_for_round(&teams);

    assert_eq!(outcome.teams[0].hearts, 3);
    assert_eq!(outcome.teams[1].hearts, 3);
    assert_eq!(outcome.teams[0].score, 60);
    assert_eq!(outcome.teams[1].score, 40);
    assert_eq!(outcome.events.len(), 2);
    assert!(
        outcome
            .events
            .iter()
            .all(|e| e.reason == REASON_ROUND_RESET)
    );
}

#[test]
fn a_team_with_full_hearts_gets_no_reset_event() {
    let teams = vec![team("team_1", 0, 3, 0)];

    let outcome = reset_hearts_for_round(&teams);

    assert!(outcome.events.is_empty());
    assert_eq!(outcome.teams[0].hearts, 3);
}

#[test]
fn manual_adjustment_is_recorded_as_a_game_master_event() {
    let teams = vec![team("team_1", 10, 3, 0)];

    let score = manual_adjustment(&teams, "team_1", 5, false);
    assert_eq!(score.teams[0].score, 15);
    assert_eq!(score.events[0].kind, EventKind::ManualAdjustment);
    assert_eq!(score.events[0].reason, REASON_GAME_MASTER);

    let hearts = manual_adjustment(&teams, "team_1", -9, true);
    assert_eq!(hearts.teams[0].hearts, 0);
    assert_eq!(hearts.events[0].kind, EventKind::ManualAdjustment);
}

#[test]
fn scoring_an_unknown_team_changes_nothing() {
    let teams = vec![team("team_1", 10, 3, 0)];

    let outcome = apply_points(&teams, "team_9", 10, REASON_GAME_MASTER);

    assert_eq!(outcome.teams, teams);
    assert!(outcome.events.is_empty());
}
