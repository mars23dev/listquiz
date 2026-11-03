---
description: "Task list for default-lists-countries"
---

# Tasks: Expand Default Lists And Add Countries

**Input**: Design documents from `/specs/014-default-lists-countries/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included. The constitution (Principle II) requires every business rule to be covered by a test, and the plan makes the rules testable. Write the tests first and confirm they fail before the implementation change.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2)
- Include exact file paths in descriptions

## Path Conventions

- **Web app**: `backend/src/`, `frontend/src/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm both toolchains run before changing anything.

- [X] T001 Run the baseline suites: `cargo test --manifest-path backend/Cargo.toml default_lists` and `npm --prefix frontend run test`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST exist before the stories.

**No foundational tasks.** The seed module, the `item.icon_name` column, `ItemIcon`, and the list-size rules all exist. Proceed to the user stories.

**Checkpoint**: Foundation ready (nothing to build).

---

## Phase 3: User Story 1 - The shipped lists carry more items (Priority: P1) 🎯 MVP

**Goal**: World History, Iconic Statues, and Blockbuster Cinema grow to 30/25/30 items, and reseeding an existing database leaves correct counts.

**Independent Test**: `cargo test default_lists` shows the three lists within 20–50 items with unique ids and finite values; a second seed run keeps counts correct.

### Tests for User Story 1

> **NOTE: Update the test FIRST, confirm it FAILS, then implement T003/T004.**

- [X] T002 [US1] In `backend/src/modules/default_lists/tests.rs`, replace the "exactly 11 items" assertion with a check that each of the three existing lists (`default_world_history`, `default_statues_monuments`, `default_cinema_box_office`) holds between 20 and 50 items, keeping the unique-id and non-empty/finite-value checks. Leave `DEFAULT_LISTS.len()` as-is for now (US2 changes it). Run `cargo test --manifest-path backend/Cargo.toml default_lists` and confirm it fails before T003.

### Implementation for User Story 1

- [X] T003 [US1] In `backend/src/modules/default_lists/mod.rs`, expand `HISTORY_ITEMS` to 30, `STATUE_ITEMS` to 25, and `CINEMA_ITEMS` to 30 items, keeping each list's existing columns, `order_direction`, icon names, unique ids, and `(column_id, display, numeric)` value shape with finite numbers.
- [X] T004 [US1] In `backend/src/modules/default_lists/mod.rs` `seed_default_lists`, upsert the `quiz_list` row instead of `INSERT OR IGNORE`: `INSERT ... ON CONFLICT(id) DO UPDATE SET title, description, order_direction, item_count` so a re-seed corrects counts/metadata on an existing database (FR-009). Keep `item` insert-or-ignore and `item_value` insert-or-replace.
- [X] T005 [US1] Run `cargo test --manifest-path backend/Cargo.toml default_lists` and confirm T002 passes and the rest of the module tests stay green.

**Checkpoint**: The three shipped lists carry 20–50 items and re-seeding keeps counts correct.

---

## Phase 4: User Story 2 - A Countries list with flags (Priority: P2)

**Goal**: A Countries list of all 195 sovereign states, each with its flag icon, area (km²), population, and date founded.

**Independent Test**: `cargo test default_lists` shows four lists with the Countries list holding 195 items, every `icon_name` starting with `flag:` and every item carrying area/population/founded; `npm test ItemIcon` shows `flag:de` renders the flag.

### Tests for User Story 2

> **NOTE: Write the flag test and update the backend test FIRST, confirm they FAIL, then implement T008/T010.**

- [X] T006 [US2] Add the `flag-icons` dependency: run `npm --prefix frontend install flag-icons`, add it to `frontend/package.json` dependencies, and import its CSS once in `frontend/src/main.tsx` (e.g. `import 'flag-icons/css/flag-icons.min.css';`).
- [X] T007 [P] [US2] Add a test in `frontend/src/components/ItemIcon.test.tsx`: `flag:de` renders the German flag (element with classes `fi` and `fi-de`, not the `HelpCircle` fallback); a lucide name (e.g. `Film`) still renders that icon; an unknown name still falls back to `HelpCircle`. Confirm it fails before T008.
- [X] T009 [P] [US2] In `backend/src/modules/default_lists/tests.rs`, update `DEFAULT_LISTS.len()` to 4 and add checks for the Countries list: 195 items, unique ids, every `icon_name` starts with `flag:`, and every item has finite `area_km2`, `population`, and `founded` values. Confirm it fails before T010.

