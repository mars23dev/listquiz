# Research: Run ListQuiz as a Docker Container

## D-2301: Add one root multi-stage Dockerfile

**Decision**: Put a single three-stage `Dockerfile` at the repository root. Stage 1 builds the frontend
with Node, stage 2 builds the backend with Rust, and stage 3 copies only the binary and built assets into
a slim runtime image.

**Rationale**: A multi-stage build keeps the toolchains out of the runtime image and lets the host build
without Rust or Node.js (FR-001, FR-002, FR-004; Principle VII). Docker's default build context is the
repository root, so no custom build script or Compose file is needed.

**Alternatives considered**: a single-stage image (large, includes toolchains and source); building on the
host and copying artifacts (requires host toolchains and can reuse stale output); Docker Compose (adds a
file and indirection for a single service).

## D-2302: Build frontend and backend from source inside the image

**Decision**: In the frontend stage run `npm ci` followed by `npm run build`. In the backend stage run
`cargo build --release --locked`, copying `Cargo.toml`, `Cargo.lock`, `.cargo/config.toml`, and `src/`.
Copy the resulting `frontend/dist` and `backend/target/release/listquiz` into the runtime stage.

**Rationale**: This satisfies FR-002 and FR-003: the build cannot accidentally use host `node_modules`,
`dist`, `target`, or `data`. `--locked` keeps the dependency set identical to the repository lockfile.
The frontend remains owned and served by the backend, as required by the constitution.

**Alternatives considered**: copying host-built artifacts (stale and toolchain-dependent); running only
`cargo build` without the frontend (the backend would start without usable UI); using an npm dev server
in the container (violates the backend-owned frontend constraint).

## D-2303: Pin base images to the repository's required toolchain lines

**Decision**: Use `node:24-bookworm-slim` for the frontend stage, `rust:1.99-bookworm` for the backend
stage, and `debian:bookworm-slim` for the runtime. All three share the Debian 12 userland.

**Rationale**: The repository documents Node.js 24, npm 11, and Rust 1.99; these tags match those lines.
Using Bookworm for the builder and runtime avoids a glibc mismatch that would break the dynamically
linked Rust binary. The slim runtime keeps the image small (FR-004).

**Alternatives considered**: `latest` tags (unpinned and non-reproducible); Alpine (musl mismatch with
the Debian-built binary or a more complex build); a distroless runtime (smaller, but harder to debug and
unnecessary for a local-only image); building the runtime from the Rust image (much larger).

## D-2304: Use the existing environment overrides for the runtime layout

**Decision**: Place the binary at `/app/listquiz`, the frontend at `/app/frontend/dist`, and the database
at `/app/data/listquiz.db`. Set `LISTQUIZ_ROOT=/app`, `LISTQUIZ_DIST=/app/frontend/dist`, and
`LISTQUIZ_DB=/app/data/listquiz.db` in the image.

**Rationale**: `backend/src/paths.rs` already resolves the packaged layout from the executable directory
and already supports these overrides. The container needs no path-resolution code change and keeps the
package-root behaviour used by the Windows zip (FR-012, FR-013; Principle V).

**Alternatives considered**: changing `paths.rs` to detect Docker (adds a special case for no present-day
need); putting the database next to the binary without a dedicated directory (harder to mount cleanly);
using `/data` alone (works, but keeping everything under `/app` makes the image layout easy to inspect).

## D-2305: Persist data in a named volume at `/app/data`

**Decision**: Declare `VOLUME ["/app/data"]` and document a named volume mount:
`-v listquiz-data:/app/data`. The container user must own `/app/data` in the image so a fresh named volume
inherits writable permissions.

**Rationale**: The SQLite database and its WAL files are the only persistent state (FR-010, FR-011). A
named volume survives container removal and avoids host-path permission differences. Running as a
non-root user with ownership of the data directory reduces accidental host-file writes and is the
standard container default.

**Alternatives considered**: no volume (data lost when the container is removed); a bind mount as the
only documented option (works, but needs host directory permissions and varies by platform); a database
inside the image (would be writable only as root and would lose data on replacement).

## D-2306: Expose container port 8080 and publish it explicitly

**Decision**: Keep the application's existing `0.0.0.0:8080` listener, add `EXPOSE 8080`, and document
`-p 8080:8080`. If the host port is busy, document `-p <host-port>:8080` without changing the container
port.

**Rationale**: The backend already binds all interfaces, which is required for LAN devices (FR-006). The
container port stays stable so the documented URL and internal configuration remain simple (FR-005,
FR-007).

**Alternatives considered**: making the container port configurable (would require a backend listener
change); publishing only on loopback (would break multi-device play); using host networking (platform
specific and unnecessary).

## D-2307: Add a root `.dockerignore` that excludes generated and local-only files

