# Implementation Plan: Expand Default Lists And Add Countries

**Branch**: `014-default-lists-countries` | **Date**: 2026-10-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/014-default-lists-countries/spec.md`

## Summary

Two content changes to the shipped seed data plus one small UI capability:

1. **Expand the three shipped lists** (World History 11→30, Iconic Statues 11→25, Blockbuster Cinema 11→30) in `backend/src/modules/default_lists/mod.rs`.
2. **Add a Countries list** of all 195 sovereign states: icon = national flag, values = area (km²), population, date founded. The list is exempt from the 100-item cap because default lists are read-only and are seeded directly, bypassing `validate_list_size` (which only guards user-created/edited lists).
3. **Render flag icons**: add the `flag-icons` package and teach `ItemIcon` a `flag:<cc>` icon-name convention (e.g. `flag:de`), offline and working on Windows.

Seeding becomes idempotent-upsert so an already-seeded database gets correct item counts and metadata, not just new rows.

## Technical Context

**Language/Version**: Rust 1.99 backend; TypeScript / React 19 frontend (Vite).

**Primary Dependencies**: one new frontend dependency, `flag-icons` (MIT). No backend dependency added.

**Storage**: unchanged schema. Seed data lives in the `default_lists` constants and is written to `quiz_list` / `item` / `item_value` / `list_item_column`; `item.icon_name` carries the flag code.

**Testing**: `cargo test` for the shipped-list rules (counts, unique ids, finite values, flag-name format, Countries size); `vitest` for `ItemIcon` flag rendering. Existing suites that assert the old counts are updated.

**Target Platform**: unchanged (backend serves the exported frontend on `0.0.0.0:8080`); flags must render on Windows.

**Constraints**: no network at runtime (flags bundled); no build state outside the repo; keyboard accessible; `the reference frontend` stays read-only.

**Scale/Scope**: ~30 + 25 + 30 expanded items and ~195 new country items authored in one seed module; one component change; seed upsert; test updates.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | Shipped-list rules are unit-tested in `default_lists`; flag rendering is component-tested. No network. | PASS |
| II. Explicit Business Logic | Seed validity is the existing named `validate_list_size`/value rules; the flag-name convention is one tested parse in `ItemIcon`. | PASS |
| III. Rust Backend | Seed data and rules stay in Rust; only presentation is in the frontend. | PASS |
| IV. Backend-Owned Frontend | Unchanged; the built frontend is served by the backend. | PASS |
| V. Small Independent Modules | Changes stay in `default_lists` and `ItemIcon`; no new module. | PASS |
| VI. Responsive and Keyboard Accessible UI | Flags are decorative images beside existing rows; no new control, no input blocking. | PASS |
| VII. Simplicity Over Abstractions | One dependency with a concrete requirement (flags cannot be done offline on Windows without one); no new state or schema. | PASS |
| Stack constraints | No migration, `the reference frontend` untouched, bundled flags (no runtime network). | PASS |

Complexity Tracking: the single added dependency (`flag-icons`) is justified by FR-005 (flags must render offline on Windows); no other exceptions.

## Project Structure

### Documentation (this feature)

```text
specs/014-default-lists-countries/
|-- plan.md
|-- spec.md
|-- research.md          # Phase 0 output
|-- data-model.md        # Phase 1 output
|-- quickstart.md        # Phase 1 output
|-- contracts/
|   `-- shipped-lists.md     # seed shape, flag-name convention, limit rule
|-- checklists/
|   `-- requirements.md
`-- tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
backend/
`-- src/modules/default_lists/
    |-- mod.rs            # expand items; add COUNTRIES_*; upsert seed (EDIT)
    `-- tests.rs          # counts, ids, values, flag format, Countries (EDIT)

frontend/
|-- package.json          # add flag-icons (EDIT)
`-- src/
    |-- main.tsx          # import flag-icons CSS (EDIT)
    |-- components/ItemIcon.tsx       # flag:<cc> rendering (EDIT)
    `-- components/ItemIcon.test.tsx  # flag render test (NEW)
```

**Structure Decision**: reuse the existing layout. Seed content and its rules live in the
`default_lists` module (the owner of shipped data); flag rendering lives in `ItemIcon` (the owner of
item icons). No new module boundary.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|--------------------------------------|
| New frontend dependency `flag-icons` | FR-005: each country shows its flag, offline, on Windows | Emoji flags do not render on Windows; vendoring ~195 SVGs is more files and upkeep than a small MIT package |

## Phase 0: Research

See [research.md](./research.md). All decisions resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — shipped list/item/value shapes, the Countries columns, the flag name.
- [contracts/shipped-lists.md](./contracts/shipped-lists.md) — seed shape, flag convention, and the size-limit rule.
- [quickstart.md](./quickstart.md) — how to verify the expanded lists, the Countries list, and flags.

## Constitution Re-check after Phase 1

Re-evaluated once the artifacts existed. One justified dependency, no schema change, changes confined to
`default_lists` and `ItemIcon`: Principles I, II, III, V, VI, and VII hold (VII with the single
dependency exception recorded above).
