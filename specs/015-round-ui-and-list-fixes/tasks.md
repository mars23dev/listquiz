---
description: "Task list for round-ui-and-list-fixes"
---

# Tasks: Round UI And List Fixes

**Input**: Design documents from `/specs/015-round-ui-and-list-fixes/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included. The constitution (Principle II) requires every business rule to be covered by a test. Write the tests first and confirm they fail before the implementation change.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1–US4)
- Include exact file paths in descriptions

## Path Conventions

- **Web app**: `backend/src/`, `backend/tests/`, `frontend/src/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm both toolchains run before changing anything.

- [X] T001 Run the baseline suites: `cargo test --manifest-path backend/Cargo.toml default_lists` and `npm --prefix frontend run test`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST exist before the stories.

**No foundational tasks.** The seed module, the board components, the Tripmine round, and the header nav all exist. Proceed to the user stories.

**Checkpoint**: Foundation ready (nothing to build).

---

## Phase 3: User Story 1 - Shipped lists are ordered by value (Priority: P1) 🎯 MVP

**Goal**: Each shipped list reads ascending by its value column, with values stored as numbers and large numbers shown with magnitude suffixes.

**Independent Test**: `cargo test default_lists` shows each list non-decreasing by its ordering column and `format_value` matching the contract examples.

### Tests for User Story 1

> **NOTE: Write the test FIRST, confirm it FAILS, then implement T003/T004.**

- [X] T002 [US1] In `backend/src/modules/default_lists/tests.rs`, add: a `format_value` unit test matching the contract (`1440000000`→"1.44 Billion", `146000000`→"146 Million", `1969`→"1969", `48`→"48", `-2560`→"-2,560", `0.49`→"0.49"); and a per-list test that each shipped list's items are non-decreasing by its ordering column (History `year`, Statues `height`, Cinema `box_office`, Countries `area_km2`). Run `cargo test --manifest-path backend/Cargo.toml default_lists` and confirm it fails before T003.

### Implementation for User Story 1

- [X] T003 [US1] In `backend/src/modules/default_lists/mod.rs`, add `format_value(n: f64) -> String` (per the contract: `>=1e12` Trillion, `>=1e9` Billion, `>=1e6` Million, else thousands-separated, trailing zeros trimmed); reorder `HISTORY_ITEMS`, `STATUE_ITEMS`, `CINEMA_ITEMS`, `COUNTRIES_ITEMS` ascending by their ordering column; convert Cinema `box_office` numerics to real dollars (e.g. `1440.0` → `1440000000.0`); and set each shipped value's `display` from `format_value(numeric)`.
- [X] T004 [US1] In `backend/src/modules/default_lists/mod.rs` `seed_default_lists`, upsert `item` (`ON CONFLICT(id) DO UPDATE SET name, position, icon_name`) so re-seeding reorders and renames items on an existing database, and write the formatted display into `item_value`.
- [X] T005 [US1] Run `cargo test --manifest-path backend/Cargo.toml default_lists` and confirm T002 passes; update any existing tests that assert old displays (e.g. `$1.44 Billion`) or old item positions, and confirm the full `cargo test --manifest-path backend/Cargo.toml` is green.

**Checkpoint**: Shipped lists read ascending with numeric values and readable large-number displays.

---

## Phase 4: User Story 2 - Connect columns look equal (Priority: P2)

**Goal**: The Connect name and value rows are the same size, so both columns look equally full.

**Independent Test**: `npm test ConnectBoard` asserts a value row and a name row share the same sizing.

### Tests for User Story 2

- [X] T006 [P] [US2] Add a test in `frontend/src/components/ConnectBoard.test.tsx` asserting a name row and a value row share the same sizing (e.g. both carry the same min-height class, or their rendered heights are equal). Confirm it fails before T007.

### Implementation for User Story 2

- [X] T007 [US2] In `frontend/src/components/ConnectBoard.tsx`, give the name cell and the value cell the same min-height and padding (and vertical centering) so the two columns render equal rows.
- [X] T008 [US2] Run `npm --prefix frontend run test -- ConnectBoard` and confirm T006 passes and the rest of the suite is green.

**Checkpoint**: Both Connect columns look the same size.

---

## Phase 5: User Story 3 - Tripmine can play 10 or 20 items (Priority: P2)

**Goal**: A Tripmine round can be configured with 10 or 20 items; 20 fits one screen with no scrolling; 10 is unchanged.

**Independent Test**: `cargo test` shows a 20-slot round samples/ranks 20 items with the mine side applied; `npm test TripmineBoard` shows 20 items render.

### Tests for User Story 3

> **NOTE: Write the tests FIRST, confirm they FAIL, then implement.**

- [X] T009 [P] [US3] Add backend tests in `backend/src/modules/tripmine_round/tests.rs` and `backend/src/modules/sessions/tests.rs`: `sample_tripmine_grid` with `slots = 20` returns 20 slots ranked `1..=20` with the mine at rank 20 (highest), 1 (lowest), or a random rank in `1..=20`; a round input with `tripmineSlots` 20 validates and one with an invalid count (e.g. 15) is refused; absent means 10. Confirm they fail before T010–T012.
- [X] T014 [P] [US3] Add a test in `frontend/src/components/TripmineBoard.test.tsx` asserting a 20-item state renders 20 items. Confirm it fails before T015.

### Implementation for User Story 3

