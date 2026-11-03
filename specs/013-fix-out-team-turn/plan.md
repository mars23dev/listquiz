# Implementation Plan: Fix Eliminated Team Turn Switch

**Branch**: `013-fix-out-team-turn` | **Date**: 2026-10-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/013-fix-out-team-turn/spec.md`

## Summary

The automatic turn rotation already skips a team at zero hearts (`scoring::advance_turn`), but the
gamemaster's manual turn switch (`apply_set_turn` in `backend/src/routes/sessions.rs`) sets
`active_team_id` to any team, so an eliminated team can be put back on turn and keep playing. The fix:
when the current round uses hearts (Order or Connect), `apply_set_turn` refuses to put a team with zero
hearts on turn and returns clear feedback; the gamemaster's turn control also marks such a team
unselectable. Tripmine (no hearts) is unaffected, and a new round resets hearts so the team is back in.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript / React 19 frontend (Vite).

**Primary Dependencies**: unchanged — no new dependency.

**Storage**: unchanged; no schema change. Hearts live on the `team` row; the round's quiz type is in
`round_config` (the same query `sessions::apply_turn_timeout` already uses).

**Testing**: `cargo test` for the turn-switch rule (unit over the DB/HTTP route, no server needed);
`vitest` for the gamemaster turn control (an out team is unselectable). Existing suites stay green.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Project Type**: web application — Rust backend that serves the exported frontend.

**Constraints**: no new dependency; no build state outside the repo; keyboard accessible;
`the reference frontend` stays read-only.

**Scale/Scope**: one route handler (`apply_set_turn`) and its test; one `<select>` option in
`Session.tsx` and its test.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | The rule is exercised by a backend route test over the in-project SQLite store; the UI check is a component test. No network. | PASS |
| II. Explicit Business Logic | The "is this team out" check is a named predicate (reusing the hearts threshold) called by the handler, not inlined in a callback. | PASS |
| III. Rust Backend | The rule is enforced in Rust; the frontend only reflects it. | PASS |
| IV. Backend-Owned Frontend | Unchanged; the session page is served by the backend. | PASS |
| V. Small Independent Modules | Change stays in `sessions.rs` route handler and the `Session` page; no new module. | PASS |
| VI. Responsive and Keyboard Accessible UI | The turn control stays a native `<select>`; disabling an option keeps it keyboard-navigable. | PASS |
| VII. Simplicity Over Abstractions | Reuse `advance_turn`'s hearts threshold and the existing quiz-type query; no new state or channel. | PASS |
| Stack constraints | No dependency, no migration, `the reference frontend` untouched. | PASS |

Complexity Tracking: no violations, so no exceptions are claimed.

## Project Structure

### Documentation (this feature)

```text
specs/013-fix-out-team-turn/
|-- plan.md
|-- spec.md
|-- research.md          # Phase 0 output
|-- data-model.md        # Phase 1 output
|-- quickstart.md        # Phase 1 output
|-- contracts/
|   `-- turn-switch.md       # when a turn switch is allowed
|-- checklists/
|   `-- requirements.md
`-- tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
backend/
|-- src/routes/sessions.rs     # apply_set_turn(): refuse a zero-heart team (EDIT)
`-- tests/                     # turn-switch refusal tests (NEW/EDIT)

frontend/
`-- src/
    |-- pages/Session.tsx              # turn <select>: disable out teams (EDIT)
    `-- test/Session.test.tsx          # out team is unselectable (EDIT)
```

**Structure Decision**: reuse the existing layout. The rule belongs in the route handler that owns the
turn switch (backend authority); the frontend mirrors it in the existing turn control. No new module
boundary.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — team hearts, the out condition, and the turn-switch rule.
- [contracts/turn-switch.md](./contracts/turn-switch.md) — when a turn switch is accepted or refused.
- [quickstart.md](./quickstart.md) — how to verify an eliminated team cannot be put on turn.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No new dependency, no schema change, one guard in one handler
plus one disabled option: Principles I, II, III, V, VI, and VII hold with no exceptions to record.
