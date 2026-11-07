# Feature Specification: Run ListQuiz as a Docker Container

**Feature Branch**: `feat/docker-image-build`

**Created**: 2026-11-07

**Status**: Draft

**Input**: User description: "Provide ListQuiz as a Docker image built from the local repository, in addition to the existing executable. Starting the container must expose a reachable URL where the complete game is playable, including list import and all existing list and session features. The image does not need to be published with releases; it only needs to be buildable locally from the repository."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Build and start ListQuiz locally with Docker (Priority: P1)

A developer clones the repository and builds a container image from the local source without installing the Rust or Node.js toolchains on the host. Starting the container serves the full application on a documented URL.

**Why this priority**: This is the core request. Without a locally buildable image and a working container, no other scenario can be exercised.

**Independent Test**: From a clean clone with Docker installed, run the documented build command, start the container, and open the documented URL in a browser. Confirm the ListQuiz landing page loads and no host Rust or Node.js installation is required.

**Acceptance Scenarios**:

1. **Given** a clean checkout of the repository, **When** the documented image build command is run, **Then** the build completes and produces a local image without requiring Rust or Node.js on the host.
2. **Given** the built image, **When** the documented container start command is run, **Then** the container stays running and the application responds at the documented URL.
3. **Given** the running container, **When** the landing page is opened in a browser, **Then** the frontend assets load and the application is usable.
4. **Given** the running container, **When** the container is stopped and started again, **Then** the application becomes reachable again without rebuilding the image.

---

### User Story 2 - Use all existing list and game features inside the container (Priority: P1)

A user plays inside the container exactly as they would with the executable build. This includes creating folders and lists, importing lists from CSV by file upload or pasted text, exporting lists, configuring and running sessions, and reconnecting to a session.

**Why this priority**: A container that starts but cannot perform the real workflows does not satisfy the request. Feature parity is required.

**Independent Test**: In a browser connected to the container, create a folder, create or import a list containing a valid number of items, start a session from that list, and complete at least one turn. Then export the list to CSV.

**Acceptance Scenarios**:

1. **Given** the running container, **When** a user creates a list manually, **Then** the list is saved and can be selected in session setup.
2. **Given** the running container, **When** a user imports a valid CSV file by upload or pasted text, **Then** the import succeeds, returns the import report, and the list is available for play.
3. **Given** a list created inside the container, **When** the user exports it, **Then** a CSV download is produced.
4. **Given** a configured session, **When** players join through the session link, **Then** the session state updates for every role as it does in the executable build.
5. **Given** the running container, **When** the user exercises any existing game mode, **Then** behaviour matches the non-container application.

---

### User Story 3 - Keep list data across container replacement (Priority: P2)

A user can recreate the container, for example to apply a new image build, without losing lists, folders, or other stored data. Data is kept in a mounted volume rather than only inside the container filesystem.

**Why this priority**: Data loss makes the container unsuitable for regular use, but the application is still demonstrable without persistence.

**Independent Test**: Create a list inside the container, remove and recreate the container with the same volume, and confirm the list is still present.

**Acceptance Scenarios**:

1. **Given** a container using a persistent volume, **When** it is stopped and started again, **Then** all previously created lists, folders, and settings remain available.
2. **Given** a container using a persistent volume, **When** it is removed and a new container is started with the same volume, **Then** the stored data is still available.
3. **Given** a new container with an empty volume, **When** it starts for the first time, **Then** the shipped default folder and default lists are seeded and playable.
4. **Given** a container with existing data, **When** it restarts, **Then** seeding neither removes nor overwrites user-created data.

---

### User Story 4 - Document the container workflow (Priority: P3)

A developer can discover the container build and run steps from the repository documentation, including the image name, port mapping, volume mapping, and the URL to open.

**Why this priority**: The container is usable without documentation, but discoverability and repeatable local verification depend on it.

**Independent Test**: Follow the repository documentation from a clean checkout and reach a playable application without consulting source files.

**Acceptance Scenarios**:

1. **Given** the repository documentation, **When** a developer reads the container section, **Then** it lists prerequisites, the build command, the run command, the published port, and the URL.
2. **Given** the documentation, **When** the developer follows it verbatim, **Then** the result is a running, playable container.

---

### Edge Cases

