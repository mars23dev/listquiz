# Tasks: Run ListQuiz as a Docker Container

**Input**: Design documents from `/specs/023-docker-container/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/container-runtime.md, quickstart.md

**Tests**: Not requested by the feature specification. No new test files are generated. Validation is
covered by the manual container scenarios in `specs/023-docker-container/quickstart.md` and by the
existing application checks.

**Organization**: Tasks are grouped by user story so each story can be implemented and verified
independently. The implementation is build infrastructure only; no backend, frontend, database, or API
source change is expected.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Every task includes an exact file path or a concrete command target

## Path Conventions

- **Repository root**: `Dockerfile`, `.dockerignore`, `README.md`
- **Feature docs**: `specs/023-docker-container/`
- **Existing application**: `backend/`, `frontend/`
- **Unchanged release path**: `scripts/package-release.ps1`, `packaging/`, `.github/workflows/release.yml`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Create the container build-context boundary before any image stage is written.

- [X] T001 Create root `.dockerignore` excluding `.git/`, `.github/`, `.specify/`, `specs/`, `tests/`, `backend/target/`, `backend/data/`, `frontend/node_modules/`, `frontend/dist/`, `frontend/.vite/`, `frontend/.npm-cache/`, `.npm-cache/`, `release/`, `*.log`, and local `.env*` files

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Build a complete image from repository source. All user stories depend on this image.

**CRITICAL**: No user story validation can begin until the image builds and contains the executable and
frontend assets.

- [X] T002 Create root `Dockerfile` with the `frontend` stage using `node:24-bookworm-slim`; copy `frontend/package.json`, `frontend/package-lock.json`, and `frontend/.npmrc`, run `npm ci`, then copy the frontend source and run `npm run build`
- [X] T003 Add the `backend` stage to root `Dockerfile` using `rust:1.99-bookworm`; copy `backend/Cargo.toml`, `backend/Cargo.lock`, `backend/.cargo/config.toml`, and `backend/src/`, then run `cargo build --release --locked`
- [X] T004 Add the runtime stage to root `Dockerfile` using `debian:bookworm-slim`; create a non-root `listquiz` user, copy `frontend/dist` from the frontend stage to `/app/frontend/dist`, copy the release binary from the backend stage to `/app/listquiz`, set `WORKDIR /app`, set `USER listquiz`, and set `CMD ["./listquiz"]`
- [X] T005 Build the image from root `Dockerfile` with `docker build --tag listquiz:local .`; fix any stage failure; verify the image contains `/app/listquiz` and `/app/frontend/dist/index.html`

**Checkpoint**: Image builds from repository source alone and contains the runtime artifacts.

---

## Phase 3: User Story 1 - Build and start ListQuiz locally with Docker (Priority: P1) - MVP

**Goal**: A developer can build and run the image locally, then reach the application at a documented URL
without host Rust or Node.js toolchains.

**Independent Test**: From a clean checkout, run `docker build --tag listquiz:local .`, start the
container with port 8080 published, open `http://localhost:8080/`, and confirm the landing page and
`GET /api/health` both respond.

### Implementation for User Story 1

- [X] T006 [US1] Add the runtime configuration to root `Dockerfile`: `EXPOSE 8080`, `LISTQUIZ_ROOT=/app`, `LISTQUIZ_DIST=/app/frontend/dist`, and `LISTQUIZ_DB=/app/data/listquiz.db`
- [X] T007 [US1] Rebuild `listquiz:local` from root `Dockerfile` and run it with `docker run --rm --name listquiz --publish 8080:8080 listquiz:local`; verify `Invoke-RestMethod http://localhost:8080/api/health` returns `status: "ok"` and the landing page loads at `http://localhost:8080/`

**Checkpoint**: User Story 1 is playable at the documented local URL with no host toolchains.

---

## Phase 4: User Story 2 - Use all existing list and game features inside the container (Priority: P1)

**Goal**: The container provides the same workflows as the executable build, including CSV list import
and normal gameplay.

**Independent Test**: In the running container, create a folder and a list, import a valid CSV by upload
and by pasted text, export a list, start a session from the imported list, join as a player, and complete
at least one turn.

### Validation for User Story 2

- [X] T008 [US2] Verify folder and list management in the running container using `specs/023-docker-container/quickstart.md` step 4: create a folder, create a list, edit its items, and confirm the list appears in session setup
- [X] T009 [US2] Verify CSV list import in the running container using `specs/023-docker-container/quickstart.md` step 4 by uploading a file and by pasting CSV text; confirm the import report appears and the imported list is selectable
- [X] T010 [US2] Verify CSV export and a complete gameplay path in the running container using `specs/023-docker-container/quickstart.md` step 4: export the imported list, create a session, open the gamemaster and player links, complete a turn, and confirm live updates
- [X] T011 [US2] Record any container-only failure found in T008-T010 and fix it only in `Dockerfile`, `.dockerignore`, or the minimal SIGTERM shutdown wiring in `backend/src/main.rs` and `backend/src/routes/api.rs`; do not change game behaviour

**Checkpoint**: User Story 2 has feature parity with the non-container build.

---

## Phase 5: User Story 3 - Keep list data across container replacement (Priority: P2)

**Goal**: Lists, folders, and sessions survive container removal when the documented volume is reused.

**Independent Test**: Create a list, stop and remove the container, start a new container with the same
named volume, and confirm the list and its folder are still present while defaults are not duplicated.

### Implementation for User Story 3

