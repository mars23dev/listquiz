# Implementation Plan: Run ListQuiz as a Docker Container

**Branch**: `feat/docker-image-build` | **Date**: 2026-11-07 | **Spec**: [spec.md](./spec.md)

**Input**: Feature specification from `/specs/023-docker-container/spec.md`

## Summary

Add a root `Dockerfile` that builds ListQuiz entirely inside Docker and a root `.dockerignore` that keeps
generated host output out of the build context. The image uses three stages: a Node stage that runs
`npm ci` and `npm run build`, a Rust stage that runs `cargo build --release --locked`, and a slim Debian
runtime that contains only the compiled binary, the built frontend, and a writable data directory.

The runtime reuses the existing path overrides. The binary is placed at `/app/listquiz`, the frontend at
`/app/frontend/dist`, and the database at `/app/data/listquiz.db`. The image exposes port 8080, runs as a
non-root user, and declares `/app/data` as its persistent volume. Documentation gains the exact local
build and run commands, with no release or registry changes.

The only application change is a minimal SIGTERM/Ctrl+C graceful-shutdown handler in
`backend/src/main.rs` and `backend/src/routes/api.rs`; no game logic, frontend, database, or HTTP API
change.

## Technical Context

**Language/Version**: Rust edition 2024, repository requirement Rust 1.99; TypeScript with React 19 and
Vite 8, repository requirement Node.js 24 and npm 11.

**Primary Dependencies**: existing application dependencies only. Local tooling is Docker with BuildKit.
Base images: `node:24-bookworm-slim`, `rust:1.99-bookworm`, and `debian:bookworm-slim`.

**Storage**: existing SQLite database, placed at `/app/data/listquiz.db` inside the container and backed
by a named Docker volume. SQLite WAL companion files live in the same directory.

**Testing**: existing `cargo test --manifest-path backend/Cargo.toml`,
`npm --prefix frontend run test`, `npm --prefix frontend run lint`, and release packaging checks remain
unchanged. Container verification is a documented `docker build` plus `docker run` flow with HTTP health,
list import, and restart/persistence checks.

**Target Platform**: Linux container image, native to the Docker host architecture, running on Docker
Desktop or Docker Engine on Windows, macOS, or Linux. The web application is reached at
`http://localhost:<published-port>/`.

**Project Type**: single-container web application. The Rust backend serves the exported React frontend;
there is no separate frontend container or runtime.

**Performance Goals**: image build completes within 15 minutes on a typical developer machine with a warm
dependency cache; container startup reaches a responsive landing page within 30 seconds; gameplay
behaviour and response times match the non-container build.

**Constraints**: no Rust or Node.js toolchain on the host; no host build output reused; no image publish
or release workflow change; no registry credentials; no new application dependencies; no change to the
existing HTTP API; default container port stays 8080; data stays in one mountable directory.

**Scale/Scope**: one image, one container, one local host, the existing single-room game scale. The
change is limited to build and run infrastructure plus documentation.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle / constraint | How the plan satisfies it | Verdict |
|---|---|---|
| I. Independently Testable Features | The container is verified by its own build/run/import/persistence scenario; existing backend and frontend tests stay runnable without Docker. | PASS |
| II. Explicit Business Logic | No business rule changes. All game, list, folder, and session logic stays in the existing named Rust units. | PASS |
| III. Rust Backend | The image compiles and runs the same Rust backend. No second backend language is introduced. | PASS |
| IV. Backend-Owned Frontend | The Node stage only produces `frontend/dist`; the Rust backend serves it exactly as in the existing build. No standalone frontend container exists. | PASS |
| V. Small Independent Modules | The only application change is shutdown wiring in the existing `main`/`api` modules; no feature module changes and no new module. | PASS |
| VI. Responsive and Keyboard Accessible UI | The frontend is unchanged, so existing responsiveness and keyboard behaviour are preserved. | PASS |
| VII. Simplicity Over Abstractions | One multi-stage Dockerfile, one ignore file, and documentation. No Compose file, orchestration layer, new script, or new service is introduced. | PASS |
| Stack: Rust backend, frontend in repo | Unchanged. The Dockerfile uses the existing `backend/` and `frontend/` source trees. | PASS |
| Stack: build locality | Compilation happens in Docker build stages from repository sources; generated host output is excluded. The Docker daemon stores image layers in its own managed storage, which is unavoidable for a container image. | PASS with documented exception |
| Stack: build artifacts ignored | No new tracked build output is introduced. `.dockerignore` mirrors the existing ignored directories for the build context. | PASS |

