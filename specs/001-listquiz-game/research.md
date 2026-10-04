# Phase 0 Research: ListQuiz

Every NEEDS CLARIFICATION in the Technical Context is resolved below, together with the decisions that
were still open when the specification was signed off.

## D-001: Frontend stack is inherited, not chosen

**Decision**: Copy the draft frontend's stack and source into `frontend/` — React 19, Vite, Tailwind 4
via `@tailwindcss/vite`, `lucide-react`, `motion`, `TypeScript 7`. Correct the eight logic conflicts
listed in the specification's Design Authority table and change nothing else visually.

**Rationale**: the user requires the draft to be reused as much as possible and the design taken 1:1.
Keeping the exact dependency set means the copy compiles and looks identical on day one, and every
divergence from the draft becomes a reviewable diff instead of a rewrite.

**Alternatives considered**: adopting a different component framework (rejected — costs weeks and
breaks 1:1 parity); keeping the draft's files in place and importing them across folders (rejected —
constitution forbids `the reference frontend` as an import source and a build input).

## D-002: Package manager is npm, not bun

**Decision**: npm, with `package-lock.json` committed.

**Rationale**: `the reference frontend` ships a `bun.lock`, but bun is not installed on this machine (checked:
`bun --version` returns nothing; Node 24.12.0 and npm 11.11.0 are present). Committing a lockfile from a
tool the project cannot run would break clean setup for everyone else.

**Alternatives considered**: bun (rejected — not installed); pnpm or yarn (rejected — no benefit, one
extra install).

## D-003: HTTP framework is axum

**Decision**: `axum` on `tokio`, with `tower-http` for static file serving and gzip.

**Rationale**: axum is on the same runtime as SSE, which is the only push mechanism needed (D-005). Its
handler-per-route shape maps directly onto the draft's screens, and `tower-http`'s `ServeDir` covers
"the backend serves the exported frontend" with one line instead of a custom static handler.

**Alternatives considered**: actix-web (rejected — heavier, no advantage here); a hand-rolled
`hyper` router (rejected — reimplementing routing is exactly the over-engineering Principle VII
forbids); serving the frontend from a separate static server (rejected — violates Principle IV).

## D-004: SQLite access is `rusqlite` with the bundled feature

**Decision**: `rusqlite` with `bundled`, WAL journaling, one connection behind a `tokio::sync::Mutex`,
inside a blocking task.

**Rationale**: `bundled` compiles SQLite into the binary, so there is no system database, no C
toolchain, and no external process — which is what "all compiling in the project folders" demands.
This product has one writer (the gamemaster) and a handful of readers, so connection pooling and
async database drivers buy nothing. `sqlx` was rejected because its compile-time checked queries
require either a live database at build time or an offline query directory, both extra machinery.

**Alternatives considered**: `sqlx` (rejected — build-time database coupling); `diesel` with
migrations (rejected — ORM and migration layer on top of a schema small enough to fit on one page);
plain JSON files instead of SQLite (rejected — the constitution mandates SQLite).

## D-005: Live updates use server-sent events

**Decision**: One `GET /api/sessions/{id}/events` SSE stream per open view. The server pushes
`{"type":"state","revision":N}`; the client then re-reads state for its role. Roles are passed as a query
parameter and the server redacts per role.

**Rationale**: every mutation originates in the gamemaster view (FR-062, no player controls exist), so
the traffic is one-directional. SSE gives that over plain HTTP with automatic reconnection and no extra
protocol. Pushing revisions instead of state bodies means hidden values are redacted in exactly one
place — `build_public_state` — so there is no path by which a player view can render a value the
gamemaster has not revealed.

**Alternatives considered**: WebSockets (rejected — bidirectional channel and reconnect logic for traffic
that is never client-initiated); polling every second (rejected — wasteful, and 1 s of staleness sits
exactly on the SC-002 boundary); embedding the whole board in the SSE payload (rejected — pushes
redaction into the client, where one mistake leaks the answer).

## D-006: Passwords are stateless, with no token issuance

**Decision**: The gamemaster password and folder passwords are compared on every request against the
value stored in SQLite. There is no login endpoint, no session token, no cookie. The frontend holds an
entered password in memory for the life of the tab and re-prompts after a reload.

**Rationale**: the constitution states these passwords need no real protection, and the specification
says the product must not imply otherwise. A token would add issuance, storage, expiry, and revocation
concerns to buy nothing, which is the over-engineering the constitution prohibits.

**Alternatives considered**: hashed passwords plus a bearer token (rejected — explicitly excluded by
the constitution and by FR-068); cookie sessions (rejected — same cost, plus CSRF surface).

## D-007: Session identifiers are `lq-` plus eight random lowercase alphanumerics

