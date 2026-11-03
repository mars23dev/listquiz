# Implementation Plan: Quick Game Connect And Random Lists

**Branch**: `018-quick-game-connect-random` | **Date**: 2026-10-08 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/018-quick-game-connect-random/spec.md`

## Summary

Two small changes:

1. **Offer Connect as a quick game.** The backend already builds a Connect quick game; only the landing
   page (and its handler/type) omit it. Add the Connect choice and widen the game-type union.
2. **Random eligible default list.** `quick_game::build_request` currently picks the first fitting list;
   change it to pick uniformly at random among the lists that fit the game type (enough items with values,
   at least one column), so repeats vary. When only one fits, that list is used; when none fits, it errors.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript / React 19 frontend (Vite).

**Primary Dependencies**: unchanged — `rand` is already a backend dependency; no new dependency.

**Storage**: unchanged; quick games use the shipped default lists.

**Testing**: `cargo test` for the random eligible-list choice (seeded rng: varies; single-fit deterministic;
no-fit errors); `vitest` for the landing page offering Connect and the quick handler accepting it.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Constraints**: no new dependency; keyboard accessible; `the reference frontend` stays read-only.

**Scale/Scope**: one function signature (`build_request` gains an rng) plus its call site; one frontend
union widened and one button added.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | The random choice is unit-tested with a seeded rng; the landing choice is a component test. | PASS |
| II. Explicit Business Logic | The eligible-list + random choice is a named function, tested, not inlined. | PASS |
| III. Rust Backend | The list choice stays in Rust; the frontend only sends the game type. | PASS |
| IV. Backend-Owned Frontend | Unchanged. | PASS |
| V. Small Independent Modules | Change stays in `quick_game` and the landing/handler; no new module. | PASS |
| VI. Responsive and Keyboard Accessible UI | A new button alongside existing ones; no input blocking. | PASS |
| VII. Simplicity Over Abstractions | Reuse the existing fitting-list predicate; pass an rng; no new state. | PASS |
| Stack constraints | No schema change, no dependency. | PASS |

Complexity Tracking: no violations.

## Project Structure

### Documentation (this feature)

```text
specs/018-quick-game-connect-random/
|-- plan.md
|-- spec.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   `-- quick-game.md
|-- checklists/
|   `-- requirements.md
`-- tasks.md
```

### Source Code (repository root)

```text
backend/
`-- src/
    |-- modules/quick_game/mod.rs    # random eligible list (EDIT)
    |-- modules/quick_game/tests.rs  # random-choice tests (EDIT)
    `-- routes/sessions.rs           # pass an rng to build_request (EDIT)

frontend/
`-- src/
    |-- pages/Landing.tsx            # Connect quick-game button + type (EDIT)
    |-- App.tsx                      # widen the quick game-type union (EDIT)
    |-- api/client.ts                # widen createQuickSession type (EDIT)
    `-- test/App.test.tsx            # quick game still starts; Connect offered (EDIT)
```

**Structure Decision**: reuse the existing layout. The random list choice belongs in the `quick_game`
module (its owner); the landing page adds one button. No new module.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — game type, eligible list, random choice.
- [contracts/quick-game.md](./contracts/quick-game.md) — eligible-list rule and the quick-game offer.
- [quickstart.md](./quickstart.md) — how to verify Connect and the random list.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No dependency, no schema change, one function signature and one
button: Principles I, II, III, V, VI, and VII hold with no exceptions to record.
