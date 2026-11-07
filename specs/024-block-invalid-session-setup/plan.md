# Implementation Plan: Block Invalid Session Setup

**Branch**: `024-block-invalid-session-setup` | **Date**: 2026-11-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/024-block-invalid-session-setup/spec.md`

## Summary

A session can currently be created from a list that cannot supply enough valued items for the chosen
mode and item count; the failure appears only when the round starts. Fix in three layers:

1. **Shared capacity rule**: add a named `setup_capacity` module that knows every mode's minimum
   (Order 11, Connect 10, Tripmine 10/20), counts only items with a value in the selected column, and
   produces a clear round-specific refusal.
2. **Session creation gate**: run the capacity rule for every round before any session, team, or round
   row is written. Quick games use the same gate, and their default-list picker also filters by valued
   item count so a usable list is not skipped.
3. **Setup UI**: list summaries carry a per-column valued-item count. The list picker hides lists that
   cannot support the mode's smallest size, while a list that becomes incompatible after a mode, size,
   or column change stays selected with an inline error and blocks creation. Unsupported Tripmine sizes
   remain selectable so the gamemaster sees the exact shortfall.

No schema migration and no new dependency.

## Technical Context

**Language/Version**: Rust edition 2024 backend; TypeScript / React 19 frontend (Vite 8).

**Primary Dependencies**: unchanged - Axum 0.8, rusqlite 0.37, serde/serde_json, React 19, Vitest. No new
dependency.

**Storage**: unchanged SQLite database. Capacity is derived from existing `item` and `item_value` rows; no
column or table is added.

**Testing**: `cargo test --manifest-path backend/Cargo.toml` for the capacity rule and session-creation
refusal; `npm --prefix frontend run test` for list hiding, retained invalid selection, and shortfall errors;
`npm --prefix frontend run lint` and `npm --prefix frontend run build` for the frontend.

**Target Platform**: unchanged web application served by the Rust backend; the built frontend is served by
the backend.

**Project Type**: web application with a Rust backend and a React frontend.

**Performance Goals**: no user-perceptible change. A list summary adds one count per column; session creation
performs one count query per round (1-10 rounds) for at most 100 items per list.

**Constraints**: no schema change; no new dependency; existing valid sessions and round starts unchanged;
lists hidden before submission never leak values beyond the summary counts already exposed; `the reference
frontend` stays read-only; build output stays inside the repository.

**Scale/Scope**: 1-10 rounds, 1-4 teams, lists of 11-100 items, a handful of folders and lists per install.
Changes are confined to list summaries, session creation, quick-game list choice, and the existing setup
page.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | The capacity rule is a pure function plus one SQL count helper, unit-tested in memory. Session refusal and setup behavior are tested without network access. | PASS |
| II. Explicit Business Logic | Minimums, counting, and refusal live in a named `setup_capacity` module with direct tests; the UI mirrors the rule but the backend remains authoritative. | PASS |
| III. Rust Backend | Capacity enforcement is Rust code in the session-creation path. The frontend only reflects it. | PASS |
| IV. Backend-Owned Frontend | Unchanged; the backend still serves the exported frontend. | PASS |
| V. Small Independent Modules | The new feature lives in `modules/setup_capacity`; existing modules receive only narrow calls or the per-column count field. | PASS |
| VI. Responsive and Keyboard Accessible UI | The setup page keeps native selects, does not block rendering, keeps controls keyboard operable, and exposes errors through the existing alert region. | PASS |
| VII. Simplicity Over Abstractions | Reuse existing per-mode constants, list summary endpoints, and the existing session transaction. Add one count field and one named rule; no new endpoint, token, cache, or schema. | PASS |
| Stack constraints | Rust backend only, React frontend in-repo, no AppData writes, no tracked build output. | PASS |

Complexity Tracking: no violations.

## Project Structure

### Documentation (this feature)

```text
specs/024-block-invalid-session-setup/
|-- plan.md
|-- spec.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   `-- setup-capacity.md
|-- checklists/
|   `-- requirements.md
`-- tasks.md            # created later by /speckit.tasks
```

### Source Code (repository root)

```text
backend/
|-- src/
|   |-- modules/
|   |   |-- mod.rs                       # register setup_capacity (EDIT)
|   |   |-- setup_capacity/
|   |   |   |-- mod.rs                   # named capacity rule + count query (NEW)
|   |   |   `-- tests.rs                 # pure threshold and message tests (NEW)
|   |   |-- default_lists/mod.rs         # per-column value count in summaries (EDIT)
|   |   |-- sessions/mod.rs              # enforce capacity before the insert (EDIT)
|   |   `-- quick_game/mod.rs            # filter default lists by valued count (EDIT)
|   `-- tests/
|       |-- session_create.rs            # refusal and no-partial-write tests (EDIT)
|       `-- quick_session.rs             # quick-game eligible-list tests (EDIT)

frontend/
|-- src/
|   |-- types/session.ts                 # ListColumnSummary.valueCount (EDIT)
|   |-- pages/SetupQuiz.tsx              # hide lists, retain invalid selection, errors (EDIT)
|   `-- test/SetupQuiz.test.tsx          # capacity UI tests and updated fixtures (EDIT)
```

**Structure Decision**: reuse the existing web-application layout. The new capacity rule gets its own
small module because it is a distinct business rule with its own tests (Principles II and V). List
summaries gain a derived `valueCount` per column instead of a new compatibility endpoint, keeping the
existing folder/list discovery flow intact.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All technical unknowns resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) - derived per-column counts, round compatibility rules, and the
  setup state that retains an incompatible selection.
- [contracts/setup-capacity.md](./contracts/setup-capacity.md) - list summary field, session-creation
  refusal, quick-game behavior, and UI guarantees.
- [quickstart.md](./quickstart.md) - automated and manual validation for every minimum and the Tripmine
  20 case.

## Constitution Re-check after Phase 1

Re-evaluated after writing the artifacts. The design adds one derived count field, one named rule module,
one call in session creation, one quick-game filter, and setup-page filtering/error logic. It adds no
dependency, schema change, endpoint, or stored state. Principles I, II, III, V, VI, and VII hold with no
exceptions to record.
