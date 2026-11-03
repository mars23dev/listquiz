---
description: "Task list for fix-reveal-reset"
---

# Tasks: Fix Reveal Reset On Next Round

**Input**: Design documents from `/specs/020-fix-reveal-reset/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included. The constitution (Principle II) requires every business rule to be covered by a test. Write the test first and confirm it fails before the implementation change.

**Organization**: One user story (P1); tasks are grouped by phase.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1)
- Include exact file paths in descriptions

## Path Conventions

- **Web app**: `backend/src/`, `backend/tests/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm the toolchain runs.

- [X] T001 Run the baseline suite: `cargo test --manifest-path backend/Cargo.toml`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST exist before the story.

**No foundational tasks.** The session, the reveal flag, and `advance_round` already exist. Proceed to the user story.

**Checkpoint**: Foundation ready.

---

## Phase 3: User Story 1 - Revealing values is per round; the next round starts hidden (Priority: P1) 🎯 MVP

**Goal**: Advancing a round resets the reveal flag so the new round starts hidden; finishing at the last round keeps the reveal.

**Independent Test**: `cargo test` shows reveal→advance clears the flag and finish→keeps it.

### Tests for User Story 1

> **NOTE: Write the test FIRST, confirm it FAILS, then implement T003.**

- [X] T002 [US1] Add a test in `backend/src/modules/sessions/tests.rs` (or `backend/tests/round_progression.rs`): set the session's `reveal_values` to 1, call `sessions::advance_round`, and assert `reveal_values` is 0 for the started round; and assert the last-round finish path leaves `reveal_values` at 1. Confirm it fails before T003.

### Implementation for User Story 1

- [X] T003 [US1] In `backend/src/modules/sessions/mod.rs` `advance_round`, add `reveal_values = 0` to the existing session update that starts the next round (the statement writing `last_message`), leaving the last-round finish branch (which sets `reveal_values = 1`) unchanged.
- [X] T004 [US1] Run `cargo test --manifest-path backend/Cargo.toml` and confirm T002 passes and the rest stays green.

**Checkpoint**: Advancing clears the reveal; finishing keeps it.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Full regression and manual validation.

- [X] T005 [P] Run `cargo fmt --manifest-path backend/Cargo.toml --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, and `cargo test --manifest-path backend/Cargo.toml`.
- [X] T006 [P] Run `npm --prefix frontend run test` and `npm --prefix frontend run build` (regression; frontend unchanged).
- [X] T007 Run the quickstart checks in `specs/020-fix-reveal-reset/quickstart.md`: reveal, advance, confirm the control resets and the new round is hidden; finish still reveals.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: None to build.
- **User Story 1 (Phase 3)**: Depends on Setup.
- **Polish (Phase 4)**: Depends on US1.

### Within User Story 1

- Test before implementation (T002 before T003); verification (T004) last.

### Parallel Opportunities

- T005 and T006 touch different toolchains and can run in parallel.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001 (baseline).
2. Complete T002 (failing test) → T003 (reset on advance) → T004 (verify).
3. **STOP and VALIDATE**: the reveal resets on the next round. Deploy/demo.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps each task to its user story for traceability.
- One statement in one function; no schema, dependency, or frontend change.
- Verify the test fails before implementing.