### Implementation for User Story 2

- [X] T008 [US2] In `frontend/src/components/ItemIcon.tsx`, when `name` starts with `flag:`, render `<span className={`fi fi-${code} ${className}`} />` for the remainder `code` (lowercase ISO 3166-1 alpha-2); otherwise keep the existing lucide switch. No `HelpCircle` fallback for a valid `flag:` name.
- [X] T010 [US2] In `backend/src/modules/default_lists/mod.rs`, add the Countries list: id `default_countries`, title/description, `order_direction: "low_to_high"`, columns `area_km2` "Area (km²)", `population` "Population", `founded` "Founded"; and `COUNTRIES_ITEMS` with all 195 sovereign states (193 UN members + Vatican City + Palestine), each `id = "c_<cc>"`, `icon_name = "flag:<cc>"` (lowercase), and finite area/population/founded values. Add it to `DEFAULT_LISTS` (now four).
- [X] T011 [US2] Run `cargo test --manifest-path backend/Cargo.toml default_lists` and `npm --prefix frontend run test -- ItemIcon`; confirm T007 and T009 pass and the rest of those suites stays green.

**Checkpoint**: The Countries list is present with flags and all three values; flag rendering works offline.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Full regression and manual validation.

- [X] T012 [P] Run `cargo fmt --manifest-path backend/Cargo.toml --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, and `cargo test --manifest-path backend/Cargo.toml`.
- [X] T013 [P] Run `npm --prefix frontend run test` and `npm --prefix frontend run build`.
- [X] T014 Run the quickstart checks in `specs/014-default-lists-countries/quickstart.md`: automated primary checks pass (backend `default_lists` incl. the 195-country test, `ItemIcon` flag test, full suites, fmt/clippy, build); the browser walkthrough (fresh DB, round from Countries, restart) is left as an optional manual confirmation for the user.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: None to build.
- **User Story 1 (Phase 3)**: Depends on Setup.
- **User Story 2 (Phase 4)**: Depends on Setup; edits the same `default_lists` files as US1, so run after US1 (or serialize the file edits).
- **Polish (Phase 5)**: Depends on US1 and US2.

### Within Each User Story

- Test written and failing before implementation (T002 before T003; T007 before T008; T009 before T010).
- Implementation before its verification (T005, T011).

### Parallel Opportunities

- T007 (frontend `ItemIcon` test) and T009 (backend tests) are different files and can be written in parallel.
- T006 (dependency/CSS) is independent of the backend work and can run alongside US1.
- T012 and T013 touch different toolchains and can run in parallel after US1/US2.

---

## Parallel Example: User Story 2

```bash
# Dependency + CSS (independent of the code paths):
Task: "Add flag-icons and import its CSS (T006)"

# Two tests in different files, in parallel:
Task: "Add the flag render test in frontend/src/components/ItemIcon.test.tsx (T007)"
Task: "Update default_lists/tests.rs for the Countries list (T009)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001 (baseline).
2. Complete T002 (failing test) → T003 (expand lists) → T004 (seed upsert) → T005 (verify).
3. **STOP and VALIDATE**: the three shipped lists carry 20–50 items. Deploy/demo.

### Incremental Delivery

1. US1 → expanded lists (MVP).
2. US2 → Countries list with flags.
3. Polish → full suites, fmt/clippy, build, quickstart.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps each task to its user story for traceability.
- US1 and US2 both edit `default_lists/mod.rs` and `default_lists/tests.rs`; keep those edits sequential.
- No schema change and no backend dependency; the one added dependency is the frontend `flag-icons` (justified in plan.md).
- Verify the US1 and US2 tests fail before their implementation tasks.
