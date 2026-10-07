//! List validation rules (T115-T116), tested without a database.

use super::*;
use crate::errors::ErrorCode;

fn item(prefix: &str, index: usize) -> ItemInput {
    let mut values = HashMap::new();
    values.insert(
        "year".to_string(),
        ValueInput {
            display: (1900 + index).to_string(),
            numeric: Some((1900 + index) as f64),
            was_defaulted: None,
        },
    );
    ItemInput {
        id: None,
        name: format!("{prefix} {index}"),
        icon_name: Some("Tag".to_string()),
        icon_image: None,
        position: Some(index as i64),
        values,
    }
}

fn input_with(title: &str, count: usize) -> ListInput {
    ListInput {
        id: None,
        title: title.to_string(),
        description: None,
        order_direction: "low_to_high".to_string(),
        columns: vec![ColumnInput {
            id: Some("year".to_string()),
            name: "Year".to_string(),
        }],
        items: (0..count).map(|index| item("Item", index)).collect(),
    }
}

// T115 ---------------------------------------------------------------------

#[test]
fn a_list_must_hold_between_eleven_and_one_hundred_items() {
    let too_few = build_new(&input_with("Small", 10)).expect_err("ten items");
    assert_eq!(too_few.code, ErrorCode::TooSmall);
    assert!(too_few.message.contains("11"));

    let too_many = build_new(&input_with("Big", 101)).expect_err("101 items");
    assert_eq!(too_many.code, ErrorCode::TooLarge);
    assert!(too_many.message.contains("100"));

    assert!(build_new(&input_with("Just right", 11)).is_ok());
}

#[test]
fn an_empty_title_is_rejected() {
    let error = build_new(&input_with("   ", 11)).expect_err("blank title");
    assert_eq!(error.code, ErrorCode::BadRequest);
}

// T116 ---------------------------------------------------------------------

#[test]
fn item_positions_are_kept_exactly_as_supplied() {
    let mut input = input_with("Ordered", 11);
    // A deliberately non-sequential order must survive untouched (FR-055).
    input.items[0].position = Some(5);
    input.items[5].position = Some(0);

    let built = build_new(&input).expect("built");
    assert_eq!(built.items[0].position, 5);
    assert_eq!(built.items[5].position, 0);
}
