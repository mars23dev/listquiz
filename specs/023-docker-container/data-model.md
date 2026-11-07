# Data Model: Run ListQuiz as a Docker Container

No database schema change. The feature introduces a container image, a container instance, a persistent
volume, and a fixed runtime filesystem layout. The existing folders, lists, sessions, and game tables are
unchanged and remain the application's data model.

## Container entities

### Image

| Attribute | Meaning |
|---|---|
| Local tag | `listquiz:local` |
| Build source | Repository root and the root `Dockerfile` |
| Contents | compiled Rust binary, exported frontend, runtime configuration |
| Excluded contents | Rust/Node toolchains, source tree, dependency caches, host build output |
| Port | 8080 exposed |
| User | non-root `listquiz` user |
| Publish status | local only; not pushed to a registry |

### Container instance

| Attribute | Meaning |
|---|---|
| Image | `listquiz:local` |
| Published port | host port mapped to container port 8080 |
| Working directory | `/app` |
| Command | `/app/listquiz` |
| Data mount | `/app/data` |
| Lifecycle | created, running, stopped, removed |

### Data volume

| Attribute | Meaning |
|---|---|
| Documented name | `listquiz-data` |
| Container path | `/app/data` |
| Contents | `listquiz.db`, plus SQLite `-wal` and `-shm` files while the server runs |
| Lifetime | independent of any one container |
| Reset | explicit volume removal only |

## Runtime filesystem layout

| Path | Source | Writable | Purpose |
|---|---|---|---|
| `/app/listquiz` | backend build stage | no | application entry point |
| `/app/frontend/dist/` | frontend build stage | no | exported frontend served by the backend |
| `/app/frontend/dist/index.html` | frontend build stage | no | landing page and history fallback |
| `/app/data/` | image directory or mounted volume | yes | persistent state directory |
| `/app/data/listquiz.db` | created at first run | yes | SQLite database |
| `/app/data/listquiz.db-wal` | SQLite while open | yes | write-ahead log |
| `/app/data/listquiz.db-shm` | SQLite while open | yes | shared-memory index |

## Configuration

| Variable | Image value | Effect |
|---|---|---|
| `LISTQUIZ_ROOT` | `/app` | packaged root used by the existing path resolver |
| `LISTQUIZ_DIST` | `/app/frontend/dist` | frontend directory served by the backend |
| `LISTQUIZ_DB` | `/app/data/listquiz.db` | SQLite file path |

The image supplies these values. A user may override them with `docker run -e`, but the documented run
command relies on the image defaults.

## Existing application data (unchanged)

The following SQLite tables remain the only application state and all live inside the data volume:

- `folder`
- `quiz_list`
- `list_item_column`
- `item`
- `item_value`
- `game_session`
- `team`
- `round_config`
- `order_state`
- `tripmine_state`
- `connect_state`
- `score_event`

Uploaded icon images are stored in the `item.icon_image` blob column, so they travel with the same
database volume. No separate upload directory exists.

## State transitions

### Image and container

```text
no image --(docker build)--> image listquiz:local
image --(docker run)-------> running container
running --(docker stop)----> stopped container
stopped --(docker start)---> running container
stopped/removed --(docker run same volume)--> running container with prior data
```

### Database initialization

```text
empty volume --(first start)--> schema created + default folder seeded + default lists seeded
existing volume --(start)-----> schema checked, existing data preserved, seeds are idempotent
```

### Persistence

```text
list/folder/session data --(write)--> /app/data/listquiz.db (and WAL)
container removed, volume kept --> data remains
new container with same volume --> data available
new container with new/empty volume --> fresh seeded state
```

## Validation rules

- The image must contain both `/app/listquiz` and `/app/frontend/dist/index.html`; otherwise the
  container must not be considered a valid build.
- The application must listen on container port 8080 on all interfaces.
- The database directory must be writable by the container user; an unwritable mount must fail visibly
  rather than silently lose data.
- An empty data directory must seed the default folder and default lists.
- A non-empty data directory must not be deleted or overwritten during startup.
- The database, WAL, and shared-memory files must remain inside `/app/data`.
- The container must not write application state into `/app/frontend` or any path outside `/app/data`.
- The image must not include host `backend/target`, `frontend/node_modules`, `frontend/dist`, or
  `backend/data` contents as build inputs.
- The documented host port may change, but the container port remains 8080.
