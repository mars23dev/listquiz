---
description: "Task list for quick-game-connect-random"
---

# Tasks: Quick Game Connect And Random Lists

**Input**: Design documents from `/specs/018-quick-game-connect-random/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included. The constitution (Principle II) requires every business rule to be covered by a test. Write the tests first and confirm they fail before the implementation change.

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

- [X] T001 Run the baseline suites: `cargo test --manifest-path backend/Cargo.toml quick_game` and `npm --prefix frontend run test -- App`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST exist before the stories.

**No foundational tasks.** The quick game endpoint, the default lists, and the landing page exist. Proceed to the user stories.

**Checkpoint**: Foundation ready (nothing to build).

---

## Phase 3: User Story 1 - Connect is available as a quick game (Priority: P1) 🎯 MVP

**Goal**: The landing page offers Connect as a quick game and it starts a playable Connect round immediately.

**Independent Test**: `npm test App` shows the landing page offers Order, Tripmine, and Connect, and starting a Connect quick game opens the player view.

### Tests for User Story 1

> **NOTE: Write the test FIRST, confirm it FAILS, then implement T003–T005.**

- [X] T002 [US1] In `frontend/src/test/App.test.tsx`, add a test that the landing page shows a Connect quick-game choice and that choosing it calls the quick endpoint with `gameType: 'CONNECT'` and opens the player view. Confirm it fails before T003–T005.

### Implementation for User Story 1

- [X] T003 [US1] In `frontend/src/api/client.ts`, widen `createQuickSession`'s `gameType` parameter to `'ORDER' | 'TRIPMINE' | 'CONNECT'`.
- [X] T004 [US1] In `frontend/src/App.tsx`, widen `startQuickGame`'s `gameType` parameter to include `'CONNECT'`.
- [X] T005 [US1] In `frontend/src/pages/Landing.tsx`, widen the `onNavigateToQuick` prop type to include `'CONNECT'` and add a Connect button next to Order and Tripmine.
- [X] T006 [US1] Run `npm --prefix frontend run test -- App` and confirm T002 passes and the rest stays green.

**Checkpoint**: Connect is playable as a quick game.

---

## Phase 4: User Story 2 - Quick games use a random default list (Priority: P2)

**Goal**: A quick game chooses its default list at random among the lists that fit the game type.

**Independent Test**: `cargo test quick_game` shows the chosen list varies across seeds when several fit; a single eligible list is always chosen; no eligible list errors.

### Tests for User Story 2

> **NOTE: Write the tests FIRST, confirm they FAIL, then implement T008/T009.**

- [X] T007 [P] [US2] In `backend/src/modules/quick_game/tests.rs`, add tests: with a seeded rng the chosen `listId` varies across different seeds when more than one list fits; with a single eligible list that list is always chosen; with no eligible list the request is refused (`not_ready`). Confirm they fail before T008.

### Implementation for User Story 2

- [X] T008 [US2] In `backend/src/modules/quick_game/mod.rs`, change `build_request` to take `rng: &mut impl rand::Rng`, collect the indices of the lists that fit the game type (at least one column and at least the required number of items with values), and pick one uniformly at random (the single-fit case returns that list; no fit returns the existing `not_ready` error).
- [X] T009 [US2] In `backend/src/routes/sessions.rs` `create_quick_session`, pass `&mut rand::rng()` to `quick_game::build_request`.
- [X] T010 [US2] Run `cargo test --manifest-path backend/Cargo.toml quick_game` and confirm T007 passes and the rest stays green.

**Checkpoint**: Repeating a quick game varies the list.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Full regression and manual validation.

- [X] T011 [P] Run `cargo fmt --manifest-path backend/Cargo.toml --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, and `cargo test --manifest-path backend/Cargo.toml`.
- [X] T012 [P] Run `npm --prefix frontend run test` and `npm --prefix frontend run build`.
- [X] T013 Run the quickstart checks in `specs/018-quick-game-connect-random/quickstart.md`: a Connect quick game starts, repeated quick games vary the list, Order/Tripmine still work.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: None to build.
- **US1 (Phase 3)**: Frontend only.
- **US2 (Phase 4)**: Backend only. Independent of US1.
- **Polish (Phase 5)**: Depends on US1 and US2.

### User Story Dependencies

- US1 and US2 are independent (different layers/files) and may run in parallel.

### Within Each User Story

- Tests written and failing before implementation (T002 before T003–T005; T007 before T008/T009).
- Implementation before verification (T006, T010).

### Parallel Opportunities

- T002 (frontend App test) and T007 (backend quick_game test) are different files and can be written in parallel.
- T011 and T012 touch different toolchains and can run in parallel.

---

## Parallel Example: Kick-off

```bash
# Two tests in two different files, in parallel:
Task: "Connect quick-game test in frontend/src/test/App.test.tsx (T002)"
Task: "Random-list test in backend/src/modules/quick_game/tests.rs (T007)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001 (baseline).
2. Complete T002 (failing test) → T003–T005 (offer Connect) → T006 (verify).
3. **STOP and VALIDATE**: Connect is playable as a quick game. Deploy/demo.

### Incremental Delivery

1. US1 → Connect quick game.
2. US2 → random default list.
3. Polish → full suites, fmt/clippy, build, quickstart.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps each task to its user story for traceability.
- One team, one round, immediate start, player view are unchanged.
- No new dependency, no schema change.
- Verify each story's tests fail before implementing.
