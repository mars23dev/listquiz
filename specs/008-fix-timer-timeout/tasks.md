---

description: "Task list for Fix Timer Timeout"
---

# Tasks: Fix Timer Timeout

**Input**: Design documents from `/specs/008-fix-timer-timeout/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/timer-timeout.md, quickstart.md

**Tests**: Included — the constitution (Principles I and II) requires every business rule to be a
named, tested unit, and the plan calls for tests.

**Organization**: Tasks are grouped by user story so each can be implemented and verified on its own.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: US1 (server-owned deadline), US2 (no client post / no bar), US3 (ticks)
- Every task names an exact file path.

## Path Conventions

- **Web app**: `backend/src/`, `frontend/src/` (mirrors plan.md)

---

## Phase 1: Setup

**Purpose**: Confirm a green baseline before touching anything.

- [x] T001 Run the baseline suites and record the result: `cargo test --manifest-path backend/Cargo.toml`, `npm --prefix frontend run lint`, `npm --prefix frontend test`. No source change.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: None — the three stories are independent (server rule, client post removal, client ticks).

**Checkpoint**: Nothing blocks the stories.

---

## Phase 3: User Story 1 - Reaching zero actually ends the turn (Priority: P1) 🎯 MVP

**Goal**: The server owns the deadline: starting a timer schedules the timeout, which fires at the end time and ends the turn with no client action.

**Independent Test**: Start a short timer and do nothing; the turn ends on its own (Order: one heart + pass; Tripmine: pass). See quickstart.md §1.

### Tests for User Story 1 ⚠️

> Write first; ensure they FAIL before implementing.

- [x] T002 [P] [US1] Unit tests for `apply_turn_timeout` in `backend/src/modules/sessions/tests.rs`: no timer running → returns `false` and changes nothing; an Order timer → on-turn team loses exactly one heart, turn passes, timer cleared, returns `true`; a Tripmine timer → turn passes with no heart/point change; the last round finishing this way sets `finished`.
- [x] T003 [P] [US1] HTTP integration test in `backend/tests/turn_timer_actions.rs`: a 1s timer ends the turn on its own (the test posts no `time_out`); the turn has advanced and the timer is clear.

### Implementation for User Story 1

- [x] T004 [US1] In `backend/src/modules/sessions/mod.rs`: add `apply_turn_timeout(conn, session_id) -> ApiResult<bool>` (no timer → `false`; else read the round's quiz type, apply `turn_timer::expire` with the matching rule, save teams + events, clear the timer, set the `"Time ran out…"` message, and `finish_if_last_round` when nobody is left; return `true`).
- [x] T005 [US1] In `backend/src/routes/sessions.rs`: make the `time_out` action call `sessions::apply_turn_timeout`; in the `start_timer` action, after recording the timer, spawn a task on the runtime that sleeps until the end time, then (only if the session's timer still has that same end time) calls `apply_turn_timeout` and `state.bump_revision`.

**Checkpoint**: A timer ends the turn by itself, even with no client watching.

---

## Phase 4: User Story 2 - No stray error bar for the timer (Priority: P2)

**Goal**: The client no longer posts `time_out`; the countdown is the only timer indicator and no error bar appears.

**Independent Test**: A countdown reaching 0 posts nothing and shows no error. See quickstart.md §2.

### Tests for User Story 2 ⚠️

- [x] T006 [P] [US2] Component test in `frontend/src/test/Session.test.tsx`: with a timer at 0, no `time_out` action is posted and no `role="alert"` appears.

### Implementation for User Story 2

- [x] T007 [US2] In `frontend/src/pages/Session.tsx`: remove the client timeout effect (the `timeoutPostedRef` and its `sendAction({ action: 'time_out' … })`); keep the countdown display only.

**Checkpoint**: No timer error bar; the turn end comes from the server.

---

## Phase 5: User Story 3 - A tick for each of the last five seconds (Priority: P3)

**Goal**: Each of the last five seconds of the countdown plays a short tick.

**Independent Test**: A timer of five or more seconds ticks on each of 5,4,3,2,1; a shorter timer ticks on each remaining second. See quickstart.md §3.

### Tests for User Story 3 ⚠️

- [x] T008 [P] [US3] Unit tests in `frontend/src/utils/sessionSounds.test.ts` for `isFinalTickSecond`: `true` for 1..=5, `false` for 0 and for 6 and above.

### Implementation for User Story 3

- [x] T009 [US3] In `frontend/src/utils/sessionSounds.ts`: add `isFinalTickSecond(secondsLeft)` (true when `1 <= secondsLeft <= 5`). In `frontend/src/pages/Session.tsx`: in the countdown effect, when `secondsLeft` decreases into a final-tick second (tracked by a ref so a refetch does not double-tick), call `playSnapSound()` (the short tick); it obeys mute through the audio module.

**Checkpoint**: The last five seconds tick; the timeout cue still plays at zero.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T010 [P] Add amendment pointers to `specs/005-session-flow/contracts/session-flow.md` §1, `specs/006-fix-turn-timer/contracts/turn-timer.md`, and `specs/001-listquiz-game/contracts/realtime.md` noting the server-owned deadline and the ticks, referencing `specs/008-fix-timer-timeout/contracts/timer-timeout.md`.
- [x] T011 Run the full verification: `cargo fmt --manifest-path backend/Cargo.toml -- --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path backend/Cargo.toml`, `npm --prefix frontend run lint`, `npm --prefix frontend test`, `npm --prefix frontend run build`; then walk quickstart.md manually.
- [x] T012 Confirm build output stayed inside the repo and `frontend/dist/` + `backend/target/` remain untracked (`git status`), per the constitution's workflow step 7.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: starts immediately.
- **Foundational (Phase 2)**: none.
- **User Stories (Phase 3+)**: US1 is independent; US2 and US3 both edit `frontend/src/pages/Session.tsx`, so do US2 T007 before US3 T009 (sequentially).
- **Polish (Phase 6)**: after the desired stories.

### Within Each Story

- Tests before implementation (write and fail first).
- T004 (rule) before T005 (action + schedule); T008 (helper test) before T009.

### Parallel Opportunities

- T002 and T003 together.
- US1 backend work is disjoint from US2/US3 frontend work.
- T006 and T008 can be written together (different concerns), but T007/T009 on the same file run in order.

---

## Parallel Example: User Story 1

```bash
Task: "Unit tests for apply_turn_timeout in backend/src/modules/sessions/tests.rs"
Task: "HTTP test that a short timer ends the turn on its own in backend/tests/turn_timer_actions.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1)

1. Phase 1 Setup → 2. Phase 3 (US1) → **STOP and validate**: a short timer ends the turn with no client action.
2. US2: remove the client post (no error bar).
3. US3: last-five-seconds ticks.
4. Polish: contract pointers, full verification, build-hygiene check.

---

## Notes

- [P] = different files, no incomplete dependency.
- Tests are written first and must fail before implementation.
- No task commits on its own; commit only when the user asks.
- `the reference frontend` stays read-only (constitution).
