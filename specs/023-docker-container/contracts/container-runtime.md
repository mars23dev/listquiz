# Contract: Local Container Runtime

This contract covers the local Docker image, the container run interface, and the unchanged HTTP surface.
It adds no application API.

## 1. Build contract

### Command

```powershell
docker build --tag listquiz:local .
```

Run from the repository root with Docker and BuildKit available. The default build context is the
repository root.

### Build inputs

The build must use repository source only:

- root `Dockerfile`
- root `.dockerignore`
- `backend/Cargo.toml`, `backend/Cargo.lock`, `backend/.cargo/config.toml`, and `backend/src/`
- `frontend/package.json`, `frontend/package-lock.json`, `frontend/.npmrc`, `frontend/index.html`,
  `frontend/vite.config.ts`, `frontend/tsconfig.json`, and `frontend/src/`

### Excluded inputs

The build context must not contain, and the build must not depend on:

- `.git/`, `.github/`, `.specify/`, `specs/`, `tests/`
- `backend/target/`, `backend/data/`
- `frontend/node_modules/`, `frontend/dist/`, `frontend/.vite/`, `frontend/.npm-cache/`
- `.npm-cache/`, `release/`, log files, local environment files

### Build success

The command exits with status 0 and creates the local image `listquiz:local`. Gateway or compiler errors
in either stage cause a non-zero exit and preserve no partial success claim.

### Build failure modes

- Missing Docker or BuildKit: Docker reports the daemon or builder error.
- Network unavailable: dependency fetch or base image pull fails with a non-zero exit.
- Frontend build failure: the Node stage fails with the frontend error.
- Backend build failure: the Rust stage fails with the compiler error.
- Invalid or missing `Cargo.lock`: `--locked` fails the backend stage.

## 2. Image contract

`docker image inspect listquiz:local` must report:

| Property | Required value |
|---|---|
| Exposed port | `8080/tcp` |
| Volume | `/app/data` |
| Working directory | `/app` |
| Runtime user | non-root `listquiz` user |
| Entry point / command | `/app/listquiz` |
| Runtime toolchain | no Rust or Node.js toolchain required |

The image must contain:

- `/app/listquiz`
- `/app/frontend/dist/index.html`
- `/app/data/` writable by the runtime user

The image must not require source files, package managers, or compilers at runtime.

## 3. Run contract

### Documented command

```powershell
docker run --rm --name listquiz --publish 8080:8080 --volume listquiz-data:/app/data listquiz:local
```

- `--rm` removes the container when it stops; the named volume remains.
- `--name listquiz` gives the container a stable local name.
- `--publish 8080:8080` maps host port 8080 to container port 8080.
- `--volume listquiz-data:/app/data` persists the SQLite database and its WAL files.

### Startup

The container starts with no shell, setup command, or interactive step. Within 30 seconds the
application must be reachable and the backend log must report the serving address.

### URL

Open:

```text
http://localhost:8080/
```

If host port 8080 is occupied, the user may publish another host port:

```powershell
docker run --rm --name listquiz --publish 8090:8080 --volume listquiz-data:/app/data listquiz:local
```

The container port remains 8080 in every case.

### LAN access

The published port is reachable from other devices on the host network, subject to the host firewall.
The application continues to listen on `0.0.0.0:8080` inside the container.

### Stop and restart

```powershell
docker stop listquiz
docker start listquiz
```

Stopping and starting the same container must preserve the mounted data. A new container started with the
same named volume must also see the same data. The backend handles SIGTERM and Ctrl+C with a graceful
shutdown. Long-lived SSE clients may delay exit until they disconnect or Docker reaches its stop timeout;
the SQLite WAL keeps committed data intact in either case.

## 4. HTTP contract

The HTTP surface is unchanged from the non-container build.

| Request | Expected result |
|---|---|
| `GET /api/health` | `200 OK`; JSON containing `status: "ok"` and the package version |
| `GET /` | `200 OK`; the ListQuiz landing page |
| `GET /api/default-lists` | existing default-list response |
| `GET /api/folders` | existing folder-list response |
| `POST /api/folders/{folderId}/lists/import` | existing CSV import response |
| `GET /api/lists/{listId}/export.csv` | existing CSV export response |
| existing session and stream routes | unchanged behaviour |

No URL prefix, API version, or authentication scheme is added by the container.

## 5. Persistence contract

| Event | Required result |
|---|---|
| First start with an empty volume | schema created; default folder and default lists seeded |
| Write a list or folder | data stored in `/app/data/listquiz.db` or its WAL |
| Stop and start the container | all data remains available |
| Remove and recreate the container with the same volume | all data remains available |
| Start with an existing database | no deletion or overwrite of user data; seeding is idempotent |
| Start with an unwritable data directory | startup fails visibly; no silent data loss |
| Reset the data | only an explicit volume removal, never an automatic startup action |

Documented reset (destructive, requires the container to be removed first):

```powershell
docker volume rm listquiz-data
```

## 6. Out of scope

- publishing or pushing an image to a registry
- attaching an image or image archive to a GitHub release
- changing `.github/workflows/release.yml` or the portable zip packaging
- multi-architecture image manifests
- Kubernetes, Compose, or other orchestration
- TLS termination or reverse-proxy configuration
- changing the application's HTTP API, database schema, or game behaviour
