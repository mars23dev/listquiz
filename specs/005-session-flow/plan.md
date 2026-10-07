# Implementation Plan: Session Flow And Quick Game

**Branch**: `005-session-flow` | **Date**: 2026-10-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/005-session-flow/spec.md`

## Summary

Six session-flow changes. First, a regression: the gamemaster's Tripmine answer key is
restored on the board (the server already sends the values; the board stops hiding them from
the gamemaster). Second, a gamemaster turn timer: a countdown attached to the current turn,
stored server-side, visible to every role, and on expiry the on-turn team loses a heart and
the turn passes — the client ticks the countdown and reports the timeout, the server judges
it. Third, a dedicated game-end dialog showing points only. Fourth, a finished game's session
is deleted once that dialog is dismissed. Fifth, a title-screen quick game that starts a
one-team session of the chosen type straight in the player view, with no gamemaster access.
Sixth, the light/dark toggle is removed and dark is the only theme.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript 7 / React 19 frontend (unchanged)

**Primary Dependencies**: unchanged — no new dependency. Timer expiry reuses the existing
scoring module for the heart loss and turn advance.

**Storage**: SQLite. One additive, idempotent change: `game_session` gains nullable
`timer_ends_at` and `timer_total_seconds`. Session deletion uses the schema's existing
`ON DELETE CASCADE` from `game_session`.

**Testing**: `cargo test` for the timer rules, the quick-game request builder, the delete
guard, and Tripmine redaction; `vitest` + Testing Library for the countdown, the end dialog,
the quick-game entry, and the removed toggle. Existing suites stay green.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Project Type**: web application — Rust backend that serves the exported frontend.

**Performance Goals**: unchanged; the timer read is O(1) and the countdown is client-side.

**Constraints**: keyboard-operable dialogs, no freeze, no full-page reload (Principle VI); no
build state outside the repo; the migration must be safe on an existing database.

**Scale/Scope**: `game_session`/view (`db.rs`, `app_state.rs`, `state_reader.rs`), a new
`turn_timer` module and a new `quick_game` module, `routes/sessions.rs`, and the frontend
(`Session.tsx`, a new `GameEndDialog`, `Landing.tsx`, `TripmineBoard.tsx`, theme removal).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | Timer rules, the quick-game builder, and the delete guard are pure/tested without the UI; component tests cover the dialog and countdown. | PASS |
| II. Explicit Business Logic | `turn_timer` (start/remaining/expire) and `quick_game::build_request` are named, tested units; the timer consequence is not inlined in a handler. | PASS |
| III. Rust Backend | Timer, delete, and quick-session rules live in Rust; the frontend ticks and renders. | PASS |
| IV. Backend-Owned Frontend | Still served by the backend; quick game is a normal session, not a standalone mode. | PASS |
| V. Small Independent Modules | New `turn_timer` and `quick_game` modules; deletion is a narrow function; no feature module is edited to add another's behavior. | PASS |
| VI. Responsive and Keyboard Accessible UI | The end dialog is a focus-managed `role="dialog"`; the timer is text, no animation-only state; no reload. | PASS |
| VI. Simplicity Over Abstractions | No new dependency; the timer stores an end time and the client ticks it; quick game reuses `create_session`. | PASS |
| Stack constraints | Additive in-tree migration; deletion relies on the existing cascade; `the reference frontend` stays read-only. | PASS |

Complexity Tracking: no violations, so no exceptions are claimed.

## Project Structure

### Documentation (this feature)

```text
specs/005-session-flow/
|-- plan.md
|-- spec.md
|-- research.md          # Phase 0 output
|-- data-model.md        # Phase 1 output
|-- quickstart.md        # Phase 1 output
|-- contracts/
|   `-- session-flow.md      # timer, delete, quick session, GM visibility
|-- checklists/
|   `-- requirements.md
`-- tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
backend/
`-- src/
    |-- db.rs                      # additive timer columns; session delete cascade
    |-- app_state.rs               # SessionState.timer; GM Tripmine visibility
    |-- state_reader.rs            # read the timer; keep GM values
    |-- modules/
    |   |-- turn_timer/mod.rs      # start/remaining/expire (NEW)
    |   |-- quick_game/mod.rs      # build a one-team request from defaults (NEW)
    |   `-- sessions/mod.rs        # delete_session(); clear timer on turn changes
    `-- routes/sessions.rs         # start_timer/stop_timer/time_out; DELETE; quick

frontend/
`-- src/
    |-- components/
    |   |-- GameEndDialog.tsx      # points-only pop-up (NEW)
    |   |-- TripmineBoard.tsx      # show the GM every value
    |   `-- ThemeToggle.tsx        # removed
    |-- pages/
    |   |-- Landing.tsx            # quick-game entry (NEW control)
    |   `-- Session.tsx            # countdown tick + time_out; end dialog; delete on dismiss
    |-- theme/                     # dark-only; toggle API removed
    |-- api/client.ts              # deleteSession(), createQuickSession()
    |-- types/session.ts           # timer shape
    `-- test/                      # countdown, dialog, quick game, no-toggle
```

**Structure Decision**: reuse the web-application layout. New behavior is split by
responsibility: `turn_timer` owns the countdown rule, `quick_game` owns the default-list
selection, `sessions::delete_session` owns cleanup, and the frontend renders. No feature
module gains another's rule.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the timer columns and view, the quick-game request, the
  delete guard, and Tripmine visibility.
- [contracts/session-flow.md](./contracts/session-flow.md) — timer state and actions, the
  delete endpoint, the quick-session endpoint, and the end-dialog/theme UI contract.
- [quickstart.md](./quickstart.md) — how to verify all six findings.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No new dependency; one additive, idempotent
migration; timer and quick-game rules are pure, tested functions; deletion rides the existing
cascade. Principles I, II, III, V, VI, and VII hold with no exceptions to record.
