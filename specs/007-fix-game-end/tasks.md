---

description: "Task list for Fix Game End After The Last Round"
---

# Tasks: Fix Game End After The Last Round

**Input**: Design documents from `/specs/007-fix-game-end/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/game-end.md, quickstart.md

**Tests**: Included — the constitution (Principles I and II) requires every business rule to be a
named, tested unit, and the plan calls for tests.

**Organization**: One user story; a small backend-only fix.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: US1
- Every task names an exact file path.

## Path Conventions

- **Web app**: `backend/src/`, `frontend/src/` (mirrors plan.md)

---

## Phase 1: Setup

**Purpose**: Confirm a green baseline before touching anything.

- [x] T001 Run the baseline suites and record the result: `cargo test --manifest-path backend/Cargo.toml`, `npm --prefix frontend run lint`, `npm --prefix frontend test`. No source change.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: None — this is a single, self-contained story with no shared prerequisite. The helper
is created within the story phase.

**Checkpoint**: Nothing blocks US1.

---

## Phase 3: User Story 1 - The game-end dialog appears when the last round ends (Priority: P1) 🎯 MVP

**Goal**: Ending the last configured round finishes the session automatically, so the existing
game-end dialog appears for every role — including in a gamemaster-less quick game.

**Independent Test**: End the last round of a one-round session (a quick game is simplest) and
confirm the session reports `finished` with no gamemaster action; end a non-final round of a
multi-round session and confirm it stays `running`. See quickstart.md.

### Tests for User Story 1 ⚠️

> Write first; ensure they FAIL before implementing.

- [x] T002 [P] [US1] Unit tests for `finish_if_last_round` in `backend/src/modules/sessions/tests.rs`: when the current round equals the round count it sets `finished` (with `reveal_values = 1`, `active_team_id = NULL`, `last_message = "Game over"`) and returns `true`; when the current round is not the last it changes nothing and returns `false`; when the session is already `finished` it is a no-op returning `false`.
- [x] T003 [P] [US1] Backend integration test in `backend/tests/game_end.rs`: a one-round Tripmine session where the gamemaster picks the mine becomes `finished` with no further action (the quick-game case, exercised via the normal create flow); a two-round session that finishes round 1 stays `running`; finishing the last round becomes `finished`.

### Implementation for User Story 1

- [x] T004 [US1] Add `finish_if_last_round(conn, session_id) -> ApiResult<bool>` to `backend/src/modules/sessions/mod.rs`: if `status != 'finished'` and `current_round >= round_count`, set `status = 'finished'`, `reveal_values = 1`, `active_team_id = NULL`, `last_message = 'Game over'` and return `true`; otherwise return `false`.
- [x] T005 [US1] Call `finish_if_last_round` from `backend/src/routes/sessions.rs` at every point a round can end: the Order confirmation when `outcome.round_over`, the Tripmine pick when `outcome.round.completed`, and the timeout (`apply_time_out`) when the turn ends with no team on turn.

**Checkpoint**: The last round's end finishes the game and the dialog appears; non-final rounds are unchanged.

---

## Phase 4: Polish & Cross-Cutting Concerns

- [x] T006 [P] Add an amendment pointer to `specs/001-listquiz-game/contracts/realtime.md` and `specs/001-listquiz-game/contracts/rest-api.md` noting that the last configured round's end finishes the session, referencing `specs/007-fix-game-end/contracts/game-end.md`.
- [x] T007 Run the full verification: `cargo fmt --manifest-path backend/Cargo.toml -- --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path backend/Cargo.toml`, `npm --prefix frontend run lint`, `npm --prefix frontend test`, `npm --prefix frontend run build`; then walk quickstart.md manually.
- [x] T008 Confirm build output stayed inside the repo and `frontend/dist/` + `backend/target/` remain untracked (`git status`), per the constitution's workflow step 7.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: starts immediately.
- **Foundational (Phase 2)**: none.
- **User Story 1 (Phase 3)**: after Setup.
- **Polish (Phase 4)**: after US1.

### Within the Story

- Tests before implementation (write and fail first).
- T004 (helper) before T005 (call sites).

### Parallel Opportunities

- T002 and T003 together (different files).
- T006 with the verification work.

---

## Parallel Example: User Story 1

```bash
Task: "Unit tests for finish_if_last_round in backend/src/modules/sessions/tests.rs"
Task: "Backend integration test in backend/tests/game_end.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1)

1. Phase 1 Setup → 2. Phase 3 (US1) → **STOP and validate**: a quick game ends and shows the dialog.
3. Polish: contract pointer, full verification, build-hygiene check.

---

## Notes

- [P] = different files, no incomplete dependency.
- Tests are written first and must fail before implementation.
- No task commits on its own; commit only when the user asks.
- `the reference frontend` stays read-only (constitution).
