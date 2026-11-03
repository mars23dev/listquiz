# Implementation Plan: Per-Game Ordering

**Branch**: `019-per-game-ordering` | **Date**: 2026-10-08 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/019-per-game-ordering/spec.md`

## Summary

Remove the per-list order direction (`order_direction`, the higher/lower choice) from list creation,
editing, CSV import, the API payloads, and the database. Ordering becomes a property of the game mode:
`QuizType::sort_order()` returns ascending for Order, Tripmine, and Connect (matching current behavior);
the Order round uses that instead of the list's direction. A migration drops the column so existing
databases upgrade cleanly.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript / React 19 frontend (Vite).

**Primary Dependencies**: unchanged — no new dependency.

**Storage**: one migration: `ALTER TABLE quiz_list DROP COLUMN order_direction` (guarded like the existing
additive migrations); the column is removed from the `CREATE TABLE` schema.

**Testing**: `cargo test` for the per-game sort order and for list create/import without a direction;
`vitest` for the list editor no longer offering a direction. Existing tests that send/assert the direction
are updated.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Constraints**: no new dependency; keyboard accessible; `the reference frontend` stays read-only.

**Scale/Scope**: remove one field across list create/edit/import and the schema; add one per-mode ordering
rule; update the Order round to use it.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | The per-mode order and the no-direction list create/import are unit-tested; no network. | PASS |
| II. Explicit Business Logic | `QuizType::sort_order()` is a named, tested rule; the removal is a field change, not inlined logic. | PASS |
| III. Rust Backend | Ordering lives in Rust; the frontend stops sending the field. | PASS |
| IV. Backend-Owned Frontend | Unchanged. | PASS |
| V. Small Independent Modules | Change stays in lists/csv_import/round_state and the list editor; no new module. | PASS |
| VI. Responsive and Keyboard Accessible UI | Removing a control; no new interaction. | PASS |
| VII. Simplicity Over Abstractions | One small enum + method; remove a field rather than add one. | PASS |
| Stack constraints | One guarded migration; `the reference frontend` untouched. | PASS |

Complexity Tracking: no violations.

## Project Structure

### Documentation (this feature)

```text
specs/019-per-game-ordering/
|-- plan.md
|-- spec.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   `-- ordering.md
|-- checklists/
|   `-- requirements.md
`-- tasks.md
```

### Source Code (repository root)

```text
backend/
`-- src/
    |-- db.rs                              # drop order_direction from schema + migration (EDIT)
    |-- modules/default_lists/mod.rs       # drop the field from DefaultList + seed (EDIT)
    |-- modules/lists/mod.rs               # drop from ListInput/NewList/ListDetail + create/update (EDIT)
    |-- modules/csv_import/mod.rs          # drop from ParsedList + parse_csv (EDIT)
    |-- routes/lists.rs                    # drop from the import request (EDIT)
    |-- modules/sessions/mod.rs            # QuizType::sort_order() (EDIT)
    `-- modules/sessions/round_state/mod.rs# use the game mode's order (EDIT)

frontend/
`-- src/
    |-- pages/ListManagement.tsx           # remove the direction selectors (EDIT)
    |-- api/client.ts                      # drop orderDirection from payloads (EDIT)
    |-- types/quiz.ts, types/session.ts    # drop orderDirection (EDIT)
    `-- test/                              # updated tests (EDIT)
```

**Structure Decision**: reuse the existing layout. The direction field is removed from the modules that
own lists/import; the per-mode rule lives on `QuizType` (sessions). No new module.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the list without a direction; the per-game ordering rule.
- [contracts/ordering.md](./contracts/ordering.md) — no direction on lists; per-mode order.
- [quickstart.md](./quickstart.md) — how to verify the removal and per-mode ordering.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No dependency, one guarded migration, one named rule: Principles
I, II, III, V, VI, and VII hold with no exceptions to record.
