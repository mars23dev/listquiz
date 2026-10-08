# Implementation Plan: Fix Reveal Reset On Next Round

**Branch**: `020-fix-reveal-reset` | **Date**: 2026-10-08 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/020-fix-reveal-reset/spec.md`

## Summary

The reveal-values flag is per round, but `advance_round` starts the next round without resetting it, so the
gamemaster's control stays "Values revealed". Fix: when `advance_round` starts the next configured round,
also set `reveal_values = 0` on the session (alongside the existing `last_message` update). The last-round
finish path already sets `reveal_values = 1` and is unchanged.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript / React 19 frontend (frontend unchanged).

**Primary Dependencies**: unchanged — no new dependency.

**Storage**: unchanged schema; the `game_session.reveal_values` column already exists.

**Testing**: `cargo test` for advancing a round resetting the reveal flag and the last-round finish keeping
it. Existing suites stay green.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Constraints**: no new dependency; no schema change; `the reference frontend` stays read-only.

**Scale/Scope**: one statement in `sessions::advance_round` and its test.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | The advance rule is exercised by backend tests needing no UI. | PASS |
| II. Explicit Business Logic | The reset is part of the named `advance_round` transition, covered by a test. | PASS |
| III. Rust Backend | The rule lives in Rust; the frontend only reads the flag. | PASS |
| IV. Backend-Owned Frontend | Unchanged. | PASS |
| V. Small Independent Modules | One statement in the sessions module. | PASS |
| VI. Responsive and Keyboard Accessible UI | No UI change; the control already reflects the flag. | PASS |
| VII. Simplicity Over Abstractions | Reuse the existing flag; one added column in an existing update. | PASS |
| Stack constraints | No schema change, no dependency. | PASS |

Complexity Tracking: no violations.

## Project Structure

### Documentation (this feature)

```text
specs/020-fix-reveal-reset/
|-- plan.md
|-- spec.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   `-- reveal-reset.md
|-- checklists/
|   `-- requirements.md
`-- tasks.md
```

### Source Code (repository root)

```text
backend/
`-- src/modules/sessions/
    |-- mod.rs          # advance_round resets reveal_values when starting a round (EDIT)
    `-- tests.rs        # advance resets reveal; finish keeps it (EDIT)
```

**Structure Decision**: reuse the existing layout. The reset belongs in `advance_round` (the owner of the
round transition); no new module.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the reveal flag and the advance transition.
- [contracts/reveal-reset.md](./contracts/reveal-reset.md) — when the flag resets.
- [quickstart.md](./quickstart.md) — how to verify the reset.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No dependency, no schema change, one statement in one function:
Principles I, II, III, V, VI, and VII hold with no exceptions to record.
