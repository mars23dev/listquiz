---

description: "Task list for Block Invalid Session Setup"
---

# Tasks: Block Invalid Session Setup

**Input**: Design documents from `/specs/024-block-invalid-session-setup/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/setup-capacity.md

**Tests**: Included because the project constitution requires every business rule to have a directly
runnable test (Principle II) and feature-local tests (Principle I).

**Organization**: Tasks are grouped by user story so each story can be implemented and tested
independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Exact file paths are included in every task

## Path Conventions

- **Web app**: `backend/src/`, `backend/tests/`, `frontend/src/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the new rule module and its test file so later phases have a home.

- [X] T001 Create `backend/src/modules/setup_capacity/mod.rs` with module documentation and an empty `#[cfg(test)] mod tests;` declaration, create `backend/src/modules/setup_capacity/tests.rs`, and register `pub mod setup_capacity;` in `backend/src/modules/mod.rs`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Named capacity rule and DB count helper required by every story.

**CRITICAL**: No user story work can begin until this phase is complete.

- [X] T002 [P] Write failing unit tests in `backend/src/modules/setup_capacity/tests.rs` for `required_valued_items`: Order returns 11, Connect returns 10, Tripmine with `None` or `Some(10)` returns 10, Tripmine with `Some(20)` returns 20, and unsupported Tripmine counts are refused
- [X] T003 Implement `required_valued_items(quiz_type, tripmine_slots) -> ApiResult<usize>` and `validate_capacity(position, quiz_type, tripmine_slots, available) -> ApiResult<()>` in `backend/src/modules/setup_capacity/mod.rs`, including a named Tripmine 20 constant beside the existing mode constants; refusal uses `400 bad_request` and names the round, mode, required count, and available count; exact minimum is valid
- [X] T004 Implement `valued_item_count(conn, list_id, column_id) -> ApiResult<usize>` and `validate_round_capacity(conn, round) -> ApiResult<()>` in `backend/src/modules/setup_capacity/mod.rs`, counting only items with an `item_value` row for the selected column
- [X] T005 [P] Add module tests in `backend/src/modules/setup_capacity/tests.rs` proving the pure rule accepts the exact minimum, rejects one below, and produces a message containing the round, mode, required count, and available count

**Checkpoint**: Capacity rule exists, is unit tested, and can be called from any creation path.

---

## Phase 3: User Story 1 - Prevent creating a session from an undersized list (Priority: P1) - MVP

**Goal**: Standard session creation refuses any round whose selected column lacks the required valued
items, before any row is written.

**Independent Test**: Submit one round of each mode at one item below the minimum and at the exact minimum;
confirm the short request gets `400 bad_request`, no session row is created, and the exact-minimum request
gets `201`.

### Tests for User Story 1

- [X] T006 [P] [US1] Add a helper in `backend/tests/session_create.rs` that plants a list with a chosen number of items and one value per item in a chosen column
- [X] T007 [US1] Add integration tests in `backend/tests/session_create.rs`: Order with 10 valued items refused and 11 accepted, Connect with 9 refused and 10 accepted, Tripmine 10 with 9 refused and 10 accepted, Tripmine 20 with 19 refused and 20 accepted
- [X] T008 [US1] Add an integration test in `backend/tests/session_create.rs` proving a refused creation writes no `game_session`, `team`, or `round_config` row

### Implementation for User Story 1

- [X] T009 [US1] Call `setup_capacity::validate_round_capacity` for every round in `backend/src/modules/sessions/mod.rs` after `validate_round_references` and `validate_round_folder_access`, before opening the insert transaction
- [X] T010 [US1] Verify the refusal path in `backend/src/modules/sessions/mod.rs` preserves the single-transaction design so one short round prevents all writes

**Checkpoint**: User Story 1 works independently; no standard setup can create an undersized session.

---

## Phase 4: User Story 2 - Reject invalid setup through any entry point (Priority: P1)

**Goal**: Direct requests, multi-round requests, and quick games all use the same capacity gate; quick game
also picks a default list by valued-item count.

**Independent Test**: Submit a multi-round request where only the last round is short; confirm no earlier
round is stored. Start each quick game mode; confirm it either starts on a fitting default list or is
refused before a session exists.

