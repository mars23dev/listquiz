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

fn seeded_conn() -> rusqlite::Connection {
    let conn = rusqlite::Connection::open_in_memory().expect("memory database");
    crate::db::apply_schema(&conn).expect("schema");
    crate::db::seed_default_folder(&conn).expect("default folder");
    crate::modules::default_lists::seed_default_lists(&conn, "default").expect("default lists");
    conn
}

// T131 ---------------------------------------------------------------------

#[test]
fn crossing_the_last_round_finishes_the_game_and_reveals_everything() {
    let conn = seeded_conn();
    let mut body = request();
    body.round_count = Some(2);
    body.rounds = vec![order_round(), order_round()];
    for round in body.rounds.iter_mut() {
        round.column_id = "year".to_string();
    }
    let session_id = create_session(&conn, &body).expect("session created");

    assert!(!advance_round(&conn, &session_id).expect("round 1").finished);
    assert!(!advance_round(&conn, &session_id).expect("round 2").finished);
    let last = advance_round(&conn, &session_id).expect("finish");
    assert!(last.finished, "crossing the last round finishes the game");

    let (status, reveal, active): (String, i64, Option<String>) = conn
        .query_row(
            "SELECT status, reveal_values, active_team_id FROM game_session WHERE id = ?1",
            [&session_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .expect("session row");
    assert_eq!(status, "finished");
    assert_eq!(reveal, 1, "the full reveal is on (FR-024)");
    assert!(active.is_none(), "nobody is on turn once finished");

    // Advancing a finished game is refused (T133).
    assert!(advance_round(&conn, &session_id).is_err());
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

#[test]
fn deleting_a_finished_session_removes_its_rows_and_is_idempotent() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    crate::db::apply_schema(&conn).unwrap();
    crate::db::insert_test_session(&conn, "s1", 1);
    conn.execute(
        "INSERT INTO team
            (id, session_id, name, color, primary_hex, accent_glow, score, hearts, turn_index)
         VALUES ('team_1', 's1', 'Team', 'cyan', '#06b6d4', 'rgba(0,0,0,0)', 0, 3, 0)",
        [],
    )
    .unwrap();

    // A running session cannot be deleted.
    let error = delete_session(&conn, "s1").expect_err("a live session is protected");
    assert_eq!(error.code, ErrorCode::NotReady);

    conn.execute(
        "UPDATE game_session SET status = 'finished' WHERE id = 's1'",
        [],
    )
    .unwrap();
    delete_session(&conn, "s1").expect("a finished session is deleted");

    let sessions: i64 = conn
        .query_row("SELECT COUNT(*) FROM game_session", [], |row| row.get(0))
        .unwrap();
    let teams: i64 = conn
        .query_row("SELECT COUNT(*) FROM team", [], |row| row.get(0))
        .unwrap();
    assert_eq!(sessions, 0);
    assert_eq!(teams, 0, "the cascade removed the session's teams");

    // Idempotent: deleting again is fine.
    delete_session(&conn, "s1").expect("a missing session is success");
}

fn session_with(conn: &rusqlite::Connection, status: &str, current: i32, count: i32) {
    crate::db::insert_test_session(conn, "s1", 1);
    conn.execute(
        "UPDATE game_session SET status = ?1, current_round = ?2, round_count = ?3 WHERE id = 's1'",
        rusqlite::params![status, current, count],
    )
    .unwrap();
}

#[test]
fn finishing_the_last_round_ends_the_game() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    crate::db::apply_schema(&conn).unwrap();
    session_with(&conn, "running", 3, 3);

    assert!(finish_if_last_round(&conn, "s1").expect("finished"));

    let (status, reveal, active, message): (String, i64, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT status, reveal_values, active_team_id, last_message
             FROM game_session WHERE id = 's1'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    assert_eq!(status, "finished");
    assert_eq!(reveal, 1, "values are revealed at game end");
    assert_eq!(active, None, "no team is on turn");
    assert_eq!(message.as_deref(), Some("Game over"));
}

#[test]
fn finishing_a_non_last_round_changes_nothing() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    crate::db::apply_schema(&conn).unwrap();
    session_with(&conn, "running", 1, 3);

    assert!(!finish_if_last_round(&conn, "s1").expect("no-op"));

    let status: String = conn
        .query_row(
            "SELECT status FROM game_session WHERE id = 's1'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "running");
}

#[test]
fn finishing_an_already_finished_game_is_a_noop() {
    let conn = rusqlite::Connection::open_in_memory().unwrap();
    crate::db::apply_schema(&conn).unwrap();
    session_with(&conn, "finished", 3, 3);

    assert!(!finish_if_last_round(&conn, "s1").expect("no-op"));
}
