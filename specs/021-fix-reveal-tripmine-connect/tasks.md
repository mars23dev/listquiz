---
description: "Task list for fix-reveal-tripmine-connect"
---

# Tasks: Reveal Values In Tripmine And Connect

**Input**: Design documents from `/specs/021-fix-reveal-tripmine-connect/`

**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/, quickstart.md

**Tests**: Included. The constitution (Principle II) requires every business rule to be covered by a test. Write the tests first and confirm they fail before the implementation change.

**Organization**: One user story (P1); tasks are grouped by phase.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1)
- Include exact file paths in descriptions

## Path Conventions

- **Web app**: `backend/src/`, `frontend/src/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm both toolchains run.

- [X] T001 Run the baseline suites: `cargo test --manifest-path backend/Cargo.toml` and `npm --prefix frontend run test`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST exist before the story.

**No foundational tasks.** The reveal flag, `redact`, and the Connect answer-key prop already exist.

**Checkpoint**: Foundation ready.

---

## Phase 3: User Story 1 - Revealing values works in Tripmine and Connect (Priority: P1) 🎯 MVP

**Goal**: Reveal unhides the hidden info in Tripmine (values + ranks + mine) and Connect (the pairing), as it does in Order.

**Independent Test**: `cargo test` shows Tripmine items revealed under the flag; `npm test` shows a player seeing the Connect key when the flag is set.

### Tests for User Story 1

> **NOTE: Write the tests FIRST, confirm they FAIL, then implement T004/T005.**

- [X] T002 [P] [US1] Add a backend test in `backend/src/app_state.rs` (tests module): redact a Tripmine state with `reveal_values = true` for role `p1` and assert every item reports `revealed = true`, a value, a `true_rank`, and the mine flag. Confirm it fails before T004.
- [X] T003 [P] [US1] Add a frontend test in `frontend/src/test/Session.test.tsx`: a Connect session-state fixture with `revealValues: true` rendered as `p1` shows the answer key (each name's correct value), and with `revealValues: false` does not. Confirm it fails before T005.

### Implementation for User Story 1

- [X] T004 [US1] In `backend/src/app_state.rs` `redact`, compute a Tripmine item's reveal as `revealed = is_gamemaster || item.revealed || reveal` so the value, rank, and mine show under the flag.
- [X] T005 [US1] In `frontend/src/pages/Session.tsx`, pass `showAnswerKey={isGamemaster || (state?.revealValues ?? false)}` to `ConnectBoard`.
- [X] T006 [US1] Run `cargo test --manifest-path backend/Cargo.toml` and `npm --prefix frontend run test`; confirm T002/T003 pass and the rest stays green.

**Checkpoint**: Reveal works in all three modes.

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: Full regression and manual validation.

- [X] T007 [P] Run `cargo fmt --manifest-path backend/Cargo.toml --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, and `cargo test --manifest-path backend/Cargo.toml`.
- [X] T008 [P] Run `npm --prefix frontend run test` and `npm --prefix frontend run build`.
- [X] T009 Run the quickstart checks in `specs/021-fix-reveal-tripmine-connect/quickstart.md`: reveal shows Tripmine values/ranks/mine and the Connect pairing; Order unchanged; reset per round.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies.
- **Foundational (Phase 2)**: None to build.
- **User Story 1 (Phase 3)**: Depends on Setup.
- **Polish (Phase 4)**: Depends on US1.

### Within User Story 1

- Tests before implementation (T002/T003 before T004/T005); verification (T006) last.

### Parallel Opportunities

- T002 (backend test) and T003 (frontend test) are different files and can be written in parallel.
- T004 and T005 touch different layers and can proceed in parallel.
- T007 and T008 touch different toolchains and can run in parallel.

---

## Parallel Example: Kick-off

```bash
# Two tests in two different files, in parallel:
Task: "Tripmine reveal redaction test in backend/src/app_state.rs (T002)"
Task: "Connect reveal test in frontend/src/test/Session.test.tsx (T003)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001 (baseline).
2. Complete T002/T003 (failing tests) → T004/T005 (reveal in both modes) → T006 (verify).
3. **STOP and VALIDATE**: reveal works in all three modes. Deploy/demo.

---

## Notes

- [P] tasks = different files, no dependencies.
- [Story] label maps each task to its user story for traceability.
- No new dependency, no schema change; scoring/hearts/turn/timer unchanged.
- Verify the tests fail before implementing.
