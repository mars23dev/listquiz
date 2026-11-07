//! Quick-game request builder, tested without a database (feature 005, US5; 018).

use rand::SeedableRng;
use rand::rngs::SmallRng;

use super::*;
use crate::errors::ErrorCode;
use crate::modules::default_lists::{DEFAULT_LISTS, DefaultList};

fn rng(seed: u64) -> SmallRng {
    SmallRng::seed_from_u64(seed)
}

/// The ids of the shipped lists that fit `game_type`.
fn eligible_ids(game_type: &str) -> Vec<&'static str> {
    let needed = required_items(QuizType::parse(game_type).expect("a game type"));
    DEFAULT_LISTS
        .iter()
        .filter(|list| {
            list.columns
                .first()
                .is_some_and(|column| valued_item_count(list, column.id) >= needed)
        })
        .map(|list| list.id)
        .collect()
}

#[test]
fn an_order_quick_game_is_one_team_and_one_round() {
    let request = build_request("ORDER", &DEFAULT_LISTS, &mut rng(1)).expect("a fitting list");

    assert_eq!(request.effective_team_count(), 1);
    assert_eq!(request.effective_round_count(), 1);
    assert_eq!(request.rounds[0].quiz_type, "ORDER");
    assert!(
        eligible_ids("ORDER").contains(&request.rounds[0].list_id.as_str()),
        "the list must be one of the fitting lists"
    );
    assert_eq!(request.rounds[0].tripmine_side, None);
    assert!(!request.rounds[0].column_id.is_empty());
}

#[test]
fn a_tripmine_quick_game_picks_a_side() {
    let request = build_request("TRIPMINE", &DEFAULT_LISTS, &mut rng(2)).expect("a fitting list");

    assert_eq!(request.rounds[0].quiz_type, "TRIPMINE");
    assert_eq!(request.rounds[0].tripmine_side.as_deref(), Some("highest"));
    assert!(eligible_ids("TRIPMINE").contains(&request.rounds[0].list_id.as_str()));
}

#[test]
fn a_connect_quick_game_is_built() {
    let request = build_request("CONNECT", &DEFAULT_LISTS, &mut rng(3)).expect("a fitting list");

    assert_eq!(request.rounds[0].quiz_type, "CONNECT");
    assert!(eligible_ids("CONNECT").contains(&request.rounds[0].list_id.as_str()));
}

#[test]
fn the_chosen_list_varies_across_seeds() {
    let ids: std::collections::BTreeSet<String> = (0..30)
        .map(|seed| {
            build_request("CONNECT", &DEFAULT_LISTS, &mut rng(seed))
                .expect("a fitting list")
                .rounds[0]
                .list_id
                .clone()
        })
        .collect();

    assert!(
        ids.len() > 1,
        "the list should vary across quick games: {ids:?}"
    );
}

#[test]
fn a_single_eligible_list_is_always_chosen() {
    let only = [DefaultList {
        id: "only",
        title: "Only",
        description: "",
        columns: DEFAULT_LISTS[0].columns,
        items: DEFAULT_LISTS[0].items,
    }];

    for seed in 0..5 {
        let request = build_request("ORDER", &only, &mut rng(seed)).expect("the only list");
        assert_eq!(request.rounds[0].list_id, "only");
    }
}

#[test]
fn an_unknown_game_type_is_rejected() {
    let error = build_request("SNAKES", &DEFAULT_LISTS, &mut rng(1)).expect_err("not a game type");
    assert_eq!(error.code, ErrorCode::BadRequest);
}

#[test]
fn items_without_the_chosen_column_value_do_not_count() {
    const ITEMS: &[crate::modules::default_lists::DefaultItem] = &[
        crate::modules::default_lists::DefaultItem {
            id: "n1",
            name: "No value",
            icon_name: "",
            values: &[],
        },
        crate::modules::default_lists::DefaultItem {
            id: "n2",
            name: "No value",
            icon_name: "",
            values: &[],
        },
        crate::modules::default_lists::DefaultItem {
            id: "n3",
            name: "No value",
            icon_name: "",
            values: &[],
        },
        crate::modules::default_lists::DefaultItem {
            id: "n4",
            name: "No value",
            icon_name: "",
            values: &[],
        },
        crate::modules::default_lists::DefaultItem {
            id: "n5",
            name: "No value",
            icon_name: "",
            values: &[],
        },
        crate::modules::default_lists::DefaultItem {
            id: "n6",
            name: "No value",
            icon_name: "",
            values: &[],
        },
        crate::modules::default_lists::DefaultItem {
            id: "n7",
            name: "No value",
            icon_name: "",
            values: &[],
        },
        crate::modules::default_lists::DefaultItem {
            id: "n8",
            name: "No value",
            icon_name: "",
            values: &[],
        },
        crate::modules::default_lists::DefaultItem {
            id: "n9",
            name: "No value",
            icon_name: "",
            values: &[],
        },
        crate::modules::default_lists::DefaultItem {
            id: "n10",
            name: "No value",
            icon_name: "",
            values: &[],
        },
        crate::modules::default_lists::DefaultItem {
            id: "n11",
            name: "No value",
            icon_name: "",
            values: &[],
        },
    ];

    let list = [DefaultList {
        id: "no-values",
        title: "No values",
        description: "",
        columns: DEFAULT_LISTS[0].columns,
        items: ITEMS,
    }];

    let error = build_request("ORDER", &list, &mut rng(1)).expect_err("nothing fits");
    assert_eq!(error.code, ErrorCode::NotReady);
}

#[test]
fn no_fitting_list_is_an_error() {
    let too_small = [DefaultList {
        id: "tiny",
        title: "Tiny",
        description: "",
        columns: &[],
        items: &[],
    }];

    let error = build_request("ORDER", &too_small, &mut rng(1)).expect_err("nothing fits");
    assert_eq!(error.code, ErrorCode::NotReady);
}
