---
description: "Task list for ListQuiz"
---

# Tasks: ListQuiz

**Input**: Design documents from `/specs/001-listquiz-game/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/rest-api.md,
contracts/realtime.md, contracts/ui-parity.md, quickstart.md

**Tests**: Test tasks ARE included. The project constitution (Principle II) requires every business
rule to be covered by a test, and plan.md names `cargo test`, `vitest`, and Playwright.

**Organization**: Tasks are grouped by user story so each story can be implemented, tested, and
delivered independently.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to
- Every task names an exact file path

## Path Conventions

- Backend: `backend/src/modules/<feature>/`, `backend/src/routes/`
- Frontend: `frontend/src/components/`, `frontend/src/pages/`, `frontend/src/api/`
- Backend tests: `backend/src/modules/<feature>/tests.rs` (unit) and `backend/tests/` (integration)
- Frontend tests: `frontend/src/**/*.test.tsx`
- E2E: `tests/e2e/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization, draft frontend copy, build locality pinning

- [ ] T001 Create the backend crate in `backend/Cargo.toml` with axum, tokio, serde, serde_json, rusqlite (bundled feature), tower-http (fs + gzip), csv, uuid, and dev-dependency `tower` with `util` feature
- [ ] T002 [P] Pin Rust build output into the repository by creating `backend/.cargo/config.toml` with `[build] target-dir = "target"`, satisfying the constitution rule that no compilation writes outside the project folders
- [ ] T003 Initialize the git repository at the repo root with `git init` and an initial commit of `.specify/` and `specs/`, since no repository exists yet
- [ ] T004 Create `.gitignore` at the repo root containing `frontend/dist/`, `backend/target/`, `backend/data/`, `node_modules/`, `.vite/`, and `.specify/.gitignore` stays as-is; verify with `git check-ignore -v frontend/dist backend/target backend/data`
- [ ] T005 Copy the draft frontend's non-page sources into `frontend/src`: `main.tsx`, `index.css`, `App.tsx`, `types/quiz.ts`, and `utils/audio.ts` — do NOT copy `utils/listStorage.ts` or `utils/sessionSync.ts`, do NOT copy the eleven `components/` files (each user story copies the components it needs), and do NOT edit anything under `the reference frontend`
- [ ] T006 Create `frontend/package.json` from the draft's dependencies, add `vitest`, `@testing-library/react`, `@testing-library/user-event`, and `jsdom` as dev dependencies, then run `npm install` in `frontend/`
- [ ] T007 [P] Pin the npm cache into the repository by creating `frontend/.npmrc` with `cache=./.npm-cache`
- [ ] T008 Update `frontend/vite.config.ts` to set `build.outDir` to `dist` and `cacheDir` to `node_modules/.vite`, keeping the draft's `@tailwindcss/vite` and `@vitejs/plugin-react` setup
- [ ] T009 Add a provenance comment to `frontend/src/App.tsx` and `frontend/src/utils/audio.ts` marking them as copied from `the reference frontend`, and confirm no file under `frontend/` imports from or reads `the reference frontend`
- [ ] T010 Configure `vitest` in `frontend/vite.config.ts` with a `jsdom` environment and a `frontend/src/test/setup.ts` file that imports `@testing-library/jest-dom`
- [ ] T011 [P] Create the shared backend test helper at `backend/tests/common/mod.rs` exposing `fn test_db() -> rusqlite::Connection` backed by `Connection::open_in_memory()` with the full schema applied

**Checkpoint**: `cargo build` succeeds, `npm run build` in `frontend/` succeeds, and `git check-ignore`
confirms the artifact folders are ignored.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**CRITICAL**: No user story work can begin until this phase is complete.

