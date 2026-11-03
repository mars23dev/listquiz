# Implementation Plan: Fix Timer Timeout

**Branch**: `008-fix-timer-timeout` | **Date**: 2026-10-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/008-fix-timer-timeout/spec.md`

## Summary

Three timer defects. First, a countdown that reaches zero does nothing: the client posts `time_out`
at zero, but the server often rejects it as "not yet expired" (the browser's timers are also throttled
in a background tab), so the turn never ends — and that rejection is what shows the stray
`"The timer has not run out yet."` bar. The fix makes the server own the deadline: when a timer
starts, the server schedules the timeout and applies it at the end time (reusing one named rule),
so the turn ends even if no client is watching; the client no longer posts `time_out`, which removes
the stray bar. Third, each of the last five seconds plays a short tick from the client countdown.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript 7 / React 19 frontend

**Primary Dependencies**: unchanged — no new dependency. The scheduled timeout uses the tokio runtime
the server already runs on; the tick reuses an existing short cue.

**Storage**: unchanged; no schema change. The timer columns from feature 005 stay.

**Testing**: `cargo test` for the timeout rule (direct call) and an HTTP test that a short timer ends
the turn on its own; `vitest` for the removed client post and the tick behavior. Existing suites stay
green.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Project Type**: web application — Rust backend that serves the exported frontend.

**Performance Goals**: unchanged; one short-lived task per started timer.

**Constraints**: no new dependency; no build state outside the repo; a scheduled timeout must be a
no-op if the timer was stopped or replaced.

**Scale/Scope**: `turn_timer`/`sessions` (the timeout rule), `routes/sessions.rs` (schedule + action),
and the session view (drop the client post, add ticks).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | The timeout rule is a named function tested directly; the schedule is thin. | PASS |
| II. Explicit Business Logic | `sessions::apply_turn_timeout` is the one named, tested timeout rule, reused by the action and the scheduled task. | PASS |
| III. Rust Backend | The deadline and its consequence live in Rust; the client only displays and ticks. | PASS |
| IV. Backend-Owned Frontend | Unchanged; still served by the backend. | PASS |
| V. Small Independent Modules | The change stays inside the sessions/turn_timer modules and the one handler. | PASS |
| VI. Responsive and Keyboard Accessible UI | No new control; ticks are sound only; the countdown is unchanged. | PASS |
| VII. Simplicity Over Abstractions | No new dependency; one rule reused twice; ticks reuse an existing cue. | PASS |
| Stack constraints | No migration; `the reference frontend` stays read-only. | PASS |

Complexity Tracking: the scheduled task is the one added mechanism; it is justified by the bug (a
client-driven deadline cannot fire from a throttled background tab) and reuses the existing runtime.

## Project Structure

### Documentation (this feature)

```text
specs/008-fix-timer-timeout/
|-- plan.md
|-- spec.md
|-- research.md          # Phase 0 output
|-- data-model.md        # Phase 1 output
|-- quickstart.md        # Phase 1 output
|-- contracts/
|   `-- timer-timeout.md     # server-owned deadline, no client post, ticks
|-- checklists/
|   `-- requirements.md
`-- tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
backend/
`-- src/
    |-- modules/sessions/mod.rs   # apply_turn_timeout() (NEW, named rule)
    |-- routes/sessions.rs        # start_timer schedules the deadline; time_out calls the rule
    `-- app_state.rs              # (unchanged) with_db + bump_revision the task uses

frontend/
`-- src/
    |-- pages/Session.tsx         # drop the client time_out post; add the last-five-seconds ticks
    `-- utils/sessionSounds.ts    # (unchanged) the start/timeout cues
```

**Structure Decision**: reuse the existing layout. The timeout rule is one function in the sessions
module; the handler schedules it on start and can still apply it directly; the frontend loses the
fragile client post and gains ticks. No new module boundary.

## Complexity Tracking

> The one added mechanism is the scheduled timeout; the table above records why it is needed.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the server-owned deadline and the timeout rule.
- [contracts/timer-timeout.md](./contracts/timer-timeout.md) — when the timeout fires, that no client
  action is needed, and the tick behavior.
- [quickstart.md](./quickstart.md) — how to verify all three findings.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No new dependency, no schema change, one named tested rule
plus a thin schedule; the frontend only removes code and adds ticks. Principles I, II, III, V, VI, and
VII hold; the scheduled task is the recorded, justified exception to "no new mechanism".
