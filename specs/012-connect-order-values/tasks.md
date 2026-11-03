---
description: "Task list for connect-order-values"
---

# Tasks: Connect Value Order And GM Answer Key

**Input**: Design documents from `/specs/012-connect-order-values/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included. The constitution (Principle II) requires every business rule to be covered by a test, and the plan makes both changes testable. Write the tests first and confirm they fail before the implementation change.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2)
- Include exact file paths in descriptions

## Path Conventions

- **Web app**: `backend/src/`, `frontend/src/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm both toolchains run before changing anything.

- [X] T001 Run the baseline suites: `cargo test --manifest-path backend/Cargo.toml connect_round` and `npm --prefix frontend run test -- ConnectBoard Session`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST exist before user stories.

**No foundational tasks.** The Connect board, its storage (`connect_state`), the `ConnectView` payload, and the shared `itemId` pairing already exist. Proceed to the user stories.

**Checkpoint**: Foundation ready (nothing to build).

---

## Phase 3: User Story 1 - Values keep list order, names shuffle (Priority: P1) 🎯 MVP

**Goal**: The Connect value column stays in the list's item order; only the name column is randomized. No sorting algorithm is added.

**Independent Test**: `sample_connect_round` returns ten values in the list's item order and ten names in a different (shuffled) order.

### Tests for User Story 1

> **NOTE: Write this test FIRST, confirm it FAILS, then implement T003.**

- [X] T002 [US1] Add a test in `backend/src/modules/connect_round/tests.rs` asserting the value column is in list order and the name column is shuffled: build `sample_connect_round(&pairs(12), &mut rng(1))`, then assert the `values` item ids are in ascending list order (`item_1`..`item_10` after the seed-1 draw) while the `names` item-id sequence differs from that order. Run `cargo test --manifest-path backend/Cargo.toml connect_round` and confirm the new test fails before T003.

### Implementation for User Story 1

- [X] T003 [US1] Change `sample_connect_round` in `backend/src/modules/connect_round/mod.rs`: draw ten pairs, emit `values` in the list's item order (sort the ten chosen indices ascending before building the values) and shuffle **only** the `names` list. Update the function/module doc comments so they no longer say both columns are shuffled. Do not add any numeric or alphabetical sorting.
- [X] T004 [US1] Re-run `cargo test --manifest-path backend/Cargo.toml connect_round` and confirm T002 passes and every existing Connect rule test stays green.

**Checkpoint**: The value column is ordered by the list and the name column is random; scoring, hearts, matching, and turns are unchanged.

---

## Phase 4: User Story 2 - Gamemaster-only answer key (Priority: P2)

**Goal**: In the gamemaster view, each unmatched name shows its correct matching value; players and the spectator never see the pairing.

**Independent Test**: Rendering the Connect board with the GM flag shows each name's matching value; rendering without it shows no pairing.

### Tests for User Story 2

- [X] T005 [US2] Add a test in `frontend/src/components/ConnectBoard.test.tsx`: rendering `ConnectBoard` with the GM flag shows each unmatched name's correct value (paired by shared `itemId`), and rendering without the flag shows no pairing. Confirm it fails before T006.

### Implementation for User Story 2

- [X] T006 [US2] Add a boolean prop (e.g. `showAnswerKey`, default false) to `frontend/src/components/ConnectBoard.tsx`; when true, render, for each **unmatched** name row, the value whose `itemId` equals the name's `itemId` (find it in `view.values`). Matched rows stay hidden as today. No new interactive control; keep it keyboard-safe.
- [X] T007 [US2] In `frontend/src/pages/Session.tsx`, pass the gamemaster flag (`isGamemaster`) to the `ConnectBoard` at the `<ConnectBoard ... />` call site.
- [X] T008 [US2] Add a test in `frontend/src/test/Session.test.tsx` using the existing Connect fixture: rendered as `gamemaster` the answer key is shown; rendered as `p1` and as `view` no pairing is shown.
- [X] T009 [US2] Run `npm --prefix frontend run test -- ConnectBoard Session` and confirm T005 and T008 pass, the T006/T007 wiring works, and the rest of those suites stays green.

**Checkpoint**: Only the gamemaster sees the pairing; players and the spectator do not; the key drops with a matched pair.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Full regression and manual validation.

- [X] T010 [P] Run `cargo fmt --manifest-path backend/Cargo.toml --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, and `cargo test --manifest-path backend/Cargo.toml`.
- [X] T011 [P] Run `npm --prefix frontend run test` and `npm --prefix frontend run build`.
- [X] T012 Run the quickstart checks in `specs/012-connect-order-values/quickstart.md`: automated primary checks pass (backend value-order test, `ConnectBoard`/`Session` GM-key tests, full suites, build); the browser walkthrough is left as an optional manual confirmation for the user.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: None to build.
- **User Story 1 (Phase 3)**: Depends on Setup; backend only.
- **User Story 2 (Phase 4)**: Depends on Setup; frontend only. Independent of US1.
- **Polish (Phase 5)**: Depends on US1 and US2.

### User Story Dependencies

- **User Story 1 (P1)**: Independent; delivers the ordering fix.
- **User Story 2 (P2)**: Independent of US1 (different layer/files). Can run in parallel.

### Within Each User Story

- Test written and failing before implementation (T002 before T003; T005 before T006).
- Implementation before its verification (T004, T009).

### Parallel Opportunities

- T010 and T011 touch different toolchains and can run in parallel after US1/US2.
- US1 (T002–T004) and US2 (T005–T009) can be worked by two developers in parallel.
- T005 (ConnectBoard.test.tsx) must not run in parallel with T008 (Session.test.tsx) only if one person edits both; they are different files.
- T002 (backend tests) and T005 (frontend test) are different files and may be written in parallel.

---

## Parallel Example: Two Developers

```bash
# Developer A (backend, US1):
Task: "Add the value-order test in backend/src/modules/connect_round/tests.rs (T002)"
Task: "Change sample_connect_round in backend/src/modules/connect_round/mod.rs (T003)"

# Developer B (frontend, US2):
Task: "Add the GM-key test in frontend/src/components/ConnectBoard.test.tsx (T005)"
Task: "Add the answer key to frontend/src/components/ConnectBoard.tsx (T006)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001 (baseline).
2. Complete T002 (failing test) → T003 (ordering change) → T004 (verify).
3. **STOP and VALIDATE**: values are in list order and names are random. Deploy/demo.

### Incremental Delivery

1. US1 → value order fixed (MVP).
2. US2 → GM answer key.
3. Polish → full suites, fmt/clippy, build, quickstart.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps each task to its user story for traceability.
- One backend function and one component prop carry the whole change; keep it minimal (Principle VII).
- Verify the US1 and US2 tests fail before their implementation tasks.
- No schema, dependency, or scoring changes.