- [ ] T012 Create the schema DDL in `backend/src/db.rs` with all twelve tables from `data-model.md`: `folder`, `list_item_column`, `quiz_list`, `item`, `item_value`, `game_session`, `team`, `round_config`, `order_state`, `tripmine_state`, `score_event`, including `PRIMARY KEY (item_id, column_id)` on `item_value`, `PRIMARY KEY (session_id, round_number)` on `round_config`, and `ON DELETE CASCADE` from `folder` to `quiz_list`, `quiz_list` to `item`, `item` to `item_value`
- [ ] T013 Add the schema statements from T012 to `backend/tests/common/mod.rs` so every integration test gets an in-memory database with the same schema
- [ ] T014 Implement `open_db(path)` and `seed_default_folder(conn)` in `backend/src/db.rs`, opening the file at `backend/data/listquiz.db`, enabling WAL journaling, and creating the directory if absent
- [ ] T015 Create the error type in `backend/src/errors.rs` mapping to the codes in `contracts/rest-api.md`: `bad_request` 400, `unauthorized` 400 with the message `Incorrect password!`, `not_found` 404, `conflict` 409, `too_small` 422, `too_large` 422, `not_ready` 409, serialized as `{ "error": { "code": ..., "message": ... } }`
- [ ] T016 Implement `build_public_state(session, role)` in `backend/src/app_state.rs` applying the redaction table from `contracts/realtime.md`: for non-gamemaster roles set `value` to `null` on un-revealed Order and Tripmine items, set `isTripmine` only on revealed items, keep the anchor's value public, and never include host action controls
- [ ] T017 Create `backend/src/app_state.rs` holding the `rusqlite::Connection` behind `std::sync::Mutex` and a `tokio::sync::broadcast::Sender<u64>` for session revisions
- [ ] T018 [P] Implement the shared scoring rules in `backend/src/modules/scoring/mod.rs` as pure functions over plain data: `apply_points`, `apply_hearts` (clamped 0-3), `advance_turn` (next team with `hearts > 0`, skipping exhausted teams), `starting_team_for_round(round_number, team_count)` returning `team_{((round_number - 1) mod team_count) + 1}`, and `reset_hearts_for_round`, with every function emitting a `score_event` row and none touching the database
- [ ] T019 Write unit tests in `backend/src/modules/scoring/tests.rs` covering: `+10` and `-10` point deltas, hearts clamped at 0 and 3, a team at 0 hearts skipped by `advance_turn`, rotation across rounds 1-3 with 3 teams, and a 1-team session whose turn never leaves team 1
- [ ] T020 [P] Implement the team rules in `backend/src/modules/teams/mod.rs`: count validation "1 to 4", the fixed colour set `cyan` / `rose` / `amber` / `emerald` with primary hexes `#06b6d4`, `#f43f5e`, `#f59e0b`, `#10b981` and the draft's accent glow values, default names from the draft's presets, and defaulting an unnamed team to its slot default
- [ ] T021 Write unit tests in `backend/src/modules/teams/tests.rs` for count rejection at 0 and 5, unique colour assignment after removing a team, and default-name fallback
- [ ] T022 [P] Create the seed data in `backend/src/modules/default_lists/mod.rs` holding the draft's three lists — `World History & Inventions` (column `Year`, 11 items), `Iconic Statues & Architectural Marvels` (columns `Height (m)` and `Year Built`, 11 items), `Blockbuster Cinema Box Office` (columns `Box Office ($)` and `Release Year`, 11 items) — copied verbatim from `the reference frontend's data/quizData.ts` and `the reference frontend's utils/listStorage.ts`, including icon names, with `BC` years stored as negative numbers and `$2.26 Billion` stored as `2260`
- [ ] T023 Create the router in `backend/src/main.rs` with `ServeDir` for `frontend/dist`, an `index.html` fallback for client routes (`/`, `/lists`, `/setup`, `/{sessionId}`, `/{sessionId}/gamemaster`, `/{sessionId}/p1`–`p4`, `/{sessionId}/view`), the `/api` router mount, and a listener on `0.0.0.0:8080`
- [ ] T024 [P] Implement the SSE endpoint in `backend/src/routes/stream.rs` at `GET /api/sessions/{id}/events?role=<role>`, emitting `event: state` with `{"revision":N}` on every broadcast, a `heartbeat` event every 15 seconds, and returning `not_found` for an unknown session or role before the stream opens
- [ ] T025 [P] Create the typed API client in `frontend/src/api/client.ts` covering every endpoint in `contracts/rest-api.md`, sending the `X-Quiz-Password` header and mapping the error shape to a typed error
- [ ] T026 [P] Create the SSE subscription hook in `frontend/src/hooks/useSessionStream.ts` that opens the stream, refetches state only when `revision` increases, retries with backoff, and never triggers a page reload
- [ ] T027 Extend `frontend/src/types/quiz.ts` with the fields added by `data-model.md`: `ScoreEvent`, `RoundConfig.tripmineSide`, `Team.turnIndex`, `ListItemValue.wasDefaulted`, and the per-role state types from `contracts/realtime.md`

**Checkpoint**: Foundation ready — `cargo test` passes for `scoring` and `teams`, the backend starts,
serves the exported frontend, and `/api/sessions/{id}/events` responds. User story work can begin.

---

## Phase 3: User Story 1 - Host a quiz session (Priority: P1) - MVP

**Goal**: Create a session with 1-10 rounds, 1-4 named teams, a required gamemaster password, and
five role links; gate the gamemaster link behind the password.

**Independent Test**: Create a 2-round, 2-team session; confirm the five role links are produced, a wrong
password is refused, the right one is accepted, and both teams start on 0 points and 3 hearts.

### Tests for User Story 1

> Write these first and confirm they FAIL before implementing.

- [ ] T028 [US1] Integration test: `POST /api/sessions` with `gmPassword` empty returns `bad_request` in `backend/tests/session_create.rs`
- [ ] T029 [US1] Integration test: `POST /api/sessions` with `roundCount` 0 and 11 returns `bad_request` in `backend/tests/session_create.rs`
- [ ] T030 [US1] Integration test: `POST /api/sessions` with `teamCount` 0 and 5 returns `bad_request` in `backend/tests/session_create.rs`
- [ ] T031 [US1] Integration test: a created session returns exactly five role links: one gamemaster link, all four player links p1 through p4 each carrying a `hasTeam` flag for its slot, and one spectator link, in `backend/tests/session_create.rs`
- [ ] T032 [P] [US1] Integration test: `POST /api/sessions/{id}/unlock` with a wrong password returns `unauthorized` and the message `Incorrect password!` in `backend/tests/session_unlock.rs`
- [ ] T033 [P] [US1] Integration test: a round missing `quizType` or `listId` cannot start and `POST /api/sessions/{id}/actions` with `next_round` returns `not_ready` in `backend/tests/session_lifecycle.rs`

### Implementation for User Story 1