- [X] T010 [US3] In `backend/src/db.rs`, add an additive migration: `ALTER TABLE round_config ADD COLUMN tripmine_slots INTEGER` guarded like the existing timer migrations, and add the column to the `CREATE TABLE round_config` statement.
- [X] T011 [US3] In `backend/src/modules/sessions/mod.rs`, add `tripmine_slots: Option<i32>` to `RoundInput` (serde camelCase, `#[serde(default)]`) and `ValidRound`; in `validate_round` accept `10` or `20` only for a Tripmine round (absent → 10) and reject other counts with a `bad_request`; include `tripmine_slots` in the `round_config` insert.
- [X] T012 [US3] In `backend/src/modules/tripmine_round/mod.rs`, change `sample_tripmine_grid` to take a `slots: usize` argument, sample and rank `1..=slots`, and apply the mine side across `slots`; in `backend/src/routes/sessions.rs` pass the round's configured slot count (default 10) when starting the Tripmine grid.
- [X] T013 [US3] In `frontend/src/pages/SetupQuiz.tsx`, add a Tripmine option to choose 10 or 20 items (only for Tripmine rounds), sent as `tripmineSlots` in the round payload.
- [X] T015 [US3] In `frontend/src/components/TripmineBoard.tsx`, scale the grid/cells so 20 items fit one screen with no scrolling (denser columns and smaller cells for the 20-item case), keeping the 10-item layout unchanged.
- [X] T016 [US3] Run `cargo test --manifest-path backend/Cargo.toml` and `npm --prefix frontend run test -- TripmineBoard SetupQuiz`; confirm T009 and T014 pass and the rest stay green.

**Checkpoint**: Tripmine supports 10 or 20 items, 20 fitting one screen.

---

## Phase 6: User Story 4 - The header view switch is gamemaster-only (Priority: P2)

**Goal**: The header view switch shows only to the gamemaster; using it opens the view in a new tab and leaves the gamemaster page.

**Independent Test**: `npm test Session` shows the switch absent for `p1`/`view`, present for the gamemaster, and opening a new tab without navigating.

### Tests for User Story 4

- [X] T017 [P] [US4] Add a test in `frontend/src/test/Session.test.tsx`: as `p1` and as `view` no view switch is rendered; as `gamemaster` the switch is present, and activating a target calls `window.open` with the target's route and does not call `onNavigate`. Confirm it fails before T018.

### Implementation for User Story 4

- [X] T018 [US4] In `frontend/src/pages/Session.tsx`, render the header role switch only when `isGamemaster`; on activation call `window.open(\`/${sessionId}/${rolePath(target)}\`, '_blank')` instead of `onNavigate`, leaving the current page unchanged. Keep the unlock screen's own control as-is.
- [X] T019 [US4] Run `npm --prefix frontend run test -- Session` and confirm T017 passes and the rest stays green.

**Checkpoint**: Players/spectators see no switch; the gamemaster opens views in a new tab.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Full regression and manual validation.

- [X] T020 [P] Run `cargo fmt --manifest-path backend/Cargo.toml --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, and `cargo test --manifest-path backend/Cargo.toml`.
- [X] T021 [P] Run `npm --prefix frontend run test` and `npm --prefix frontend run build`.
- [X] T022 Run the quickstart checks in `specs/015-round-ui-and-list-fixes/quickstart.md`: ordered lists with magnitude displays, equal Connect columns, 10/20 Tripmine on one screen, gamemaster-only switch opening a new tab.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: None to build.
- **US1 (Phase 3)**: Backend seed only.
- **US2 (Phase 4)**: Frontend `ConnectBoard` only.
- **US3 (Phase 5)**: Backend config/sampling + frontend setup/board.
- **US4 (Phase 6)**: Frontend `Session` only.
- **Polish (Phase 7)**: Depends on all stories.

### User Story Dependencies

- US1, US2, US4 are independent (different files) and may run in parallel.
- US3 is independent of the others; within it, the sampling change (T012) must land before T016.

### Within Each User Story

- Tests written and failing before implementation (T002 before T003/T004; T006 before T007; T009/T014 before T010–T015; T017 before T018).
- Implementation before verification (T005, T008, T016, T019).

### Parallel Opportunities

- T002 (backend seed test), T006 (ConnectBoard test), T017 (Session test) are different files and can be written in parallel.
- T009 and T014 (different files) can be written in parallel.
- T020 and T021 touch different toolchains and can run in parallel.

---

## Parallel Example: Kick-off

```bash
# Three tests in three different files, in parallel:
Task: "Seed ordering + formatter test in backend/src/modules/default_lists/tests.rs (T002)"
Task: "Connect equal-size test in frontend/src/components/ConnectBoard.test.tsx (T006)"
Task: "View-switch test in frontend/src/test/Session.test.tsx (T017)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001 (baseline).
2. Complete T002 (failing test) → T003/T004 (order + format + upsert) → T005 (verify).
3. **STOP and VALIDATE**: shipped lists read ascending with magnitude displays. Deploy/demo.

### Incremental Delivery

1. US1 → ordered numeric lists (MVP).
2. US2 → equal Connect columns.
3. US3 → Tripmine 10/20.
4. US4 → gamemaster-only switch.
5. Polish → full suites, fmt/clippy, build, quickstart.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps each task to its user story for traceability.
- US3 is the largest; keep its backend (T010–T012) ahead of its UI verification (T016).
- Only one additive migration (`tripmine_slots`); no new dependency.
- Verify each story's tests fail before implementing.
