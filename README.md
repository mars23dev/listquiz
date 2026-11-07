# ListQuiz

A single-room, gamemaster-hosted quiz game. A Rust backend owns the game state, serves the exported
React frontend, and keeps sessions, folders, and lists in a SQLite file inside the repository.

## How this project was built

This is a vibecoding project. It was built with AI assistance from Google AI Studio (Gemini) and
opencode (deepseek v4.1 flash), with human review and manual testing.

The feature specification, plan, data model, contracts, and task breakdown live in
[`specs/001-listquiz-game/`](specs/001-listquiz-game/). Start with
[`quickstart.md`](specs/001-listquiz-game/quickstart.md) for the full run-and-verify guide.

## Prerequisites

| Tool | Required version | Check |
|---|---|---|
| Rust | 1.99 or newer | `cargo --version` |
| Node.js | 24 or newer | `node --version` |
| npm | 11 or newer | `npm --version` |

These are build prerequisites only. The release zip runs without them.

`bun` is not used. All build state stays inside the repository (Rust `backend/target/`, the npm cache,
Vite's cache, and `backend/data/listquiz.db`), so nothing is written under `%APPDATA%`.

## Build and run

```bash
# one-time frontend install
cd frontend
npm install
cd ..

# build everything (frontend export, then backend)
npm --prefix frontend run build
cargo build --manifest-path backend/Cargo.toml --release

# run
cargo run --manifest-path backend/Cargo.toml --release
```

The backend listens on `0.0.0.0:8080`, creates `backend/data/listquiz.db` on first run, and seeds the
read-only default folder. Open `http://localhost:8080/`.

There is no separate frontend server: the backend serves `frontend/dist`, so the frontend never runs
without it.

## Docker container (local)

The image builds the frontend and backend inside Docker, so the host needs Docker but no Rust or Node.js
toolchain. The image is local only; it is not published with releases.

Build from the repository root:

```powershell
docker build --tag listquiz:local .
```

Run it with a named volume so lists, folders, and sessions survive container replacement:

```powershell
docker run --rm --name listquiz --publish 8080:8080 --volume listquiz-data:/app/data listquiz:local
```

Open `http://localhost:8080/`. The database lives in the `listquiz-data` volume at
`/app/data/listquiz.db`. Stop the container with `docker stop listquiz`; the volume remains.

If host port 8080 is busy, publish a different host port and keep container port 8080:

```powershell
docker run --rm --name listquiz --publish 8090:8080 --volume listquiz-data:/app/data listquiz:local
```

Then open `http://localhost:8090/`. To reset the game data, remove the volume explicitly (destructive):

```powershell
docker volume rm listquiz-data
```

## Portable release

The release artifact is a zip containing everything a Windows PC needs at runtime: the compiled
`listquiz.exe`, the built frontend, the license, a `README.txt`, and `start.bat`. No bare `.exe` is
published. The Rust backend must still be compiled to an executable; `start.bat` is the launcher, not a
replacement for it.

Build the same zip locally:

```powershell
powershell -ExecutionPolicy Bypass -File scripts/package-release.ps1
```

That writes `release/listquiz-windows-x64.zip` and its SHA-256 checksum. Extract it on another PC and
run `start.bat`; the game opens at `http://localhost:8080` and keeps its database in `data\listquiz.db`
inside the extracted folder. No Rust, Node.js, or npm is needed on the target machine.

The backend resolves paths at runtime: in development it still uses `backend/data/` and
`frontend/dist/`; in the release zip it uses the folder next to the executable. `LISTQUIZ_ROOT`,
`LISTQUIZ_DB`, and `LISTQUIZ_DIST` override those locations if needed.

Publishing a GitHub release runs `.github/workflows/release.yml`, builds the frontend and backend, and
attaches only the portable zip plus its checksum.

## Checks

```bash
cargo fmt --manifest-path backend/Cargo.toml --check
cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path backend/Cargo.toml        # business logic + storage, no server needed

npm --prefix frontend run lint                       # type check
npm --prefix frontend test                           # component + unit tests
npm --prefix frontend run build                      # production export
```

The two-device browser scenario lives in `tests/e2e/` (Playwright). It needs the built frontend served
by the backend, plus `npm i -D @playwright/test && npx playwright install`; see `playwright.config.ts`.