- [ ] T034 [US1] Implement session creation in `backend/src/modules/sessions/mod.rs`: validate `roundCount` in 1-10 and `teamCount` in 1-4, require a non-empty `gmPassword`, verify every referenced `listId` and `columnId` exists, and insert `game_session`, `team`, and `round_config` rows per `data-model.md`
- [ ] T035 [US1] Implement the session identifier generator in `backend/src/modules/sessions/mod.rs` producing `lq-` plus 8 random lowercase alphanumerics, retrying on the unique-index collision
- [ ] T036 [P] [US1] Write unit tests in `backend/src/modules/sessions/tests.rs` for identifier format, round-count bounds, and the rule that a Tripmine round requires `tripmineSide` while an Order round must not carry one
- [ ] T037 [US1] Add the HTTP surface in `backend/src/routes/sessions.rs`: `POST /api/sessions`, `POST /api/sessions/{id}/unlock`, and `GET /api/sessions/{id}/state?role=<role>` returning `not_found` for an unknown session
- [ ] T038 [US1] Build the role links in `backend/src/routes/sessions.rs` as `/lq-xxxxxxxx/gamemaster`, all four player links `/lq-xxxxxxxx/p1`–`/lq-xxxxxxxx/p4` each carrying a `hasTeam` flag for its slot, and `/lq-xxxxxxxx/view`, plus the `/lq-xxxxxxxx` → `p1` redirect in `backend/src/main.rs`
- [ ] T039 [US1] Build the three-step setup wizard in `frontend/src/pages/SetupQuiz.tsx`, correcting the draft copy: `Step 1: Participants & Password` makes the password mandatory, team count buttons become `1 Team` through `4 Teams`, and `Step 2: Round Configuration` supports 1-10 rounds with add/remove rows
- [ ] T040 [US1] Build the session links step in `frontend/src/pages/SetupQuiz.tsx` keeping the draft's `Session Ready:`, `HOST` badge, `Copy Link`, `Launch GM`, per-player pills, `PUBLIC` spectator row, and the `Start Game as Gamemaster` button
- [ ] T041 [US1] Wire the wizard to the API in `frontend/src/pages/SetupQuiz.tsx` using `frontend/src/api/client.ts`, removing the draft's local `initNewSession` call
- [ ] T042 [US1] Build the gamemaster password gate in `frontend/src/pages/Session.tsx` using the draft's exact copy: `Gamemaster Access`, `Enter the moderator password to unlock host controls.`, placeholder `Enter password...`, error `Incorrect password!`, buttons `Switch to View` and `Unlock`
- [ ] T043 [US1] Build the landing page in `frontend/src/pages/Landing.tsx` from the draft's `LandingPage.tsx`, keeping the `LIST`+`QUIZ` wordmark and both cards, and removing the `Quick Start Sample` action

**Checkpoint**: A gamemaster can create a session and unlock their view. Test independently before
moving on.

---

## Phase 4: User Story 2 - Prepare lists in password-protected folders (Priority: P1)

**Goal**: Folder-gated list access — create a password-protected folder, unlock it to see its lists,
with a fixed read-only default folder and no way to create a list outside a folder.

**Independent Test**: Create a folder with a password, confirm a locked folder reveals no list names,
confirm the default folder cannot be written to, and confirm the sensitive-data hint appears on the
editor and import screens.

### Tests for User Story 2

- [ ] T044 [US2] Integration test: `POST /api/folders` with an empty name or empty password returns `bad_request` in `backend/tests/folder_lifecycle.rs`
- [ ] T045 [US2] Integration test: `POST /api/folders` with a duplicate name returns `conflict` in `backend/tests/folder_lifecycle.rs`
- [ ] T046 [US2] Integration test: unlocking with a wrong password returns `unauthorized` and no list data in `backend/tests/folder_lifecycle.rs`
- [ ] T047 [US2] Integration test: any write aimed at the default folder returns `conflict` in `backend/tests/folder_lifecycle.rs`
- [ ] T048 [US2] Integration test: `GET /api/folders` reports `locked: true` for an unlocked-by-nobody folder and never includes list names for it in `backend/tests/folder_lifecycle.rs`
- [ ] T049 [P] [US2] Integration test: `POST /api/folders/{id}/delete` with a wrong `confirmPassword` returns `unauthorized`, and with the correct one removes its lists while leaving the default folder intact in `backend/tests/folder_delete.rs`
- [ ] T050 [US2] Integration test: `POST /api/folders/{id}/rename` to an existing name returns `conflict` in `backend/tests/folder_lifecycle.rs`

### Implementation for User Story 2

- [ ] T051 [US2] Implement the folder rules in `backend/src/modules/folders/mod.rs` enforcing the `data-model.md` constraints: name "trimmed, 1-60 chars, unique among non-default folders", password non-empty except for `id = 'default'`, identifiers `fld-<8 random alphanumerics>`
- [ ] T052 [P] [US2] Write unit tests in `backend/src/modules/folders/tests.rs` for name trimming, the 60-character bound, and the default folder's empty-password exemption
- [ ] T053 [US2] Implement `unlock`, `rename`, and `delete` in `backend/src/modules/folders/mod.rs`, with delete requiring the password re-entered as confirmation, returning the folder's list count, and cascading deletion to its lists only
- [ ] T054 [US2] Add the HTTP surface in `backend/src/routes/folders.rs`: `POST /api/folders`, `POST /api/folders/{id}/unlock`, `POST /api/folders/{id}/rename`, `POST /api/folders/{id}/delete` (204), and `GET /api/folders`
- [ ] T055 [US2] Build the folder bar in `frontend/src/pages/ListManagement.tsx` above the draft's list grid: create folder, unlock, rename, and delete with a password confirmation that displays the list count
- [ ] T056 [US2] Render the default folder read-only in `frontend/src/pages/ListManagement.tsx`, hiding `Create List`, `Import CSV`, `Duplicate List`, `Delete List`, and rename controls for its cards
- [ ] T057 [US2] Add the sensitive-data hint to `frontend/src/pages/ListManagement.tsx` on both the list editor and the CSV import modal, stating the data is not protected and that sensitive data must not be added
- [ ] T058 [US2] Add `GET /api/default-lists` in `backend/src/routes/lists.rs` returning the seeded lists with no password, calling `backend/src/modules/default_lists/mod.rs`