Complexity Tracking records the Docker-managed layer storage exception. No other violations.

## Project Structure

### Documentation (this feature)

```text
specs/023-docker-container/
|-- spec.md
|-- plan.md
|-- research.md
|-- data-model.md
|-- quickstart.md
|-- contracts/
|   `-- container-runtime.md
|-- checklists/            # existing empty directory, unchanged
`-- tasks.md               # created later by /speckit.tasks
```

### Source Code (repository root)

```text
Dockerfile                 # NEW: three-stage frontend, backend, runtime build
.dockerignore              # NEW: keep generated and local-only files out of the context
README.md                  # EDIT: local Docker build and run section

backend/
|-- Cargo.toml
|-- Cargo.lock
|-- .cargo/config.toml
|-- src/
|   |-- main.rs                    # EDIT: SIGTERM/Ctrl+C shutdown signal
|   `-- routes/api.rs              # EDIT: axum graceful shutdown
`-- tests/

frontend/                  # unchanged
|-- package.json
|-- package-lock.json
|-- .npmrc
|-- index.html
|-- vite.config.ts
`-- src/

scripts/package-release.ps1        # unchanged
packaging/                          # unchanged
.github/workflows/release.yml       # unchanged
```

**Structure Decision**: keep the existing single-project web-application layout. Docker support is
repository-level build infrastructure, not an application module. No `backend/` or `frontend/` source
directory is added or restructured; the backend change is limited to the existing entry point and API
serve function for graceful shutdown. The only new files are at the repository root, where `docker
build` expects them by default.

### Runtime layout inside the image

```text
/app/
|-- listquiz                     # compiled Rust binary
|-- frontend/
|   `-- dist/                    # exported frontend served by the backend
`-- data/
    `-- listquiz.db              # SQLite database, on the persistent volume
```

The existing `LISTQUIZ_ROOT=/app`, `LISTQUIZ_DB=/app/data/listquiz.db`, and
`LISTQUIZ_DIST=/app/frontend/dist` overrides make this layout explicit.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|---|---|---|
| Docker daemon image and layer storage lives in Docker-managed storage outside the repository | A runnable container image is the feature; Docker owns image layers and container filesystems | Keeping image layers inside the repository would not produce an image Docker can run, and would still duplicate the daemon's storage model |

No other constitution violations.

## Phase 0: Research

See [research.md](./research.md). All technical unknowns are resolved; there are no open clarifications.

## Phase 1: Design

- [data-model.md](./data-model.md) - image, container, volume, runtime paths, configuration, and
  persistence rules.
- [contracts/container-runtime.md](./contracts/container-runtime.md) - build context, image, run, HTTP,
  and persistence contracts.
- [quickstart.md](./quickstart.md) - local build, run, health, list import, rebuild, and persistence
  validation.

## Post-Design Constitution Re-Check

| Principle / constraint | Re-check result |
|---|---|
| I. Independently Testable Features | PASS - container scenario is isolated from existing tests |
| II. Explicit Business Logic | PASS - no rule change |
| III. Rust Backend | PASS - same binary |
| IV. Backend-Owned Frontend | PASS - frontend only built, never served separately |
| V. Small Independent Modules | PASS - no feature-module edits; shutdown wiring stays in existing modules |
| VI. Responsive and Keyboard Accessible UI | PASS - frontend unchanged |
| VII. Simplicity Over Abstractions | PASS - one Dockerfile, one ignore file, documentation only |
| Stack and build constraints | PASS with the documented Docker daemon storage exception |

Design introduces no new gate failure.
