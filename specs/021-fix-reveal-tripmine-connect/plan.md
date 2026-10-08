# Implementation Plan: Reveal Values In Tripmine And Connect

**Branch**: `021-fix-reveal-tripmine-connect` | **Date**: 2026-10-08 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/021-fix-reveal-tripmine-connect/spec.md`

## Summary

"Reveal values" is a no-op in Tripmine and Connect. Fix:

1. **Tripmine**: in `app_state::redact`, a player's item becomes revealed when the session's reveal flag is
   set (`revealed = is_gamemaster || item.revealed || reveal`), so the value, rank, and mine show — the
   board already keys visibility on `revealed`.
2. **Connect**: the values are public, so the hidden thing is the pairing. The board already has an
   `showAnswerKey` prop (gamemaster-only); the page passes it for the gamemaster **or** when the reveal flag
   is set.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript / React 19 frontend (Vite).

**Primary Dependencies**: unchanged — no new dependency.

**Storage**: unchanged; `game_session.reveal_values` already exists.

**Testing**: `cargo test` for the Tripmine redaction under the reveal flag; `vitest` for the Connect answer
key being shown to a player when the reveal flag is set (and hidden otherwise). Existing suites stay green.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Constraints**: no new dependency; no schema change; `the reference frontend` stays read-only.

**Scale/Scope**: one expression in `redact` and one prop expression in `Session.tsx`, plus tests.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | The redaction is a unit test; the Connect key is a component/session test. No network. | PASS |
| II. Explicit Business Logic | The reveal rule is a clear conditional in `redact`; the Connect rule reuses the existing prop. | PASS |
| III. Rust Backend | The Tripmine rule lives in Rust; the frontend only reflects the flag. | PASS |
| IV. Backend-Owned Frontend | Unchanged. | PASS |
| V. Small Independent Modules | Changes stay in `app_state` and `Session`; no new module. | PASS |
| VI. Responsive and Keyboard Accessible UI | No new control; the existing board updates. | PASS |
| VII. Simplicity Over Abstractions | Reuse the reveal flag and the existing `showAnswerKey` prop. | PASS |
| Stack constraints | No schema change, no dependency. | PASS |

Complexity Tracking: no violations.

## Project Structure

### Documentation (this feature)

```text
specs/021-fix-reveal-tripmine-connect/
|-- plan.md
|-- spec.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   `-- reveal-modes.md
|-- checklists/
|   `-- requirements.md
`-- tasks.md
```

### Source Code (repository root)

```text
backend/
`-- src/app_state.rs          # Tripmine items reveal under the flag (EDIT + test)

frontend/
`-- src/
    |-- pages/Session.tsx     # Connect answer key shown when revealed (EDIT)
    `-- test/Session.test.tsx # reveal shows the Connect key to a player (EDIT)
```

**Structure Decision**: reuse the existing layout. The Tripmine rule belongs in `redact` (the owner of
role redaction); the Connect rule reuses the board's existing answer-key prop at the page. No new module.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the reveal flag and what each mode unhides.
- [contracts/reveal-modes.md](./contracts/reveal-modes.md) — per-mode reveal behavior.
- [quickstart.md](./quickstart.md) — how to verify the reveal in all three modes.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No dependency, no schema change, one redaction expression and one
prop expression: Principles I, II, III, V, VI, and VII hold with no exceptions to record.
