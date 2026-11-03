# Implementation Plan: Connect Value Order And GM Answer Key

**Branch**: `012-connect-order-values` | **Date**: 2026-10-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/012-connect-order-values/spec.md`

## Summary

Two connected improvements to the Connect game:

1. **Value column keeps the list's order.** Today `sample_connect_round` shuffles both columns. Change
   it so only the **names** are shuffled; the drawn **values** stay in the list's item order (the same
   order `load_pairs` already returns, `ORDER BY i.position`). No sorting algorithm is added.
2. **Gamemaster-only answer key.** In the gamemaster view, each name row shows its correct matching
   value for the whole round; players and the spectator never see it. The pairing is already derivable
   from the payload (a name and its value share `itemId`), so this is a display rule applied in the
   client for the gamemaster role — no backend role filtering and no schema change.

Both are presentation/ordering changes; scoring, hearts, matching, and turn logic are untouched.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript / React 19 frontend (Vite).

**Primary Dependencies**: unchanged — `rand` (backend), React/Tailwind (frontend). No new dependency.

**Storage**: unchanged; no schema change. The board (names, values, matched, pending) is already saved
as JSON in `connect_state`; keeping values in list order needs no migration.

**Testing**: `cargo test` for `sample_connect_round` (values in list order, names shuffled);
`vitest` for `ConnectBoard` (GM key shown, players/spectator not) and `Session` (role wiring). Existing
suites stay green.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Project Type**: web application — Rust backend that serves the exported frontend.

**Constraints**: no new dependency; keyboard accessible; no build state outside the repo;
`the reference frontend` stays read-only.

**Scale/Scope**: one backend function (`sample_connect_round`) plus its tests; one component prop
(`ConnectBoard`) and its `Session` call site plus tests.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | Ordering is unit-tested in `connect_round`; the GM key is component-tested in `ConnectBoard`; neither needs the full app. | PASS |
| II. Explicit Business Logic | The value order is a named rule in `sample_connect_round`; the key is a small, testable derivation in the component. | PASS |
| III. Rust Backend | The ordering rule lives in Rust; the frontend only presents it. | PASS |
| IV. Backend-Owned Frontend | Unchanged; the board is served by the backend. | PASS |
| V. Small Independent Modules | Change stays in `connect_round` and `ConnectBoard`; no new module. | PASS |
| VI. Responsive and Keyboard Accessible UI | The GM key is added to existing name rows; no new interactive control, no input blocking. | PASS |
| VII. Simplicity Over Abstractions | No sorting, no schema, no backend role channel; reuse the existing `itemId` pairing. | PASS |
| Stack constraints | No dependency, no migration, `the reference frontend` untouched. | PASS |

Complexity Tracking: no violations, so no exceptions are claimed.

## Project Structure

### Documentation (this feature)

```text
specs/012-connect-order-values/
|-- plan.md
|-- spec.md
|-- research.md          # Phase 0 output
|-- data-model.md        # Phase 1 output
|-- quickstart.md        # Phase 1 output
|-- contracts/
|   `-- connect-board.md     # value order + GM answer key visibility
|-- checklists/
|   `-- requirements.md
`-- tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
backend/
`-- src/modules/connect_round/
    |-- mod.rs            # sample_connect_round(): stop shuffling values (EDIT)
    `-- tests.rs          # assert list order + shuffled names (EDIT)

frontend/
`-- src/
    |-- components/ConnectBoard.tsx       # GM-only answer key prop + render (EDIT)
    |-- components/ConnectBoard.test.tsx  # GM vs player visibility (EDIT)
    |-- pages/Session.tsx                 # pass the gamemaster flag (EDIT)
    `-- test/Session.test.tsx             # role wiring (EDIT)
```

**Structure Decision**: reuse the existing layout. The ordering rule is one function in the
`connect_round` module (the owner of the board); the answer key is a render decision in `ConnectBoard`
(the owner of the board UI). No new module boundary.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the board columns, list order, and the derived GM answer key.
- [contracts/connect-board.md](./contracts/connect-board.md) — value order rule and key visibility.
- [quickstart.md](./quickstart.md) — how to verify both changes.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No new dependency, no schema change, one ordering rule in one
function and one render decision in one component: Principles I, II, III, V, VI, and VII hold with no
exceptions to record.
