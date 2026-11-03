# Implementation Plan: Fix Turn Timer

**Branch**: `006-fix-turn-timer` | **Date**: 2026-10-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/006-fix-turn-timer/spec.md`

## Summary

Three corrections to the turn timer added in feature 005. First, a timeout's consequence must match
the game type: in an Order round the on-turn team loses one heart and the turn passes, but a Tripmine
round has no heart mechanic, so a timeout there only passes the turn. Today `turn_timer::expire`
always removes a heart. Second, the timer is silent: starting it and letting it run out should each
play a cue, driven (like every other cue) from the reflected state, without double-sounding an Order
timeout that also drops a heart. Third, the gamemaster's free-text duration field is replaced by
three presets — 15s, 30s, 60s.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript 7 / React 19 frontend (both unchanged)

**Primary Dependencies**: unchanged — no new dependency. Cues reuse the existing synthesized effects.

**Storage**: unchanged; no schema change. The timer columns from feature 005 stay.

**Testing**: `cargo test` for the per-type timeout rule and the HTTP action; `vitest` for the new
timer cues and the preset controls. Existing suites stay green.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Project Type**: web application — Rust backend that serves the exported frontend.

**Performance Goals**: unchanged.

**Constraints**: keyboard-operable controls, no freeze, no reload (Principle VI); no build state
outside the repo.

**Scale/Scope**: the `turn_timer` rule, the `time_out` handler, the cue derivation, and the timer
controls in the session view.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | The timeout rule and the cue derivation are pure and unit-tested; the action has an HTTP test. | PASS |
| II. Explicit Business Logic | `turn_timer::expire` (now rule-aware) and the cue mapping are named, tested units. | PASS |
| III. Rust Backend | The timeout consequence lives in Rust; the frontend renders and cues. | PASS |
| IV. Backend-Owned Frontend | Unchanged; still served by the backend. | PASS |
| V. Small Independent Modules | The change stays inside `turn_timer`, the `time_out` handler, and the cue module. | PASS |
| VI. Responsive and Keyboard Accessible UI | The three preset controls are native buttons with visible focus. | PASS |
| VII. Simplicity Over Abstractions | No new dependency; a rule parameter instead of a second module; presets as a fixed list. | PASS |
| Stack constraints | No migration; `the reference frontend` stays read-only. | PASS |

Complexity Tracking: no violations, so no exceptions are claimed.

## Project Structure

### Documentation (this feature)

```text
specs/006-fix-turn-timer/
|-- plan.md
|-- spec.md
|-- research.md          # Phase 0 output
|-- data-model.md        # Phase 1 output
|-- quickstart.md        # Phase 1 output
|-- contracts/
|   `-- turn-timer.md        # timeout rule per type, cues, presets
|-- checklists/
|   `-- requirements.md
`-- tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
backend/
`-- src/
    |-- modules/turn_timer/mod.rs   # expire(teams, active, rule)
    `-- routes/sessions.rs          # apply_time_out picks the rule from the round type

frontend/
`-- src/
    |-- utils/sessionSounds.ts      # 'buzzer' start cue + timeout cue
    `-- pages/Session.tsx           # 15/30/60 preset controls; play the start cue
```

**Structure Decision**: reuse the existing layout. The timeout rule stays in `turn_timer` (the module
that owns it); the handler supplies the round's type; the cue mapping stays in the one frontend
module. No new module boundary.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the timeout rule per game type and the cue mapping.
- [contracts/turn-timer.md](./contracts/turn-timer.md) — the `time_out` consequence per type, the
  timer cues, and the preset durations.
- [quickstart.md](./quickstart.md) — how to verify all three findings.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No new dependency, no schema change, one rule-aware pure
function and a small cue-table addition: Principles I, II, III, V, VI, and VII hold with no
exceptions to record.
