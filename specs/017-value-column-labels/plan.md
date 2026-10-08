# Implementation Plan: Value Column Labels

**Branch**: `017-value-column-labels` | **Date**: 2026-10-08 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/017-value-column-labels/spec.md`

## Summary

Show the round's value-column name (e.g. "Area (km²)") wherever a value is displayed, in every game mode,
without changing value visibility. The name already exists in the session state
(`state.round.columnName`); pass it to the boards and render it as a label next to the values (the Connect
value column heading, a label on the Order bar/anchor, and a label on the Tripmine values).

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript / React 19 frontend (Vite).

**Primary Dependencies**: unchanged — no new dependency.

**Storage**: unchanged; the round's column name is already served (`RoundSummary.columnName`).

**Testing**: `vitest` for each board showing the column label where values are shown, and not where they
are hidden. Backend unchanged, so `cargo test` stays green as a regression check.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Constraints**: no new dependency; keyboard accessible; the label must not add scrolling; `the reference frontend`
stays read-only.

**Scale/Scope**: one prop threaded to three board components plus small label markup in each.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | Each board's label is a component test; no backend or network. | PASS |
| II. Explicit Business Logic | The label is the round's column name (data); the "show only where a value shows" rule is a clear conditional. | PASS |
| III. Rust Backend | No backend change; the name is already served. | PASS |
| IV. Backend-Owned Frontend | Unchanged. | PASS |
| V. Small Independent Modules | One prop on existing board components; no new module. | PASS |
| VI. Responsive and Keyboard Accessible UI | A non-interactive label; no new control, no input blocking. | PASS |
| VII. Simplicity Over Abstractions | Reuse the existing `columnName`; no new state or type. | PASS |
| Stack constraints | No schema change, no dependency. | PASS |

Complexity Tracking: no violations.

## Project Structure

### Documentation (this feature)

```text
specs/017-value-column-labels/
|-- plan.md
|-- spec.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   `-- value-labels.md
|-- checklists/
|   `-- requirements.md
`-- tasks.md
```

### Source Code (repository root)

```text
frontend/
`-- src/
    |-- pages/Session.tsx               # pass state.round.columnName to the boards (EDIT)
    |-- components/ConnectBoard.tsx     # value-column heading (EDIT)
    |-- components/TopBarOrdering.tsx   # label beside the Order values (EDIT)
    |-- components/TripmineBoard.tsx    # label beside the Tripmine values (EDIT)
    `-- components/*.test.tsx           # label shown where values show (EDIT)
```

**Structure Decision**: reuse the existing layout. The column name is data the page already holds, so it
is passed to the board that renders the values; each board adds a small label. No new module.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the column name, where values show, and the label rule.
- [contracts/value-labels.md](./contracts/value-labels.md) — when the label is shown.
- [quickstart.md](./quickstart.md) — how to verify the label in every mode.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No dependency, no backend change, one prop and small labels:
Principles I, II, III, V, VI, and VII hold with no exceptions to record.
