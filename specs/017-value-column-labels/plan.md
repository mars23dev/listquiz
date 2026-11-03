# Implementation Plan: Value Column Labels

**Branch**: `017-value-column-labels` | **Date**: 2026-10-08 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/017-value-column-labels/spec.md`

## Summary

Show the round's value-column name wherever a value is displayed, in every game mode, without changing
value visibility. The name already exists in the session state (`state.round.columnName`); pass it to the
boards and render it as a label next to the values. Additionally (clarification): a revealed **Order** item
must show its value **and** column name in the **middle of the card**, in the same position as the anchor —
not in the card footer.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript / React 19 frontend (Vite).

**Primary Dependencies**: unchanged — no new dependency.

**Storage**: unchanged; the round's column name is already served (`RoundSummary.columnName`).

**Testing**: `vitest` for each board showing the column label where values are shown, for the Order slot
value appearing in the middle (not the footer), and for no label where a value is hidden. Backend unchanged.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Constraints**: no new dependency; keyboard accessible; the label must not add scrolling; `the reference frontend`
stays read-only.

**Scale/Scope**: one prop threaded to three board components plus label markup; one Order slot layout move
(footer → middle) in `TopBarOrdering.tsx`.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | Each board's label and the Order slot placement are component tests; no backend or network. | PASS |
| II. Explicit Business Logic | The label is the round's column name (data); "show only where a value shows" and "middle, like the anchor" are clear conditions. | PASS |
| III. Rust Backend | No backend change; the name is already served. | PASS |
| IV. Backend-Owned Frontend | Unchanged. | PASS |
| V. Small Independent Modules | One prop on existing board components; no new module. | PASS |
| VI. Responsive and Keyboard Accessible UI | Non-interactive label; the slot value moves within the same button; no new control. | PASS |
| VII. Simplicity Over Abstractions | Reuse the existing `columnName` and the anchor's value-box markup; no new state or type. | PASS |
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
    |-- components/TopBarOrdering.tsx   # label + move revealed slot value to the middle (EDIT)
    |-- components/TripmineBoard.tsx    # label beside the Tripmine values (EDIT)
    `-- components/*.test.tsx           # label + Order placement tests (EDIT)
```

**Structure Decision**: reuse the existing layout. The column name is data the page already holds; the
Order slot value reuses the anchor's value-box markup, moved from the footer to the middle. No new module.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the column name, where values show, and the label/placement rules.
- [contracts/value-labels.md](./contracts/value-labels.md) — when the label is shown and where.
- [quickstart.md](./quickstart.md) — how to verify the label and the Order placement.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No dependency, no backend change, one prop and small labels plus
one layout move: Principles I, II, III, V, VI, and VII hold with no exceptions to record.
