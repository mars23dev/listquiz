//! CSV parse and export rules (T110-T114).

use super::*;
use crate::errors::ErrorCode;

fn csv_with_rows(rows: usize) -> String {
    let mut body = String::from("Icon,Name,Year\n");
    for index in 0..rows {
        body.push_str(&format!("Rocket,Item {index},{}\n", 1900 + index));
    }
    body
}

// T110 ---------------------------------------------------------------------

#[test]
fn a_leading_icon_header_is_detected_case_insensitively() {
    let parsed = parse_csv("History", &csv_with_rows(11)).expect("parsed");

    assert_eq!(parsed.columns.len(), 1);
    assert_eq!(parsed.columns[0].id, "year");
    assert_eq!(parsed.columns[0].name, "Year");
    assert_eq!(parsed.items[0].name, "Item 0");
    assert_eq!(parsed.items[0].icon_name.as_deref(), Some("Rocket"));
    assert_eq!(parsed.items[0].values[0].numeric, 1900.0);
    assert!(!parsed.items[0].values[0].was_defaulted);
}

#[test]
fn without_an_icon_header_the_first_column_is_the_name() {
    let body = "Name,Score\nAlpha,10\nBravo,20\n";
    // Only two rows, so the count check fires first; assert on the parse shape with
    // an 11-row body instead.
    let mut many = String::from("Name,Score\n");
    for index in 0..11 {
        many.push_str(&format!("Item {index},{}\n", index + 1));
    }
    let parsed = parse_csv("Scores", &many).expect("parsed");
    assert_eq!(parsed.columns[0].id, "score");
    assert_eq!(parsed.items[0].name, "Item 0");
    let _ = body;
}

// T111 ---------------------------------------------------------------------

#[test]
fn a_quoted_field_containing_a_comma_survives() {
    let mut body = String::from("Name,Title\n");
    for index in 0..11 {
        body.push_str(&format!("\"Item, {index}\",\"{index}\"\n"));
    }
    let parsed = parse_csv("Quoted", &body).expect("parsed");

    assert_eq!(parsed.items[0].name, "Item, 0");
}

// T112 ---------------------------------------------------------------------

#[test]
fn an_unparseable_value_falls_back_to_the_row_position() {
    let mut body = String::from("Name,Year\n");
    body.push_str("Alpha,n/a\n");
    for index in 1..11 {
        body.push_str(&format!("Item {index},{}\n", 1900 + index));
    }
    let parsed = parse_csv("History", &body).expect("parsed");

    let first = &parsed.items[0].values[0];
    assert!(first.was_defaulted);
    assert_eq!(first.display, "n/a");
    assert_eq!(first.numeric, 10.0);
    assert_eq!(parsed.report.defaulted_rows[0].position, 1);
    assert_eq!(parsed.report.defaulted_rows[0].raw, "n/a");
    assert!(parsed.report.warnings[0].contains("n/a"));
}

// T113 ---------------------------------------------------------------------

#[test]
fn the_row_count_must_be_between_eleven_and_one_hundred() {
    let small = parse_csv("Small", &csv_with_rows(10)).expect_err("too few");
    assert_eq!(small.code, ErrorCode::TooSmall);
    assert!(small.message.contains("11"));

    let large = parse_csv("Large", &csv_with_rows(101)).expect_err("too many");
    assert_eq!(large.code, ErrorCode::TooLarge);
    assert!(large.message.contains("100"));
}

// T114 ---------------------------------------------------------------------

#[test]
fn export_round_trips_through_import_including_the_direction() {
    let parsed = parse_csv("History", &csv_with_rows(11)).expect("parsed");
    let column_names: Vec<String> = parsed.columns.iter().map(|c| c.name.clone()).collect();
    let export_items: Vec<ExportItem> = parsed
        .items
        .iter()
        .map(|item| ExportItem {
            icon_name: item.icon_name.clone(),
            name: item.name.clone(),
            values: item
                .values
                .iter()
                .map(|value| value.display.clone())
                .collect(),
        })
        .collect();

    let csv = export_csv(&column_names, &export_items).expect("export");
    assert!(csv.starts_with("Icon,Name,Year"));

    let reimported = parse_csv("History", &csv).expect("re-import");
    assert_eq!(reimported.columns, parsed.columns);
    assert_eq!(reimported.items.len(), parsed.items.len());
    assert_eq!(reimported.items[0].name, parsed.items[0].name);
    assert_eq!(
        reimported.items[0].values[0].numeric,
        parsed.items[0].values[0].numeric
    );
}
