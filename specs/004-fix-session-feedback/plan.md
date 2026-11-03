# Implementation Plan: Session Feedback And Order Reveal

**Branch**: `004-fix-session-feedback` | **Date**: 2026-10-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/004-fix-session-feedback/spec.md`

## Summary

Three corrections to the running-session experience. First, the game is silent: the cue
functions exist in `frontend/src/utils/audio.ts` but no event is wired to them, and the audio
context is never unlocked, so the browser shows no audio indicator. Cues become **server-driven**:
a pure function diffs the previously rendered session state against the newly fetched one and
returns the one cue for that transition (matched to the reference draft's effect set). Second, an
item's border currently encodes higher/lower (Order) or mine/safe (Tripmine); it becomes the
**color of the team that placed or picked the item**. Order needs a new placing-team association
(the Tripmine picker is already stored), so `order_state` gains a `placed_by` map and both views
expose the acting team. Third, revealing values in an Order round becomes a presentation of the
answer: every item sorts into its correct bar position with a neutral border and **zero** score or
heart change (the stored board is not mutated); Tripmine reveal is unchanged.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript 7 / React 19 frontend (both unchanged)

**Primary Dependencies**: unchanged — no new dependency. Audio reuses the existing synthesized
cue functions in `frontend/src/utils/audio.ts`.

**Storage**: SQLite. One additive, idempotent schema change: `order_state` gains a `placed_by`
JSON column (item id → placing team id). No other table changes.

**Testing**: `cargo test` for the pure Order arrangement rule, the placing-team association, and
the view fields; `vitest` + Testing Library for the cue derivation function, the audio unlock, the
team-colored borders, and the neutral reveal. Existing suites must stay green. The Playwright
harness covers the end-to-end session.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Project Type**: web application — Rust backend that serves the exported frontend.

**Performance Goals**: unchanged; cue derivation is O(items) per state update, arrangement is
O(n log n) on at most eleven items.

**Constraints**: keyboard-operable, no freeze, no full-page reload (Principle VI); no build state
outside the repo; migration must be safe on an existing database.

**Scale/Scope**: the session state view (`app_state.rs`, `state_reader.rs`), the Order rules
(`order_round`), the schema (`db.rs`), the session stream hook, and the three board components
(`Session.tsx`, `TopBarOrdering.tsx`, `TripmineBoard.tsx`).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | Cue derivation and Order arrangement are pure functions with unit tests that need no server; border rendering is covered by component tests. | PASS |
| II. Explicit Business Logic | Cue mapping (`deriveSessionCue`), reveal arrangement (`arrange_revealed_order`), and team-border styling are named, directly callable, tested units — not inline in handlers. | PASS |
| III. Rust Backend | All rules stay in Rust; the frontend only renders and diffs state. | PASS |
| IV. Backend-Owned Frontend | Still served by the backend; no standalone mode. | PASS |
| V. Small Independent Modules | The arrangement rule lives in `order_round`; the view exposes teams through the existing narrow DTO boundary; components read props. | PASS |
| VI. Responsive and Keyboard Accessible UI | Audio unlocks on first pointer **or keyboard** interaction; no freeze or reload; controls unchanged. | PASS |
| VII. Simplicity Over Abstractions | No new dependency; one diff function and one sort function; borders reuse the team's existing color. | PASS |
| Stack constraints | Additive in-tree migration only; no build-state change; `the reference frontend` stays read-only. | PASS |

Complexity Tracking: no violations, so no exceptions are claimed.

## Project Structure

### Documentation (this feature)

```text
specs/004-fix-session-feedback/
|-- plan.md
|-- spec.md
|-- research.md          # Phase 0 output
|-- data-model.md        # Phase 1 output
|-- quickstart.md        # Phase 1 output
|-- contracts/
|   `-- session-feedback.md  # state additions + cue mapping + reveal rule
|-- checklists/
|   `-- requirements.md
`-- tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
backend/
`-- src/
    |-- db.rs                      # additive order_state.placed_by migration
    |-- app_state.rs               # OrderItem.placedBy, TripmineItem.pickedBy, reveal passes through
    |-- state_reader.rs            # read placed_by; arrange the bar when reveal is on
    |-- modules/
    |   |-- order_round/mod.rs     # record placed_by on confirm; arrange_revealed_order()
    |   `-- sessions/round_state/  # (unchanged; ListItemValue carries numeric for arrangement)
    `-- tests/                     # arrangement + association tests

frontend/
`-- src/
    |-- utils/
    |   |-- audio.ts               # add unlockAudio(); cue functions already present
    |   `-- sessionSounds.ts       # deriveSessionCue(prev, next) + mapping (NEW)
    |-- utils/teamColors.ts        # teamBorderStyle() helper (NEW)
    |-- hooks/useSessionStream.ts  # unchanged (revision contract already correct)
    |-- pages/Session.tsx          # wire cues; pass teams to boards; unlock audio on gesture
    |-- components/
    |   |-- TopBarOrdering.tsx     # team-color borders; neutral on reveal
    |   `-- TripmineBoard.tsx      # team-color border for the picking team
    |-- types/session.ts           # OrderItem.placedBy, TripmineItem.pickedBy
    `-- test/                      # cue, border, reveal tests
```

**Structure Decision**: reuse the existing web-application layout. The new behavior is split by
responsibility: cue derivation in `frontend/src/utils/sessionSounds.ts`, reveal arrangement inside
`order_round` (the module that owns the bar rule), and the placing-team association inside
`order_round` + `order_state`. Components only render what the view exposes. No new module boundary
is crossed beyond the existing `modules → app_state/state_reader → routes` layering.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the placing-team association, the picked-team field, the cue
  event, and the reveal arrangement.
- [contracts/session-feedback.md](./contracts/session-feedback.md) — new `SessionState` fields, the
  cue-mapping contract, and the reveal/neutral-border rule.
- [quickstart.md](./quickstart.md) — how to verify sound, team-colored borders, and the Order reveal.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No new dependency; the only schema change is one additive,
idempotent column; cue mapping and arrangement are pure, tested functions; borders reuse each team's
existing color. Principles I, II, III, V, VI, and VII hold with no exceptions to record.
