# Implementation Plan: ListQuiz

**Branch**: `001-listquiz-game` | **Date**: 2026-10-04 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/001-listquiz-game/spec.md`

## Summary

ListQuiz is a single-room, gamemaster-hosted quiz game. A Rust backend owns all game state, serves the
exported frontend, and holds session, folder, and list data in a SQLite database inside the project
tree. The frontend is React + Vite, adopted from the read-only draft in `the reference frontend`: its screens,
navigation, copy, colour set, and audio cues are copied into this repository and corrected where the
spec overrides them. All mutations originate from the gamemaster view; player and spectator views are
read-only and receive updates over server-sent events, so hidden item values are redacted on the
server and never reach a player.

## Technical Context

**Language/Version**: Rust 1.99 (edition 2024) for the backend; TypeScript 7 / React 19 for the
frontend (versions inherited from `the reference frontend`)

**Primary Dependencies**: `axum` (HTTP + SSE), `tokio` (async runtime), `rusqlite` with the bundled
SQLite feature (no external database process, no C toolchain needed), `serde` / `serde_json`,
`tower-http` (static file serving, gzip), `csv` (import and export); frontend keeps the draft's
`react`, `react-dom`, `vite`, `@tailwindcss/vite`, `lucide-react`, `motion`

**Storage**: SQLite database file at `backend/data/listquiz.db`, created on first run and gitignored.
Seeded with the read-only default folder and the draft's three default lists.

**Testing**: `cargo test` for all backend business logic (pure modules, no HTTP required) and
integration tests against an in-memory SQLite database; `vitest` + Testing Library for frontend
components; Playwright for the two-device manual scenario in `quickstart.md`.

**Target Platform**: Single machine on a local network. The backend listens on `0.0.0.0:8080`; players
reach it by IP or by sharing the machine's address. No cloud, no container, no reverse proxy.

**Project Type**: Web application - Rust backend that also serves the exported frontend

**Performance Goals**: Every gamemaster action reaches an open player view within one second
(SC-002); local interactions never block input or trigger a full-page reload (SC-007); board render
stays at 60 fps with 10 board items and up to 4 teams.

**Constraints**:
- Frontend source lives in `frontend/`, copied from `the reference frontend`; `the reference frontend` is never read as a build
  input and never edited.
- The build exports the frontend to `frontend/dist`, and the backend serves that directory. The
  frontend does not run standalone against a stub.
- All compilation and tool state stays inside the repository. `CARGO_TARGET_DIR`, the npm cache, and
  Vite's cache directory are pinned to project-local paths via `.cargo/config.toml` and `.npmrc`, so
  no build writes under `%APPDATA%`.
- `frontend/dist`, `backend/target`, `backend/data`, `node_modules`, and `.vite` are gitignored.
- No timer and no buzzer anywhere in the product.
- Passwords are stored as entered, never hashed, and the product states that this is not protection.

**Scale/Scope**: 1 gamemaster plus up to roughly 10 concurrent player/spectator views in one room;
up to 10 rounds per session; 11-100 items per list; 1-4 teams per session; a handful of folders per
installation. Single feature, single process, single SQLite file.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | Every feature is a Rust module with its own tests: `folders`, `lists`, `csv_import`, `order_round`, `tripmine_round`, `scoring`, `sessions`, `teams`. No module's tests need HTTP, a network, or another feature's fixtures. | PASS |
| II. Explicit Business Logic | Rules live in pure functions over plain data - placement validation, rank assignment, scoring, turn rotation, CSV parsing. Handlers only translate JSON to calls. Every rule gets unit tests. | PASS |
| III. Rust Backend | One backend crate in Rust. No second backend language. The draft's client-side `listStorage.ts` and `sessionSync.ts` logic is not ported; it is replaced by Rust modules. | PASS |
| IV. Backend-Owned Frontend | `frontend/dist` is served by the backend. No dev-only frontend runtime, no mock backend, no static-only hosting. A frontend opened against nothing is unsupported. | PASS |
| V. Small Independent Modules | `backend/src/modules/<feature>/` with one public API per module; cross-module calls go through those APIs. Frontend keeps the draft's component-per-screen split. | PASS |
| VI. Responsive and Keyboard Accessible UI | Native `button` and `input` elements in reused components so keyboard operation comes for free; pool and bar slots are buttons in DOM order, so the board is fully tabbable; no full-page reload anywhere. | PASS |
| VII. Simplicity Over Abstractions | One process, one SQLite file, no cache layer, no message broker, no ORM, no repository trait per table, no plugin points. SSE instead of a bidirectional protocol because every write already comes from one client. | PASS |
| Stack: Rust backend, build exports frontend | One `build` task runs `vite build` then `cargo build`; the backend serves `frontend/dist`. | PASS |
| Stack: all compilation inside project folders | `.cargo/config.toml` pins `target-dir`, `.npmrc` pins the cache, Vite `cacheDir` pinned; documented in `quickstart.md`. | PASS |
| Stack: build artifact folder gitignored | `frontend/dist` is the artifact folder and is listed in `.gitignore` | PASS |
| Stack: session data in SQLite, passwords unprotected | `backend/data/listquiz.db`; `gm_password` and folder passwords stored as plain text by design | PASS |
| Stack: `the reference frontend` read-only orientation | Frontend is copied into `frontend/`; the draft path appears in no build config, no `package.json`, and no import | PASS |

Complexity Tracking: no violations, so no exceptions are claimed.

**Constitution follow-up (completed)**: constitution v2.1.1 removed the `TODO(FRONTEND_SOURCE_PATH)` and
`TODO(BUILD_ARTIFACT_PATH)` placeholders and now defers the concrete directory to the feature plan. This
plan is that authority: it pins `frontend/` and `frontend/dist`.

## Project Structure

### Documentation (this feature)

```text
specs/001-listquiz-game/
|-- plan.md              # This file
|-- spec.md              # Feature specification
|-- research.md          # Phase 0: technology decisions
|-- data-model.md        # Phase 1: entities, states, validation
|-- quickstart.md        # Phase 1: run and verify end to end
|-- contracts/
|   |-- rest-api.md      # HTTP endpoints, request and response shapes
|   |-- realtime.md      # SSE event stream and role-based redaction
|   `-- ui-parity.md     # Draft screens kept, changed, and removed
`-- tasks.md             # Phase 2: executable task list (/speckit.tasks)
```

### Source Code (repository root)

```text
backend/
|-- Cargo.toml
|-- .cargo/config.toml           # target-dir pinned into the repo
|-- data/                        # SQLite file lives here (gitignored)
`-- src/
    |-- main.rs                  # process startup, router assembly, static serving
    |-- app_state.rs             # shared handles: db handle + broadcast channel
    |-- db.rs                    # connection, schema, seed of the default folder
    |-- errors.rs                # one error type mapped to HTTP responses
    |-- routes/
    |   |-- mod.rs
    |   |-- folders.rs           # HTTP surface for folders
    |   |-- lists.rs             # HTTP surface for lists and CSV import/export
    |   |-- sessions.rs          # HTTP surface for session create/unlock/actions
    |   `-- stream.rs            # SSE endpoint
    `-- modules/
        |-- folders/             # create, unlock, rename, delete, access rule
        |-- lists/               # list CRUD, item ordering, direction, validation
        |-- csv_import/          # parse, export, numeric extraction, error reporting
        |-- default_lists/       # seed data and the read-only default folder
        |-- sessions/            # creation, role links, password gate, lifecycle
        |-- order_round/         # sampling, bar state, placement validation
        |-- tripmine_round/      # sampling, ranks, searched position, reveal history
        |-- scoring/             # point and heart deltas, turn rotation, zero-heart skip
        `-- teams/               # colours, default names, count rules
