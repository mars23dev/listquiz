# Implementation Plan: Board Icon And Layout Fixes

**Branch**: `016-board-icon-fixes` | **Date**: 2026-10-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/016-board-icon-fixes/spec.md`

## Summary

Three fixes:

1. **Item icons in every game type.** Order and Tripmine item payloads omit the list icon, so their boards
   show a generic placeholder while Connect shows the real icon. Carry `icon_name` through
   `ListItemValue` → `OrderItem`/`TripmineItem` (and `AnchorItem`) and through the frontend types and
   mappers, so every board renders the item's own icon (flags included). No game type overrides it.
2. **Tripmine bar matches the item count.** The rank ladder is hardcoded `[1..10]`; build it from the
   number of items (10 or 20).
3. **Connect rows equal height.** Give both the name cell and the value cell one fixed height so the two
   columns are identical and aligned, with the icon not affecting the row height.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript / React 19 frontend (Vite).

**Primary Dependencies**: unchanged — no new dependency.

**Storage**: unchanged; `item.icon_name` already exists. The session views read it in their queries.

**Testing**: `cargo test` for the icon carried into the Order/Tripmine views; `vitest` for the Tripmine
bar position count and the Connect equal-height rows. Existing suites stay green.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Constraints**: no new dependency; keyboard accessible; no build state outside the repo; `the reference frontend`
stays read-only.

**Scale/Scope**: one DTO field across three structs + their queries/mappers; one bar range; one CSS
height.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | The icon is asserted in the backend view tests; the bar and rows are component tests. No network. | PASS |
| II. Explicit Business Logic | The icon comes from the item (data), not from a mode; the bar count is derived from the item list. | PASS |
| III. Rust Backend | The DTO/queries stay in Rust; the frontend only renders. | PASS |
| IV. Backend-Owned Frontend | Unchanged. | PASS |
| V. Small Independent Modules | Changes stay in `state_reader`/`app_state`, the mappers, and the two components. | PASS |
| VI. Responsive and Keyboard Accessible UI | No new control; the fixed row height keeps rows aligned. | PASS |
| VII. Simplicity Over Abstractions | Add one existing field to existing structs; no new type or channel. | PASS |
| Stack constraints | No schema change, no dependency. | PASS |

Complexity Tracking: no violations.

## Project Structure

### Documentation (this feature)

```text
specs/016-board-icon-fixes/
|-- plan.md
|-- spec.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   `-- board-fixes.md
|-- checklists/
|   `-- requirements.md
`-- tasks.md
```

### Source Code (repository root)

```text
backend/
`-- src/
    |-- app_state.rs        # icon_name on OrderItem/TripmineItem/AnchorItem (EDIT)
    |-- state_reader.rs     # load + map the icon in the Order/Tripmine views (EDIT)
    `-- modules/sessions/round_state/mod.rs   # ListItemValue.icon_name (EDIT)

frontend/
`-- src/
    |-- types/session.ts             # iconName on OrderItem/TripmineItem (EDIT)
    |-- pages/Session.tsx            # map iconName in toQuizItem/toTripmineItem (EDIT)
    |-- components/TripmineBoard.tsx # rank ladder from items.length (EDIT)
    `-- components/ConnectBoard.tsx  # fixed equal row height (EDIT)
```

**Structure Decision**: reuse the existing layout. The icon is data on the item, so it is added to the
existing item structs and carried through the existing mappers; the bar and row fixes are local to the
two components. No new module.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the icon field on the item views; the bar and row rules.
- [contracts/board-fixes.md](./contracts/board-fixes.md) — icon source, bar count, row sizing.
- [quickstart.md](./quickstart.md) — how to verify all three fixes.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No dependency, no schema change, one carried field and two
local fixes: Principles I, II, III, V, VI, and VII hold with no exceptions to record.
