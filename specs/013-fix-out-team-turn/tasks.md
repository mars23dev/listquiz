---
description: "Task list for fix-out-team-turn"
---

# Tasks: Fix Eliminated Team Turn Switch

**Input**: Design documents from `/specs/013-fix-out-team-turn/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included. The constitution (Principle II) requires every business rule to be covered by a test, and the plan makes the rule testable. Write the tests first and confirm they fail before the implementation change.

**Organization**: One user story (P1); tasks are grouped by phase.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1)
- Include exact file paths in descriptions

## Path Conventions

- **Web app**: `backend/src/`, `backend/tests/`, `frontend/src/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm both toolchains run before changing anything.

- [X] T001 Run the baseline suites: `cargo test --manifest-path backend/Cargo.toml` and `npm --prefix frontend run test -- Session`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST exist before the story.

**No foundational tasks.** The `team` hearts column, the `round_config.quiz_type` lookup (used by `sessions::apply_turn_timeout`), the `set_turn` action, and the gamemaster turn control all exist. Proceed to the user story.

**Checkpoint**: Foundation ready (nothing to build).

---

## Phase 3: User Story 1 - An eliminated team cannot be put back on turn (Priority: P1) 🎯 MVP

**Goal**: In Order and Connect, a team with zero hearts cannot be put on turn by the gamemaster's turn switch (backend refuses, UI marks it unselectable); Tripmine is unaffected; a new round re-enables the team.

**Independent Test**: Drive a team to zero hearts, attempt a `set_turn` to it, and confirm it is refused and the active team is unchanged; confirm a switch to a team with hearts still succeeds and a Tripmine switch is unrestricted.

### Tests for User Story 1

> **NOTE: Write the backend test FIRST, confirm it FAILS, then implement T003. Write the frontend test FIRST, confirm it FAILS, then implement T006.**

- [X] T002 [US1] Add an integration test in a new `backend/tests/turn_switch.rs` (start with `mod common;`, use `common::seeded_db`, `common::test_app`, `common::call`, `common::gm_action_request`, `common::json_request`, `common::valid_session_body`): start an Order round, drive `team_2` to zero hearts with an `adjust_hearts` action (`delta: -3`), then assert a `set_turn` to `team_2` is refused (non-OK status) and the state's active team is unchanged; assert a `set_turn` to `team_1` succeeds; start a Tripmine round, drive a team to zero hearts, and assert a `set_turn` to it succeeds. Run `cargo test --manifest-path backend/Cargo.toml turn_switch` and confirm the Order refusal case fails before T003.
- [X] T005 [US1] Add a test in `frontend/src/test/Session.test.tsx`: in a heart-using round (Order/Connect), the gamemaster turn `<select>` renders the zero-heart team's `<option>` as `disabled` while a team with hearts is enabled; confirm it fails before T006.

### Implementation for User Story 1

- [X] T003 [US1] In `backend/src/routes/sessions.rs`, change `apply_set_turn`: after the team-exists check, read the current round's `quiz_type` (same query as `sessions::apply_turn_timeout`: `SELECT quiz_type FROM round_config WHERE session_id = ?1 AND round_number = (SELECT current_round FROM game_session WHERE id = ?1)`); when it is `ORDER` or `CONNECT`, load teams with `order_round::load_teams` and return an `ApiError` (e.g. `not_ready`, message that the team is out of hearts) if the target team's `hearts <= scoring::MIN_HEARTS`; otherwise update `active_team_id` as today. Tripmine (and any non-heart round) is not restricted.
- [X] T004 [US1] Run `cargo test --manifest-path backend/Cargo.toml turn_switch` and confirm T002 passes; run the full `cargo test --manifest-path backend/Cargo.toml` and confirm existing suites stay green.
- [X] T006 [US1] In `frontend/src/pages/Session.tsx`, in the gamemaster turn `<select>` option map, add `disabled={showHearts && team.hearts <= 0}` to the `<option>` so an out team cannot be selected in Order/Connect; keep the control a native `<select>` and leave Tripmine (no hearts) unrestricted.
- [X] T007 [US1] Run `npm --prefix frontend run test -- Session` and confirm T005 passes and the rest of the suite stays green.

**Checkpoint**: An eliminated team cannot be put on turn (backend refuses, UI disables); Tripmine and teams with hearts are unaffected; a new round re-enables the team.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Full regression and manual validation.

- [X] T008 [P] Run `cargo fmt --manifest-path backend/Cargo.toml --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, and `cargo test --manifest-path backend/Cargo.toml`.
- [X] T009 [P] Run `npm --prefix frontend run test` and `npm --prefix frontend run build`.
- [X] T010 Run the quickstart checks in `specs/013-fix-out-team-turn/quickstart.md`: automated primary checks pass (backend turn-switch refusal tests, `Session` disabled-option test, full suites, fmt/clippy, build); the browser walkthrough is left as an optional manual confirmation for the user.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: None to build.
- **User Story 1 (Phase 3)**: Depends on Setup.
- **Polish (Phase 5)**: Depends on US1.

### Within User Story 1

- Backend test before backend implementation (T002 before T003); T004 verifies.
- Frontend test before frontend implementation (T005 before T006); T007 verifies.
- The backend and frontend halves are independent (different files) and may be worked in parallel by two people.

### Parallel Opportunities

- T002 (backend test) and T005 (frontend test) are different files and may be written in parallel.
- T008 and T009 touch different toolchains and can run in parallel after US1.
- T005 must not run in parallel with T006 (test before implementation) nor with T007.

---

## Parallel Example: Two Developers

```bash
# Developer A (backend):
Task: "Add the turn-switch refusal test in backend/tests/turn_switch.rs (T002)"
Task: "Refuse a zero-heart team in apply_set_turn in backend/src/routes/sessions.rs (T003)"

# Developer B (frontend):
Task: "Add the disabled-option test in frontend/src/test/Session.test.tsx (T005)"
Task: "Disable the out team's option in frontend/src/pages/Session.tsx (T006)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001 (baseline).
2. Backend: T002 (failing test) → T003 (refuse) → T004 (verify).
3. Frontend: T005 (failing test) → T006 (disable) → T007 (verify).
4. **STOP and VALIDATE**: an eliminated team cannot be put on turn. Deploy/demo.

### Incremental Delivery

1. US1 backend (authority) → US1 frontend (mirror).
2. Polish → full suites, fmt/clippy, build, quickstart.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps each task to its user story for traceability.
- One guard in one handler and one disabled option carry the change; keep it minimal (Principle VII).
- Verify the backend and frontend tests fail before their implementation tasks.
- No schema, dependency, or scoring changes.
