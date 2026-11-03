---
description: "Task list for fix-connect-start"
---

# Tasks: Fix Connect Round Start Panel

**Input**: Design documents from `/specs/011-fix-connect-start/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included. The constitution (Principle II) requires every business rule to be covered by a test, and the plan makes the prompt-visibility rule a tested predicate. Write the tests first and confirm they fail before the implementation change.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2)
- Include exact file paths in descriptions

## Path Conventions

- **Web app**: `backend/src/`, `frontend/src/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm the existing frontend test toolchain runs before changing anything.

- [X] T001 Run the frontend unit suite once to establish a green baseline: `npm --prefix frontend run test -- Session`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST exist before the fix can be applied.

**No foundational tasks.** The streamed session state, the `connect` field (`state.connect`), and the pre-round prompt already exist; this feature only changes when the prompt renders. Proceed directly to Phase 3.

**Checkpoint**: Foundation ready (nothing to build).

---

## Phase 3: User Story 1 - Start panel clears once a Connect round begins (Priority: P1) 🎯 MVP

**Goal**: Once a Connect round is active, the gamemaster's "Start Round" button and players' "Waiting for the game to be started…" message disappear for every role, and starting a round does not end the game.

**Independent Test**: With a Connect session-state fixture where `connect` is present, render the gamemaster and player views and assert neither the start button nor the waiting message is in the DOM.

### Tests for User Story 1

> **NOTE: Write this test FIRST, confirm it FAILS against the current guard, then implement T003.**

- [X] T002 [US1] Add a failing test in `frontend/src/test/Session.test.tsx` under the pre-round prompt describe block: a Connect session-state fixture (copy `orderState()` shape but with a non-null `connect` object and `order: null, tripmine: null`) rendering role `gamemaster` asserts there is no button named `/start round/i`, and rendering role `p1` asserts `/waiting for the game to be started/i` is absent; also assert the Connect board is present. Run `npm --prefix frontend run test -- Session` and confirm this test fails before T003.

### Implementation for User Story 1

- [X] T003 [US1] In `frontend/src/pages/Session.tsx`, change the pre-round prompt guard (currently `state && !order && !tripmine && state.status !== 'finished'`, around line 757) to also exclude a Connect board: `state && !connect && !order && !tripmine && state.status !== 'finished'`. Change only this expression; keep the surrounding markup.
- [X] T004 [US1] Re-run `npm --prefix frontend run test -- Session` and confirm T002's test now passes and existing pre-round prompt tests stay green.

**Checkpoint**: User Story 1 fully functional — a started Connect round clears the prompt for every role and cannot be ended by the former control.

---

## Phase 4: User Story 2 - Starting a Connect round matches the other game types (Priority: P2)

**Goal**: The pre-round prompt is shown only while no game type is active, identically for Connect, Order, and Tripmine.

**Independent Test**: A test that asserts the prompt rule holds for all three board types: absent when any board is active, present when none is active and the session is not finished.

### Tests for User Story 2

- [X] T005 [US2] Extend `frontend/src/test/Session.test.tsx` with a consistency case: for each active board (Connect via the T002 fixture, Order via `orderState()`, Tripmine via `tripmineState()`), assert the start button and waiting message are absent for a non-finished session; and for the pre-round fixture (`NOT_STARTED`) assert the gamemaster sees the button and a player sees the waiting message.

### Implementation for User Story 2

- [X] T006 [US2] Confirm the T003 guard satisfies the consistency case without further production changes; run `npm --prefix frontend run test` and confirm the whole frontend suite is green. If the consistency case reveals a gap, adjust only the single guard expression from T003.

**Checkpoint**: All three game types behave identically for the pre-round prompt.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Validation and regression safety across the changed surface.

- [X] T007 [P] Run the full frontend suite `npm --prefix frontend run test` and the frontend build `npm --prefix frontend run build` to confirm nothing regressed and the app still builds.
- [X] T008 [P] Run the quickstart validation in `specs/011-fix-connect-start/quickstart.md`: the automated primary check (`Session.test.tsx` Connect visibility cases) passes; the browser walkthrough is left as an optional manual confirmation for the user. (gamemaster + player + spectator on a Connect session): prompt clears on start, game does not end, Order/Tripmine prompt behavior unchanged.
- [X] T009 Verify keyboard operability is unchanged (Principle VI): in a pre-round state, the gamemaster's "Start Round" is reachable and activatable by keyboard alone with visible focus, and no stale control remains after starting a Connect round.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: None to build.
- **User Story 1 (Phase 3)**: Depends on Setup. Delivers the fix.
- **User Story 2 (Phase 4)**: Depends on US1's guard edit (T003); adds consistency coverage.
- **Polish (Phase 5)**: Depends on US1 and US2.

### User Story Dependencies

- **User Story 1 (P1)**: Independent. The fix and its direct test.
- **User Story 2 (P2)**: Builds on US1's guard; adds cross-game-type coverage. No new production code expected.

### Within Each User Story

- Test written and failing before implementation (T002 before T003).
- Production change (T003) before its verification (T004).

### Parallel Opportunities

- T007 and T008 touch different concerns and can run in parallel after US1/US2 complete.
- T005 must not run in parallel with T002 (same file, `Session.test.tsx`).

---

## Parallel Example: Polish Phase

```bash
# After US1 and US2 are complete, run independent verifications together:
Task: "Run full frontend suite and build (T007)"
Task: "Run quickstart manual Connect checks (T008)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001 (baseline).
2. Complete T002 (failing test) → T003 (guard fix) → T004 (verify).
3. **STOP and VALIDATE**: T002's test passes; a started Connect round clears the prompt and cannot end the game. Deploy/demo.

### Incremental Delivery

1. US1 → prompt clears for Connect (MVP).
2. US2 → consistency coverage across Connect/Order/Tripmine.
3. Polish → full suite, build, quickstart, keyboard check.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps each task to its user story for traceability.
- The production change is a single guard expression in one file; keep it minimal (Principle VII).
- Verify the US1 test fails before applying T003.
- No backend, schema, or dependency changes.