**Checkpoint**: US1 and US2 both work independently. Locked folders expose nothing.

---

## Phase 5: User Story 3 - Play an Order round (Priority: P1)

**Goal**: The 11-slot bar, 3 hearts, 10-point correct placements, heart loss on a wrong placement, and
the per-button value reveal.

**Independent Test**: Play a full Order round with two teams and confirm `+10` for a correct placement,
no points and `-1` heart for a wrong one, the turn passing either way, and a 0-heart team skipped.

### Tests for User Story 3

- [ ] T059 [US3] Unit test: `sample_order_round` picks 5 items below the anchor and 5 above, never the anchor, in `backend/src/modules/order_round/tests.rs`
- [ ] T060 [US3] Unit test: a correct left placement awards exactly 10 points and a correct right placement awards exactly 10 points in `backend/src/modules/order_round/tests.rs`
- [ ] T061 [US3] Unit test: a wrong-side placement awards 0 points and removes exactly 1 heart in `backend/src/modules/order_round/tests.rs`
- [ ] T062 [US3] Unit test: `validate_placement` accepts only left, between-two, and right positions, and rejects an out-of-range `slot_index` in `backend/src/modules/order_round/tests.rs`
- [ ] T063 [US3] Unit test: an item with a value equal to an existing neighbour is accepted on the correct side in `backend/src/modules/order_round/tests.rs`
- [ ] T064 [US3] Unit test: the bar keeps placement order and is never re-sorted by value in `backend/src/modules/order_round/tests.rs`
- [ ] T065 [US3] Unit test: the round ends when all items are placed or when no team has hearts in `backend/src/modules/order_round/tests.rs`
- [ ] T066 [US3] Integration test: `reveal_values` sets `revealValues` on the session and un-redacts values for player roles in `backend/tests/order_actions.rs`
- [ ] T067 [US3] Integration test: `select_item` and `choose_slot` from a team that is not on turn return `not_ready` in `backend/tests/order_actions.rs`

### Implementation for User Story 3

- [ ] T068 [US3] Implement `start_order_round(list, column_id, order_direction, rng)` in `backend/src/modules/order_round/mod.rs`: sort by the column's numeric value in the list's direction, take the middle item as the anchor with `FR-026`'s distinct-colour marker, then sample 5 at random from below and 5 at random from above
- [ ] T069 [US3] Write unit tests in `backend/src/modules/order_round/tests.rs` for `start_order_round`'s anchor selection at list lengths 11 and 20, including topping up the shorter side when the anchor is not centred
- [ ] T070 [US3] Implement `validate_placement(state, side, slot_index)` in `backend/src/modules/order_round/mod.rs` checking the anchor side, the sequence order of the affected side, and the equal-value case
- [ ] T071 [US3] Implement `confirm_placement` and `return_to_pool` in `backend/src/modules/order_round/mod.rs`, delegating every point and heart change to `backend/src/modules/scoring/` and recording a `score_event` with reason `correct_placement` or `game_master`
- [ ] T072 [US3] Persist `order_state` in `backend/src/modules/order_round/mod.rs` using the JSON columns `pool`, `left_ids`, `right_ids`, and `pending` from `data-model.md`
- [ ] T073 [US3] Add the Order actions to `backend/src/routes/sessions.rs`: `select_item`, `choose_slot`, `cancel_selection`, `return_to_pool`, `reveal_values`
- [ ] T074 [US3] Copy `the reference frontend's components/TopBarOrdering.tsx` to `frontend/src/components/TopBarOrdering.tsx` unchanged, including the 5-left / anchor / 5-right layout, the `CONFIRM PLACEMENT` and `CANCEL` buttons, and the pending slot
- [ ] T075 [US3] Copy `the reference frontend's components/MiddleItemGrid.tsx` to `frontend/src/components/MiddleItemGrid.tsx` unchanged, including the 2-to-5 column responsive grid and hiding placed items
- [ ] T076 [US3] Convert both board components to keyboard operation in `frontend/src/components/TopBarOrdering.tsx` and `frontend/src/components/MiddleItemGrid.tsx`: every pool item and bar slot becomes a `button` in DOM order with a visible focus ring, and confirm or cancel is reachable by keystroke
- [ ] T077 [US3] Wire the Order board to the API in `frontend/src/pages/Session.tsx`, keeping the draft's waiting banner copy `Waiting for <team> to place an item...`
- [ ] T078 [US3] Add the reveal control to the gamemaster dashboard in `frontend/src/pages/Session.tsx` calling `reveal_values` for both quiz types

**Checkpoint**: US1, US2, US3 independently functional.

---

## Phase 6: User Story 4 - Play a Tripmine round (Priority: P1)

**Goal**: The 10-item grid, gamemaster-chosen tripmine side, position reveal, and the four scoring
outcomes including the 20-point final find.

**Independent Test**: Play a full Tripmine round and confirm `+10` for the searched item, `+5` for a
wrong non-tripmine pick, `-10` for the picker plus `+10` for everyone else on a tripmine hit, and
`+20` when the valid item is picked with two unrevealed items left.

### Tests for User Story 4

