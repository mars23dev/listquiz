# ListQuiz

A single-room, gamemaster-hosted quiz game. A Rust backend owns the game state, serves the exported
React frontend, and keeps sessions, folders, and lists in a SQLite file inside the repository.

The feature specification, plan, data model, contracts, and task breakdown live in
[`specs/001-listquiz-game/`](specs/001-listquiz-game/). Start with
[`quickstart.md`](specs/001-listquiz-game/quickstart.md) for the full run-and-verify guide.

## Prerequisites

| Tool | Required version | Check |
|---|---|---|
| Rust | 1.99 or newer | `cargo --version` |
| Node.js | 24 or newer | `node --version` |
| npm | 11 or newer | `npm --version` |

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