### Tests for User Story 2

- [X] T011 [P] [US2] Add an integration test in `backend/tests/session_create.rs` proving a three-round request with only round 3 short is refused entirely and no round 1 or 2 row is written
- [X] T012 [P] [US2] Add quick-game tests in `backend/tests/quick_session.rs` for Order, Tripmine, and Connect that assert the chosen default list has enough valued items and the session starts
- [X] T013 [US2] Add a quick-game test in `backend/tests/quick_session.rs` proving no session is created when no default list can satisfy the mode

### Implementation for User Story 2

- [X] T014 [US2] Update `required_items` filtering in `backend/src/modules/quick_game/mod.rs` to count only values in the chosen first column, so an undersized default list is not selected while a fitting list exists
- [X] T015 [US2] Confirm `backend/src/routes/sessions.rs` quick-session path still reuses `sessions::create_session` so the shared gate applies without a second validation path

**Checkpoint**: User Stories 1 and 2 both pass; every creation path is gated.

---

## Phase 5: User Story 3 - Hide unsupported lists and flag unsupported sizes (Priority: P2)

**Goal**: Setup hides lists that cannot support the chosen mode's smallest size, keeps a list that becomes
incompatible selected with a shortfall error, and blocks creation until corrected.

**Independent Test**: With a 10-item list and Order selected, the list is absent. With a 15-item list and
Tripmine selected, the list is present, selecting 20 shows the shortfall, selecting 10 allows creation.

### Tests for User Story 3

- [X] T016 [P] [US3] Update list fixtures in `frontend/src/test/SetupQuiz.test.tsx` to include `columns[].valueCount`
- [X] T017 [P] [US3] Add frontend tests in `frontend/src/test/SetupQuiz.test.tsx`: a list below the mode minimum is absent from the picker; a 10-only Tripmine list stays visible; selecting 20 shows the shortfall and blocks creation; correcting the size clears the error
- [X] T018 [US3] Add frontend tests in `frontend/src/test/SetupQuiz.test.tsx`: changing mode, size, or column to make the current list incompatible keeps it selected with an error; no compatible list shows an empty state; the error uses the existing alert region

### Implementation for User Story 3

- [X] T019 [P] [US3] Add `valueCount?: number` to `ListColumnSummary` in `frontend/src/types/session.ts` with the rule "items with a value in that column; `0..=itemCount`"; the backend always returns it, while editor-only drafts may omit it
- [X] T020 [P] [US3] Add `value_count: i64` to `ColumnSummary` in `backend/src/modules/default_lists/mod.rs` and compute it in `columns_of` by counting `item` rows joined to `item_value` for that column; update any affected expectations in `backend/src/modules/default_lists/tests.rs` and `backend/tests/list_crud.rs`
- [X] T021 [US3] Add named helpers in `frontend/src/pages/SetupQuiz.tsx` for required counts (Order 11, Connect 10, Tripmine 10/20), smallest-size eligibility (Order 11, Connect 10, Tripmine 10), current-column eligibility, and the shortfall message
- [X] T022 [US3] Update the list picker in `frontend/src/pages/SetupQuiz.tsx` to offer only eligible lists, keep the currently selected list visible when it becomes incompatible, and show an empty-state explanation when no list qualifies
- [X] T023 [US3] Update mode, item-count, and column change handlers in `frontend/src/pages/SetupQuiz.tsx` to retain an incompatible selection, show the shortfall error, and clear the error when the selection becomes valid again
- [X] T024 [US3] Extend `validateStepTwo` in `frontend/src/pages/SetupQuiz.tsx` to check every round's selected column `valueCount` against the required count and block creation with the round-specific message
- [X] T025 [US3] Keep the existing keyboard-operable native selects and `role="alert"` error region in `frontend/src/pages/SetupQuiz.tsx`; confirm no new control traps focus or hides the error

**Checkpoint**: All three stories work; the original Tripmine 20 bug is blocked in both UI and backend.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Verify the complete feature and keep the contract and tests consistent.