- [ ] T079 [US4] Unit test: `start_tripmine_round` picks 10 items, assigns `true_rank` 1 (lowest) to 10 (highest), and marks the tripmine as rank 10 when the side is `highest` and rank 1 when `lowest` in `backend/src/modules/tripmine_round/tests.rs`
- [ ] T080 [US4] Unit test: the initial `searched_rank` is 1 for a `highest` tripmine and 10 for a `lowest` tripmine in `backend/src/modules/tripmine_round/tests.rs`
- [ ] T081 [US4] Unit test: picking the searched position awards 10 and advances `searched_rank` to the next unrevealed position in the search direction in `backend/src/modules/tripmine_round/tests.rs`
- [ ] T082 [US4] Unit test: picking a wrong non-tripmine item awards 5, reveals only that position, and leaves `searched_rank` unchanged in `backend/src/modules/tripmine_round/tests.rs`
- [ ] T083 [US4] Unit test: picking the tripmine removes 10 from the picker, adds 10 to every other team, reveals all positions, and ends the round in `backend/src/modules/tripmine_round/tests.rs`
- [ ] T084 [US4] Unit test: with exactly two unrevealed items a correct pick awards 20 instead of 10 and ends the round in `backend/src/modules/tripmine_round/tests.rs`
- [ ] T085 [US4] Unit test: a pick from a team that is not on turn is refused, except for the gamemaster acting on the active team's behalf in `backend/src/modules/tripmine_round/tests.rs`
- [ ] T086 [US4] Unit test: changing the tripmine side after the round starts is refused in `backend/src/modules/tripmine_round/tests.rs`
- [ ] T087 [P] [US4] Integration test: a player role never receives `isTripmine: true` for an unrevealed item in `backend/tests/tripmine_actions.rs`

### Implementation for User Story 4

- [ ] T088 [US4] Implement `start_tripmine_round(list, column_id, side, rng)` in `backend/src/modules/tripmine_round/mod.rs` sampling 10 items, ranking them by numeric value ascending, setting `searched_rank` from the side, and storing the JSON `items` array from `data-model.md`
- [ ] T089 [US4] Implement `pick_item` in `backend/src/modules/tripmine_round/mod.rs` with the four branches — searched, wrong non-tripmine, tripmine hit, and the two-remaining final find — delegating scoring to `backend/src/modules/scoring/` with reasons `searched_item_found`, `safe_pick`, `tripmine_hit`, and `final_item`
- [ ] T090 [US4] Implement `next_searched_rank` in `backend/src/modules/tripmine_round/mod.rs` scanning upward from the hit rank when the tripmine is highest and downward when it is lowest, skipping already-revealed positions
- [ ] T091 [US4] Persist `tripmine_state` in `backend/src/modules/tripmine_round/mod.rs`, replacing it only at a round start so the ten items never change mid-round
- [ ] T092 [US4] Add the `pick_tripitem` action to `backend/src/routes/sessions.rs` and reject a side change once `status = 'running'`
- [ ] T093 [US4] Copy `the reference frontend's components/TripmineBoard.tsx` to `frontend/src/components/TripmineBoard.tsx` unchanged, including the searched-rank display, the grid, and the draft's feedback strings `BOOM!`, `PERFECT DEFUSE!`, `TARGET HIT!`, and `Safe pick!`
- [ ] T094 [US4] Remove the in-round tripmine side switch from `frontend/src/components/TripmineBoard.tsx` and move it to setup only, per the clarification that the side is locked once play begins
- [ ] T095 [US4] Wire the Tripmine board to the API in `frontend/src/pages/Session.tsx`, keeping the draft's grid behaviour and making every cell a keyboard-reachable `button`

**Checkpoint**: Both quiz types work independently.

---

## Phase 7: User Story 5 - Watch and steer from separate views (Priority: P2)

**Goal**: Five role views on separate devices converging on one board state, with read-only player and
spectator views and no hidden value leakage.

**Independent Test**: Open the gamemaster link and a player link in two browser contexts, act as
gamemaster, and confirm the player view updates within a second with no host controls and no unrevealed
values.

### Tests for User Story 5

- [ ] T096 [US5] Integration test: a gamemaster action is visible on the next state read for the player role in `backend/tests/session_sync.rs`
- [ ] T097 [US5] Integration test: a player role state contains no `pending`, no control list, and no un-revealed values in `backend/tests/session_sync.rs`
- [ ] T098 [US5] Integration test: an unknown session id returns `not_found` on state read in `backend/tests/session_sync.rs`
- [ ] T099 [US5] Integration test: the revision counter increases after every accepted action in `backend/tests/session_sync.rs`
- [ ] T100 [US5] Integration test: `set_muted` applies to every role's state in `backend/tests/session_sync.rs`

### Implementation for User Story 5

- [ ] T101 [US5] Wire `GET /api/sessions/{id}/events` into `frontend/src/hooks/useSessionStream.ts` so a state event with a newer revision triggers exactly one refetch, and a heartbeat with no change triggers none
- [ ] T102 [US5] Build the session view shell in `frontend/src/pages/Session.tsx` keeping the draft's top bar with `ListQuiz`, the session id link, `Round N of M (TYPE)`, and the role switcher labelled `Role:` with `GM`, `P1`–`P4`, and `View`
- [ ] T103 [US5] Build the perspective banner in `frontend/src/pages/Session.tsx` keeping `Moderator Controls Active`, `Stage Broadcast Screen`, and the player variants `(YOUR TURN!)` and `(Waiting...)`
- [ ] T104 [US5] Remove the draft's timer block entirely from `frontend/src/pages/Session.tsx` — the `Counter:` control, the `15s`/`20s`/`30s` buttons, the `Stop` button, and the `Turn Timer Running:` bar — since the product has no timer
- [ ] T105 [US5] Build the read-only player and spectator rendering in `frontend/src/pages/Session.tsx` so no control issues an action, and the spectator view shows no turn emphasis and no team badge
- [ ] T106 [P] [US5] Copy `the reference frontend's components/BottomTeams.tsx` to `frontend/src/components/BottomTeams.tsx` unchanged, including colour, score, hearts, active-turn highlight, and the `YOU` badge
- [ ] T107 [US5] Copy `the reference frontend's components/StageLighting.tsx` to `frontend/src/components/StageLighting.tsx` unchanged and render it in `frontend/src/pages/Session.tsx`
- [ ] T108 [US5] Add the mute toggle to the gamemaster dashboard in `frontend/src/pages/Session.tsx`, calling `set_muted`, and suppress audio in `frontend/src/utils/audio.ts` when the session is muted
- [ ] T109 [US5] Add the not-found view in `frontend/src/pages/Session.tsx` shown when the state read returns `not_found`, replacing the draft's blank board

