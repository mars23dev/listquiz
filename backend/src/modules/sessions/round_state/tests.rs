//! Unit tests for the round builder: value loading and its sort direction.

use super::*;
use crate::modules::sessions::QuizType;

#[test]
fn items_load_sorted_low_to_high_unless_the_list_says_otherwise() {
    let conn = rusqlite::Connection::open_in_memory().expect("memory database");
    crate::db::apply_schema(&conn).expect("schema");
    conn.execute(
        "INSERT INTO folder (id, name, password, is_default, created_at)
         VALUES ('default', 'Default', '', 1, 0)",
        [],
    )
    .expect("folder");
    conn.execute(
        "INSERT INTO quiz_list (id, folder_id, title, description, item_count, order_direction, created_at, updated_at)
         VALUES ('l', 'default', 'List', '', 3, 'low_to_high', 0, 0)",
        [],
    )
    .expect("list");
    conn.execute(
        "INSERT INTO list_item_column (id, list_id, name, position) VALUES ('c', 'l', 'Number', 0)",
        [],
    )
    .expect("column");
    for (id, name, numeric) in [("b", "Bravo", 20.0), ("a", "Alpha", 10.0), ("c", "C", 5.0)] {
        conn.execute(
            "INSERT INTO item (id, list_id, name, icon_name, position) VALUES (?1, 'l', ?2, 'Tag', 0)",
            rusqlite::params![id, name],
        )
        .expect("item");
        conn.execute(
            "INSERT INTO item_value (item_id, column_id, display, numeric) VALUES (?1, 'c', ?2, ?3)",
            rusqlite::params![id, numeric.to_string(), numeric],
        )
        .expect("value");
    }

    let ascending = load_ordered_items(&conn, "l", "c", true).expect("ascending");
    assert_eq!(
        ascending
            .iter()
            .map(|i| i.item_id.as_str())
            .collect::<Vec<_>>(),
        vec!["c", "a", "b"]
    );
    assert_eq!(ascending[0].display, "5");

    let descending = load_ordered_items(&conn, "l", "c", false).expect("descending");
    assert_eq!(
        descending
            .iter()
            .map(|i| i.item_id.as_str())
            .collect::<Vec<_>>(),
        vec!["b", "a", "c"]
    );

    assert!(
        load_ordered_items(&conn, "missing", "c", true)
            .expect("unknown list is not an error here")
            .is_empty()
    );
}

#[test]
fn quiz_type_and_side_survive_a_round_trip_through_strings() {
    assert_eq!(
        QuizType::parse(QuizType::Order.as_str()),
        Some(QuizType::Order)
    );
    assert_eq!(
        QuizType::parse(QuizType::Tripmine.as_str()),
        Some(QuizType::Tripmine)
    );
    assert_eq!(
        TripmineSide::parse(TripmineSide::Highest.as_str()),
        Some(TripmineSide::Highest)
    );
    assert_eq!(
        TripmineSide::parse(TripmineSide::Lowest.as_str()),
        Some(TripmineSide::Lowest)
    );
}
