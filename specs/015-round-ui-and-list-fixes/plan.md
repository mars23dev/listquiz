# Implementation Plan: Round UI And List Fixes

**Branch**: `015-round-ui-and-list-fixes` | **Date**: 2026-10-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/015-round-ui-and-list-fixes/spec.md`

## Summary

Four independent fixes in one feature:

1. **Ordered, numeric shipped lists.** Seed each list ascending by its value column (History Year, Statues
   Height, Cinema Box Office, Countries Area). Store the true numeric value and derive the display with a
   magnitude formatter (`146000000` → "146 Million", `1440000000` → "1.44 Billion"); cinema numerics become
   real dollars. Seeding upserts item positions so an existing database is reordered too.
2. **Connect columns equal size.** Give name and value rows the same min-height and padding so both columns
   look equally full.
3. **Tripmine 10 or 20 items.** Add an optional per-round item count (10 or 20) to setup and round config,
   sample/rank across the chosen count, and scale the board to fit one screen with no scrolling.
4. **Gamemaster-only view switch.** Show the header role switch only to the gamemaster; activating it opens
   the chosen view in a new tab and leaves the gamemaster's page in place.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript / React 19 frontend (Vite).

**Primary Dependencies**: unchanged — no new dependency.

**Storage**: one additive migration: `round_config.tripmine_slots INTEGER` (nullable, default ten when
absent), following the existing `ALTER TABLE ... ADD COLUMN` migration pattern in `db.rs`. No other schema
change; shipped item positions are refreshed by the seed upsert.

**Testing**: `cargo test` for the formatter, list ordering, and Tripmine slot count/ranking; `vitest` for
`ConnectBoard` equal sizing, `TripmineBoard` twenty-item layout, and the `Session` view switch. Existing
suites that assert old displays/positions are updated.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Constraints**: no new dependency; keyboard accessible; no build state outside the repo; `the reference frontend`
stays read-only; twenty-item Tripmine must fit the standard window without scrolling.

**Scale/Scope**: one formatter + seed reorder/upsert; one component sizing tweak; a round-config field
across validation/sampling/UI; one header control change.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | Formatter, ordering, and slot rules are unit-tested in Rust; the UI checks are component tests. No network. | PASS |
| II. Explicit Business Logic | Value formatting and Tripmine slot/rank rules are named functions, tested, not inlined. | PASS |
| III. Rust Backend | Ordering, formatting, and the slot count live in Rust; the frontend only presents them. | PASS |
| IV. Backend-Owned Frontend | Unchanged; the built frontend is served by the backend. | PASS |
| V. Small Independent Modules | Changes stay in `default_lists`, `tripmine_round`/`sessions`, and three components; no new module beyond a tiny value-format helper. | PASS |
| VI. Responsive and Keyboard Accessible UI | The switch stays a button; the twenty-item board scales, no input blocking. | PASS |
| VII. Simplicity Over Abstractions | No dependency; one formatter, one nullable column, one min-height; reuse existing upsert/migration patterns. | PASS |
| Stack constraints | Additive migration only; `the reference frontend` untouched. | PASS |

Complexity Tracking: no violations.

## Project Structure

### Documentation (this feature)

```text
specs/015-round-ui-and-list-fixes/
|-- plan.md
|-- spec.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   `-- round-fixes.md
|-- checklists/
|   `-- requirements.md
`-- tasks.md
```

### Source Code (repository root)

```text
backend/
|-- src/db.rs                          # additive tripmine_slots column (EDIT)
|-- src/modules/default_lists/mod.rs   # reorder + numeric display + item upsert (EDIT)
|-- src/modules/default_lists/tests.rs # ordering + formatter tests (EDIT)
|-- src/modules/sessions/mod.rs        # RoundInput/ValidRound tripmineSlots (EDIT)
|-- src/modules/tripmine_round/mod.rs  # sample/rank by slot count (EDIT)
`-- src/routes/sessions.rs             # pass the slot count when starting Tripmine (EDIT)

frontend/
`-- src/
    |-- components/ConnectBoard.tsx       # equal cell sizing (EDIT)
    |-- components/TripmineBoard.tsx      # fit 20 items on one screen (EDIT)
    |-- pages/Session.tsx                 # GM-only switch, new-tab open (EDIT)
    |-- pages/SetupQuiz.tsx               # Tripmine 10/20 choice (EDIT)
    `-- test/                             # component/session tests (EDIT/NEW)
```

**Structure Decision**: reuse the existing layout. Each fix stays in the module that owns it
(`default_lists` for data, `tripmine_round`/`sessions` for the round option, the board components for
presentation). No new module boundary except a small value-format helper inside `default_lists`.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — value formatting, list ordering, the Tripmine slot field, the view switch.
- [contracts/round-fixes.md](./contracts/round-fixes.md) — value display, Tripmine slots, view switch.
- [quickstart.md](./quickstart.md) — how to verify all four fixes.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No dependency, one additive migration, named tested rules:
Principles I, II, III, V, VI, and VII hold with no exceptions to record.