**Checkpoint**: Two devices stay in step; no leakage.

---

## Phase 8: User Story 6 - Manage list content and ordering (Priority: P2)

**Goal**: Manual list editing with drag ordering, CSV import and export, and the 11-100 item rule.

**Independent Test**: Create an 11-item list by hand, import an equivalent CSV, confirm both produce a
usable round, confirm a 10-row import is rejected naming the range, and confirm dragging changes item
order.

### Tests for User Story 6

- [ ] T110 [US6] Unit test: `parse_csv` detects a leading `Icon` header case-insensitively and maps the remaining headers to value columns in `backend/src/modules/csv_import/tests.rs`
- [ ] T111 [US6] Unit test: `parse_csv` keeps a quoted field containing a comma whole in `backend/src/modules/csv_import/tests.rs`
- [ ] T112 [US6] Unit test: a non-numeric value falls back to the row position, sets `wasDefaulted`, and is reported in `backend/src/modules/csv_import/tests.rs`
- [ ] T113 [US6] Unit test: a CSV with 10 data rows returns `too_small` and one with 101 returns `too_large`, both naming the 11-100 range in `backend/src/modules/csv_import/tests.rs`
- [ ] T114 [US6] Unit test: `export_csv` emits the header `Icon,Name,<column names...>` and re-imports to an equivalent list including the direction in `backend/src/modules/csv_import/tests.rs`
- [ ] T115 [US6] Unit test: creating or updating a list with fewer than 11 or more than 100 items is rejected, and an empty title is rejected in `backend/src/modules/lists/tests.rs`
- [ ] T116 [US6] Unit test: item `position` values are stored exactly as supplied in `backend/src/modules/lists/tests.rs`
- [ ] T117 [P] [US6] Integration test: `GET /api/lists/{id}/export.csv` returns `text/csv` and `POST /api/folders/{id}/lists/import` returns the created list plus a report in `backend/tests/list_crud.rs`

### Implementation for User Story 6

- [ ] T118 [US6] Implement `parse_csv` in `backend/src/modules/csv_import/mod.rs` using the `csv` crate, with the numeric extraction rule "strips everything except digits, `-` and `.`", and `parseFloat`, falling back to `row_index * 10`-equivalent position order when unparseable
- [ ] T119 [US6] Implement `export_csv` in `backend/src/modules/csv_import/mod.rs` preserving column order and the list's direction
- [ ] T120 [US6] Implement the list rules in `backend/src/modules/lists/mod.rs` enforcing the `data-model.md` constraints: title "1-80 chars, mandatory", item count "11-100 enforced on every write path", "at least one value column per list", `icon_name` and `icon_image` mutually exclusive, and a list in the default folder rejected
- [ ] T121 [US6] Implement list CRUD in `backend/src/modules/lists/mod.rs`: create, read, update, delete, duplicate with a ` (Copy)` suffix, and the `columns[0]` default-value-column rule
- [ ] T122 [US6] Add the HTTP surface in `backend/src/routes/lists.rs`: `GET /api/folders/{id}/lists`, `GET /api/lists/{id}`, `POST /api/folders/{id}/lists`, `PUT /api/lists/{id}`, `DELETE /api/lists/{id}`, `GET /api/lists/{id}/export.csv`, `POST /api/folders/{id}/lists/import`
- [ ] T123 [P] [US6] Copy `the reference frontend's components/IconPickerModal.tsx` and `the reference frontend's components/ItemIcon.tsx` into `frontend/src/components/`, unchanged
- [ ] T124 [US6] Build the list editor in `frontend/src/pages/ListManagement.tsx` from the draft's editor, keeping `List Form & Table Editor`, `List Title (Mandatory)`, `Order Direction` with `Low to High (Ascending / Earliest to Latest)` and `High to Low (Descending / Highest to Lowest)`, `Table Columns`, `Add Value Column`, `Add Row`, the drag grip, and the `Drag the grip icon on the left to rearrange rows` hint
- [ ] T125 [US6] Add inline 11-100 item-count validation and the mandatory-title validation to `frontend/src/pages/ListManagement.tsx`
- [ ] T126 [US6] Build the CSV import modal in `frontend/src/pages/ListManagement.tsx` keeping `Import List from CSV`, `Upload .csv File OR Paste CSV Below`, the `Icon,Name,Year,Value2` placeholder, and the cancel and `Import List` actions, and render the import report's `defaultedRows` warnings
- [ ] T127 [US6] Add the `Select List:` and `Value Column:` dropdowns to the round configuration step in `frontend/src/pages/SetupQuiz.tsx`, showing `{title} ({itemCount} items)` and offering the value-column picker only when the list has more than one column

**Checkpoint**: US6 works with US2's folders. Round configuration can use any unlocked list.

---

