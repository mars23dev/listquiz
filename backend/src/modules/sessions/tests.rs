//! Unit tests for the setup-screen rules (T028-T031). Integration tests in
//! `backend/tests/session_create.rs` cover the same rules through HTTP; these pin
//! the messages and the pure logic so a router change cannot weaken them.

use super::*;
use crate::errors::ErrorCode;

fn order_round() -> RoundInput {
    RoundInput {
        round_number: 0,
        quiz_type: "ORDER".to_string(),
        list_id: "default_world_history".to_string(),
        column_id: "default_world_history_year".to_string(),
        tripmine_side: None,
    }
}

fn tripmine_round(side: Option<&str>) -> RoundInput {
    RoundInput {
        round_number: 0,
        quiz_type: "TRIPMINE".to_string(),
        list_id: "default_world_history".to_string(),
        column_id: "default_world_history_year".to_string(),
        tripmine_side: side.map(str::to_string),
    }
}

fn request() -> CreateSessionRequest {
    CreateSessionRequest {
        gm_password: "letmein".to_string(),
        team_count: Some(2),
        teams: Vec::new(),
        round_count: Some(1),
        rounds: vec![order_round()],
    }
}

#[test]
fn a_generated_session_id_has_the_documented_shape() {
    for _ in 0..200 {
        let id = generate_session_id();
        assert!(id.starts_with(SESSION_ID_PREFIX), "{id}");
        assert_eq!(id.len(), SESSION_ID_PREFIX.len() + SESSION_ID_BODY_CHARS);
        assert!(
            id[SESSION_ID_PREFIX.len()..]
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()),
            "{id}"
        );
        assert!(is_valid_session_id(&id));
    }
    assert!(is_valid_session_id("lq-abcd1234"));
    assert!(!is_valid_session_id("lq-ABCD1234"));
    assert!(!is_valid_session_id("lq-abc"));
    assert!(!is_valid_session_id("lq-waytoolong"));
    assert!(!is_valid_session_id("abcd1234"));
    assert!(!is_valid_session_id(""));
}

#[test]
fn generated_session_ids_do_not_repeat() {
    let ids: std::collections::HashSet<String> = (0..500).map(|_| generate_session_id()).collect();
    assert_eq!(ids.len(), 500);
}

#[test]
fn a_blank_gamemaster_password_is_refused() {
    for password in ["", "   ", "\t\n"] {
        let mut body = request();
        body.gm_password = password.to_string();
        let error = validate_request(&body).expect_err("blank password must be refused");
        assert_eq!(error.code, ErrorCode::BadRequest);
        assert!(error.message.contains("password"));
    }
}

#[test]
fn round_and_team_counts_are_bounded() {
    for count in [0, -1, 11, 99] {
        let mut body = request();
        body.round_count = Some(count);
        let error = validate_request(&body).expect_err("out-of-range round count");
        assert_eq!(error.code, ErrorCode::BadRequest);
        assert!(error.message.contains("Round count"));
    }
    for count in [1, 10] {
        let mut body = request();
        body.round_count = Some(count);
        body.rounds = (1..=count).map(|_| order_round()).collect();
        assert!(validate_request(&body).is_ok());
    }

    for count in [0, 5, -2] {
        let mut body = request();
        body.team_count = Some(count);
        let error = validate_request(&body).expect_err("out-of-range team count");
        assert_eq!(error.code, ErrorCode::BadRequest);
    }
    for count in 1..=4 {
        let mut body = request();
        body.team_count = Some(count);
        assert!(
            validate_request(&body).is_ok(),
            "{count} teams must be allowed"
        );
    }
}

#[test]
fn a_round_count_that_disagrees_with_the_round_list_is_refused() {
    let mut body = request();
    body.round_count = Some(3);
    let error = validate_request(&body).expect_err("mismatched round count");
    assert_eq!(error.code, ErrorCode::BadRequest);
    assert!(error.message.contains("roundCount"));
}

