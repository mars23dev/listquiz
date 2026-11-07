//! Capacity-rule tests: thresholds, exact-minimum acceptance, and refusal copy.

use super::*;

#[test]
fn each_mode_reports_its_minimum() {
    assert_eq!(
        required_valued_items(QuizType::Order, None).unwrap(),
        order_round::MIN_ITEMS_FOR_ORDER
    );
    assert_eq!(
        required_valued_items(QuizType::Connect, None).unwrap(),
        connect_round::PAIRS
    );
    assert_eq!(
        required_valued_items(QuizType::Tripmine, None).unwrap(),
        tripmine_round::TRIPMINE_SLOTS
    );
    assert_eq!(
        required_valued_items(QuizType::Tripmine, Some(10)).unwrap(),
        tripmine_round::TRIPMINE_SLOTS
    );
    assert_eq!(
        required_valued_items(QuizType::Tripmine, Some(20)).unwrap(),
        tripmine_round::TRIPMINE_SLOTS_LARGE
    );
}

#[test]
fn an_unsupported_tripmine_size_is_refused() {
    let error = required_valued_items(QuizType::Tripmine, Some(15)).expect_err("15 is not a size");
    assert_eq!(error.code, crate::errors::ErrorCode::BadRequest);
}

#[test]
fn the_exact_minimum_is_valid() {
    assert!(validate_capacity(1, QuizType::Order, None, 11).is_ok());
    assert!(validate_capacity(2, QuizType::Connect, None, 10).is_ok());
    assert!(validate_capacity(3, QuizType::Tripmine, Some(10), 10).is_ok());
    assert!(validate_capacity(4, QuizType::Tripmine, Some(20), 20).is_ok());
}

#[test]
fn one_below_the_minimum_is_refused_with_the_shortfall() {
    let error = validate_capacity(2, QuizType::Tripmine, Some(20), 19).expect_err("19 is short");

    assert_eq!(error.code, crate::errors::ErrorCode::BadRequest);
    assert!(error.message.contains("Round 2"), "{}", error.message);
    assert!(
        error.message.contains("Tripmine with 20 items"),
        "{}",
        error.message
    );
    assert!(error.message.contains("at least 20"), "{}", error.message);
    assert!(error.message.contains("list has 19"), "{}", error.message);
}