## Phase 9: User Story 7 - Carry scores across rounds and finish the game (Priority: P2)

**Goal**: Points accumulate across rounds, the first turn rotates one team per round, hearts reset each
round, and the game ends with a final ranking.

**Independent Test**: Play two rounds and confirm round two opens on round one's totals, the starting
team rotates, and the final screen shows the cross-round ranking.

### Tests for User Story 7

- [ ] T128 [US7] Unit test: `starting_team_for_round` returns team 1 in round 1, team 2 in round 2, and wraps at the team count in `backend/src/modules/scoring/tests.rs`
- [ ] T129 [US7] Unit test: the first turn of a round skips a team with 0 hearts in `backend/src/modules/scoring/tests.rs`
- [ ] T130 [US7] Unit test: starting a round resets hearts to 3 and leaves scores untouched in `backend/src/modules/scoring/tests.rs`
- [ ] T131 [US7] Unit test: crossing the last round sets status `finished` and enables the full reveal in `backend/src/modules/sessions/tests.rs`
- [ ] T132 [US7] Integration test: a team's round-two opening score equals its round-one closing score in `backend/tests/round_progression.rs`
- [ ] T133 [US7] Integration test: `next_round` beyond the last round returns `not_ready` in `backend/tests/round_progression.rs`

### Implementation for User Story 7

- [ ] T134 [US7] Implement `advance_round` in `backend/src/modules/sessions/mod.rs`: reset hearts to 3, set the active team from `starting_team_for_round`, rebuild `order_state` or `tripmine_state` for the new round's list and column, and set `lastMessage` to `Round N Started: <list title> (<QUIZ_TYPE>)`
- [ ] T135 [US7] Implement game completion in `backend/src/modules/sessions/mod.rs` when the last round ends: set `status = 'finished'`, set `revealValues` true, and expose the aggregated `score_event` totals as the final ranking
- [ ] T136 [US7] Add `next_round`, `pass_turn`, `set_turn`, `adjust_score`, and `adjust_hearts` to `backend/src/routes/sessions.rs`, clamping hearts to 0-3 and recording a `manual_adjustment` score event with reason `game_master`
- [ ] T137 [US7] Build the gamemaster dashboard controls in `frontend/src/pages/Session.tsx` keeping the draft's `Pass Turn` and `Next Round` buttons and its score and heart steppers
- [ ] T138 [US7] Build the final ranking view in `frontend/src/pages/Session.tsx` showing cross-round totals with each team's colour, and the full value reveal

**Checkpoint**: US1 through US7 all independently functional.

---

## Phase 10: User Story 8 - Use the interface in either theme and without a mouse (Priority: P3)

**Goal**: A real dark and light theme that persists, plus full keyboard operability and no blocking on
any screen.

**Independent Test**: Complete setup, a round of each quiz type, and list management using only the
keyboard in each theme, with no input freeze and no full-page reload.

### Tests for User Story 8

- [ ] T139 [P] [US8] Unit test: every required CSS variable resolves under both `:root[data-theme="dark"]` and `:root[data-theme="light"]` in `frontend/src/theme/theme.test.ts`
- [ ] T140 [P] [US8] Component test: the theme toggle flips `data-theme` and persists the choice in `frontend/src/components/ThemeToggle.test.tsx`
- [ ] T141 [P] [US8] Component test: the pool grid is fully reachable and operable by keyboard, with a visible focus indicator, in `frontend/src/components/MiddleItemGrid.test.tsx`
- [ ] T142 [P] [US8] Component test: a bar slot can be chosen and the pending placement confirmed or cancelled by keyboard alone in `frontend/src/components/TopBarOrdering.test.tsx`
- [ ] T143 [P] [US8] Component test: a Tripmine grid cell is reachable and operable by keyboard alone in `frontend/src/components/TripmineBoard.test.tsx`
- [ ] T144 [P] [US8] Component test: the setup wizard, list editor, and import modal are fully keyboard operable in `frontend/src/pages/SetupQuiz.test.tsx` and `frontend/src/pages/ListManagement.test.tsx`

### Implementation for User Story 8

- [ ] T145 [US8] Define the palette in `frontend/src/index.css` as CSS custom properties under `:root[data-theme="dark"]` and `:root[data-theme="light"]`, keeping the draft's team accents `#06b6d4`, `#f43f5e`, `#f59e0b`, `#10b981` and status colours identical in both themes
- [ ] T146 [US8] Create the theme context in `frontend/src/theme/ThemeProvider.tsx` that reads the operating system preference on first visit, applies the `data-theme` attribute, and stores the override locally
- [ ] T147 [US8] Replace hardcoded palette classes with theme variables across `frontend/src/components/StageLighting.tsx`, `frontend/src/components/BottomTeams.tsx`, `frontend/src/components/TopBarOrdering.tsx`, `frontend/src/components/MiddleItemGrid.tsx`, `frontend/src/components/TripmineBoard.tsx`, `frontend/src/pages/Landing.tsx`, `frontend/src/pages/ListManagement.tsx`, and `frontend/src/pages/Session.tsx`, changing no visual decision the draft made in dark mode
- [ ] T148 [P] [US8] Build the theme toggle in `frontend/src/components/ThemeToggle.tsx` and place it in the landing, list-management, and session headers
- [ ] T149 [US8] Add focus-visible styling in `frontend/src/index.css` so every interactive control shows a visible focus indicator
- [ ] T150 [US8] Verify in `frontend/src/App.tsx` that no local action triggers a full-page reload and that state survives a theme switch without loss
- [ ] T151 [P] [US8] Add the Playwright harness in `tests/e2e/` with two browser contexts on one run — a gamemaster context and a player context — and the scenario steps 4 through 8 of `quickstart.md`