- The host port used by the application is already occupied: the start command fails clearly or the documentation explains how to choose another host port.
- The image is built on a different CPU architecture than the Docker host: the documentation states the supported architecture, or the build produces a runnable image for the host.
- The database volume is empty: the application seeds defaults and starts normally.
- The database volume contains an existing database from an older build: the application starts without deleting it, and any required migration behaviour matches the non-container build.
- The volume path is not writable by the container user: the container fails with a clear error rather than silently discarding data.
- The frontend build fails during the image build: the build fails and reports the frontend stage error.
- The backend build fails during the image build: the build fails and reports the backend stage error.
- The image is rebuilt after source changes: build output must not be reused from stale host artifacts such as `frontend/node_modules`, `frontend/dist`, or `backend/target`.
- The container receives a stop signal: it shuts down without leaving the database corrupted.
- No network access is available during the container build: the build fails clearly; offline builds are out of scope.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The repository MUST contain a container definition that builds ListQuiz from the local source tree.
- **FR-002**: The image build MUST build the frontend and the backend inside the build environment, so the host needs no Rust or Node.js toolchain.
- **FR-003**: The image build MUST NOT depend on prebuilt host artifacts, including `frontend/node_modules`, `frontend/dist`, `backend/target`, or `backend/data`.
- **FR-004**: The runtime image MUST contain the compiled backend executable and the built frontend assets required to serve the complete application.
- **FR-005**: The container MUST start without manual setup beyond the documented run command and MUST serve the application on a documented container port.
- **FR-006**: The container MUST listen on all interfaces so that the published host port reaches the application.
- **FR-007**: The documented start command MUST allow the user to reach the application at a documented `http://localhost:<port>/` URL.
- **FR-008**: The container MUST provide the same user-visible behaviour as the executable build, including list creation, list editing, list import from CSV by upload or pasted text, list export, folder management, session setup, gameplay, and reconnect.
- **FR-009**: The container MUST seed the shipped default folder and default lists when its database is empty.
- **FR-010**: Stored data MUST live in a documented container path that can be mounted as a volume.
- **FR-011**: The documented run command MUST show how to mount a volume so data survives container removal and recreation.
- **FR-012**: The application MUST accept configuration, through documented environment variables or equivalent, to override the database location, the frontend asset location, and the runtime root used by the existing build.
- **FR-013**: The container MUST keep the existing default database and frontend layouts working without requiring the executable release layout to change.
- **FR-014**: The repository MUST include a container ignore file that excludes build output, dependency directories, local databases, version control data, and other non-source files from the build context.
- **FR-015**: The repository documentation MUST describe the container prerequisites, build command, run command, port mapping, volume mapping, and the URL to open.
- **FR-016**: Building the image MUST remain a local workflow only; it MUST NOT be added to the release workflow and MUST NOT require publishing or pushing an image.
- **FR-017**: The existing portable executable release workflow MUST continue to work unchanged.
- **FR-018**: Rebuilding the image after source changes MUST include those changes and MUST NOT rely on stale host build output.
- **FR-019**: The container MUST start in a reasonable time for local use, without an interactive shell or manual service startup step.
- **FR-020**: The container MUST stop cleanly when asked to stop and MUST NOT leave the database in a corrupt state.

### Key Entities

- **Container Image**: The locally built artifact containing the application runtime, compiled backend, built frontend, and runtime configuration.
- **Container Instance**: A running instance of the image that serves the application on a published host port.
- **Data Volume**: The host-mounted storage location that holds the SQLite database and therefore all lists, folders, sessions, and settings.
- **Image Build Context**: The set of repository files included in the build; it must contain source and build definitions and exclude generated or local-only files.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A developer with a clean checkout and Docker installed can build the image and start a playable container in under 15 minutes on a typical developer machine.
- **SC-002**: Opening the documented URL after starting the container shows the ListQuiz landing page in 100% of documented verification runs.
- **SC-003**: A list imported from CSV inside the container is available for session setup and survives a container removal and recreation with the same volume in 100% of verification runs.
- **SC-004**: All existing backend and frontend automated checks continue to pass, and the existing portable release build remains unchanged.
- **SC-005**: The container can be rebuilt from scratch after deleting all host build output, and the resulting image serves the application with no missing assets.
- **SC-006**: No image publish step is required, and no release artifact changes are introduced by this feature.

## Assumptions

- The primary container runtime is Docker with a Linux container image.
- The container target architecture is the same architecture as the local Docker host used for the build and run verification.
- The application remains a single service: the backend serves the built frontend, and no separate frontend container is required.
- The application continues to listen on TCP port 8080 inside the container; only the host-side published port is configurable.
- The SQLite database remains the only persistent state.
- Docker Compose is optional; a plain `docker build` and `docker run` workflow satisfies the requirement.
- Published images, registry authentication, multi-architecture builds, and Kubernetes deployment are out of scope.
- The existing `LISTQUIZ_ROOT`, `LISTQUIZ_DB`, and `LISTQUIZ_DIST` overrides remain the configuration mechanism unless implementation adds an equivalent.
- Internet access is available during the image build to fetch base images and dependencies.
