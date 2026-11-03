# Implementation Plan: Tripmine Mode

**Branch**: `009-tripmine-mode` | **Date**: 2026-10-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/009-tripmine-mode/spec.md`

## Summary

Three Tripmine changes. First, hearts are Order-only: a Tripmine round must not show or change hearts
(the server already never touches them, so this is a display change). Second, the end-of-round cue
sequence: when a round's end also finishes the game, the round cue (e.g. the mine explosion) must play
before the game-end cue; today the `finished` check returns the game-end cue first and skips the
explosion. The cue derivation becomes an ordered list. Third, a new **Random (Hard)** Tripmine side:
the mine is a random item and the searched target is a random remaining non-mine item, reusing the
existing Tripmine scoring.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript 7 / React 19 frontend

**Primary Dependencies**: unchanged — no new dependency.

**Storage**: unchanged; no schema change. The Tripmine side is already a stored string.

**Testing**: `cargo test` for the random mine/target selection and the side parsing; `vitest` for the
ordered cue list, the hidden hearts, and the new setup option. Existing suites stay green.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`).

**Project Type**: web application — Rust backend that serves the exported frontend.

**Performance Goals**: unchanged.

**Constraints**: no new dependency; no build state outside the repo.

**Scale/Scope**: the Tripmine side enum and selection rules, the cue derivation, and the Tripmine UI
(setup option, board label, hidden hearts).

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | The random mine/target selection and the cue list are pure and unit-tested. | PASS |
| II. Explicit Business Logic | `sample_tripmine_grid` / `next_searched_rank` gain the Random rule as named, tested functions; the cue list is one named function. | PASS |
| III. Rust Backend | The side rules live in Rust; the frontend renders the label and hides hearts. | PASS |
| IV. Backend-Owned Frontend | Unchanged; still served by the backend. | PASS |
| V. Small Independent Modules | The change stays in `tripmine_round`, `sessions` (the side enum), and the cue module. | PASS |
| VI. Responsive and Keyboard Accessible UI | The setup option is a native select; no new interactive control elsewhere. | PASS |
| VII. Simplicity Over Abstractions | No new dependency; Random reuses the existing scoring and flow; hearts are only hidden. | PASS |
| Stack constraints | No migration; `the reference frontend` stays read-only. | PASS |

Complexity Tracking: no violations, so no exceptions are claimed.

## Project Structure

### Documentation (this feature)

```text
specs/009-tripmine-mode/
|-- plan.md
|-- spec.md
|-- research.md          # Phase 0 output
|-- data-model.md        # Phase 1 output
|-- quickstart.md        # Phase 1 output
|-- contracts/
|   `-- tripmine-mode.md     # sides, random rule, hearts, cue order
|-- checklists/
|   `-- requirements.md
`-- tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
backend/
`-- src/modules/
    |-- sessions/mod.rs           # TripmineSide::Random (parse/as_str)
    `-- tripmine_round/mod.rs     # random mine + random searched target

frontend/
`-- src/
    |-- pages/SetupQuiz.tsx       # Random (Hard) option
    |-- pages/Session.tsx         # pass the side through; hide hearts; play cue list
    |-- components/BottomTeams.tsx    # showHearts prop
    |-- components/TripmineBoard.tsx  # side label incl. Random
    |-- types/quiz.ts             # TripmineState side gains 'random'
    `-- utils/sessionSounds.ts    # deriveSessionCues (ordered)
```

**Structure Decision**: reuse the existing layout. The Random rule lives in `tripmine_round` (the
module that owns the mine and the search); the side enum gains one variant; the cue derivation becomes
an ordered list. No new module boundary.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the side variants, the Random selection rules, and the cue list.
- [contracts/tripmine-mode.md](./contracts/tripmine-mode.md) — the sides, hearts, and cue order.
- [quickstart.md](./quickstart.md) — how to verify all three findings.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. No new dependency, no schema change, one side variant and one
ordered cue function: Principles I, II, III, V, VI, and VII hold with no exceptions to record.
