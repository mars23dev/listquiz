// ListQuiz backend library.
//
// The binary in `src/main.rs` is a thin startup shell; everything testable lives
// here so `backend/tests/` integration tests can reach it. Principle II: business
// rules are named units in `modules/`, not inline in HTTP handlers.
pub mod app_state;
pub mod db;
pub mod errors;
pub mod modules;
pub mod paths;
pub mod routes;
pub mod state_reader;
