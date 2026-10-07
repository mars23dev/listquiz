# Implementation Plan: Fix Game End After The Last Round

**Branch**: `007-fix-game-end` | **Date**: 2026-10-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/007-fix-game-end/spec.md`

## Summary

The game-end dialog only appears when a session reaches the `finished` status, and that status is
set only by the gamemaster's `advance_round` (the "Finish Game" button). A quick game has no
gamemaster, so its session never finishes and the dialog never appears; in a normal game the end is
also easily missed. The fix: when the **last configured round** ends, the session finishes itself —
revealing values and clearing the active team, exactly as the manual finish does — so the existing
game-end dialog pops up for every role. Ending a non-final round is unchanged. This is a backend
behavior change; the frontend already renders the dialog off `status === 'finished'`.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript 7 / React 19 frontend (frontend unchanged)

**Primary Dependencies**: unchanged — no new dependency.

**Storage**: unchanged; no schema change. The `finished` status and `reveal_values` already exist.

**Testing**: `cargo test` for the finish-if-last-round rule (unit over the DB + HTTP integration);
`vitest` only if a frontend assertion is needed (the dialog already has coverage). Existing suites
stay green.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Project Type**: web application — Rust backend that serves the exported frontend.

**Performance Goals**: unchanged.

**Constraints**: no new dependency; no build state outside the repo; safe when the game is already
finished.

**Scale/Scope**: one new session helper and its call sites in the round-ending action handlers.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | The finish rule is exercised by backend tests that need no UI. | PASS |
| II. Explicit Business Logic | `finish_if_last_round` is a named, tested function, not inlined in each handler. | PASS |
| III. Rust Backend | The rule lives in Rust; the frontend is unchanged. | PASS |
| IV. Backend-Owned Frontend | The dialog is still served by the backend's finished status. | PASS |
| V. Small Independent Modules | One helper in the sessions module; handlers call it. | PASS |
| VI. Responsive and Keyboard Accessible UI | No UI change; the existing dialog already satisfies this. | PASS |
| VII. Simplicity Over Abstractions | No schema change, no new dependency; reuse the existing finish behavior. | PASS |
| Stack constraints | No migration; `the reference frontend` stays read-only. | PASS |

Complexity Tracking: no violations, so no exceptions are claimed.

## Project Structure

### Documentation (this feature)

```text
specs/007-fix-game-end/
|-- plan.md
|-- spec.md
|-- research.md          # Phase 0 output
|-- data-model.md        # Phase 1 output
|-- quickstart.md        # Phase 1 output
|-- contracts/
|   `-- game-end.md          # the last-round finish transition
|-- checklists/
|   `-- requirements.md
`-- tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
backend/
`-- src/
    |-- modules/sessions/mod.rs   # finish_if_last_round() (NEW)
    |-- routes/sessions.rs        # call it where a round can end (order confirm, tripmine pick, time_out)
    `-- tests/                    # finish-on-last-round tests

frontend/                          # unchanged
```

**Structure Decision**: reuse the existing layout. The rule is one function in the sessions module
(the owner of session status); the round-ending handlers call it. No new module boundary and no
frontend change.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the last-round finish transition and its guards.
- [contracts/game-end.md](./contracts/game-end.md) — when the session becomes `finished` and what
  the client sees.
- [quickstart.md](./quickstart.md) — how to verify the end dialog appears after the last round.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No new dependency, no schema change, one named tested
function: Principles I, II, III, V, VI, and VII hold with no exceptions to record.