#[test]
fn the_team_count_defaults_to_the_team_list_length() {
    let mut body = request();
    body.team_count = None;
    body.teams = vec![
        TeamColorInput {
            name: Some("Blau".to_string()),
            primary_hex: String::new(),
        },
        TeamColorInput {
            name: Some("Gelb".to_string()),
            primary_hex: String::new(),
        },
    ];
    let (team_count, _, teams, _) = validate_request(&body).expect("two named teams");
    assert_eq!(team_count, 2);
    assert_eq!(teams[0].name, "Blau");
    assert_eq!(teams[1].name, "Gelb");
}

#[test]
fn tripmine_side_is_required_only_for_tripmine_rounds() {
    let mut body = request();
    body.rounds = vec![tripmine_round(None)];
    let error = validate_request(&body).expect_err("TRIPMINE without a side");
    assert_eq!(error.code, ErrorCode::BadRequest);
    assert!(error.message.contains("tripmineSide"));

    let mut body = request();
    body.rounds = vec![tripmine_round(Some("highest"))];
    assert!(validate_request(&body).is_ok());

    let mut body = request();
    body.rounds = vec![tripmine_round(Some("sideways"))];
    assert_eq!(
        validate_request(&body).expect_err("unknown side").code,
        ErrorCode::BadRequest
    );

    let mut body = request();
    let mut round = order_round();
    round.tripmine_side = Some("highest".to_string());
    body.rounds = vec![round];
    assert_eq!(
        validate_request(&body).expect_err("ORDER with a side").code,
        ErrorCode::BadRequest
    );
}

#[test]
fn an_unknown_quiz_type_is_refused() {
    let mut body = request();
    let mut round = order_round();
    round.quiz_type = "PICK_TWO".to_string();
    body.rounds = vec![round];
    assert_eq!(
        validate_request(&body).expect_err("unknown quiz type").code,
        ErrorCode::BadRequest
    );
}

#[test]
fn a_round_without_list_or_column_is_refused() {
    let mut body = request();
    let mut round = order_round();
    round.list_id = "  ".to_string();
    body.rounds = vec![round];
    assert!(
        validate_request(&body)
            .expect_err("no list")
            .message
            .contains("listId")
    );

    let mut body = request();
    let mut round = order_round();
    round.column_id = String::new();
    body.rounds = vec![round];
    assert!(
        validate_request(&body)
            .expect_err("no column")
            .message
            .contains("columnId")
    );
}

#[test]
fn quiz_type_and_side_parse_case_insensitively() {
    assert_eq!(QuizType::parse("order"), Some(QuizType::Order));
    assert_eq!(QuizType::parse(" Tripmine "), Some(QuizType::Tripmine));
    assert_eq!(QuizType::parse("tripmine"), Some(QuizType::Tripmine));
    assert_eq!(QuizType::parse(""), None);
    assert_eq!(TripmineSide::parse("HIGHEST"), Some(TripmineSide::Highest));
    assert_eq!(TripmineSide::parse("lowest"), Some(TripmineSide::Lowest));
    assert_eq!(TripmineSide::parse("middle"), None);
}

#[test]
fn round_numbers_default_to_their_position() {
    let mut body = request();
    body.round_count = Some(2);
    let mut second = order_round();
    second.round_number = 0;
    body.rounds = vec![order_round(), second];
    let (_, _, _, rounds) = validate_request(&body).expect("two rounds");
    assert_eq!(rounds[0].round_number, 1);
    assert_eq!(rounds[1].round_number, 2);

    let mut body = request();
    body.round_count = Some(2);
    let mut numbered = order_round();
    numbered.round_number = 7;
    body.rounds = vec![order_round(), numbered];
    let (_, _, _, rounds) = validate_request(&body).expect("explicit round number kept");
    assert_eq!(rounds[1].round_number, 7);
}
