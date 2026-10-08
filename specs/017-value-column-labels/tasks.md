---
description: "Task list for value-column-labels"
---

# Tasks: Value Column Labels

**Input**: Design documents from `/specs/017-value-column-labels/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included. The constitution (Principle II) requires every business rule to be covered by a test. Write the tests first and confirm they fail before the implementation change.

**Organization**: One user story (P1). The label work (T002–T006) is already implemented and committed; the Order placement change (T007+) is the remaining work.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1)
- Include exact file paths in descriptions

## Path Conventions

- **Web app**: `frontend/src/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm the toolchain runs.

- [X] T001 Run the baseline suite: `npm --prefix frontend run test`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST exist before the story.

**No foundational tasks.** `state.round.columnName` is already served and the boards already render values.

**Checkpoint**: Foundation ready.

---

## Phase 3: User Story 1 - Values show their column name in every game mode (Priority: P1) 🎯 MVP

**Goal**: Wherever a value is shown (Order, Tripmine, Connect), the round's column name is shown with it; a revealed Order item shows its value + column name in the middle of the card (like the anchor), not the footer.

**Independent Test**: `npm test` shows each board rendering the column name where values are shown, the Order revealed value in the middle, and no label where a value is hidden.

### Labels (already implemented — committed in 298dda0/123014b)

- [X] T002 [US1] `frontend/src/components/ConnectBoard.tsx`: optional `columnName` prop as the value-column heading.
- [X] T003 [US1] `frontend/src/components/TopBarOrdering.tsx`: optional `columnName` prop shown with the anchor value.
- [X] T004 [US1] `frontend/src/components/TripmineBoard.tsx`: optional `columnName` prop shown with the Tripmine value ("Year 1957").
- [X] T005 [US1] `frontend/src/pages/Session.tsx`: pass `columnName={state?.round?.columnName}` to the three boards.
- [X] T006 [US1] Tests in `ConnectBoard.test.tsx`, `TopBarOrdering.test.tsx`, `TripmineBoard.test.tsx` for the labels.

### Order revealed-value placement (remaining)

> **NOTE: Write the test FIRST, confirm it FAILS, then implement T008.**

- [X] T007 [US1] Add a test in `frontend/src/components/TopBarOrdering.test.tsx`: with a revealed slot item, the value and its column name render in the middle of the card (e.g. inside the item's middle content), not in a footer line. Confirm it fails before T008.
- [X] T008 [US1] In `frontend/src/components/TopBarOrdering.tsx`, move the revealed slot item's value (and its column name) from the footer into the middle of the card, using the same value-box markup as the anchor, for both the left and right slot groups. Keep visibility unchanged (value shown only when `revealAllValues`).
- [X] T009 [US1] Run `npm --prefix frontend run test -- TopBarOrdering ConnectBoard TripmineBoard` and confirm T007 passes and the rest stays green.

**Checkpoint**: Every displayed value carries the column name; Order revealed values sit in the middle like the anchor.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Full regression and manual validation.

- [X] T010 [P] Run `cargo test --manifest-path backend/Cargo.toml` (regression; backend unchanged).
- [X] T011 [P] Run `npm --prefix frontend run test` and `npm --prefix frontend run build`.
- [X] T012 Run the quickstart checks in `specs/017-value-column-labels/quickstart.md`: the column name shows with values in Order, Tripmine, and Connect; a revealed Order value sits in the middle; never with a hidden value.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: None to build.
- **User Story 1 (Phase 3)**: Depends on Setup.
- **Polish (Phase 4)**: Depends on US1.

### Within User Story 1

- T007 (test) before T008 (implementation); T009 verifies.
- T002–T006 are already complete.

### Parallel Opportunities

- T010 and T011 touch different toolchains and can run in parallel.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T007 (failing test) → T008 (move the value to the middle) → T009 (verify).
2. **STOP and VALIDATE**: the Order revealed value sits in the middle like the anchor.

### Incremental Delivery

1. US1 labels → already done.
2. US1 Order placement → this change.
3. Polish → full suites, build, quickstart.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps each task to its user story for traceability.
- Presentation only: value visibility, scoring, and rules are unchanged.
- The column name is already in the session state; no backend change.
- Verify the T007 test fails before implementing T008.
