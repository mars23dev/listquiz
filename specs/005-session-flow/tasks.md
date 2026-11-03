---

description: "Task list for Session Flow And Quick Game"
---

# Tasks: Session Flow And Quick Game

**Input**: Design documents from `/specs/005-session-flow/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/session-flow.md, quickstart.md

**Tests**: Included — the constitution (Principles I and II) requires every business rule to be a
named, tested unit, and the plan calls for tests.

**Organization**: Tasks are grouped by user story so each can be implemented and verified on its own.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: US1 (GM Tripmine values), US2 (turn timer), US3 (end dialog), US4 (delete session), US5 (quick game), US6 (dark only)
- Every task names an exact file path.

## Path Conventions

- **Web app**: `backend/src/`, `frontend/src/` (mirrors plan.md)

---

## Phase 1: Setup

**Purpose**: Confirm a green baseline before touching anything.

- [x] T001 Run the baseline suites and record the result: `cargo test --manifest-path backend/Cargo.toml`, `npm --prefix frontend run lint`, `npm --prefix frontend test`. No source change; note any pre-existing failures.

---

## Phase 2: Foundational (Blocking Prerequisite)

**Purpose**: The one shared, additive schema change the timer story reads from.

**⚠️ CRITICAL**: Complete before the user stories.

- [x] T002 [P] Add the idempotent migration to `backend/src/db.rs`: after `execute_batch(SCHEMA)`, inspect `PRAGMA table_info(game_session)` and, when `timer_ends_at` is absent, run `ALTER TABLE game_session ADD COLUMN timer_ends_at INTEGER` and `... ADD COLUMN timer_total_seconds INTEGER`. Safe on every open; both are `NULL` together (no timer).
- [x] T003 [P] Add a test (in `backend/src/db.rs` tests) that `apply_schema` adds both timer columns to a pre-existing `game_session` table, is safe to run twice, and reads `NULL` for an existing row.

**Checkpoint**: The session store upgrades in place.

---

## Phase 3: User Story 1 - The gamemaster always sees every Tripmine value (Priority: P1) 🎯 MVP

**Goal**: The gamemaster's board shows every item's value and rank, picked or not; players are unchanged.

**Independent Test**: As gamemaster, confirm every Tripmine item shows its value/rank before a pick; as a player, confirm unpicked items hide them. See quickstart.md §1.

### Tests for User Story 1 ⚠️

- [x] T004 [P] [US1] Component test in `frontend/src/components/TripmineBoard.test.tsx`: with `currentRole="gamemaster"`, an unpicked item renders its value and rank; with `currentRole="view"`, the same unpicked item shows neither.

### Implementation for User Story 1

- [x] T005 [US1] In `frontend/src/components/TripmineBoard.tsx`: render each item's value (and the GM rank) whenever `currentRole === 'gamemaster'`, regardless of the picked flag; keep the value hidden for players until revealed. No redaction change server-side.

**Checkpoint**: The gamemaster's answer key is back on the board; players unchanged.

---

## Phase 4: User Story 2 - Turn timer (Priority: P2)

**Goal**: The gamemaster starts a countdown on the current turn; every role sees it; on expiry the on-turn team loses one heart and the turn passes; the timer clears on any turn end.

**Independent Test**: Start a timer, watch it count down in every view, let it expire (one heart, turn passes), start/stop one early (no heart), and end a turn another way (timer clears). See quickstart.md §2.

### Tests for User Story 2 ⚠️

- [x] T006 [P] [US2] Unit tests in `backend/src/modules/turn_timer/tests.rs`: `start` (seconds > 0, `ends_at = now + seconds*1000`); `remaining` (ceil, floored at 0); `is_expired`; `expire` (on-turn team loses exactly one heart, never below 0; turn passes to the next team with hearts; nobody left → `None`).
- [x] T007 [P] [US2] Backend integration test in `backend/tests/turn_timer_actions.rs`: `start_timer`/`stop_timer` are gamemaster-only; `time_out` is refused before expiry, accepted at/after expiry for the gamemaster or the on-turn player (not another player), applies one heart and advances, and is refused a second time (idempotent); any other turn action clears the timer.
- [x] T008 [P] [US2] Frontend component test in `frontend/src/test/Session.test.tsx`: a state with a running timer renders a countdown and, at zero, posts exactly one `time_out` action.

### Implementation for User Story 2

- [x] T009 [US2] Create `backend/src/modules/turn_timer/mod.rs`: `TurnTimer { ends_at, total_seconds }`, `start(now, seconds)`, `remaining(ends_at, now)`, `is_expired(ends_at, now)`, and `expire(teams, active_team_id) -> (Vec<TeamStanding>, Option<String>)` reusing the scoring module (one heart, next team with hearts). Register the module in `backend/src/modules/mod.rs`.
- [x] T010 [US2] In `backend/src/state_reader.rs` and `backend/src/app_state.rs`: add `timer: Option<TimerView>` (`{ totalSeconds, secondsLeft }`) to `SessionState`, computed at read time from `timer_ends_at`/`timer_total_seconds`; `null` when no timer runs.
- [x] T011 [US2] In `backend/src/routes/sessions.rs` and `backend/src/modules/sessions/mod.rs`: add the `start_timer` (gamemaster), `stop_timer` (gamemaster), and `time_out` actions (accepted only when a timer runs, `now >= ends_at`, and the caller is the gamemaster or the on-turn team); add a `clear_timer` helper and call it from `confirm_placement` handling, Tripmine pick, `pass_turn`, `set_turn`, and `next_round`.
- [x] T012 [US2] In `frontend/src/types/session.ts`: add `timer: { totalSeconds: number; secondsLeft: number } | null`; in `frontend/src/pages/Session.tsx`: count the timer down locally from `secondsLeft`, post `time_out` once at zero, and render the countdown for every role.
- [x] T013 [US2] In `frontend/src/pages/Session.tsx`: add gamemaster timer controls (choose seconds and Start; Stop) that post `start_timer`/`stop_timer`, keyboard-operable.

**Checkpoint**: The timer runs, expires as a failed turn, and clears on any turn end.

---

## Phase 5: User Story 3 - A game-end dialog that shows points only (Priority: P3)

**Goal**: A dedicated modal dialog at game end lists teams by points, no hearts.

**Independent Test**: Finish a game; confirm the pop-up ranks teams by points with no hearts; end only a round; confirm no dialog. See quickstart.md §3.

### Tests for User Story 3 ⚠️

- [x] T014 [P] [US3] Component test in `frontend/src/components/GameEndDialog.test.tsx`: renders a `role="dialog"`, lists teams ordered by points descending, shows each team's points, and shows no heart count anywhere.

### Implementation for User Story 3

- [x] T015 [US3] Create `frontend/src/components/GameEndDialog.tsx`: a focus-managed `role="dialog"` modal listing teams by points (points only), with a keyboard-operable dismiss button and an `onDismiss` callback.
- [x] T016 [US3] In `frontend/src/pages/Session.tsx`: show `GameEndDialog` when `state.status === 'finished'` and remove the inline final-ranking section.

**Checkpoint**: The game-end dialog appears only at game end and shows points only.

---

## Phase 6: User Story 4 - A finished game's session is cleaned up (Priority: P4)

**Goal**: Dismissing the end dialog deletes the finished session; between-rounds sessions are never deleted.

**Independent Test**: Finish a game, dismiss the dialog, confirm the session reports not-found; end only a round and confirm the session still loads. See quickstart.md §4.

**Depends on**: US3 (the dialog dismissal triggers deletion).

### Tests for User Story 4 ⚠️

- [x] T017 [P] [US4] Backend integration test in `backend/tests/session_delete.rs`: deleting a `finished` session removes it and its teams/round state (cascade); deleting a non-finished session is refused (`not_ready`); deleting a missing session returns success; quiz lists are untouched.

### Implementation for User Story 4

- [x] T018 [US4] In `backend/src/modules/sessions/mod.rs`: add `delete_session(conn, id)` (success when missing; `not_ready` unless `finished`; otherwise `DELETE FROM game_session`, relying on the cascade). In `backend/src/routes/sessions.rs`: add `DELETE /sessions/{session_id}` and mount it.
- [x] T019 [US4] In `frontend/src/api/client.ts`: add `deleteSession(sessionId)`. In `frontend/src/pages/Session.tsx`: on end-dialog dismiss, call `deleteSession`, then `onNavigate('/')`.

**Checkpoint**: Finished sessions are removed on dismissal; live sessions survive.

---

## Phase 7: User Story 5 - Quick game from the title screen (Priority: P5)

**Goal**: Choose a game type on the title screen and start a one-team game immediately in the player view, with no gamemaster access.

**Independent Test**: From the title screen choose a type; a one-team game starts in the player view; no gamemaster link is offered. See quickstart.md §5.

### Tests for User Story 5 ⚠️

- [x] T020 [P] [US5] Unit tests in `backend/src/modules/quick_game/tests.rs`: `build_request(game_type, default_lists)` returns a one-team, one-round request whose default list fits the type (Order needs ≥ 11 valued items, Tripmine ≥ 10); picks the first fitting default list; errors when none fits.
- [x] T021 [P] [US5] Backend integration test in `backend/tests/quick_session.rs`: `POST /api/sessions/quick {gameType}` returns a created session with links and a running round of the chosen type; an unknown game type is rejected.

### Implementation for User Story 5

- [x] T022 [US5] Create `backend/src/modules/quick_game/mod.rs`: `build_request(game_type, default_lists) -> ApiResult<CreateSessionRequest>` (one team, one round, a fitting default list/column, an unused `gm_password`). Register the module in `backend/src/modules/mod.rs`.
- [x] T023 [US5] In `backend/src/routes/sessions.rs`: add `POST /sessions/quick` that builds the request from `default_lists::DEFAULT_LISTS` (and their seeded list ids/columns), calls `sessions::create_session`, starts round 1, and returns the normal `CreateSessionResponse`; mount it.
- [x] T024 [US5] In `frontend/src/api/client.ts`: add `createQuickSession(gameType)`. In `frontend/src/pages/Landing.tsx`: add a Quick Game control offering Order/Tripmine. In `frontend/src/App.tsx`: wire the Landing callback to create the quick session and navigate to `/{sessionId}/p1`.
- [x] T025 [P] [US5] Frontend test in `frontend/src/test/App.test.tsx`: the title screen offers Quick Game; choosing a type creates a session and navigates to the p1 view without exposing the gamemaster link.

**Checkpoint**: Quick game starts in the player view with no gamemaster access.

---

## Phase 8: User Story 6 - Dark mode only (Priority: P6)

**Goal**: No theme toggle anywhere; the dark theme is the only theme.

**Independent Test**: Load every screen; no toggle is present and the dark theme is used. See quickstart.md §6.

### Tests for User Story 6 ⚠️

- [x] T026 [P] [US6] Frontend test (in `frontend/src/test/App.test.tsx` or `frontend/src/theme/theme.test.ts`): no element matching a theme-toggle accessible name is rendered on the landing, lists, or session screens.

### Implementation for User Story 6

- [x] T027 [US6] Delete `frontend/src/components/ThemeToggle.tsx` and its usages in `frontend/src/pages/Landing.tsx`, `frontend/src/pages/ListManagement.tsx`, and `frontend/src/pages/Session.tsx`; make `frontend/src/theme/ThemeProvider.tsx` apply the dark theme only (drop the toggle API); update `frontend/src/theme/theme.test.ts` accordingly.

**Checkpoint**: Dark is the only theme; no toggle exists.

---

## Phase 9: Polish & Cross-Cutting Concerns

- [x] T028 [P] Add an amendment pointer to `specs/001-listquiz-game/contracts/realtime.md` and `specs/001-listquiz-game/contracts/rest-api.md` for the timer state/actions, the delete endpoint, and the quick-session endpoint, referencing `specs/005-session-flow/contracts/session-flow.md`.
- [x] T029 [P] Update `README.md` if it documents session controls, to mention the timer, quick game, and the removal of the theme toggle.
- [x] T030 Run the full verification: `cargo fmt --manifest-path backend/Cargo.toml -- --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path backend/Cargo.toml`, `npm --prefix frontend run lint`, `npm --prefix frontend test`, `npm --prefix frontend run build`; then walk quickstart.md manually (two tabs).
- [x] T031 Confirm build output stayed inside the repo and `frontend/dist/` + `backend/target/` remain untracked (`git status`), per the constitution's workflow step 7.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: starts immediately.
- **Foundational (Phase 2)**: after Setup — the timer columns.
- **User Stories (Phase 3+)**: after Foundational. US1, US2, US3, US5, US6 are independent; **US4 depends on US3** (deletion is triggered by the dialog's dismissal).
- **Polish (Phase 9)**: after the desired stories.

### Within Each Story

- Tests before implementation (write and fail first).
- Rules before endpoints; view fields before component wiring.

### Parallel Opportunities

- T002/T003 together.
- US1 T004 with any other test.
- US2 tests T006–T008 together; US5 tests T020/T021 together; US6 T026 early.
- US3/US5/US6 frontend work is disjoint from US2 backend work.

---

## Parallel Example: User Story 2

```bash
Task: "Unit tests for turn_timer in backend/src/modules/turn_timer/tests.rs"
Task: "Backend integration test in backend/tests/turn_timer_actions.rs"
Task: "Frontend countdown test in frontend/src/test/Session.test.tsx"
```

---

## Implementation Strategy

### MVP First (User Story 1)

1. Phase 1 Setup → 2. Phase 2 Foundational → 3. Phase 3 (US1) → **STOP and validate** the GM Tripmine fix independently → demo.

### Incremental Delivery

1. US1: GM sees the answer key (MVP).
2. US2: turn timer.
3. US3: points-only end dialog.
4. US4: finished-session cleanup (needs US3).
5. US5: quick game.
6. US6: dark only.
7. Polish: contract/README pointers, full verification, build-hygiene check.

---

## Notes

- [P] = different files, no incomplete dependency.
- Tests are written first and must fail before implementation.
- No task commits on its own; commit only when the user asks.
- `the reference frontend` stays read-only (constitution).
