use super::*;

#[test]
fn a_single_team_session_is_allowed() {
    let teams = build_teams(1, &[]).unwrap();

    assert_eq!(teams.len(), 1);
    assert_eq!(teams[0].id, "team_1");
}

#[test]
fn four_teams_are_allowed() {
    assert_eq!(build_teams(4, &[]).unwrap().len(), 4);
}

#[test]
fn zero_or_five_teams_are_rejected() {
    assert!(build_teams(0, &[]).is_err());
    assert!(build_teams(5, &[]).is_err());
}

#[test]
fn default_teams_keep_the_draft_names_and_colors() {
    let teams = build_teams(4, &[]).unwrap();

    assert_eq!(teams[0].name, "Team Cyan Pulsar");
    assert_eq!(teams[0].primary_hex, "#06b6d4");
    assert_eq!(teams[0].accent_glow, "rgba(6, 182, 212, 0.45)");
    assert_eq!(teams[3].name, "Team Emerald Volt");
    assert_eq!(teams[3].primary_hex, "#10b981");
}

#[test]
fn every_team_starts_at_zero_points_and_three_hearts() {
    let teams = build_teams(3, &[]).unwrap();

    for (index, team) in teams.iter().enumerate() {
        assert_eq!(team.score, 0);
        assert_eq!(team.hearts, 3);
        assert_eq!(team.turn_index, index as i32);
    }
}

#[test]
fn turn_order_follows_team_id_order() {
    let teams = build_teams(4, &[]).unwrap();

    let ids: Vec<&str> = teams.iter().map(|t| t.id.as_str()).collect();

    assert_eq!(ids, ["team_1", "team_2", "team_3", "team_4"]);
}

#[test]
fn a_custom_color_replaces_the_palette_and_derives_the_glow() {
    let teams = build_teams(
        2,
        &[
            TeamColorInput {
                name: None,
                primary_hex: "#ABCDEF".into(),
            },
            TeamColorInput {
                name: None,
                primary_hex: "#000000".into(),
            },
        ],
    )
    .unwrap();

    assert_eq!(teams[0].primary_hex, "#abcdef");
    assert_eq!(teams[0].accent_glow, "rgba(171, 205, 239, 0.45)");
    assert_eq!(teams[0].name, "Team Cyan Pulsar", "name is untouched");
    assert_eq!(teams[1].accent_glow, "rgba(0, 0, 0, 0.45)");
}

#[test]
fn a_custom_name_replaces_the_default_team_name() {
    let teams = build_teams(
        1,
        &[TeamColorInput {
            name: Some("  The Brain Trust  ".into()),
            primary_hex: "#06b6d4".into(),
        }],
    )
    .unwrap();

    assert_eq!(teams[0].name, "The Brain Trust");
}

#[test]
fn more_colors_than_teams_is_rejected() {
    let overrides = vec![
        TeamColorInput {
            name: None,
            primary_hex: "#06b6d4".into(),
        };
        2
    ];

    assert!(build_teams(1, &overrides).is_err());
}

#[test]
fn an_invalid_hex_is_rejected() {
    for bad in ["blue", "#12345", "#gggggg", "", "06b6d4"] {
        assert!(
            normalize_hex(bad).is_err(),
            "'{bad}' should not be accepted as a hex colour"
        );
    }
}

#[test]
fn a_three_digit_hex_is_expanded() {
    assert_eq!(normalize_hex("#0cf").unwrap(), "#00ccff");
    assert_eq!(glow_from_hex("#0cf").unwrap(), "rgba(0, 204, 255, 0.45)");
}

#[test]
fn a_blank_or_overlong_team_name_is_rejected() {
    assert!(validate_team_name("   ").is_err());
    assert!(validate_team_name(&"x".repeat(41)).is_err());
    assert!(validate_team_name(&"x".repeat(40)).is_ok());
}

#[test]
fn dropping_a_team_leaves_the_remaining_colours_unique() {
    // The setup screen removes team 2 from a four-team session; the survivors must
    // not end up sharing a colour badge.
    let all = build_teams(4, &[]).unwrap();
    let kept: Vec<NewTeam> = all.into_iter().filter(|team| team.id != "team_2").collect();

    let mut colours: Vec<&str> = kept.iter().map(|t| t.color_name.as_str()).collect();
    colours.sort_unstable();

    assert_eq!(colours, ["amber", "cyan", "emerald"]);
    assert_eq!(
        kept.iter().map(|t| t.turn_index).collect::<Vec<_>>(),
        [0, 2, 3]
    );
}

#[test]
fn team_count_validation_matches_the_documented_range() {
    assert!(validate_team_count(1).is_ok());
    assert!(validate_team_count(4).is_ok());
    assert!(validate_team_count(0).is_err());
    assert!(validate_team_count(-1).is_err());
    assert!(validate_team_count(5).is_err());
}
