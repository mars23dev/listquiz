---

description: "Task list for Fix Turn Timer"
---

# Tasks: Fix Turn Timer

**Input**: Design documents from `/specs/006-fix-turn-timer/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/turn-timer.md, quickstart.md

**Tests**: Included — the constitution (Principles I and II) requires every business rule to be a
named, tested unit, and the plan calls for tests.

**Organization**: Tasks are grouped by user story so each can be implemented and verified on its own.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: US1 (timeout per type), US2 (timer sounds), US3 (presets)
- Every task names an exact file path.

## Path Conventions

- **Web app**: `backend/src/`, `frontend/src/` (mirrors plan.md)

---

## Phase 1: Setup

**Purpose**: Confirm a green baseline before touching anything.

- [x] T001 Run the baseline suites and record the result: `cargo test --manifest-path backend/Cargo.toml`, `npm --prefix frontend run lint`, `npm --prefix frontend test`. No source change.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: None — the three stories touch disjoint code (the timeout rule, the cue module, the
timer controls).

**Checkpoint**: Nothing blocks the stories.

---

## Phase 3: User Story 1 - A timeout ends the turn correctly for each game type (Priority: P1) 🎯 MVP

**Goal**: An Order timeout costs one heart and passes the turn; a Tripmine timeout passes the turn with no heart or point change.

**Independent Test**: Start a short timer in each game type and let it expire; Order loses a heart and advances, Tripmine only advances. See quickstart.md §1.

### Tests for User Story 1 ⚠️

> Write first; ensure they FAIL before implementing.

- [x] T002 [P] [US1] Unit tests in `backend/src/modules/turn_timer/tests.rs`: `expire` with the `LoseHeart` rule removes exactly one heart and advances; with `PassOnly` it changes no heart and emits no events and advances; both never drop a heart below zero; no team with hearts left → `None`.
- [x] T003 [P] [US1] HTTP integration test in `backend/tests/turn_timer_actions.rs`: an Order timeout removes one heart and passes the turn; a Tripmine timeout changes no team's hearts and passes the turn; both clear the timer and set a timeout message.

### Implementation for User Story 1

- [x] T004 [US1] In `backend/src/modules/turn_timer/mod.rs`: add `pub enum TimeoutRule { LoseHeart, PassOnly }` and change `expire(teams, active_team_id, rule)` to apply a heart only for `LoseHeart` (reusing the scoring module) and otherwise just advance the turn; keep returning `(teams, events, next)`.
- [x] T005 [US1] In `backend/src/routes/sessions.rs::apply_time_out`: read the current round's quiz type (`round_config.quiz_type`), pass `LoseHeart` for Order and `PassOnly` for Tripmine, and set a timeout `last_message` beginning `"Time ran out"` for both.

**Checkpoint**: A timeout's consequence matches the game type.

---

## Phase 4: User Story 2 - The timer makes a sound (Priority: P2)

**Goal**: Starting the timer plays a cue and a timeout plays a cue, for every role, once, obeying mute.

**Independent Test**: In an unmuted session, start a timer (start cue) and let it expire (timeout cue); mute and confirm neither plays. See quickstart.md §2.

### Tests for User Story 2 ⚠️

- [x] T006 [P] [US2] Unit tests in `frontend/src/utils/sessionSounds.test.ts`: a timer appearing where there was none → `buzzer`; a timer clearing with a `"Time ran out"` message → `failure`; a placement/`pass_turn` clearing the timer → `null`; an Order timeout (timer clears AND a heart drops) → exactly one `failure`.

### Implementation for User Story 2

- [x] T007 [US2] In `frontend/src/utils/sessionSounds.ts`: add `buzzer` to `SessionCue`; after the round-change guard, return `buzzer` when `prev.timer === null && next.timer !== null`, and return `failure` when `prev.timer !== null && next.timer === null && (next.lastMessage ?? '').startsWith('Time ran out')` (this runs before the heart-loss check, so an Order timeout is one cue).
- [x] T008 [US2] In `frontend/src/pages/Session.tsx`: handle the `buzzer` cue by calling `playBuzzerSound()` in the cue effect.

**Checkpoint**: The timer is audible on start and on timeout, once, and silent when muted.

---

## Phase 5: User Story 3 - Fixed timer durations (Priority: P3)

**Goal**: The gamemaster starts the timer from exactly 15s, 30s, and 60s; no free-text field.

**Independent Test**: The timer controls show three presets and no input; picking one runs a countdown of that length. See quickstart.md §3.

### Tests for User Story 3 ⚠️

- [x] T009 [P] [US3] Component test in `frontend/src/test/Session.test.tsx`: the gamemaster sees buttons `15s`, `30s`, `60s` and no `spinbutton`/number input; clicking `30s` posts `start_timer` with `seconds: 30`.

### Implementation for User Story 3

- [x] T010 [US3] In `frontend/src/pages/Session.tsx`: replace the seconds number input with three preset buttons (15s, 30s, 60s) that each call `handleStartTimer(seconds)`, keeping the Stop control; remove the `timerSeconds` input state.

**Checkpoint**: The timer offers only the three presets.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T011 [P] Add an amendment pointer to `specs/005-session-flow/contracts/session-flow.md` §1 (and `specs/001-listquiz-game/contracts/realtime.md`) noting the per-type timeout and the timer cues, referencing `specs/006-fix-turn-timer/contracts/turn-timer.md`.
- [x] T012 Run the full verification: `cargo fmt --manifest-path backend/Cargo.toml -- --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path backend/Cargo.toml`, `npm --prefix frontend run lint`, `npm --prefix frontend test`, `npm --prefix frontend run build`; then walk quickstart.md manually.
- [x] T013 Confirm build output stayed inside the repo and `frontend/dist/` + `backend/target/` remain untracked (`git status`), per the constitution's workflow step 7.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: starts immediately.
- **Foundational (Phase 2)**: none.
- **User Stories (Phase 3+)**: US1, US2, US3 are independent; US2's Session.tsx cue wiring and US3's control change touch the same file, so do US2 T008 before US3 T010 (or sequentially).
- **Polish (Phase 6)**: after the desired stories.

### Within Each Story

- Tests before implementation (write and fail first).
- T004 (rule) before T005 (handler); T007 (cue) before T008 (wiring).

### Parallel Opportunities

- T002 and T003 together.
- T006 (cue tests) with US1 work; T009 (preset test) with US1 work.
- US1 backend work is disjoint from US2/US3 frontend work.

---

## Parallel Example: User Story 1

```bash
Task: "Unit tests for expire rules in backend/src/modules/turn_timer/tests.rs"
Task: "HTTP test for the per-type timeout in backend/tests/turn_timer_actions.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1)

1. Phase 1 Setup → 2. Phase 3 (US1) → **STOP and validate**: the timeout consequence matches the type.
2. US2: timer cues.
3. US3: preset durations.
4. Polish: contract pointer, full verification, build-hygiene check.

---

## Notes

- [P] = different files, no incomplete dependency.
- Tests are written first and must fail before implementation.
- No task commits on its own; commit only when the user asks.
- `the reference frontend` stays read-only (constitution).
