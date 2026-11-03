---
description: "Task list for per-game-ordering"
---

# Tasks: Per-Game Ordering

**Input**: Design documents from `/specs/019-per-game-ordering/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included. The constitution (Principle II) requires every business rule to be covered by a test. Write the tests first and confirm they fail before the implementation change.

**Organization**: One user story (P1); tasks are grouped by phase.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1)
- Include exact file paths in descriptions

## Path Conventions

- **Web app**: `backend/src/`, `frontend/src/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm both toolchains run.

- [X] T001 Run the baseline suites: `cargo test --manifest-path backend/Cargo.toml` and `npm --prefix frontend run test`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST exist before the story.

**No foundational tasks.** The list modules, the game modes, and the Order round all exist. Proceed to the user story.

**Checkpoint**: Foundation ready.

---

## Phase 3: User Story 1 - Lists no longer carry a direction; each game mode orders (Priority: P1) 🎯 MVP

**Goal**: Remove the list-level order direction; each game mode defines its own ordering (Order/Tripmine/Connect ascending).

**Independent Test**: `cargo test` creates/imports a list with no direction and shows the per-mode sort order ascending; `npm test` shows no direction selector.

### Tests for User Story 1

> **NOTE: Write the tests FIRST, confirm they FAIL, then implement T003–T012.**

- [X] T002 [P] [US1] Update backend tests in `backend/src/modules/lists/tests.rs`, `backend/src/modules/csv_import/tests.rs`, `backend/src/modules/default_lists/tests.rs`, and `backend/src/modules/sessions/round_state/tests.rs`: create/import a list with no `order_direction`; assert `QuizType::sort_order()` is ascending for Order, Tripmine, and Connect; assert the Order round orders the column ascending. Confirm they fail before T003–T009.
- [X] T003 [P] [US1] Update frontend tests in `frontend/src/test/ListManagement.test.tsx`, `frontend/src/test/SetupQuiz.test.tsx`, `frontend/src/test/client.test.ts`, and `frontend/src/test/App.test.tsx` to drop `orderDirection` and assert no direction selector is rendered. Confirm they fail before T010–T011.

### Implementation for User Story 1

- [X] T004 [US1] In `backend/src/db.rs`, remove `order_direction` from the `quiz_list` `CREATE TABLE`, and add a guarded migration `ALTER TABLE quiz_list DROP COLUMN order_direction` (called from `apply_schema`, guarded by `column_exists`).
- [X] T005 [US1] In `backend/src/modules/default_lists/mod.rs`, remove `order_direction` from `DefaultList` and from the four shipped list definitions and the `seed_default_lists` insert.
- [X] T006 [US1] In `backend/src/modules/csv_import/mod.rs`, remove `order_direction` from `ParsedList` and the `parse_csv` parameter; update its body and `backend/src/routes/lists.rs` import request accordingly.
- [X] T007 [US1] In `backend/src/modules/lists/mod.rs`, remove `order_direction` from `ListInput`, `NewList`, and `ListDetail`, the validation match, and the `create`/`update`/import inserts and selects.
- [X] T008 [US1] In `backend/src/modules/sessions/mod.rs`, add `pub enum SortOrder { Ascending, Descending }` and `impl QuizType { pub fn sort_order(self) -> SortOrder }` returning `Ascending` for Order, Tripmine, and Connect.
- [X] T009 [US1] In `backend/src/modules/sessions/round_state/mod.rs`, remove `low_to_high` from `RoundToStart` and the `order_direction` select in `load_round`; order the round column using the game mode's `sort_order()` (Order ascending), leaving Tripmine/Connect ascending.
- [X] T010 [US1] In `frontend/src/pages/ListManagement.tsx`, remove the higher/lower direction selector from the create form and the list editor, and stop sending it.
- [X] T011 [US1] In `frontend/src/api/client.ts`, `frontend/src/types/quiz.ts`, and `frontend/src/types/session.ts`, remove `orderDirection` from the list payloads/types.
- [X] T012 [US1] Run `cargo test --manifest-path backend/Cargo.toml` and `npm --prefix frontend run test`; confirm T002/T003 pass and the rest stays green.

**Checkpoint**: No list direction anywhere; each game mode orders (all ascending).

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Full regression and manual validation.

- [X] T013 [P] Run `cargo fmt --manifest-path backend/Cargo.toml --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, and `cargo test --manifest-path backend/Cargo.toml`.
- [X] T014 [P] Run `npm --prefix frontend run test` and `npm --prefix frontend run build`.
- [X] T015 Run the quickstart checks in `specs/019-per-game-ordering/quickstart.md`: no direction in create/import/edit; Order/Tripmine/Connect order ascending; an existing database upgrades cleanly.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: None to build.
- **User Story 1 (Phase 3)**: Depends on Setup.
- **Polish (Phase 4)**: Depends on US1.

### Within User Story 1

- Backend tests before backend implementation (T002 before T004–T009); frontend tests before frontend implementation (T003 before T010–T011).
- The schema/migration (T004) before the modules that insert (T005–T007).
- Verification (T012) last.

### Parallel Opportunities

- T002 (backend tests) and T003 (frontend tests) are different files and can be written in parallel.
- T005–T008 touch different backend modules and can proceed in parallel once T004 lands.
- T010 and T011 touch different frontend files and can proceed in parallel.
- T013 and T014 touch different toolchains and can run in parallel.

---

## Parallel Example: Kick-off

```bash
# Backend and frontend tests, in parallel:
Task: "No-direction backend tests in backend/src/modules/lists/tests.rs (+ csv/default/round_state) (T002)"
Task: "No-direction frontend tests in frontend/src/test/ListManagement.test.tsx (+ SetupQuiz/client/App) (T003)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001 (baseline).
2. Complete T002/T003 (failing tests) → T004–T011 (remove the field; per-mode rule) → T012 (verify).
3. **STOP and VALIDATE**: no direction anywhere; per-mode ordering. Deploy/demo.

### Incremental Delivery

1. Backend removal + per-mode rule.
2. Frontend removal.
3. Polish → full suites, fmt/clippy, build, quickstart.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps each task to its user story for traceability.
- One guarded migration drops the column; no new dependency.
- Scoring, hearts, the Tripmine side, and matching are unchanged.
- Verify the tests fail before implementing.