**Decision**: Ignore `.git`, `.github`, `.specify`, `specs`, `tests`, `release`, `backend/target`,
`backend/data`, `frontend/node_modules`, `frontend/dist`, `frontend/.vite`, `frontend/.npm-cache`,
`.npm-cache`, log files, and local environment files.

**Rationale**: It keeps the context small and prevents host build output from entering an image build
(FR-003, FR-014, FR-018). It mirrors the existing `.gitignore` categories so local state cannot leak into
the image.

**Alternatives considered**: relying on `.gitignore` alone (Docker ignores it for the build context); a
custom build script that copies selected files (more moving parts than a `.dockerignore`); no ignore file
(the context would include `node_modules` and `target`, making builds slow and potentially stale).

## D-2308: Document the container workflow in the README

**Decision**: Add a Docker section to `README.md` with prerequisites, `docker build -t listquiz:local .`,
`docker run --rm -p 8080:8080 -v listquiz-data:/app/data listquiz:local`, the URL, the volume reset
command, and the optional host-port override.

**Rationale**: FR-015 and the discovery scenario require the commands to be in the repository, not only
in the feature folder. The README is the existing canonical run guide.

**Alternatives considered**: a separate `DOCKER.md` (less discoverable); comments only in the Dockerfile
(users start at the README); no documentation (fails the user story).

## D-2309: Keep image publication and release packaging out of scope

**Decision**: Do not edit `.github/workflows/release.yml`, `scripts/package-release.ps1`, or
`packaging/`. Do not add a registry push, image export, or image attachment to releases.

**Rationale**: FR-016 and FR-017 explicitly limit the feature to local image creation. The existing
portable zip remains the release artifact and must keep working unchanged.

**Alternatives considered**: attaching an image tar to releases (out of scope and adds large release
assets); publishing to GitHub Container Registry (out of scope and introduces credentials); replacing
the zip with the image (breaks the no-Docker release path).

## D-2310: Verify with build, run, feature, rebuild, and persistence checks

**Decision**: The validation guide covers: build from a clean context; start the container; verify
`GET /api/health` and the landing page; create and import a list; export it; remove and recreate the
container with the same volume; rebuild after deleting host build output; and stop the container.

**Rationale**: These checks map directly to the success criteria and exercise the container boundary,
not only the application unit tests (Principle I). They can be repeated locally without a release.

**Alternatives considered**: only running existing tests (does not verify Docker); only checking the
landing page (does not verify list import or persistence); automated container CI (not required by the
feature and would require a new workflow).

## D-2311: Add a minimal graceful-shutdown path for the container stop signal

**Decision**: Add a small shutdown future in `backend/src/main.rs` that resolves on Ctrl+C or Unix
SIGTERM, and pass it to `axum::serve(...).with_graceful_shutdown(...)` in
`backend/src/routes/api.rs`. Keep SQLite WAL enabled as the recovery safety net.

**Rationale**: A Docker stop sends SIGTERM. Without a handler the process can be killed at Docker's
timeout, which is not the clean stop required by FR-020. The change is limited to lifecycle wiring in
existing modules and does not alter game rules, the HTTP API, or the database schema (Principle V,
Principle VII). The WAL continues to protect committed data if the process is force-stopped.

**Alternatives considered**: relying only on WAL recovery (data-safe but leaves the process to be
SIGKILLed at the Docker timeout); setting a shorter stop timeout (makes the kill faster, not cleaner);
`docker kill` as the normal stop path (abrupt); a full shutdown coordinator or new module (overkill for
one signal).

**Observed behaviour**: With no open clients, `docker stop` completes in under one second. With a
long-lived SSE browser stream open, graceful shutdown waits for that connection and Docker may reach its
stop timeout; the database remains intact.

## D-2312: Keep the local image tag simple

**Decision**: Document the tag `listquiz:local` and use the same tag for build, run, and rebuild.

**Rationale**: A local-only image needs no registry naming, version scheme, or push step. The tag makes
the commands short and clearly marks the image as non-published (FR-016).

**Alternatives considered**: `latest` only (ambiguous between local and published images); a full
registry reference (implies publishing); a versioned tag per build (adds bookkeeping with no local use).

## D-2313: Target the native Docker host architecture

**Decision**: Build for the host's native architecture. Document that the image is local and that
cross-architecture builds are out of scope.

**Rationale**: The feature asks for a locally buildable image. Native builds avoid emulation and keep
the build simple (Principle VII). Docker Desktop on Windows and macOS supplies a Linux VM, so the image
runs there normally.

**Alternatives considered**: multi-architecture builds (out of scope and slower); forcing `linux/amd64`
on every host (fails on ARM hosts without emulation); publishing per-architecture images (requires a
registry).