frontend/
|-- package.json                 # adopted from the reference frontend, no draft path reference
|-- vite.config.ts               # outDir stays dist/, cacheDir pinned in-repo
|-- .npmrc                       # cache pinned in-repo
`-- src/
    |-- App.tsx                  # routing per the reference frontend, real data source
    |-- theme/                   # dark/light theme context and toggle
    |-- components/              # copied from the reference frontend, corrected per spec
    |-- pages/                   # Landing, ListManagement, Setup, Session
    |-- api/                     # typed client for contracts/rest-api.md
    `-- hooks/                   # SSE subscription, session state, keyboard helpers
tests/
`-- e2e/                         # Playwright: two-context gamemaster + player scenario
```

**Structure Decision**: Option 2, a web application with a Rust backend that also serves the exported
frontend, plus one extra top-level `frontend/` sibling. `backend/` and `frontend/` stay separate so the
frontend can be type-checked and unit-tested on its own, but there is exactly one runtime process: the
backend serves `frontend/dist`, so the frontend never runs without it. The draft's `components`, `data`,
`types`, and `utils` layout is preserved inside `frontend/src` so the copy is a near-mechanical move
rather than a rewrite; `utils/listStorage.ts` and `utils/sessionSync.ts` are the only draft files not
carried over, because folders, storage, and live sync become Rust modules.

## Complexity Tracking

> No constitution violations. No table entries.

## Re-check after Phase 1 design

Re-evaluated once `research.md`, `data-model.md`, `contracts/`, and `quickstart.md` existed. The design
introduced no new dependency and no cross-module shortcut: `scoring` is the only module allowed to
mutate points and hearts, `order_round` and `tripmine_round` call it rather than writing scores, and
both quiz-type modules are pure with respect to transport. Principle VII holds with no exceptions to
record.