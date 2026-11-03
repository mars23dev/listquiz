# Implementation Plan: Fix Play Findings

**Branch**: `002-fix-play-findings` | **Date**: 2026-10-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/002-fix-play-findings/spec.md`

## Summary

Six findings from a play session: the active team must be able to act from its own player view
(amending the original "host-only mutations" rule), the session must show a start affordance to the
host and a waiting state to the room before a round begins, the Tripmine grid must be shown in a
shuffled order and its hint must name the target value, the setup team-name fields must keep focus,
and the list editor must show real icons with a table that fits. Five are frontend corrections; the
player-action change and the Tripmine target value are small, tested backend changes.

## Technical Context

**Language/Version**: Rust 1.99 (edition 2024) backend; TypeScript 7 / React 19 frontend (unchanged)

**Primary Dependencies**: `axum` + `tokio` (backend, unchanged), `serde`/`serde_json`; frontend keeps
`react`, `motion`, `lucide-react` (unchanged). No new dependency is introduced.

**Storage**: SQLite at `backend/data/listquiz.db`; no schema change. The Tripmine grid order is already
stored as a JSON array, so shuffling changes the array's sequence only.

**Testing**: `cargo test` for the backend rule changes (player-turn authorization, Tripmine shuffle,
target value); `vitest` + Testing Library for the component corrections; the existing Playwright
harness for the two-context scenario.

**Target Platform**: a single machine on a local network; backend serves the exported frontend on
`0.0.0.0:8080` (unchanged).

**Project Type**: web application — Rust backend that also serves the exported frontend.

**Performance Goals**: an accepted action still reaches an open view within one second (SC-002 of the
original feature); the fixes add no measurable latency.

**Constraints**: keep responsive and keyboard-accessible (Principle VI); no timer; passwords remain
unprotected; no build state under `%APPDATA%`.

**Scale/Scope**: 1 gamemaster plus up to ~10 player/spectator views; the fixes touch one backend view
field, one action-channel authorization rule, and the existing frontend pages/components.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | The player-turn rule and the Tripmine shuffle/target are Rust functions with their own tests; the UI fixes are covered by component tests. No test needs a running server. | PASS |
| II. Explicit Business Logic | "Which role may act" and "the Tripmine target value" become named, tested functions/fields, not inlined in handlers. | PASS |
| III. Rust Backend | One Rust backend; no second language. | PASS |
| IV. Backend-Owned Frontend | The built frontend is still served by the backend; no standalone mode. | PASS |
| V. Small Independent Modules | Changes stay inside `tripmine_round`, `routes/sessions`, `state_reader`/`app_state`, and the existing frontend components; no cross-module reach-in. | PASS |
| VI. Responsive and Keyboard Accessible UI | Every corrected control stays keyboard-operable with visible focus; the player board gains the same native-button controls the host uses. | PASS |
| VII. Simplicity Over Abstractions | The player action reuses the existing action channel with an added role field; the shuffle reuses the existing sampling shuffle; no new layer. | PASS |
| Stack: passwords unprotected | Player actions are authorized by role/team, not by a password; consistent with the existing model. | PASS |
| `the reference frontend` read-only | No draft file is referenced at build or run time. | PASS |

Complexity Tracking: no violations, so no exceptions are claimed.

## Project Structure

### Documentation (this feature)

```text
specs/002-fix-play-findings/
|-- plan.md
|-- spec.md
|-- research.md          # Phase 0 output
|-- data-model.md        # Phase 1 output (view/field deltas)
|-- quickstart.md        # Phase 1 output
|-- contracts/
|   |-- actions-amendment.md   # player-role actions on the action channel
|   `-- realtime-amendment.md  # tripmine target value + player controls
`-- checklists/
    `-- requirements.md
```

### Source Code (repository root)

```text
backend/
`-- src/
    |-- app_state.rs                 # TripmineView gains targetValue
    |-- state_reader.rs              # computes the target value for the round's column
    |-- modules/
    |   |-- tripmine_round/mod.rs    # shuffle the stored grid order at round start
    |   `-- sessions/mod.rs          # role/team authorization helper for in-round actions
    `-- routes/sessions.rs           # action channel accepts a player role for the active team

frontend/
`-- src/
    |-- pages/
    |   |-- Session.tsx              # start affordance, waiting state, player-action wiring,
    |   |                            #   Tripmine target-value hint
    |   |-- SetupQuiz.tsx            # stable team-name field keys
    |   `-- ListManagement.tsx       # real icon control + fitted editor table
    `-- components/
        |-- TripmineBoard.tsx        # hint shows the target value
        `-- (existing board components reused by the player view)
```

**Structure Decision**: reuse the existing web-application layout from feature 001. The backend gains
one view field, one authorization helper, and one ordering change; the frontend corrections live in
the pages/components already present. No new top-level project or module is added.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions are resolved with no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the `TripmineView.targetValue` field and the stored grid order.
- [contracts/actions-amendment.md](./contracts/actions-amendment.md) — who may post an action.
- [contracts/realtime-amendment.md](./contracts/realtime-amendment.md) — Tripmine target value and
  player controls in the state view.
- [quickstart.md](./quickstart.md) — how to verify each finding.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. The design introduces no new dependency and no cross-module
shortcut: authorization is one named function, the shuffle reuses the round's own sampler, and the
target value is a redaction-safe view field. Principles I, II, V, VI, and VII hold with no exceptions
to record.
