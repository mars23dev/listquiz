# Implementation Plan: Select Custom List In Session Setup

**Branch**: `022-fix-custom-list-setup` | **Date**: 2026-11-03 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/022-fix-custom-list-setup/spec.md`

## Summary

The setup wizard loads only `GET /api/default-lists`, so lists in user-created protected folders are
unreachable and cannot be selected for a round. Fix:

1. **Setup frontend**: load the folder list, show the default folder as always available, and let the
   gamemaster unlock protected folders inline with their password. After a successful unlock, load that
   folder's lists and add them to the round's source-list selector.
2. **Session request**: send the passwords for the folders actually referenced by the selected rounds as a
   request-scoped `folderPasswords` map.
3. **Backend enforcement**: before creating a session, check every round's list folder against that map with
   the existing folder password rule. A list in a locked folder is refused even if its id is sent directly.

No schema change and no new dependency. Passwords remain in memory only for the setup wizard's lifetime.

## Technical Context

**Language/Version**: Rust edition 2024 backend; TypeScript / React 19 frontend (Vite 8).

**Primary Dependencies**: unchanged — Axum 0.8, rusqlite 0.37, serde/serde_json, React, Vitest. No new
dependency.

**Storage**: unchanged SQLite database. The folder-password proof is request-scoped and is not stored.

**Testing**: `cargo test --manifest-path backend/Cargo.toml` for session-creation access rules;
`npm --prefix frontend run test` for the setup wizard; `npm --prefix frontend run lint` and
`npm --prefix frontend run build` for the frontend.

**Target Platform**: unchanged web application served by the Rust backend on the local machine or LAN.

**Project Type**: web application with a Rust backend and a React frontend served from the backend's export.

**Performance Goals**: no change to round rendering; folder unlock and list loading are ordinary local
requests, and session creation performs at most one password check per configured round (1-10).

**Constraints**: folder passwords MUST stay out of URLs; locked folder list names MUST NOT be fetched or
rendered before unlock; no schema change; no dependency change; build output stays inside the repository;
`the reference frontend` stays read-only.

**Scale/Scope**: a handful of folders and lists per installation, 1-10 rounds, 1-4 teams. One combined
source-list selector plus a folder access panel in the existing setup page.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | Backend access rule is tested over an in-memory SQLite database; the setup behavior is tested with mocked fetch. No network needed. | PASS |
| II. Explicit Business Logic | The locked-list rule is a named `validate_round_folder_access` function using the existing `folders::password_matches`; the setup state transitions are explicit. | PASS |
| III. Rust Backend | Folder-password enforcement lives in Rust. The frontend only collects the password and sends it. | PASS |
| IV. Backend-Owned Frontend | Unchanged; the backend still serves the built frontend. | PASS |
| V. Small Independent Modules | The access rule stays in `sessions`; a narrow `lists::folder_id` helper exposes the list-to-folder lookup; setup changes stay in the setup page. | PASS |
| VI. Responsive and Keyboard Accessible UI | The unlock control is a normal form, loading does not block the page, and every new control is keyboard reachable with visible focus. | PASS |
| VII. Simplicity Over Abstractions | Reuse the existing folders/unlock/list endpoints and password rule; add one request field and one named validation function. No new endpoint or token system. | PASS |
| Stack constraints | No schema change, no dependency change, no state written outside the repository. | PASS |

Complexity Tracking: no violations.

## Project Structure

### Documentation (this feature)

```text
specs/022-fix-custom-list-setup/
|-- plan.md
|-- spec.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   `-- setup-list-access.md
`-- tasks.md            # created later by /speckit.tasks
```

### Source Code (repository root)

```text
backend/
|-- src/
|   |-- modules/
|   |   |-- lists/mod.rs          # narrow folder-id lookup for a list (EDIT)
|   |   `-- sessions/mod.rs       # folderPasswords field + access validation (EDIT)
|   `-- routes/
|       `-- lists.rs              # reuse the shared list-folder lookup (EDIT)
`-- tests/
    `-- session_create.rs         # custom-list access tests (EDIT)

frontend/
`-- src/
    |-- api/client.ts             # session request carries folderPasswords (EDIT)
    |-- pages/SetupQuiz.tsx       # folder access panel + combined list selector (EDIT)
    `-- test/SetupQuiz.test.tsx   # unlock/select/create coverage (EDIT)
```

**Structure Decision**: reuse the existing web-application layout. No new page or backend module is
introduced. The only new shared helper is `lists::folder_id`, because both the list routes and session
access validation need the same lookup.

## Complexity Tracking

> No constitution violations. No table entries.

## Phase 0: Research

See [research.md](./research.md). All technical unknowns resolved; no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) — the existing folder/list entities, the request-scoped password map,
  and the setup state transitions.
- [contracts/setup-list-access.md](./contracts/setup-list-access.md) — folder discovery, unlock, list loading,
  and the `POST /api/sessions` access field.
- [quickstart.md](./quickstart.md) — automated and manual validation of the custom-list flow.

## Constitution Re-check after Phase 1

Re-evaluated after writing the artifacts. The design adds one request field, one named validation rule, one
narrow lookup helper, and setup-page controls. It adds no dependency, schema change, token store, or new
module. Principles I, II, III, V, VI, and VII hold with no exceptions to record.
