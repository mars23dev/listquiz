---
description: "Task list for value-column-labels"
---

# Tasks: Value Column Labels

**Input**: Design documents from `/specs/017-value-column-labels/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included. The constitution (Principle II) requires every business rule to be covered by a test. Write the tests first and confirm they fail before the implementation change.

**Organization**: One user story (P1); tasks are grouped by phase.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1)
- Include exact file paths in descriptions

## Path Conventions

- **Web app**: `frontend/src/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm the toolchain runs before changing anything.

- [X] T001 Run the baseline suite: `npm --prefix frontend run test`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST exist before the story.

**No foundational tasks.** `state.round.columnName` is already served and the boards already render values. Proceed to the user story.

**Checkpoint**: Foundation ready (nothing to build).

---

## Phase 3: User Story 1 - Values show their column name in every game mode (Priority: P1) 🎯 MVP

**Goal**: Wherever a value is shown (Order, Tripmine, Connect), the round's column name is shown with it, without changing value visibility.

**Independent Test**: `npm test` shows each board rendering the column name where values are shown, and no label where a value is hidden.

### Tests for User Story 1

> **NOTE: Write the tests FIRST, confirm they FAIL, then implement T005–T007.**

- [X] T002 [P] [US1] Add a test in `frontend/src/components/ConnectBoard.test.tsx`: rendering with a `columnName` prop shows that name as the value-column heading. Confirm it fails before T005.
- [X] T003 [P] [US1] Add a test in `frontend/src/components/TopBarOrdering.test.tsx`: rendering with a `columnName` prop shows the name beside the displayed (anchor) value. Confirm it fails before T006.
- [X] T004 [P] [US1] Add a test in `frontend/src/components/TripmineBoard.test.tsx`: rendering with a `columnName` prop shows the name with the displayed values, and a board with no visible value shows no label. Confirm it fails before T007.

### Implementation for User Story 1

- [X] T005 [US1] In `frontend/src/components/ConnectBoard.tsx`, add an optional `columnName?: string` prop and render it as a heading above the value column (only when present). Do not change value visibility.
- [X] T006 [US1] In `frontend/src/components/TopBarOrdering.tsx`, add an optional `columnName?: string` prop and render it as a small label beside the values the board displays (the anchor and any revealed values).
- [X] T007 [US1] In `frontend/src/components/TripmineBoard.tsx`, add an optional `columnName?: string` prop and render it as a small label beside the values the board displays (revealed values and the gamemaster's full view).
- [X] T008 [US1] In `frontend/src/pages/Session.tsx`, pass `columnName={state?.round?.columnName}` to `TopBarOrdering`, `TripmineBoard`, and `ConnectBoard` (leave `MiddleItemGrid` unchanged).
- [X] T009 [US1] Run `npm --prefix frontend run test` and confirm T002–T004 pass and the rest of the suite stays green.

**Checkpoint**: Every displayed value carries the round's column name; hidden values show no label.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Full regression and manual validation.

- [X] T010 [P] Run `cargo test --manifest-path backend/Cargo.toml` (regression; backend unchanged).
- [X] T011 [P] Run `npm --prefix frontend run test` and `npm --prefix frontend run build`.
- [X] T012 Run the quickstart checks in `specs/017-value-column-labels/quickstart.md`: the column name shows with values in Order, Tripmine, and Connect, and never with a hidden value.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: None to build.
- **User Story 1 (Phase 3)**: Depends on Setup.
- **Polish (Phase 4)**: Depends on US1.

### Within User Story 1

- Tests written and failing before implementation (T002–T004 before T005–T007).
- The page wiring (T008) after the board props exist (T005–T007).
- Verification (T009) last.

### Parallel Opportunities

- T002, T003, T004 are different files and can be written in parallel.
- T005, T006, T007 are different components and can be implemented in parallel.
- T010 and T011 touch different toolchains and can run in parallel.

---

## Parallel Example: Implementation

```bash
# Three components, in parallel:
Task: "Connect value-column heading in frontend/src/components/ConnectBoard.tsx (T005)"
Task: "Order value label in frontend/src/components/TopBarOrdering.tsx (T006)"
Task: "Tripmine value label in frontend/src/components/TripmineBoard.tsx (T007)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001 (baseline).
2. Complete T002–T004 (failing tests) → T005–T007 (labels) → T008 (wiring) → T009 (verify).
3. **STOP and VALIDATE**: every mode shows the column name with its values. Deploy/demo.

### Incremental Delivery

1. US1 → column labels everywhere values show.
2. Polish → full suites, build, quickstart.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps each task to its user story for traceability.
- Presentation only: value visibility, scoring, and rules are unchanged.
- The column name is already in the session state; no backend change.
- Verify each test fails before implementing.
