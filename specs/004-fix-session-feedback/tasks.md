---

description: "Task list for Session Feedback And Order Reveal"
---

# Tasks: Session Feedback And Order Reveal

**Input**: Design documents from `/specs/004-fix-session-feedback/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/session-feedback.md, quickstart.md

**Tests**: Included — the constitution (Principles I and II) requires every business rule to be a
named, tested unit, and the spec/plan call for tests.

**Organization**: Tasks are grouped by user story so each can be implemented and verified on its own.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: US1 (sound), US2 (team-colored borders), US3 (Order reveal)
- Every task names an exact file path.

## Path Conventions

- **Web app**: `backend/src/`, `frontend/src/` (mirrors plan.md)

---

## Phase 1: Setup

**Purpose**: Confirm a green baseline before touching anything.

- [x] T001 Run the baseline suites and record the result: `cargo test --manifest-path backend/Cargo.toml`, `npm --prefix frontend run lint`, `npm --prefix frontend test`. No source change; note any pre-existing failures so this feature is not blamed for them.

---

## Phase 2: Foundational (Blocking Prerequisite)

**Purpose**: The one shared, additive schema change both Order stories read from.

**⚠️ CRITICAL**: Complete before the Order stories (US2, US3).

- [x] T002 [P] Add the idempotent migration to `backend/src/db.rs`: after `execute_batch(SCHEMA)`, inspect `PRAGMA table_info(order_state)` and, when `placed_by` is absent, run `ALTER TABLE order_state ADD COLUMN placed_by TEXT`. Safe to run on every open.
- [x] T003 [P] Add a test (in `backend/tests/session_sync.rs` or `backend/src/db.rs` tests) that `apply_schema` adds `placed_by` to a pre-existing `order_state` table and is safe to run twice; a `null`/missing value must be read as an empty map.

**Checkpoint**: The session store upgrades in place; Order stories can proceed.

---

## Phase 3: User Story 1 - Game sounds actually play (Priority: P1) 🎯 MVP

**Goal**: Every key event plays its cue, driven only by the reflected session state; the acting team never hears a doubled cue; mute silences all; audio unlocks on the first interaction.

**Independent Test**: Feed hand-built `SessionState` pairs to `deriveSessionCue` and assert the cue per transition and the `null` guards; in the browser, trigger each event and confirm the cue plays once (unmuted), never on reconnect, and nothing while muted. See quickstart.md §1.

### Tests for User Story 1 ⚠️

> Write first; ensure they FAIL before implementing.

- [x] T004 [P] [US1] Unit tests for `deriveSessionCue` in `frontend/src/utils/sessionSounds.test.ts`: selection→slide; side growth→slide/chime; score rise→chime; heart drop→failure; mine reveal→explosion; safe reveal→chime; round complete→chime; pool growth→remove; status `finished`→bonus; and `null` for `prev === null`, `next.revision !== prev.revision + 1`, and mute/other-only changes.
- [x] T005 [P] [US1] Unit tests for audio unlock in `frontend/src/utils/audio.test.ts`: `unlockAudio()` resumes a suspended context; a play call while muted produces no oscillator.

### Implementation for User Story 1

- [x] T006 [P] [US1] Add `unlockAudio()` to `frontend/src/utils/audio.ts`: create the `AudioContext` and `resume()` it; keep the lazy `resume()` inside the play functions as a fallback.
- [x] T007 [US1] Create `frontend/src/utils/sessionSounds.ts`: `deriveSessionCue(prev, next): SessionCue | null` implementing the priority table in contracts/session-feedback.md §2, reading public data only, emitting at most one cue per transition, and returning `null` unless `prev !== null` and `next.revision === prev.revision + 1`.
- [x] T008 [US1] Wire cues into `frontend/src/pages/Session.tsx`: keep the previous `SessionState` in a ref, call `deriveSessionCue` on each new state, dispatch to the matching `play*` function, and register a one-time `pointerdown`/`keydown` listener that calls `unlockAudio()`.
- [x] T009 [P] [US1] Remove the local interaction cues from `frontend/src/components/MiddleItemGrid.tsx` and `frontend/src/components/TopBarOrdering.tsx` (drop the `playSlideSound` imports/calls) so only the reflected event emits a cue.

**Checkpoint**: Cues fire from the reflected event in every role, once, and obey mute.

---

## Phase 4: User Story 2 - Item border shows the choosing team's color (Priority: P2)

**Goal**: A placed (Order) or revealed (Tripmine) item's border takes the acting team's color; higher/lower and mine/safe move off the border but stay visible.

**Independent Test**: Play both round types with two teams; assert each item's border matches its placing/picking team's `primaryHex`, the pending preview matches the active team, and no border encodes higher/lower or mine/safe. See quickstart.md §2.

### Tests for User Story 2 ⚠️

- [x] T010 [P] [US2] Backend test in `backend/tests/order_placed_by.rs`: a correct placement records `placed_by[item_id] = pending.team_id`; a wrong placement records nothing; `return_to_pool` removes the entry; the map survives a `save_board`→`load_board` round trip.
- [x] T011 [P] [US2] Backend tests in `backend/src/app_state.rs`: `order.left/right[].placedBy` is populated for placed items and `null` for pool items; `tripmine.items[].pickedBy` is the picker for revealed items and `null` while hidden; both are visible to every role.
- [x] T012 [P] [US2] Unit test in `frontend/src/utils/teamColors.test.ts`: `teamBorderStyle` returns the team's `primaryHex` for a known team and a neutral border when the team is unknown.
- [x] T013 [P] [US2] Component tests in `frontend/src/test/Session.test.tsx` and `frontend/src/components/TopBarOrdering.test.tsx`: a placed item and the pending preview render with the acting team's color; a revealed Tripmine item renders with the picking team's color while the mine/safe badge and point text remain.

### Implementation for User Story 2

- [x] T014 [US2] In `backend/src/modules/order_round/mod.rs`: add `placed_by: BTreeMap<String, String>` to `OrderBoard` (`#[serde(default)]`); in `confirm_placement` record `placed_by[item_id] = pending.team_id` **only when correct**; in `return_to_pool` drop the entry; persist the map in `save_board` and read it in `load_board` from the new `placed_by` column (missing/`null` → empty map).
- [x] T015 [US2] In `backend/src/state_reader.rs`: populate `OrderItem.placed_by` from the stored map and `TripmineItem.picked_by` from `TripmineSlot.revealed_by` (guard hidden → `None`). In `backend/src/app_state.rs`: add `placed_by: Option<String>` to `OrderItem` and `picked_by: Option<String>` to `TripmineItem`, and pass both through `redact` unchanged.
- [x] T016 [US2] Mirror the fields in `frontend/src/types/session.ts`: `OrderItem.placedBy: string | null` and `TripmineItem.pickedBy: string | null`.
- [x] T017 [P] [US2] Create `frontend/src/utils/teamColors.ts`: `teamBorderStyle(teamId, teams)` returning a border style from the team's `primaryHex` (neutral when the team is unknown), plus a neutral constant for the reveal case.
- [x] T018 [US2] In `frontend/src/components/TopBarOrdering.tsx`: take a `teams` prop; render each placed item's border from its `placedBy` team color and the pending preview from the active team's color; stop using the cyan/rose side color as the item's team signal (keep it only as an empty-slot tint).
- [x] T019 [US2] In `frontend/src/components/TripmineBoard.tsx`: take the teams/color lookup; frame each revealed item with its `pickedBy` team color; keep the mine/safe signal on the RANK badge, the bomb/check icon, and the point text (FR-009).
- [x] T020 [US2] In `frontend/src/pages/Session.tsx`: pass the team list into `TopBarOrdering` and `TripmineBoard`, carry `placedBy` into the order `QuizItem`s and `pickedBy` into `QuizTripmineItem.revealedByTeamId`.