---

## Phase 11: Polish & Cross-Cutting Concerns

- [ ] T152 [P] Run the full verification pass from `quickstart.md` and record the outcome of every step, including the AppData check for escaped build state and the `git check-ignore` check
- [ ] T153 [P] Verify the 1:1 parity contract in `contracts/ui-parity.md`: confirm no file under `frontend/` references `the reference frontend` and that every deviation is limited to the eight Design Authority corrections
- [ ] T154 [P] Add the `README.md` at the repo root with the prerequisites, build and run commands from `quickstart.md`, and a pointer to `specs/001-listquiz-game/`
- [ ] T155 Confirm `cargo test` and `npm run test` pass with no skipped test and no test requiring a running HTTP server, per constitution Principle II
- [ ] T156 Run a contrast review of both themes on every screen and record the result against SC-012
- [ ] T157 Remove the Sync Impact Report comment from `.specify/memory/constitution.md` before committing, per the report's own instruction

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 — BLOCKS all user stories
- **User Stories (Phases 3-10)**: All depend on Phase 2 completion; then proceed in priority order
- **Polish (Phase 11)**: Depends on the desired stories being complete

### User Story Dependencies

- **US1 (P1)**: No story dependency. Uses the seeded default folder's lists.
- **US2 (P1)**: No story dependency. Must land before US6 so lists have a home.
- **US3 (P1)**: No story dependency beyond `scoring`. Can start after Phase 2.
- **US4 (P1)**: No story dependency beyond `scoring`. Can start after Phase 2.
- **US5 (P2)**: Depends on US1 for sessions and on US3 or US4 for a board to watch; its redaction work can start after Phase 2.
- **US6 (P2)**: Depends on US2 for folder-gated storage.
- **US7 (P2)**: Depends on US3 and US4 for rounds, and on US1 for sessions.
- **US8 (P3)**: Touches every screen, so apply it after the screens it restyles exist; its theme variables and test harness can start after Phase 2.

### Within Each User Story

- Tests MUST be written and confirmed failing before implementation
- Models and pure rules before services
- Pure rules before HTTP routes
- Core implementation before frontend wiring
- Story complete before moving to the next priority

### Parallel Opportunities

- All `[P]` tasks in Phase 1 and Phase 2
- After Phase 2: US2, US3, US4 can run fully in parallel with each other; US1 alongside them
- After US1: US5 can start; US6 needs US2
- Tests within a story marked `[P]` run in parallel; a story's frontend tasks marked `[P]` run
  in parallel with its backend tasks

---

## Parallel Example: User Story 3

```bash
# All Order unit tests together, written first and expected to fail
Task: "Unit test: sample_order_round picks 5 below and 5 above, never the anchor"
Task: "Unit test: a correct left placement awards exactly 10 points"
Task: "Unit test: a wrong-side placement awards 0 points and removes exactly 1 heart"
Task: "Unit test: validate_placement accepts only left, between-two, and right"

# Board components copied unchanged, in parallel
Task: "Copy TopBarOrdering.tsx to frontend/src/components/TopBarOrdering.tsx unchanged"
Task: "Copy MiddleItemGrid.tsx to frontend/src/components/MiddleItemGrid.tsx unchanged"
```

---

## Parallel Example: User Story 4

```bash
# All Tripmine unit tests together
Task: "Unit test: start_tripmine_round assigns true_rank 1..10 and marks the tripmine"
Task: "Unit test: initial searched_rank is 1 for highest and 10 for lowest"
Task: "Unit test: picking the searched position awards 10 and advances the search"
Task: "Unit test: tripmine hit removes 10 from the picker and adds 10 to others"
Task: "Unit test: with two unrevealed items a correct pick awards 20"

# Board component copy in parallel with the integration test
Task: "Copy TripmineBoard.tsx to frontend/src/components/TripmineBoard.tsx unchanged"
Task: "Integration test: a player role never receives isTripmine for an unrevealed item"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational — this blocks everything
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: create a 2-round, 2-team session, confirm the links, the password gate, and the starting scoreboard
5. Demo it — a session exists but no board is playable yet

### Incremental Delivery

1. Phase 1 + Phase 2 → foundation ready
2. US1 → session creation and the gamemaster gate → demo
3. US2 → folders and the default folder → demo
4. US3 → Order playable → demo, first genuinely playable game
5. US4 → Tripmine playable → demo, both quiz types
6. US5 → multi-device views → demo, the real hosting experience
7. US6 → list management and CSV → demo
8. US7 → cross-round scoring and final ranking → demo, complete game
9. US8 → themes and keyboard → demo, accessibility complete
10. Phase 11 → verification and parity check

### Parallel Team Strategy

With multiple developers:

1. Team completes Phase 1 + Phase 2 together
2. Once Phase 2 is done:
   - Developer A: US3 (Order)
   - Developer B: US4 (Tripmine)
   - Developer C: US2 then US6 (folders and lists)
   - Developer D: US1 (sessions)
3. Then US5, US7, and US8, which integrate across the boards

---

## Notes

- `[P]` tasks touch different files and have no dependency on incomplete tasks
- `[Story]` labels give traceability back to `spec.md`
- Every business rule needs a unit test in its own module's `tests.rs`; a rule that needs HTTP, the
  network, or another feature's fixtures belongs in a pure function instead
- Stop at each checkpoint and validate that story on its own before continuing
- Commit after each task or logical group; never commit `frontend/dist`, `backend/target`, or
  `backend/data`
