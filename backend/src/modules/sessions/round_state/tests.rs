//! Unit tests for the round builder. The Order anchor and the Tripmine ranking are
//! the two places where a plausible-looking shortcut would quietly break a game
//! rule, so both are pinned here.

use super::*;
use crate::modules::sessions::QuizType;

fn items(count: usize) -> Vec<ListItemValue> {
    (1..=count)
        .map(|n| ListItemValue {
            item_id: format!("i{n}"),
            name: format!("Item {n}"),
            display: n.to_string(),
            numeric: n as f64 * 10.0,
        })
        .collect()
}

#[test]
fn the_order_anchor_is_the_median_item_with_five_on_each_side() {
    let layout = build_order_layout(items(13)).expect("layout");

    assert_eq!(layout.anchor.item_id, "i7");
    assert_eq!(layout.left.len(), ORDER_SIDE_ITEMS);
    assert_eq!(layout.right.len(), ORDER_SIDE_ITEMS);
    assert_eq!(layout.left.last().unwrap().item_id, "i6");
    assert_eq!(layout.right.first().unwrap().item_id, "i8");
}

#[test]
fn the_leftover_items_stay_in_the_pool() {
    let layout = build_order_layout(items(20)).expect("layout");

    assert_eq!(layout.pool.len(), 20 - 11);
    let placed: Vec<&str> = layout
        .left
        .iter()
        .chain(layout.right.iter())
        .chain(std::iter::once(&layout.anchor))
        .map(|i| i.item_id.as_str())
        .collect();
    for item in &layout.pool {
        assert!(!placed.contains(&item.item_id.as_str()), "{}", item.item_id);
    }
}

#[test]
fn the_smallest_possible_list_still_builds() {
    let layout = build_order_layout(items(MIN_ITEMS_FOR_ORDER)).expect("layout");

    assert_eq!(layout.anchor.item_id, "i6");
    assert_eq!(layout.left.len(), 5);
    assert_eq!(layout.right.len(), 5);
    assert!(layout.pool.is_empty());
}

#[test]
fn a_list_too_small_for_an_order_round_is_not_ready() {
    let error = build_order_layout(items(MIN_ITEMS_FOR_ORDER - 1))
        .expect_err("ten items cannot make an order round");
    assert_eq!(error.code, crate::errors::ErrorCode::NotReady);
    assert!(error.message.contains("at least 11"));
}

#[test]
fn a_highest_tripmine_mines_the_top_of_the_ranking() {
    let grid = build_tripmine_grid(&items(12), TripmineSide::Highest).expect("grid");

    assert_eq!(grid.len(), TRIPMINE_SLOTS);
    assert_eq!(grid[0].item.item_id, "i1");
    assert_eq!(grid[0].true_rank, 1);
    let mine = grid.iter().find(|slot| slot.is_tripmine).expect("one mine");
    assert_eq!(mine.true_rank, TRIPMINE_SLOTS as i32);
    assert_eq!(mine.item.numeric, 100.0);
    assert_eq!(grid.iter().filter(|s| s.is_tripmine).count(), 1);
    assert_eq!(starting_search_rank(TripmineSide::Highest), 1);
}

#[test]
fn a_lowest_tripmine_mines_the_bottom_of_the_ranking() {
    let grid = build_tripmine_grid(&items(12), TripmineSide::Lowest).expect("grid");

    let mine = grid.iter().find(|slot| slot.is_tripmine).expect("one mine");
    assert_eq!(mine.true_rank, 1);
    assert_eq!(mine.item.numeric, 10.0);
    assert_eq!(mine.item.item_id, "i1");
    assert_eq!(starting_search_rank(TripmineSide::Lowest), 10);
}

#[test]
fn the_search_walks_towards_the_mined_rank() {
    let highest = TripmineSide::Highest;
    assert_eq!(next_search_rank(highest, 1), Some(2));
    assert_eq!(next_search_rank(highest, 9), Some(10));
    assert_eq!(next_search_rank(highest, 10), None);

    let lowest = TripmineSide::Lowest;
    assert_eq!(next_search_rank(lowest, 10), Some(9));
    assert_eq!(next_search_rank(lowest, 2), Some(1));
    assert_eq!(next_search_rank(lowest, 1), None);
}

#[test]
fn a_tripmine_grid_starts_hidden() {
    let grid = build_tripmine_grid(&items(12), TripmineSide::Highest).expect("grid");

    assert!(grid.iter().all(|slot| !slot.revealed));
    assert!(grid.iter().all(|slot| slot.revealed_by.is_none()));
    assert!(grid.iter().all(|slot| slot.points_awarded.is_none()));
}

#[test]
fn fewer_than_ten_items_is_not_ready_for_tripmine() {
    let error = build_tripmine_grid(&items(9), TripmineSide::Highest)
        .expect_err("nine items cannot fill a tripmine grid");
    assert_eq!(error.code, crate::errors::ErrorCode::NotReady);
}

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
