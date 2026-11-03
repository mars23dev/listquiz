# Tasks: Fix Order And Setup

**Input**: Design documents from `/specs/003-fix-order-setup/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included — the project constitution requires named, tested behaviour, and each fix changes
user-visible behaviour, so each carries a test.

**Organization**: Tasks are grouped by user story so each fix is independently implementable and
testable.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2)
- Exact file paths are given in each description.

## Path Conventions

- Frontend: `frontend/src/`

---

## Phase 1: Setup

**Purpose**: Confirm a green baseline before changing anything.

- [x] T001 Verify baseline: `cargo test --manifest-path backend/Cargo.toml` and `npm --prefix frontend test` pass, and `npm --prefix frontend run build` succeeds.

---

## Phase 2: Foundational

**Purpose**: Blocking prerequisites for the user stories.

No shared prerequisites: the two fixes touch different files and neither depends on the other. The
gap-to-position helper belongs to US1.

---

## Phase 3: User Story 1 - Order placement works (Priority: P1) MVP

**Goal**: A team drops an item into a gap (between two items or beside an end) on a contiguous side
and gets a neighbour-judged, scored result, never the internal "slot is not on this side" error.

**Independent Test**: Start an Order round, select an item, drop it into a gap, and confirm; the
result is points or a lost heart, never an error, on both sides and once a side has items.

### Tests for User Story 1

- [x] T002 [P] [US1] Unit test for the gap→position mapping in `frontend/src/utils/orderBar.test.ts`: an empty side (0 items) has one gap and maps every dropped column to position `0`; a side holding n items has gaps `0..=n` and every dropped column maps to a position in that range; the result is clamped so no drop ever yields a value outside `0..=n`.
- [x] T003 [P] [US1] Component test in `frontend/src/test/Session.test.tsx`: confirming a placement on a freshly started Order round posts `choose_slot` with `slotIndex: 0` for an empty side and shows no slot-range error; a wrong-side confirm surfaces an incorrect (heart) outcome, not an error.

### Implementation for User Story 1

- [x] T004 [US1] Implement the named rule `positionForGap(side, column, sideLength) -> number` in `frontend/src/utils/orderBar.ts`: translate the dropped bar column to the gap's insertion position on a contiguous side and clamp it to `0..=sideLength` (contracts/order-placement.md).
- [x] T005 [US1] Use `positionForGap` in `handleChooseSlot` and `handleDropToPending` in `frontend/src/pages/Session.tsx` so the `choose_slot` `slotIndex` is always a valid gap on the named side, for both the gamemaster and the active-team player.

**Checkpoint**: Any gap the bar offers is accepted and scored; the round is playable and sides stay contiguous.

---

## Phase 4: User Story 2 - Tripmine side only for Tripmine rounds (Priority: P2)

**Goal**: The Setup round configuration shows the lowest/highest control only when the round type is
Tripmine.

**Independent Test**: Switch a round between Order and Tripmine in Setup; the control appears only for
Tripmine.

### Tests for User Story 2

- [x] T006 [P] [US2] Component test in `frontend/src/test/SetupQuiz.test.tsx`: the tripmine side control is absent while the round type is Order and present when it is Tripmine; switching back to Order hides it; the created session body carries no `tripmineSide` for an Order round.

### Implementation for User Story 2

- [x] T007 [US2] Render the tripmine side control only when `round.quizType === 'TRIPMINE'` and omit `tripmineSide` from an Order round in `frontend/src/pages/SetupQuiz.tsx`.

**Checkpoint**: Order rounds show no tripmine side control; Tripmine rounds still do.

---

## Phase 5: Polish & Cross-Cutting Concerns

- [x] T008 Run the full verification pass: `cargo fmt --manifest-path backend/Cargo.toml --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, `cargo test`, `npm --prefix frontend run lint`, `npm --prefix frontend test`, `npm --prefix frontend run build`, and the `specs/003-fix-order-setup/quickstart.md` steps.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (T001)**: no dependencies.
- **Foundational**: none.
- **US1 (T002–T005)** and **US2 (T006–T007)**: independent of each other; both start after T001.
- **Polish (T008)**: after the stories it verifies.

### Within Each User Story

- Tests before implementation; T002 before T004, T003 after T005 (it exercises the wired behaviour).
- T004 (helper) before T005 (wiring).
- US2: T006 (test) before T007 (implementation).

### Parallel Opportunities

- T002 and T003 are separate test files and can run in parallel once T001 is done.
- US1 and US2 touch different files and can proceed in parallel.

## Parallel Example: User Story 1

```bash
Task: "Unit test for the gap→position mapping in frontend/src/utils/orderBar.test.ts"
Task: "Component test for a valid choose_slot in frontend/src/test/Session.test.tsx"
```

---

## Implementation Strategy

### MVP First (User Story 1)

1. T001 baseline.
2. US1 (T002–T005) → Order is playable again.
3. Validate US1 by dropping an item into a gap.

### Incremental Delivery

1. US1 (P1) → the core round works.
2. US2 (P2) → Setup no longer shows the tripmine control for Order.

---

## Notes

- [P] tasks touch different files with no incomplete dependency.
- Verify each test fails before implementing.
- Commit after each story.
