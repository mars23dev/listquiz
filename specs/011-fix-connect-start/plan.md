# Implementation Plan: Fix Connect Round Start Panel

**Branch**: `011-fix-connect-start` | **Date**: 2026-10-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/011-fix-connect-start/spec.md`

## Summary

The pre-round prompt (the gamemaster's "Start Round" button and everyone else's "Waiting for the
game to be started…" message) is shown whenever no **Order** and no **Tripmine** board exists. That
guard does not account for the **Connect** game type, so once a Connect round is active the prompt
stays on screen: the gamemaster can press the leftover button again, which re-issues `next_round` and
finishes the game. The fix is to include the active Connect board in the pre-round guard, matching how
Order and Tripmine already suppress the prompt. Frontend-only; no backend or data change.

## Technical Context

**Language/Version**: TypeScript / React 19 frontend (Vite); Rust backend unchanged.

**Primary Dependencies**: unchanged — React, Vite, Tailwind, `lucide-react`. No new dependency.

**Storage**: N/A — the guard reads the already-streamed session state; no persistence change.

**Testing**: `vitest` + `@testing-library/react` in `frontend/src/test/Session.test.tsx` (a Connect
session-state fixture asserting the prompt is hidden once the Connect board exists, and shown before
it does). Existing `vitest` and `cargo test` suites stay green.

**Target Platform**: browser served by the Rust backend on `0.0.0.0:8080` (unchanged).

**Project Type**: web application — Rust backend that serves the exported frontend.

**Performance Goals**: unchanged; one boolean condition evaluated during render.

**Constraints**: no new dependency; no build state outside the repo; keyboard accessible (unchanged
controls); `the reference frontend` stays read-only.

**Scale/Scope**: one guard expression in `frontend/src/pages/Session.tsx` and one added test.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | The visibility rule is exercised by a frontend test with a stubbed state; no backend or network needed. | PASS |
| II. Explicit Business Logic | The prompt-visibility predicate is a named, testable expression over session state, not scattered across branches. | PASS |
| III. Rust Backend | No backend change; the client only reads server state. | PASS |
| IV. Backend-Owned Frontend | Unchanged; the frontend stays served by the backend. | PASS |
| V. Small Independent Modules | One guard in the sessions page; no other module touched. | PASS |
| VI. Responsive and Keyboard Accessible UI | Removing a stale panel improves the UI; no interactive control is altered. | PASS |
| VII. Simplicity Over Abstractions | One added condition; no new helper, state, or abstraction. | PASS |
| Stack constraints | No dependency, no schema, `the reference frontend` untouched. | PASS |

Complexity Tracking: no violations, so no exceptions are claimed.

## Project Structure

### Documentation (this feature)

```text
specs/011-fix-connect-start/
|-- plan.md
|-- spec.md
|-- research.md          # Phase 0 output
|-- data-model.md        # Phase 1 output
|-- quickstart.md        # Phase 1 output
|-- contracts/
|   `-- pre-round-prompt.md   # prompt visibility contract
|-- checklists/
|   `-- requirements.md
`-- tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
frontend/
|-- src/
|   |-- pages/Session.tsx          # pre-round prompt guard: add the Connect board (EDIT)
|   `-- test/Session.test.tsx      # Connect board hides the prompt (NEW test)

backend/                            # unchanged
```

**Structure Decision**: reuse the existing layout. The prompt lives in the sessions page and keys off
the same `state` it already reads; the only change is extending the guard that already excludes Order
and Tripmine to also exclude Connect. No new module boundary.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the pre-round prompt visibility predicate and its inputs.
- [contracts/pre-round-prompt.md](./contracts/pre-round-prompt.md) — when the prompt is shown or hidden.
- [quickstart.md](./quickstart.md) — how to verify the Connect start prompt clears.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No new dependency, no schema change, one guarded condition in
one page: Principles I, II, III, V, VI, and VII hold with no exceptions to record.