- [X] T026 [P] Run `cargo test --manifest-path backend/Cargo.toml` and fix any failing test in `backend/src/modules/setup_capacity/`, `backend/src/modules/sessions/mod.rs`, `backend/tests/session_create.rs`, or `backend/tests/quick_session.rs`
- [X] T027 [P] Run `npm --prefix frontend run test` and fix any failing test in `frontend/src/test/SetupQuiz.test.tsx`
- [X] T028 Run `npm --prefix frontend run lint` and `npm --prefix frontend run build`, fixing type or build errors in `frontend/src/pages/SetupQuiz.tsx` and `frontend/src/types/session.ts`
- [X] T029 Verify the scenarios in `specs/024-block-invalid-session-setup/quickstart.md` through automated coverage and source inspection, including the 15-item Tripmine 20 refusal in `backend/tests/session_create.rs` and `frontend/src/test/SetupQuiz.test.tsx` and the 10-item Order hidden-list case; manual app launch skipped per the repository no-executable rule
- [X] T030 Review `specs/024-block-invalid-session-setup/contracts/setup-capacity.md` against the implemented JSON field, status code, message fields, and UI guarantees; correct the contract or code if they diverge

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies; starts immediately.
- **Foundational (Phase 2)**: Depends on T001; blocks all user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational.
- **User Story 2 (Phase 4)**: Depends on Foundational; uses the same gate but has its own quick-game work.
- **User Story 3 (Phase 5)**: Depends on Foundational; `valueCount` backend field (T020) is needed for real UI data.
- **Polish (Phase 6)**: Depends on all desired stories.

### User Story Dependencies

- **US1 (P1)**: No dependency on US2 or US3.
- **US2 (P1)**: No dependency on US1 implementation beyond the shared Foundational rule; independently testable through HTTP.
- **US3 (P2)**: Independent frontend behavior with mocked `valueCount`; end-to-end needs T020 and the backend gate from US1.

### Within Each User Story

- Tests before implementation.
- Rule implementation before call sites.
- Backend field before UI consumption.
- UI behavior before final manual validation.

### Parallel Opportunities

- T002 and T005 test work can start alongside T003/T004 implementation once signatures are agreed.
- After Phase 2, US1 backend tests (T006-T008) can run parallel with US3 fixtures (T016) and US3 type/backend field work (T019-T020), because they touch different files.
- T011 and T012 are parallel backend test files.
- T019 and T020 are parallel frontend/backend contract work.
- T026 and T027 are parallel test suites.
- T028 is a separate type/build pass.

---

## Parallel Example: User Story 1

```text
Task: "Add list-with-values helper in backend/tests/session_create.rs"
Task: "Add minimum-boundary integration tests in backend/tests/session_create.rs"
Task: "Add no-partial-write test in backend/tests/session_create.rs"
```

## Parallel Example: User Story 3

```text
Task: "Add valueCount to frontend/src/types/session.ts"
Task: "Add value_count to backend/src/modules/default_lists/mod.rs"
Task: "Update list fixtures in frontend/src/test/SetupQuiz.test.tsx"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 and Phase 2.
2. Complete Phase 3 (US1) and run `cargo test`.
3. **STOP and VALIDATE**: Standard setup now refuses undersized lists before creating a session.

### Incremental Delivery

1. Foundation: shared capacity rule + unit tests.
2. US1: standard creation gate and no-partial-write guarantee.
3. US2: quick game and multi-round coverage.
4. US3: setup page hides impossible lists and explains retained invalid selections.
5. Polish: full test suite, lint, build, and quickstart scenarios.

### Parallel Team Strategy

1. Team completes Phase 1 and Phase 2 together.
2. Developer A takes US1 backend gate and tests.
3. Developer B takes US2 quick-game eligibility and tests.
4. Developer C takes US3 list summaries, types, and setup UI.
5. Join for Phase 6 verification.

---

## Notes

- [P] tasks touch different files and have no incomplete dependency.
- [US1] [US2] [US3] labels map to spec.md user stories.
- Backend refusal is authoritative; the UI mirrors it for usability.
- Do not change the existing minimums: Order 11, Connect 10, Tripmine 10/20.
- Do not add a schema migration, new endpoint, or dependency.
- Verify each story at its checkpoint before moving on.
