use super::*;

#[test]
fn four_default_lists_are_seeded() {
    assert_eq!(DEFAULT_LISTS.len(), 4);
    assert_eq!(DEFAULT_LISTS[0].title, "World History & Inventions");
    assert_eq!(
        DEFAULT_LISTS[1].title,
        "Iconic Statues & Architectural Marvels"
    );
    assert_eq!(DEFAULT_LISTS[2].title, "Blockbuster Cinema Box Office");
    assert_eq!(DEFAULT_LISTS[3].title, "Countries of the World");
}

#[test]
fn the_countries_list_holds_every_sovereign_state_with_a_flag_and_three_values() {
    let list = DEFAULT_LISTS
        .iter()
        .find(|l| l.id == "default_countries")
        .expect("the countries list");

    assert_eq!(list.items.len(), 195, "every sovereign state");
    for item in list.items.iter() {
        assert!(
            item.icon_name.starts_with("flag:"),
            "{} has no flag icon (icon_name = {})",
            item.name,
            item.icon_name
        );
        for column in ["area_km2", "population", "founded"] {
            let value = item
                .values
                .iter()
                .find(|(c, _, _)| *c == column)
                .unwrap_or_else(|| panic!("{} is missing the {column} value", item.name));
            assert!(!value.1.is_empty(), "{} has an empty {column}", item.name);
            assert!(
                value.2.is_finite(),
                "{} has a non-finite {column}",
                item.name
            );
        }
    }
}

#[test]
fn the_expanded_default_lists_hold_twenty_to_fifty_items() {
    for id in [
        "default_world_history",
        "default_statues_monuments",
        "default_cinema_box_office",
    ] {
        let list = DEFAULT_LISTS.iter().find(|l| l.id == id).expect(id);
        assert!(
            (20..=50).contains(&list.items.len()),
            "{} has {} items",
            list.title,
            list.items.len()
        );
        validate_list_size(list.items.len()).unwrap();
    }
}

#[test]
fn every_default_list_has_at_least_one_column() {
    for list in DEFAULT_LISTS.iter() {
        assert!(!list.columns.is_empty(), "{} has no columns", list.title);
        validate_column_name(list.columns[0].name).unwrap();
    }
}

#[test]
fn every_default_value_row_has_a_matching_column() {
    for list in DEFAULT_LISTS.iter() {
        for item in list.items.iter() {
            for (column_id, display, numeric) in item.values.iter() {
                assert!(
                    list.columns.iter().any(|c| c.id == *column_id),
                    "{} item {} references unknown column {}",
                    list.title,
                    item.name,
                    column_id
                );
                assert!(!display.is_empty());
                assert!(numeric.is_finite());
            }
        }
    }
}

#[test]
fn default_item_ids_are_unique_within_each_list() {
    for list in DEFAULT_LISTS.iter() {
        let mut ids: Vec<&str> = list.items.iter().map(|i| i.id).collect();
        let count = ids.len();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), count, "{} has duplicate item ids", list.title);
    }
}

#[test]
fn eleven_items_pass_and_ten_or_more_than_a_hundred_do_not() {
    assert!(validate_list_size(11).is_ok());
    assert!(validate_list_size(100).is_ok());

    let too_small = validate_list_size(10).unwrap_err();
    assert_eq!(too_small.code, crate::errors::ErrorCode::TooSmall);

    let too_large = validate_list_size(101).unwrap_err();
    assert_eq!(too_large.code, crate::errors::ErrorCode::TooLarge);
}

#[test]
fn an_empty_list_is_too_small() {
    assert!(validate_list_size(0).is_err());
}

#[test]
fn list_titles_are_limited_to_eighty_characters() {
    assert!(validate_list_title("Top Grossing Films").is_ok());
    assert!(validate_list_title(&"x".repeat(80)).is_ok());
    assert!(validate_list_title(&"x".repeat(81)).is_err());
    assert!(validate_list_title("   ").is_err());
}

#[test]
fn column_names_are_limited_to_forty_characters() {
    assert!(validate_column_name("Year Built").is_ok());
    assert!(validate_column_name(&"x".repeat(40)).is_ok());
    assert!(validate_column_name(&"x".repeat(41)).is_err());
    assert!(validate_column_name("").is_err());
}

#[test]
fn seeding_writes_every_list_item_and_value() {
    let conn = Connection::open_in_memory().unwrap();
    crate::db::apply_schema(&conn).unwrap();
    crate::db::seed_default_folder(&conn).unwrap();
    seed_default_lists(&conn, "default").unwrap();

    let lists: i64 = conn
        .query_row("SELECT COUNT(*) FROM quiz_list", [], |r| r.get(0))
        .unwrap();
    let items: i64 = conn
        .query_row("SELECT COUNT(*) FROM item", [], |r| r.get(0))
        .unwrap();
    let values: i64 = conn
        .query_row("SELECT COUNT(*) FROM item_value", [], |r| r.get(0))
        .unwrap();

    assert_eq!(lists, DEFAULT_LISTS.len() as i64);
    assert_eq!(items, expected_default_item_count());
    assert_eq!(values, expected_default_value_count());
}

