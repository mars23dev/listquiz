# Tasks: Fix Play Findings

**Input**: Design documents from `/specs/002-fix-play-findings/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Tests**: Included — the project constitution requires named, tested business rules, and the codebase
already tests every rule and control, so each fix carries a test.

**Organization**: Tasks are grouped by user story so each fix is independently implementable and
testable.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1…US6)
- Exact file paths are given in each description.

## Path Conventions

- Backend: `backend/src/`, `backend/tests/`
- Frontend: `frontend/src/`

---

## Phase 1: Setup

**Purpose**: Confirm a green baseline before touching anything.

- [x] T001 Verify baseline: `cargo test --manifest-path backend/Cargo.toml` and `npm --prefix frontend test` both pass, and `npm --prefix frontend run build` succeeds.

---

## Phase 2: Foundational

**Purpose**: The one shared request-shape change every player-action task needs.

- [x] T002 Add an optional `role` field (defaulted, serde camelCase) to `ActionRequest` in `backend/src/routes/sessions.rs`. No behaviour change yet.

**Checkpoint**: The action request can carry a requester role.

---

## Phase 3: User Story 1 - The team on turn plays from its own view (Priority: P1) MVP

**Goal**: The active team's player link can perform the in-round actions; every other player view and
the spectator view stay read-only; the gamemaster keeps acting on the team's behalf.

**Independent Test**: In two contexts, the active team's player link can select and confirm an Order
placement and pick a Tripmine cell, while another player link and the spectator link cannot, and the
gamemaster still can.

### Tests for User Story 1

- [x] T003 [P] [US1] Integration test in `backend/tests/player_actions.rs`: an active-team player role (`p2` for team_2 on turn) posts `select_item`/`choose_slot` and `pick_tripitem` without `X-Quiz-Password` and is accepted; a non-active player role and `view` are refused with `not_ready`; the gamemaster path still requires and accepts the password.
- [x] T004 [P] [US1] Component test in `frontend/src/test/Session.test.tsx`: a player role that owns the team on turn gets an interactive board and its actions post with `role` and no password header; a non-active player role is read-only; the spectator view has no controls.

### Implementation for User Story 1

- [x] T005 [US1] Implement the named rule `actor_may_act(conn, session_id, role, team_id) -> ApiResult<()>` in `backend/src/modules/sessions/mod.rs`: the gamemaster may always act; a player role may act only for the team on turn (reuse `scoring::is_active_turn`); any other role or team is `not_ready`.
- [x] T006 [US1] Wire authorization into `session_action` in `backend/src/routes/sessions.rs`: require `X-Quiz-Password` only when `role` is the gamemaster (or absent); for a player role call `actor_may_act`; reject `view`; keep `return_to_pool` gamemaster-only.
- [x] T007 [US1] Make the player board interactive when the role owns the team on turn and send `role` (no password) with player actions for Order and Tripmine in `frontend/src/pages/Session.tsx`; leave spectator read-only and the gamemaster path unchanged.

**Checkpoint**: The active team plays from its own view; other views cannot.

---

## Phase 4: User Story 2 - Start affordance and waiting state (Priority: P1)

**Goal**: Before a round starts, the host sees an obvious start control and the room sees a waiting
message in the board area.

**Independent Test**: On a fresh session, the gamemaster link shows a large start control, a player and
a spectator link show a waiting message, and starting the round replaces both with the live board.

### Tests for User Story 2

- [x] T008 [P] [US2] Component test in `frontend/src/test/Session.test.tsx`: with no board present, the gamemaster sees a start control and players/spectators see a waiting message; after a board exists, neither is shown.

### Implementation for User Story 2

- [x] T009 [US2] Render a large start control (calls the existing `next_round`) for the gamemaster and a waiting-for-start message for players/spectators, shown only while no Order and no Tripmine board exists, in `frontend/src/pages/Session.tsx`.

**Checkpoint**: A fresh session is never a blank page for any role.

---

## Phase 5: User Story 3 - Tripmine grid in random order (Priority: P2)

**Goal**: The ten Tripmine items are shown in a shuffled order unrelated to their ranks; outcomes are
unchanged.

**Independent Test**: Two Tripmine starts on the same list show different item orders, and a pick still
reveals the correct position and advances the search.

### Tests for User Story 3

- [x] T010 [P] [US3] Unit test in `backend/src/modules/tripmine_round/tests.rs`: the stored grid order from two different seeds differs and does not equal the ascending/descending rank order, while each slot's `true_rank` and the single `is_tripmine` are unchanged, and a pick still scores and advances the searched rank.

### Implementation for User Story 3

- [x] T011 [US3] Shuffle the grid array order in `start_tripmine_round` in `backend/src/modules/tripmine_round/mod.rs` before it is stored (each slot keeps its `true_rank`/`is_tripmine`), reusing the module's Fisher–Yates shuffle.

**Checkpoint**: Tripmine is no longer sweepable left-to-right.

---

## Phase 6: User Story 4 - Team names can be typed in Setup (Priority: P2)

**Goal**: A multi-character team name is entered without losing focus.

**Independent Test**: Typing a full team name in Setup retains the whole name and keeps focus.

### Tests for User Story 4

- [x] T012 [P] [US4] Component test in `frontend/src/test/SetupQuiz.test.tsx`: typing a multi-character team name retains the full value and the field stays focused across keystrokes.

### Implementation for User Story 4

- [x] T013 [US4] Key the team-name inputs by stable index (not by the changing team name) in `frontend/src/pages/SetupQuiz.tsx`.

**Checkpoint**: Team naming works.

---

## Phase 7: User Story 6 - Tripmine hint names the target value (Priority: P2)

**Goal**: The Tripmine hint shows the mined item's value next to the highest/lowest text; cell values
stay hidden until picked.

**Independent Test**: The hint shows the target value, no unpicked cell shows a value, and the value
matches the mined item once picked.

### Tests for User Story 6

- [x] T014 [P] [US6] Integration test in `backend/tests/tripmine_actions.rs`: `tripmine.targetValue` is present for gamemaster, player, and spectator roles, equals the mined item's value, and unrevealed `items[].value` stays `null`.

### Implementation for User Story 6

- [x] T015 [US6] Add `target_value: Option<String>` to `TripmineView` in `backend/src/app_state.rs` and compute it in `read_tripmine` in `backend/src/state_reader.rs` from the mined slot's value for the round's column.
- [x] T016 [US6] Add `targetValue` to the Tripmine view type in `frontend/src/types/session.ts` and show it next to the highest/lowest text in `frontend/src/components/TripmineBoard.tsx`.

**Checkpoint**: The room knows the value to hunt without knowing the cell.

---

## Phase 8: User Story 5 - List editor icons and table fit (Priority: P3)

**Goal**: Each editor row shows a real icon control (never the literal word) with a table that fits.

**Independent Test**: Open the editor and confirm icon controls render glyphs and the row fields align
under their headings without an oversized empty column, down to a narrow width.

### Tests for User Story 5

- [x] T017 [P] [US5] Component test in `frontend/src/test/ListManagement.test.tsx`: a row's icon control does not read as the literal placeholder word and exposes an accessible label; the editor table renders name and value fields.

### Implementation for User Story 5

- [x] T018 [US5] Render the chosen icon via the existing icon component with a neutral fallback and an accessible label, and constrain the editor table column widths so fields sit under their headings, in `frontend/src/pages/ListManagement.tsx`.

**Checkpoint**: The editor control reads as an icon and the table fits.

---

## Phase 9: Polish & Cross-Cutting Concerns

- [x] T019 [P] Add a one-line pointer in `specs/001-listquiz-game/contracts/realtime.md` and `contracts/rest-api.md` to the `002-fix-play-findings` amendments, so the "host-only mutations" line is not read as current.
- [x] T020 Run the full verification pass: `cargo fmt --manifest-path backend/Cargo.toml --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, `cargo test`, `npm --prefix frontend run lint`, `npm --prefix frontend test`, `npm --prefix frontend run build`, and the `specs/002-fix-play-findings/quickstart.md` steps.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (T001)**: no dependencies.
- **Foundational (T002)**: after T001; blocks US1 only.
- **User Stories**: US1 needs T002; US2, US3, US4, US6, US5 are independent of T002 and of each other.
- **Polish (T019, T020)**: after the stories they cover.