- [X] T012 [US3] Add `VOLUME ["/app/data"]` to the runtime stage in root `Dockerfile` and ensure `/app/data` is created and owned by the `listquiz` user before `USER listquiz`
- [X] T013 [US3] Rebuild `listquiz:local` from root `Dockerfile` and run it with `docker run --rm --name listquiz --publish 8080:8080 --volume listquiz-data:/app/data listquiz:local`
- [X] T014 [US3] Verify persistence using `specs/023-docker-container/quickstart.md` step 5: create a custom list, stop the container, start a new container with the same `listquiz-data` volume, and confirm the list and folder remain; then start with a new empty volume and confirm the default folder and default lists are seeded

**Checkpoint**: User Story 3 keeps all SQLite state across container replacement.

---

## Phase 6: User Story 4 - Document the container workflow (Priority: P3)

**Goal**: A developer can discover and follow the local container workflow from the repository README.

**Independent Test**: Follow only the README Docker section from a clean checkout and reach a playable
application at the documented URL.

### Implementation for User Story 4

- [X] T015 [US4] Add a Docker section to root `README.md` covering Docker prerequisites, `docker build --tag listquiz:local .`, the `docker run` command with `--publish 8080:8080` and `--volume listquiz-data:/app/data`, the URL to open, the alternate host-port example, and the explicit volume reset command

**Checkpoint**: User Story 4 documentation is sufficient to reproduce the local container run.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Confirm the feature did not disturb existing builds, tests, or release packaging.

- [X] T016 [P] Run `cargo fmt --manifest-path backend/Cargo.toml --check`, `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`, and `cargo test --manifest-path backend/Cargo.toml`
- [X] T017 [P] Run `npm --prefix frontend run lint`, `npm --prefix frontend test`, and `npm --prefix frontend run build`
- [X] T018 [P] Confirm `.github/workflows/release.yml`, `scripts/package-release.ps1`, and `packaging/` are unchanged, and that the portable zip workflow still has no Docker dependency
- [X] T019 Run the full validation sequence in `specs/023-docker-container/quickstart.md`, including the clean rebuild with host `frontend/dist`, `frontend/node_modules`, `backend/target`, and `backend/data` removed
- [X] T020 Inspect `listquiz:local` with `docker image inspect` and confirm the root `Dockerfile` image exposes the required port, volume, working directory, non-root user, and contains no build toolchains or source files
- [X] T021 Review root `Dockerfile` and `.dockerignore` for accidental host-path references, stale build inputs, or writes outside `/app/data`; remove any such reference

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: T001 can start immediately.
- **Foundational (Phase 2)**: T002-T005 depend on T001 and must complete before any user story validation.
- **User Story 1 (Phase 3)**: T006-T007 depend on T005.
- **User Story 2 (Phase 4)**: T008-T011 depend on T007. These tasks exercise the same running container and should run sequentially.
- **User Story 3 (Phase 5)**: T012-T014 depend on T007; T014 also depends on T013.
- **User Story 4 (Phase 6)**: T015 depends on T007 and T013 so the documented run command is final.
- **Polish (Phase 7)**: T016-T021 depend on all desired user stories.

### User Story Dependencies

- **US1 (P1)**: First runnable increment; no dependency on US2-US4.
- **US2 (P1)**: Depends on US1 for a running container; independent of US3 and US4.
- **US3 (P2)**: Depends on US1 for a runnable image; independent of US2 and US4.
- **US4 (P3)**: Depends on US1 and US3 for the final commands; independent of US2.

### Within Each User Story

- US1: runtime configuration (T006) before the build/run validation (T007).
- US2: list management validation (T008), import validation (T009), export/gameplay validation (T010), then any container-only fix (T011).
- US3: volume declaration (T012) before rebuild/run (T013), then persistence validation (T014).
- US4: one documentation task (T015) after the commands are final.

---

## Parallel Opportunities

- T016, T017, and T018 can run in parallel; they cover backend checks, frontend checks, and release-file review.
- T020 and T021 can run in parallel after the image is built; one inspects the image, the other reviews build files.
- T001 has no parallel partner in this small setup phase.
- Dockerfile stages T002-T004 must remain sequential because they edit the same file.
- US2 container validations should not run in parallel because they share one application instance and database.

### Parallel Example: Polish

```text
Task: "Run backend fmt, clippy, and cargo test"
Task: "Run frontend lint, test, and build"
Task: "Confirm release workflow, packaging script, and packaging files are unchanged"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete T001: build-context boundary.
2. Complete T002-T005: image builds and contains artifacts.
3. Complete T006-T007: public port, runtime paths, health check, landing page.
4. **STOP and VALIDATE**: run the US1 independent test.
5. The MVP is a locally built and runnable ListQuiz container.

### Incremental Delivery

1. Setup + Foundational -> image builds.
2. US1 -> image runs and serves the app.
3. US2 -> verify feature parity, including CSV import and gameplay.
4. US3 -> add and verify persistent data.
5. US4 -> document the final build/run workflow.
6. Polish -> confirm existing tests and release packaging are unchanged.

### Parallel Team Strategy

With more than one person after T007:

- Developer A: US2 feature-parity validation.
- Developer B: US3 volume and persistence work.
- Developer C: US4 README section, then polish checks.

Use separate container names and host ports if US2 and US3 run at the same time.

---

## Notes

- `[P]` tasks use different files or independent checks.
- The feature does not add application tests; it verifies the existing application through the container.
- No task modifies `backend/`, `frontend/`, the database schema, or the HTTP API.
- No task publishes an image or edits the release workflow.
- Commit after each logical group only when the user asks for a commit.