**Decision**: Generate the identifier at session creation from 48 bits of randomness, formatted as
`lq-xxxxxxxx`, and retry on the (checked, unique-indexed) collision. The bare `/<sessionId>` path
redirects to `/<sessionId>/p1`, and any unrecognised identifier renders the not-found view (FR-013).

**Rationale**: the draft's four-digit random suffix gives roughly 6 500 identifiers before collisions
become likely, which is reachable by accident in a long-lived installation and silently produces a
wrong game. Eight characters is short enough to read aloud across a room and long enough to be safe.

**Alternatives considered**: full UUID (rejected — 36 characters to dictate aloud); draft's four digits
(rejected — collision risk).

## D-008: Theme is a `data-theme` attribute plus CSS variables

**Decision**: One palette expressed as CSS custom properties under `:root[data-theme="dark"]` and
`:root[data-theme="light"]`. Every surface, border, and text colour in the draft is replaced by the
matching variable; team accent colours and status colours stay fixed because they must read the same in
both themes. The attribute is set from the operating system preference on first visit and stored
locally afterwards (FR-063).

**Rationale**: the draft hardcodes Tailwind palette classes, which is why it is dark-only. Variables are
the smallest change that produces a real light theme without forking every component or inventing a
second design, which the clarification session ruled out.

**Alternatives considered**: Tailwind's `dark:` variant everywhere (rejected — doubles the class surface
and forces a decision in every component); authoring a distinct light design (rejected — rejected by the
user during clarification).

## D-009: CSV is parsed with the `csv` crate

**Decision**: Use the `csv` crate for both import and export.

**Rationale**: the draft splits on commas and strips quotes by hand, which breaks on any quoted field
containing a comma. FR requires quoted fields with commas to survive whole. The crate is a
well-scoped dependency that removes a class of bug rather than adding architecture.

**Alternatives considered**: porting the draft's hand-rolled parser (rejected — knowingly reimplementing
a known-broken parser); a spreadsheet-typed reader (rejected — the specification defines one comma
format).

## D-010: Board interaction is click-and-confirm first, drag as an enhancement

**Decision**: Keep the draft's flow — pick an item, drop or click a bar slot, confirm or cancel — but
make every step reachable by keyboard: the pool is a grid of buttons, the bar slots are buttons in DOM
order, and the pending item can be confirmed or cancelled with a keystroke.

**Rationale**: FR-065 requires keyboard operation and FR-028 limits placement to left, between-two, and
right. The draft's drag-and-drop already computes exactly those three outcomes and its confirm step
already exists, so keeping it means accessibility is added on top of the reused design rather than
rebuilt alongside it.

**Alternatives considered**: dropping drag-and-drop entirely (rejected — it is core to the 1:1 design
and is the fastest path for a gamemaster placing on behalf of players); custom keyboard grid navigation
with roving tabindex (rejected for v1 — deferred, since plain button order already satisfies FR-065).

## D-011: Numbers that cannot be parsed fall back to row position

**Decision**: When a CSV value cell yields no number, the item keeps its display string and receives a
comparable value derived from its row index, and the import response lists the affected rows.

**Rationale**: the specification requires an unreadable value to degrade rather than fail, and requires
the gamemaster to be told. Deriving the fallback from row position is deterministic, so two imports of
the same file produce the same list.

**Alternatives considered**: rejecting the import (rejected — contradicts FR-057); leaving the value
uncomparable (rejected — the round could not be built).

## D-012: Cross-device state without a broadcast bus

**Decision**: One process holds the authoritative session state. A `tokio::sync::broadcast` channel
carries revision numbers to SSE subscribers; SQLite holds the durable copies. No Redis, no message
broker, no separate state service.

**Rationale**: single-room scale does not justify one, and Principle VII requires naming the present-day
requirement that an abstraction serves. There is none.

**Alternatives considered**: Redis pub/sub (rejected — no requirement); polling the database per action
(rejected — the broadcast channel is a few lines and keeps reads off the hot path).

## Resolved Technical Context unknowns

| Unknown | Resolution |
|---|---|
| Language/version | Rust 1.99 (edition 2024); TypeScript 7 + React 19 in the frontend (D-001) |
| Primary dependencies | D-003, D-004, D-009 |
| Storage | SQLite at `backend/data/listquiz.db` (D-004) |
| Testing | `cargo test`, `vitest`, Playwright (plan Technical Context) |
| Target platform | Single machine, `0.0.0.0:8080`, no proxy (plan Technical Context) |
| Project type | Web application, backend serves exported frontend (D-003) |
| Performance goals | 1 s action-to-player-view propagation, 60 fps board (plan Technical Context) |
| Constraints | Project-local build state, no timer, passwords unprotected (plan Technical Context) |
| Scale/scope | 1 gamemaster, ~10 views, 10 rounds, 100 items, 4 teams (plan Technical Context) |

No technical-context unknown remains unresolved.