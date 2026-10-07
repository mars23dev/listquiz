# Implementation Plan: Connect Game And Tripmine Marker

**Branch**: `010-connect-game` | **Date**: 2026-10-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/010-connect-game/spec.md`

## Summary

Two changes. First, a new turn-based game type, **Connect**: the screen splits into ten item
**names** (left, with icons) and their ten **values** (right, shuffled, no icons). On a turn the team
picks one name and one value into a middle box and confirms; a matching pair scores 10, a mismatch
costs one heart. Each team starts the round with three hearts and the round goes read-only when no
team has hearts — the Order model. It is a new backend round module, a new stored board, a new view,
a new board component, and setup/quick-game plumbing. Second, the **Tripmine bar marks the mine's
position from the start** for every role, without revealing the item.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript 7 / React 19 frontend

**Primary Dependencies**: unchanged — no new dependency. Connect reuses the scoring, teams, and
round-state helpers.

**Storage**: SQLite. One additive table `connect_state` (a `CREATE TABLE IF NOT EXISTS`, like
`order_state`/`tripmine_state`); no change to existing tables.

**Testing**: `cargo test` for the Connect board rules (sample, pick, confirm, round-over) and the
view; `vitest` for the board component and the Tripmine marker. Existing suites stay green.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Project Type**: web application — Rust backend that serves the exported frontend.

**Performance Goals**: unchanged; ten pairs per round.

**Constraints**: no new dependency; no build state outside the repo; hearts reset per round.

**Scale/Scope**: a new `connect_round` module, a `connect_state` table, the view/redaction, the
action dispatch, and the frontend board + setup.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | The Connect board rules are plain functions over plain data, tested without a database. | PASS |
| II. Explicit Business Logic | `sample_connect_round`, `pick_name`/`pick_value`/`confirm`, and `round_is_over` are named, tested units. | PASS |
| III. Rust Backend | Connect rules and storage live in Rust; the frontend renders and acts. | PASS |
| IV. Backend-Owned Frontend | Still served by the backend; Connect is a normal round. | PASS |
| V. Small Independent Modules | A new `connect_round` module; the Tripmine marker is one field + one render. | PASS |
| VI. Responsive and Keyboard Accessible UI | The board's items are native buttons in DOM order with focus; no reload. | PASS |
| VII. Simplicity Over Abstractions | No new dependency; Connect reuses scoring/teams/round-state; the marker is display-only. | PASS |
| Stack constraints | Additive table; `the reference frontend` stays read-only. | PASS |

Complexity Tracking: no violations. The new module and table are the feature's own shape.

## Project Structure

### Documentation (this feature)

```text
specs/010-connect-game/
|-- plan.md
|-- spec.md
|-- research.md          # Phase 0 output
|-- data-model.md        # Phase 1 output
|-- quickstart.md        # Phase 1 output
|-- contracts/
|   `-- connect.md           # the Connect round, actions, view, and the Tripmine marker
|-- checklists/
|   `-- requirements.md
`-- tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
backend/
`-- src/
    |-- db.rs                      # connect_state table
    |-- app_state.rs               # ConnectView; TripmineView.mineRank; redaction
    |-- state_reader.rs            # read_connect; mineRank
    |-- modules/
    |   |-- connect_round/mod.rs   # board rules + storage (NEW)
    |   |-- sessions/mod.rs        # QuizType::Connect; round validation; clear connect_state
    |   |-- sessions/round_state/  # start a Connect round
    |   `-- quick_game/mod.rs      # Connect allowed
    `-- routes/sessions.rs         # connect actions

frontend/
`-- src/
    |-- components/ConnectBoard.tsx    # split screen + middle box (NEW)
    |-- pages/Session.tsx              # render + handlers + cues
    |-- pages/SetupQuiz.tsx            # Connect round option
    |-- components/TripmineBoard.tsx   # mark the mine position
    |-- types/quiz.ts, types/session.ts
    `-- utils/sessionSounds.ts         # Connect correct/wrong cues
```

**Structure Decision**: reuse the layout. Connect is a self-contained `connect_round` module with
its own board and storage, mirroring `order_round`; the marker is one field and one render. No
existing feature module gains Connect's rules.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the Connect board and storage, the actions, and the Tripmine marker.
- [contracts/connect.md](./contracts/connect.md) — the Connect round, its actions/view, and the marker.
- [quickstart.md](./quickstart.md) — how to verify both findings.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No new dependency, one additive table, a new self-contained
module reusing the shared scoring/teams/round-state: Principles I, II, III, V, VI, and VII hold with
no exceptions to record.