/// The number of items the shipped seed holds, read from the constants so the test
/// tracks content changes instead of hard-coding a count.
fn expected_default_item_count() -> i64 {
    DEFAULT_LISTS.iter().map(|l| l.items.len() as i64).sum()
}

/// The number of value rows the shipped seed holds.
fn expected_default_value_count() -> i64 {
    DEFAULT_LISTS
        .iter()
        .flat_map(|l| l.items.iter())
        .map(|i| i.values.len() as i64)
        .sum()
}

#[test]
fn seeding_twice_does_not_duplicate_anything() {
    let conn = Connection::open_in_memory().unwrap();
    crate::db::apply_schema(&conn).unwrap();
    crate::db::seed_default_folder(&conn).unwrap();
    seed_default_lists(&conn, "default").unwrap();
    seed_default_lists(&conn, "default").unwrap();

    let items: i64 = conn
        .query_row("SELECT COUNT(*) FROM item", [], |r| r.get(0))
        .unwrap();
    assert_eq!(items, expected_default_item_count());
}

#[test]
fn seeded_lists_live_in_the_default_folder_and_are_read_only() {
    let conn = Connection::open_in_memory().unwrap();
    crate::db::apply_schema(&conn).unwrap();
    crate::db::seed_default_folder(&conn).unwrap();
    seed_default_lists(&conn, "default").unwrap();

    let row: (String, i64) = conn
        .query_row(
            "SELECT folder_id, is_default FROM quiz_list WHERE id = 'default_world_history'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();

    assert_eq!(row, ("default".to_string(), 1));
}

#[test]
fn every_seeded_list_can_host_an_order_round() {
    // FR-026 takes the anchor from the middle of the list, FR-027 then samples five
    // items below and five above it, so each seeded column needs a median with at
    // least five values on either side.
    for list in DEFAULT_LISTS.iter() {
        let column = list.columns[0].id;
        let mut values: Vec<f64> = list
            .items
            .iter()
            .filter_map(|i| {
                i.values
                    .iter()
                    .find(|(c, _, _)| *c == column)
                    .map(|(_, _, n)| *n)
            })
            .collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = values[values.len() / 2];
        let below = values.iter().filter(|v| **v < median).count();
        let above = values.iter().filter(|v| **v > median).count();

        assert!(
            below >= 5 && above >= 5,
            "{}: median {median} has {below} below and {above} above",
            list.title
        );
    }
}

/// The column whose numeric value orders a shipped list (feature 015).
fn ordering_column(list_id: &str) -> &'static str {
    match list_id {
        "default_world_history" => "year",
        "default_statues_monuments" => "height",
        "default_cinema_box_office" => "box_office",
        "default_countries" => "area_km2",
        other => panic!("no ordering column for {other}"),
    }
}

#[test]
fn format_value_renders_magnitudes_and_plain_numbers() {
    assert_eq!(format_value(1_440_000_000.0), "1.44 Billion");
    assert_eq!(format_value(2_920_000_000.0), "2.92 Billion");
    assert_eq!(format_value(146_000_000.0), "146 Million");
    assert_eq!(format_value(652_230.0), "652,230");
    assert_eq!(format_value(1969.0), "1969");
    assert_eq!(format_value(48.0), "48");
    assert_eq!(format_value(-2560.0), "-2560");
    assert_eq!(format_value(0.49), "0.49");
}

#[test]
fn seeding_stores_each_shipped_list_in_ascending_value_order() {
    let conn = Connection::open_in_memory().unwrap();
    crate::db::apply_schema(&conn).unwrap();
    crate::db::seed_default_folder(&conn).unwrap();
    seed_default_lists(&conn, "default").unwrap();

    for list in DEFAULT_LISTS.iter() {
        let column = ordering_column(list.id);
        let mut statement = conn
            .prepare(
                "SELECT v.numeric FROM item i
                 JOIN item_value v ON v.item_id = i.id
                 WHERE i.list_id = ?1 AND v.column_id = ?2
                 ORDER BY i.position",
            )
            .unwrap();
        let values: Vec<f64> = statement
            .query_map(params![list.id, column], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<f64>, _>>()
            .unwrap();

        for window in values.windows(2) {
            assert!(
                window[0] <= window[1],
                "{} is not ascending by {column}: {} then {}",
                list.title,
                window[0],
                window[1]
            );
        }
    }
}
