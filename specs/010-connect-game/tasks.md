---

description: "Task list for Connect Game And Tripmine Marker"
---

# Tasks: Connect Game And Tripmine Marker

**Input**: Design documents from `/specs/010-connect-game/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/connect.md, quickstart.md

**Tests**: Included — the constitution (Principles I and II) requires every business rule to be a
named, tested unit, and the plan calls for tests.

**Organization**: Tasks are grouped by user story so each can be implemented and verified on its own.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: US1 (Connect game), US2 (Tripmine marker)
- Every task names an exact file path.

## Path Conventions

- **Web app**: `backend/src/`, `frontend/src/` (mirrors plan.md)

---

## Phase 1: Setup

**Purpose**: Confirm a green baseline before touching anything.

- [x] T001 Run the baseline suites and record the result: `cargo test --manifest-path backend/Cargo.toml`, `npm --prefix frontend run lint`, `npm --prefix frontend test`. No source change.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: None — the Connect story is self-contained; the marker story is a small display change.

**Checkpoint**: Nothing blocks the stories.

---

## Phase 3: User Story 1 - The Connect game type (Priority: P1) 🎯 MVP

**Goal**: A new turn-based game: ten names (left, with icons) and ten shuffled values (right, no icons); pick one of each into a middle box, confirm → +10 or −1 heart; three hearts; read-only at zero.

**Independent Test**: Play a Connect round: correct pair scores +10, wrong pair loses a heart, zero hearts goes read-only, all matched ends the round. See quickstart.md §1.

### Tests for User Story 1 ⚠️

> Write first; ensure they FAIL before implementing.

- [x] T002 [P] [US1] Unit tests in `backend/src/modules/connect_round/tests.rs`: `sample_connect_round` yields ten names and ten values (values shuffled) and refuses a list under ten items; `pick_name`/`pick_value` set the pending sides and reject an already-matched id; `confirm` on a matching pair awards 10 and marks it matched; on a mismatch loses exactly one heart (never below 0) and leaves the pair unmatched; both always advance the turn; `round_is_over` is true when all matched or no team has hearts.
- [x] T003 [P] [US1] Backend integration test in `backend/tests/connect_actions.rs`: create a Connect session and start the round; pick a matching pair and confirm (+10, turn passes); pick a mismatching pair and confirm (−1 heart, turn passes); drive a team to zero hearts and confirm read-only; the state exposes `connect.names`/`connect.values`/`connect.matched`/`connect.pending` to a player.

### Implementation for User Story 1

- [x] T004 [US1] Add the `connect_state` table to `backend/src/db.rs` (`CREATE TABLE IF NOT EXISTS`, columns `session_id` PK, `names`, `values`, `matched`, `pending`); add `connect_state` to the `clear_round_state` list in `backend/src/modules/sessions/round_state/mod.rs`.
- [x] T005 [US1] Create `backend/src/modules/connect_round/mod.rs`: `ConnectBoard { names, values, matched, pending }`, `sample_connect_round(items) -> ApiResult<ConnectBoard>`, `pick_name`, `pick_value`, `confirm(board, teams, team_id) -> ConfirmOutcome`, `round_is_over`, and `load_board`/`save_board` over `connect_state`. Register the module in `backend/src/modules/mod.rs`.
- [x] T006 [US1] In `backend/src/modules/sessions/mod.rs`: add `QuizType::Connect` (`parse`/`as_str` = `"CONNECT"`); in `validate_round` treat Connect like Order (no tripmine side) and update the type message; in `backend/src/modules/sessions/round_state/mod.rs::store_round_start` add the Connect branch that samples and saves the board.
- [x] T007 [US1] In `backend/src/app_state.rs` and `backend/src/state_reader.rs`: add `ConnectView { names, values, matched, pending }` to `SessionState` and read it (values public — Connect is a matching game); redaction passes it through unchanged.
- [x] T008 [US1] In `backend/src/routes/sessions.rs`: add `pick_name`, `pick_value`, and `confirm_connect` handling (gamemaster, or the on-turn team), include them in `is_player_action`, dispatch them in `apply_action`, clear the timer on a confirm, and call `sessions::finish_if_last_round` when the round is over.
- [x] T009 [US1] In `backend/src/modules/quick_game/mod.rs`: allow `CONNECT` (needs ten valued items) so a Connect quick game can start.
- [x] T010 [P] [US1] Component test in `frontend/src/components/ConnectBoard.test.tsx`: ten names with icons on the left, ten values (no icons) on the right, a middle box, and confirm/cancel; selecting a name and a value then confirming calls the handler.
- [x] T011 [US1] Frontend types: add `'CONNECT'` to `QuizType` and a `ConnectView` to `frontend/src/types/session.ts` and `frontend/src/types/quiz.ts`; widen the create-session `quizType` in `frontend/src/api/client.ts`.
- [x] T012 [US1] Create `frontend/src/components/ConnectBoard.tsx`: the split screen, the middle box, and the pick/confirm controls as native buttons.
- [x] T013 [US1] In `frontend/src/pages/Session.tsx`: render `ConnectBoard` when the round is Connect, map the view to the board, and add the pick/confirm handlers; in `frontend/src/pages/SetupQuiz.tsx`: add the Connect round option.
- [x] T014 [US1] In `frontend/src/utils/sessionSounds.ts`: a Connect correct pair (matched count grew while the score rose) plays the success cue; a wrong pair is the existing heart-drop failure cue.

**Checkpoint**: Connect plays end-to-end, read-only at zero hearts, and finishes the last round.

---

## Phase 4: User Story 2 - The Tripmine bar shows the mine position (Priority: P2)

**Goal**: The Tripmine progress bar marks the mine's position from the start for every role, without revealing the item.

**Independent Test**: Start a Tripmine round and confirm the bar marks the mine's position immediately while the item stays hidden. See quickstart.md §2.

### Tests for User Story 2 ⚠️

- [x] T015 [P] [US2] Backend test (in `backend/src/app_state.rs` tests) and component test in `frontend/src/components/TripmineBoard.test.tsx`: `tripmine.mineRank` is present for every role (gamemaster and player), and the board marks that rank from the start while the mine's item stays hidden.

### Implementation for User Story 2

- [x] T016 [US2] In `backend/src/app_state.rs` and `backend/src/state_reader.rs`: add `mine_rank: i32` to `TripmineView`, populated from the mine's slot, public to every role; mirror `mineRank` in `frontend/src/types/session.ts`.
- [x] T017 [US2] In `frontend/src/components/TripmineBoard.tsx`: mark the `mineRank` position on the progress ladder from the start (distinct from the revealed-mine styling).

**Checkpoint**: The mine's position is visible from the start; the item stays hidden.

---

## Phase 5: Polish & Cross-Cutting Concerns

- [x] T018 [P] Add amendment pointers to `specs/001-listquiz-game/contracts/realtime.md` and `specs/001-listquiz-game/contracts/rest-api.md` for the Connect round and the Tripmine marker, referencing `specs/010-connect-game/contracts/connect.md`.
- [x] T019 Run the full verification: `cargo fmt --manifest-path backend/Cargo.toml -- --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path backend/Cargo.toml`, `npm --prefix frontend run lint`, `npm --prefix frontend test`, `npm --prefix frontend run build`; then walk quickstart.md manually.
- [x] T020 Confirm build output stayed inside the repo and `frontend/dist/` + `backend/target/` remain untracked (`git status`), per the constitution's workflow step 7.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: starts immediately.
- **Foundational (Phase 2)**: none.
- **User Stories (Phase 3+)**: US2 (the marker) is independent of US1 (Connect) and can be done in parallel.
- **Polish (Phase 5)**: after the desired stories.

### Within User Story 1

- T004 (table) before T005 (module); T005 before T006 (start a round); T006/T007 before T008 (actions);
  T011 before T012/T013; T005's rule before T014 (cues).

### Parallel Opportunities

- T002 and T003 together.
- US2 (T015–T017) in parallel with US1 backend work (disjoint files).
- T010 (board test) and T011 (types) with the backend module.

---

## Parallel Example: User Story 1

```bash
Task: "Unit tests for connect_round in backend/src/modules/connect_round/tests.rs"
Task: "Backend integration test in backend/tests/connect_actions.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1)

1. Phase 1 Setup → 2. Phase 3 (US1) → **STOP and validate**: a Connect round plays end-to-end.
2. US2: the Tripmine marker.
3. Polish: contract pointers, full verification, build-hygiene check.

---

## Notes

- [P] = different files, no incomplete dependency.
- Tests are written first and must fail before implementation.
- No task commits on its own; commit only when the user asks.
- `the reference frontend` stays read-only (constitution).