### User Story Dependencies

- **US1 (P1)**: depends on T002. No dependency on other stories.
- **US2 (P1)**: independent.
- **US3 (P2)**: independent (backend Tripmine module).
- **US4 (P2)**: independent (Setup page).
- **US6 (P2)**: independent of US3 in behaviour, but both touch Tripmine display; run T011 before T015/T016 to avoid confusing the manual check.
- **US5 (P3)**: independent (List page).

### Within Each User Story

- Tests first (fail), then implementation.
- Backend rule (T005) before its route wiring (T006); route before the frontend that depends on it (T007).
- T015 before T016 (backend field before the UI that reads it).

### Parallel Opportunities

- T003 and T004 (different files).
- T008, T010, T012, T014, T017 are single test tasks that can run in parallel once their prerequisites exist.
- US2, US3, US4, US5 touch different files and can proceed in parallel.

---

## Parallel Example: User Story 1

```bash
# Tests first, together:
Task: "Integration test for player-role actions in backend/tests/player_actions.rs"
Task: "Component test for the interactive player board in frontend/src/test/Session.test.tsx"
```

---

## Implementation Strategy

### MVP First (User Story 1)

1. T001 baseline, T002 role field.
2. US1 (T003–T007) → the active team can play from its own link.
3. Validate US1 independently.

### Incremental Delivery

1. US1 (P1) → player play works.
2. US2 (P1) → nothing is a blank page; the host can start.
3. US3, US4, US6, US5 (P2/P3) → each lands and is validated on its own.

---

## Notes

- [P] tasks touch different files with no incomplete dependency.
- Each user story is independently completable and testable.
- Verify tests fail before implementing.
- Commit after each story.
