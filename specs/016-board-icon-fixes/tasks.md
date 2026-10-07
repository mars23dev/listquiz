---
description: "Task list for board-icon-fixes"
---

# Tasks: Board Icon And Layout Fixes

**Input**: Design documents from `/specs/016-board-icon-fixes/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included. The constitution (Principle II) requires every business rule to be covered by a test. Write the tests first and confirm they fail before the implementation change.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1–US3)
- Include exact file paths in descriptions

## Path Conventions

- **Web app**: `backend/src/`, `backend/tests/`, `frontend/src/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm both toolchains run before changing anything.

- [X] T001 Run the baseline suites: `cargo test --manifest-path backend/Cargo.toml` and `npm --prefix frontend run test`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST exist before the stories.

**No foundational tasks.** The `item.icon_name` column, the boards' `ItemIcon` calls, and the item DTOs already exist. Proceed to the user stories.

**Checkpoint**: Foundation ready (nothing to build).

---

## Phase 3: User Story 1 - Every game type shows each item's own icon (Priority: P1) 🎯 MVP

**Goal**: Order and Tripmine carry the list item's icon (flags included), so every game type renders the item's own icon.

**Independent Test**: An Order item view and a Tripmine item view for a Country report `iconName = "flag:<cc>"`; the boards render it.

### Tests for User Story 1

> **NOTE: Write the test FIRST, confirm it FAILS, then implement T003/T004.**

- [X] T002 [US1] Add a backend test (in the `backend/src/state_reader.rs` test module or `backend/tests/`) asserting that an Order item view and a Tripmine item view each carry the item's `iconName` (e.g. a Countries item reports `flag:<cc>`). Run `cargo test --manifest-path backend/Cargo.toml` and confirm it fails before T003.

### Implementation for User Story 1

- [X] T003 [US1] In `backend/src/modules/sessions/round_state/mod.rs`, add `icon_name: Option<String>` to `ListItemValue` and select `i.icon_name` in `load_ordered_items`. In `backend/src/app_state.rs`, add `icon_name: Option<String>` (serde camelCase) to `OrderItem`, `TripmineItem`, and `AnchorItem`. In `backend/src/state_reader.rs`, populate `icon_name` in `read_order_items`, `to_order_items`, the anchor, and the Tripmine item construction (query `i.icon_name` where the name/value are read).
- [X] T004 [US1] In `frontend/src/types/session.ts`, add `iconName: string | null` to `OrderItem` and `TripmineItem`; in `frontend/src/pages/Session.tsx`, map `iconName: item.iconName` in `toQuizItem` and `toTripmineItem` so `QuizItem`/`QuizTripmineItem` carry it to the boards.
- [X] T005 [US1] Run `cargo test --manifest-path backend/Cargo.toml` and `npm --prefix frontend run test`; confirm T002 passes and existing suites stay green (update any test that asserts an item view's exact fields if needed).

**Checkpoint**: Order and Tripmine show the list item's own icon; countries show flags in all three game types.

---

## Phase 4: User Story 2 - The Tripmine bar matches the item count (Priority: P2)

**Goal**: The Tripmine rank bar shows one position per item (10 or 20).

**Independent Test**: A 20-item state renders a 20-position bar; a 10-item state renders 10.

### Tests for User Story 2

- [X] T006 [P] [US2] Add a test in `frontend/src/components/TripmineBoard.test.tsx` asserting a 20-item state renders bar positions up to `#20` (and no `#21`), and a 10-item state renders up to `#10`. Confirm it fails before T007.

### Implementation for User Story 2

- [X] T007 [US2] In `frontend/src/components/TripmineBoard.tsx`, build the rank ladder from the item count (`Array.from({ length: items.length }, (_, i) => i + 1)`) instead of the hardcoded `[1..10]`, keeping the mine/search markers working within the chosen count.
- [X] T008 [US2] Run `npm --prefix frontend run test -- TripmineBoard` and confirm T006 passes and the rest stays green.

**Checkpoint**: The bar always matches the round's item count.

---

## Phase 5: User Story 3 - Connect columns are the same size and aligned (Priority: P2)

**Goal**: Connect name and value rows are the same fixed height and align row-for-row.

**Independent Test**: A Connect board's name row and value row carry the same fixed-height class.

### Tests for User Story 3

- [X] T009 [P] [US3] Update the equal-size test in `frontend/src/components/ConnectBoard.test.tsx` to assert the name row and the value row carry the same fixed-height class (e.g. `h-12`) rather than only a minimum. Confirm it fails before T010.

### Implementation for User Story 3

- [X] T010 [US3] In `frontend/src/components/ConnectBoard.tsx`, replace the shared `min-h-[44px]` with one fixed height (e.g. `h-12`) on both the name cell and the value cell, keep content vertically centered, and constrain the icon so it cannot make the name row taller.
- [X] T011 [US3] Run `npm --prefix frontend run test -- ConnectBoard` and confirm T009 passes and the rest stays green.

**Checkpoint**: Connect columns are equal and aligned.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Full regression and manual validation.

- [X] T012 [P] Run `cargo fmt --manifest-path backend/Cargo.toml --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, and `cargo test --manifest-path backend/Cargo.toml`.
- [X] T013 [P] Run `npm --prefix frontend run test` and `npm --prefix frontend run build`.
- [X] T014 Run the quickstart checks in `specs/016-board-icon-fixes/quickstart.md`: flags in all three game types, icons on other lists, 20/10 Tripmine bar, equal Connect rows.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: None to build.
- **US1 (Phase 3)**: Backend DTOs/mappers + frontend types/mappers.
- **US2 (Phase 4)**: Frontend `TripmineBoard` only.
- **US3 (Phase 5)**: Frontend `ConnectBoard` only.
- **Polish (Phase 6)**: Depends on all stories.

### User Story Dependencies

- US1, US2, US3 are independent (different files) and may run in parallel.

### Within Each User Story

- Tests written and failing before implementation (T002 before T003/T004; T006 before T007; T009 before T010).
- Implementation before verification (T005, T008, T011).

### Parallel Opportunities

- T002 (backend test), T006 (TripmineBoard test), T009 (ConnectBoard test) are different files and can be written in parallel.
- T012 and T013 touch different toolchains and can run in parallel.

---

## Parallel Example: Kick-off

```bash
# Three tests in three different files, in parallel:
Task: "Icon-carried test in backend state_reader (T002)"
Task: "Bar-count test in frontend/src/components/TripmineBoard.test.tsx (T006)"
Task: "Equal-row test in frontend/src/components/ConnectBoard.test.tsx (T009)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001 (baseline).
2. Complete T002 (failing test) → T003/T004 (carry the icon) → T005 (verify).
3. **STOP and VALIDATE**: every game type shows the item's icon. Deploy/demo.

### Incremental Delivery

1. US1 → icons in every game type (MVP).
2. US2 → Tripmine bar matches the count.
3. US3 → Connect rows equal.
4. Polish → full suites, fmt/clippy, build, quickstart.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps each task to its user story for traceability.
- The icon is data; no game type chooses an icon (Principle VII).
- The Countries flags are never changed.
- Verify each story's tests fail before implementing.
