---

description: "Task list for Tripmine Mode"
---

# Tasks: Tripmine Mode

**Input**: Design documents from `/specs/009-tripmine-mode/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/tripmine-mode.md, quickstart.md

**Tests**: Included — the constitution (Principles I and II) requires every business rule to be a
named, tested unit, and the plan calls for tests.

**Organization**: Tasks are grouped by user story so each can be implemented and verified on its own.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: US1 (no hearts), US2 (cue order), US3 (random side)
- Every task names an exact file path.

## Path Conventions

- **Web app**: `backend/src/`, `frontend/src/` (mirrors plan.md)

---

## Phase 1: Setup

**Purpose**: Confirm a green baseline before touching anything.

- [x] T001 Run the baseline suites and record the result: `cargo test --manifest-path backend/Cargo.toml`, `npm --prefix frontend run lint`, `npm --prefix frontend test`. No source change.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: None — the three stories touch different concerns (heart display, cue list, side rule).

**Checkpoint**: Nothing blocks the stories.

---

## Phase 3: User Story 1 - Tripmine has no hearts (Priority: P1) 🎯 MVP

**Goal**: A Tripmine round shows no heart indicator and changes no hearts; Order is unchanged.

**Independent Test**: Play a Tripmine round — no hearts shown, none change; play Order — hearts as before. See quickstart.md §1.

### Tests for User Story 1 ⚠️

- [x] T002 [P] [US1] Component test (in `frontend/src/test/Session.test.tsx` or `frontend/src/components/BottomTeams.test.tsx`): a Tripmine round renders no heart indicator; an Order round renders it.

### Implementation for User Story 1

- [x] T003 [US1] In `frontend/src/components/BottomTeams.tsx`: add a `showHearts` prop (default true) and skip the heart row when false. In `frontend/src/pages/Session.tsx`: pass `showHearts={state?.round?.quizType !== 'TRIPMINE'}` and hide the gamemaster heart-adjust controls when hearts are hidden.

**Checkpoint**: Hearts appear only in Order rounds.

---

## Phase 4: User Story 2 - Round end plays before game end (Priority: P2)

**Goal**: A round-ending cue plays before the game-end cue when the same transition finishes the game.

**Independent Test**: Hit the mine on the last round → explosion then game-end cue; a non-final round end → one cue. See quickstart.md §2.

### Tests for User Story 2 ⚠️

- [x] T004 [P] [US2] Unit tests in `frontend/src/utils/sessionSounds.test.ts`: for `deriveSessionCues`, a mine reveal that also finishes the game returns `['explosion', 'bonus']`; a correct placement that finishes the game returns `['chime', 'bonus']`; a round end that does not finish returns a single cue; a plain finish (no event) returns `['bonus']`.

### Implementation for User Story 2

- [x] T005 [US2] In `frontend/src/utils/sessionSounds.ts`: replace `deriveSessionCue` with `deriveSessionCues(prev, next): SessionCue[]` — compute the event cue first, then append `'bonus'` when `next.status === 'finished'` and `prev.status !== 'finished'`; remove the early `bonus` return. Keep the other guards (revision, reveal toggle, round change).
- [x] T006 [US2] In `frontend/src/pages/Session.tsx`: play every cue from `deriveSessionCues` in order.

**Checkpoint**: The mine's explosion is no longer skipped at game end.

---

## Phase 5: User Story 3 - Random (Hard) Tripmine mode (Priority: P3)

**Goal**: A third Tripmine side where the mine is random and the searched target is a random remaining non-mine item.

**Independent Test**: Setup shows Lowest/Highest/Random (Hard); a Random round's mine can be any item and the search target is a random remaining non-mine. See quickstart.md §3.

### Tests for User Story 3 ⚠️

- [x] T007 [P] [US3] Backend unit tests in `backend/src/modules/tripmine_round/tests.rs`: `TripmineSide::parse("random")` works; `sample_tripmine_grid(side=Random)` yields exactly one mine whose rank is within 1..=10 (and varies across seeds) with the grid ranked ascending; `next_searched_rank` with Random returns an unrevealed, non-mine rank, and with only the mine and one item left returns that item.
- [x] T008 [P] [US3] Component test in `frontend/src/pages/SetupQuiz` test (or `frontend/src/test/SetupQuiz.test.tsx`): the Tripmine side control offers Lowest, Highest, and Random (labelled "Hard").

### Implementation for User Story 3

- [x] T009 [US3] In `backend/src/modules/sessions/mod.rs`: add `TripmineSide::Random` with `parse("random")` and `as_str()` returning `"random"`.
- [x] T010 [US3] In `backend/src/modules/tripmine_round/mod.rs`: for `TripmineSide::Random`, `sample_tripmine_grid` picks a random mined rank (1..=10); `start_tripmine_round` chooses the initial searched rank as a random unrevealed non-mine item; `next_searched_rank` for Random returns a random unrevealed non-mine rank (using the runtime RNG); update `ensure_side_unchanged`'s message to include random.
- [x] T011 [US3] Frontend: in `frontend/src/pages/SetupQuiz.tsx` add the Random (Hard) option to the Tripmine side select; in `frontend/src/types/quiz.ts` widen the Tripmine side type to include `'random'`; in `frontend/src/components/TripmineBoard.tsx` show a label for Random; in `frontend/src/pages/Session.tsx` pass the `random` side through in `toTripmineState`.

**Checkpoint**: Random (Hard) plays with the same scoring and the existing flow.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T012 [P] Add amendment pointers to `specs/001-listquiz-game/contracts/realtime.md` and `specs/001-listquiz-game/contracts/rest-api.md` for the Random side, hidden hearts, and the cue order, referencing `specs/009-tripmine-mode/contracts/tripmine-mode.md`.
- [x] T013 Run the full verification: `cargo fmt --manifest-path backend/Cargo.toml -- --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path backend/Cargo.toml`, `npm --prefix frontend run lint`, `npm --prefix frontend test`, `npm --prefix frontend run build`; then walk quickstart.md manually.
- [x] T014 Confirm build output stayed inside the repo and `frontend/dist/` + `backend/target/` remain untracked (`git status`), per the constitution's workflow step 7.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: starts immediately.
- **Foundational (Phase 2)**: none.
- **User Stories (Phase 3+)**: US1, US2, US3 are independent in rules; US1 T003, US2 T006, and US3 T011 all edit `frontend/src/pages/Session.tsx`, so run them sequentially.
- **Polish (Phase 6)**: after the desired stories.

### Within Each Story

- Tests before implementation (write and fail first).
- T004 (cue list) before T006 (play the list); T009 (side variant) before T010 (random rules).

### Parallel Opportunities

- T002, T004, T007, T008 can be authored together (different files).
- US3 backend (T009/T010) is disjoint from the frontend work.

---

## Parallel Example: User Story 3

```bash
Task: "Backend random-side tests in backend/src/modules/tripmine_round/tests.rs"
Task: "Setup option test for Random (Hard)"
```

---

## Implementation Strategy

### MVP First (User Story 1)

1. Phase 1 Setup → 2. Phase 3 (US1) → **STOP and validate**: hearts hidden in Tripmine.
2. US2: ordered cues (mine explosion before game end).
3. US3: Random (Hard) side.
4. Polish: contract pointers, full verification, build-hygiene check.

---

## Notes

- [P] = different files, no incomplete dependency.
- Tests are written first and must fail before implementation.
- No task commits on its own; commit only when the user asks.
- `the reference frontend` stays read-only (constitution).
