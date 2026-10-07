# Implementation Plan: Fix Order And Setup

**Branch**: `003-fix-order-setup` | **Date**: 2026-10-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/003-fix-order-setup/spec.md`

## Summary

Two corrections. First, an Order placement errors out because the session view sends the visual bar
column as if it were the item's insertion position, while a side grows item by item and its items stay
contiguous. The player drops a new item into a **gap** — between two items already on that side or
beside either end — and the game judges it from its neighbours (already implemented server-side). The
client must translate the dropped gap into the insertion position the side actually has, so a drop is
scored and never returns the internal "slot is not on this side" error. Second, the Setup round
configuration shows the tripmine side control for Order rounds; it must be shown only for Tripmine.
Both are frontend corrections; the server's neighbour-based judgement is already correct and stays
strict.

## Technical Context

**Language/Version**: Rust 1.99 backend (unchanged); TypeScript 7 / React 19 frontend (unchanged)

**Primary Dependencies**: unchanged (no new dependency).

**Storage**: unchanged; no schema change. Each bar side already stores its items in placement order,
contiguous.

**Testing**: `vitest` + Testing Library for the two UI fixes; the existing `cargo test` suites must stay
green (the server is not changed). The Playwright harness covers the end-to-end placement.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Project Type**: web application — Rust backend that serves the exported frontend.

**Performance Goals**: unchanged; the fixes add no latency.

**Constraints**: keyboard-operable, no freeze, no full-page reload (Principle VI); no build state
outside the repo.

**Scale/Scope**: two frontend files' behavior — the Order bar's gap-to-position translation in the
session view and the tripmine-side control in Setup.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | Both fixes are covered by component tests that run without a server; the server suites are unchanged. | PASS |
| II. Explicit Business Logic | The gap-to-position translation becomes a named, tested function; neighbour judgement already lives in a named, tested server rule. | PASS |
| III. Rust Backend | Only frontend code changes; the backend is untouched. | PASS |
| IV. Backend-Owned Frontend | Still served by the backend; no standalone mode. | PASS |
| V. Small Independent Modules | Changes stay inside the Order bar component/creation path and the Setup round configuration. | PASS |
| VI. Responsive and Keyboard Accessible UI | The corrected drop targets stay native buttons in DOM order with visible focus; no freeze or reload. | PASS |
| VII. Simplicity Over Abstractions | Reuse the existing bar and its columns; add one small translation function rather than redesigning the layout. | PASS |
| Stack constraints | No new dependency, no schema change, no build-state change. | PASS |

Complexity Tracking: no violations, so no exceptions are claimed.

## Project Structure

### Documentation (this feature)

```text
specs/003-fix-order-setup/
|-- plan.md
|-- spec.md
|-- research.md          # Phase 0 output
|-- data-model.md        # Phase 1 output (no schema change; bar/gap semantics)
|-- quickstart.md        # Phase 1 output
|-- contracts/
|   `-- order-placement.md   # the gap the client may send; Setup visibility
`-- checklists/
    `-- requirements.md
```

### Source Code (repository root)

```text
frontend/
`-- src/
    |-- pages/
    |   |-- Session.tsx        # translate the dropped gap to an insertion position
    |   `-- SetupQuiz.tsx      # show the tripmine side control only for Tripmine rounds
    |-- components/
    |   `-- TopBarOrdering.tsx # bar columns; the drop targets between and beside items
    |-- utils/                 # (if needed) the named gap→position function
    `-- test/                  # component tests for both fixes

backend/                       # unchanged
```

**Structure Decision**: reuse the existing web-application layout. The Order-bar fix lives with the
session view and the bar component; the Setup fix lives in the setup wizard. No new module or
dependency is added.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions are resolved with no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the contiguous side and its gaps; the round-configuration control.
- [contracts/order-placement.md](./contracts/order-placement.md) — the gap a client may send and when
  the tripmine side control is shown.
- [quickstart.md](./quickstart.md) — how to verify both findings.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No new dependency, no schema change, and no cross-module
shortcut: the gap→position translation is one named function, and the Setup change is a visibility
condition. Principles I, II, V, VI, and VII hold with no exceptions to record.