**Checkpoint**: Borders name the responsible team in both games; no information was lost from the border.

---

## Phase 5: User Story 3 - Order reveal arranges the whole bar (Priority: P3)

**Goal**: Revealing an Order round sorts every item into its correct bar position with a neutral border and changes no score or hearts; Tripmine reveal is unchanged.

**Independent Test**: Play an Order round, reveal, and confirm the arranged bar, neutral borders, values shown, and byte-identical scores/hearts. See quickstart.md §3.

### Tests for User Story 3 ⚠️

- [x] T021 [P] [US3] Unit tests for `arrange_revealed_order` in `backend/src/modules/order_round/tests.rs`: values `<=` anchor go left ascending and `>` go right ascending; ties preserve input order; a legal round yields five per side; the five-per-side cap holds on an over-full input.
- [x] T022 [P] [US3] Backend test (in `backend/src/state_reader.rs` or `backend/src/app_state.rs` tests): with `reveal_values` set, the Order view is arranged and `pool` is empty; scores and hearts are identical before and after (no placement scored).
- [x] T023 [P] [US3] Component test in `frontend/src/components/TopBarOrdering.test.tsx`: with `revealAllValues` true, the arranged items render and every item border is neutral (not a team color, not a side color).

### Implementation for User Story 3

- [x] T024 [US3] Implement `arrange_revealed_order(anchor: &ListItemValue, items: &[ListItemValue]) -> (Vec<ListItemValue>, Vec<ListItemValue>)` in `backend/src/modules/order_round/mod.rs`: left = `numeric <= anchor.numeric` ascending, right = `numeric > anchor.numeric` ascending, each a stable sort on `(numeric, original index)`, clamped to five per side. Pure; no board mutation.
- [x] T025 [US3] In `backend/src/state_reader.rs::read_order`: when the session's `reveal_values` is set, rebuild `left`/`right`/`pool` from the anchor plus all bar and pool items via `arrange_revealed_order` (left, anchor, right; pool empty); otherwise unchanged. The stored board and scores are not touched.
- [x] T026 [US3] In `frontend/src/components/TopBarOrdering.tsx`: when `revealAllValues` is true, render item borders and side tints neutral (use `teamBorderStyle`'s neutral constant), overriding the team-color path from US2.

**Checkpoint**: The reveal shows the full answer with neutral borders and zero scoring; Tripmine reveal is untouched.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T027 [P] Add an amendment pointer to `specs/001-listquiz-game/contracts/realtime.md` noting the new `order[].placedBy` and `tripmine.items[].pickedBy` fields and the reveal arrangement, referencing `specs/004-fix-session-feedback/contracts/session-feedback.md`.
- [x] T028 [P] Update `README.md` if it documents session audio/reveal behavior, to match the server-driven cues and the arranged reveal.
- [x] T029 Run the full verification: `cargo fmt --manifest-path backend/Cargo.toml -- --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, `cargo test --manifest-path backend/Cargo.toml`, `npm --prefix frontend run lint`, `npm --prefix frontend test`, `npm --prefix frontend run build`; then walk quickstart.md manually (build the frontend, run the backend release, open two tabs).
- [x] T030 Confirm the build stayed inside the repo and `frontend/dist/` + `backend/target/` remain untracked (`git status`), per the constitution's workflow step 7.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: starts immediately.
- **Foundational (Phase 2)**: after Setup — blocks US2 and US3 (the `placed_by` column).
- **User Stories (Phase 3+)**: US1 needs only Setup; US2 and US3 need Foundational.
- **Polish (Phase 6)**: after the desired stories.

### Story Dependencies

- **US1 (P1)**: independent (frontend audio only).
- **US2 (P2)**: needs Foundational; independent of US1.
- **US3 (P3)**: needs Foundational; the neutral-border render (T026) overrides US2's team-color path in the same component, so do US2's T018 before T026.

### Within Each Story

- Tests before implementation (write and fail first).
- `order_round` storage before view exposure; view fields before component wiring.

### Parallel Opportunities

- T002 and T003 together (different files).
- US1 tests T004/T005 together; T006 and T009 together.
- US2 tests T010–T013 together; T017 with T016.
- US3 tests T021–T023 together.
- Polish T027/T028 together.

---

## Parallel Example: User Story 1

```bash
Task: "Unit tests for deriveSessionCue in frontend/src/utils/sessionSounds.test.ts"
Task: "Unit tests for audio unlock in frontend/src/utils/audio.test.ts"
```

## Parallel Example: User Story 2

```bash
Task: "Backend test placed_by in backend/tests/order_placed_by.rs"
Task: "Unit test teamBorderStyle in frontend/src/utils/teamColors.test.ts"
Task: "Component tests for team borders in frontend/src/test/Session.test.tsx"
```

---

## Implementation Strategy

### MVP First (User Story 1)

1. Phase 1 Setup → 2. Phase 2 Foundational → 3. Phase 3 (US1) → **STOP and validate** the sound fix independently → demo.

### Incremental Delivery

1. US1: cues play (MVP).
2. US2: borders name the team (adds the `placed_by` column and view fields).
3. US3: the reveal finishes the round with neutral, arranged borders.
4. Polish: contract/README pointers, full verification, build-hygiene check.

---

## Notes

- [P] = different files, no incomplete dependency.
- Tests are written first and must fail before implementation.
- No task commits on its own; commit only when the user asks.
- The `the reference frontend` reference stays read-only (constitution).
