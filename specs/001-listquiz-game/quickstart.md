# Quickstart: Run and Verify ListQuiz

Validation guide. It describes how to build, run, and prove the feature works end to end. It contains no
implementation code; task-level detail belongs in `tasks.md`.

## Prerequisites

| Tool | Required version | Check |
|---|---|---|
| Rust | 1.99 or newer | `cargo --version` |
| Node.js | 24 or newer | `node --version` |
| npm | 11 or newer | `npm --version` |

bun is **not** required and must not be used; the project locks npm so a clean checkout installs the same
tree on every machine (research D-002).

All build state is pinned inside the repository so nothing is written under `%APPDATA%`:

| What | Pinned by | Path |
|---|---|---|
| Rust build output | `backend/.cargo/config.toml` | `backend/target/` |
| npm cache | `frontend/.npmrc` | `frontend/.npmrc` cache setting |
| Vite cache | `vite.config.ts` `cacheDir` | `frontend/node_modules/.vite/` |
| SQLite database | backend config | `backend/data/listquiz.db` |

## Build and run

```bash
# one-time frontend install
cd frontend
npm install

# build everything (frontend export, then backend)
cd ..
npm --prefix frontend run build
cargo build --manifest-path backend/Cargo.toml --release

# run
cargo run --manifest-path backend/Cargo.toml --release
```

The backend listens on `0.0.0.0:8080`, creates `backend/data/listquiz.db` on first run, and seeds the
default folder. Open `http://localhost:8080/`.

There is no separate frontend server. The backend serves `frontend/dist`, so the frontend cannot run
without it (Principle IV).

## Verifying no build state escapes the repository

```bash
# expect no matches for a build artifact path under %APPDATA%
dir /s /b "%APPDATA%" 2>nul | findstr /i "listquiz cargo vite"
```

Expected: no output. Repeat after a clean `cargo clean` and a fresh `npm install`.

## Verify the artifact folder is ignored

```bash
git check-ignore -v frontend/dist backend/target backend/data
```

Expected: all three reported as ignored.

## Automated checks

```bash
cargo test --manifest-path backend/Cargo.toml      # business logic + storage
npm --prefix frontend run lint                      # type check
npm --prefix frontend run test                     # component tests
```

Business-logic tests must run without starting a server or touching the network. If any test needs a
running HTTP server, that is a Principle II violation and the rule belongs in a `modules/` crate
function instead (see `data-model.md`).

## Manual end-to-end scenario

Two browser contexts: one gamemaster, one player. Use `http://localhost:8080` for the gamemaster and the
machine's LAN address for the player, to prove the two devices really are separate clients.

### 1. Folder and list preparation

1. Open `/`, choose `Manage Lists`.
2. Confirm the default folder is visible, read-only, and holds the draft's three lists
   (`World History & Inventions`, `Iconic Statues & Architectural Marvels`, `Blockbuster Cinema
   Box Office`) with 11 items each.
3. Create a folder named `Test folder` with a password. Confirm no other list appears anywhere.
4. Unlock it with a wrong password, then the right one. Confirm the wrong attempt reveals nothing.
5. Import a CSV with an `Icon` column, a `Name` column, and one value column, 11 data rows, including
   one quoted field containing a comma. Confirm the quoted field survives whole and the list shows 11
   items.
6. Import a CSV with 10 data rows. Confirm rejection naming the 11-100 range.
7. Confirm the sensitive-data hint is visible on both the editor and the import screen.

### 2. Session setup

1. Open `/setup`.
2. Leave the gamemaster password blank and try to continue. Confirm it is refused.
3. Set a password, choose 2 teams, name both.
4. Choose 3 rounds; set round 1 to Order on the imported list, round 2 to Tripmine with the tripmine side
   set to `Highest (Find #1 Lowest)`, round 3 to Order on the default folder's list.
5. Unlock the `Test folder` password when the list is required, and confirm locked folders are not offered.
6. Create the session. Confirm 4 links: gamemaster, P1, P2, spectator.

### 3. Role gating

1. Open the gamemaster link in context A. Confirm the password gate shows `Gamemaster Access`.
2. Enter a wrong password. Confirm `Incorrect password!` and no host controls.
3. Enter the right password. Confirm the host dashboard appears.
4. Open the player link in context B. Confirm no host controls and no timer anywhere.

### 4. Order round, scoring, and hearts

1. Confirm 3 hearts per team, team 1 on turn and highlighted, one anchor item in the middle showing its
   value, and 10 pool items in random order with no values.
2. As gamemaster, place an item on the correct side. Confirm `+10` for team 1, the item joins the bar
   aligned to its neighbours, and the turn passes to team 2.
3. Place an item on the wrong side. Confirm the item stays in the pool, team 2 loses 1 heart, no points
   are awarded, and the turn still passes.
4. Place items between two existing bar items. Confirm only left, between-two, and right are offered.
5. Confirm the player view in context B reflects each change within a second without a reload, and shows
   no value that has not been revealed.
6. Press reveal. Confirm every value becomes visible in both contexts.
7. Repeat steps 2-3 until a team reaches 0 hearts. Confirm it is skipped and cannot score for the rest of
   the round.

### 5. Tripmine round

1. Advance to round 2. Confirm team 2 holds the first turn.
2. Confirm 10 items in a grid with no values and `searching item 1` displayed.
3. Pick the item at position 1. Confirm `+10`, position 1 revealed, and the searched position advances.
4. Pick a wrong item that is not the tripmine. Confirm `+5` and no change to the searched position.
5. Try to pick with the player view in context B. Confirm the pick is refused.
6. Reach the state with two unrevealed items and pick the valid one. Confirm `+20` and the round ends.
7. Start round 2 again and pick the tripmine instead. Confirm the picker loses 10, every other team gains
   10, all positions are revealed, and the round ends.
8. Confirm the tripmine side cannot be changed once the round has started.

### 6. Cross-round carry-over

1. Note the scores at the end of round 2. Confirm round 3 opens on exactly those totals (SC-005).
2. Confirm the game ends after round 3 and shows the cross-round ranking.

### 7. Session persistence

1. Close both browsers. Reopen the gamemaster link.
2. Confirm identical scores, hearts, turn, and board state (SC-011).
3. Restart the backend process, then reload. Confirm the same state again.

### 8. Theme and keyboard

1. Toggle the theme. Confirm both themes are legible on every screen, the choice persists across a
   reload, and no screen loses state (SC-012).
2. Complete list import, session setup, one Order placement, and one Tripmine pick using only the
   keyboard. Confirm every control is reachable, operable, and visibly focused (SC-006).
3. Confirm no local action froze input or caused a full-page reload (SC-007).

### 9. Folder deletion

1. Delete the `Test folder`, confirming the list count and re-entering its password. Confirm its lists are
   gone and the default folder is untouched.
2. Confirm the already-running session still finishes, and a later round referencing the deleted list
   reports that the list is gone.

## Expected results summary

| Spec outcome | Verified by step |
|---|---|
| SC-001 setup in under 5 minutes | 2 |
| SC-002 action visible in the player view within 1 s | 4.5 |
| SC-003 no hidden value ever leaks to a player view | 4.5 |
| SC-004 scoring matches the spec in both quiz types | 4, 5 |
| SC-005 cross-round carry-over | 6.1 |
| SC-006 full keyboard operation | 8.2 |
| SC-007 no input freeze, no full-page reload | 8.3 |
| SC-008 11-100 enforced at creation and import | 1.6 |
| SC-009 locked folder never renders list names | 1.4 |
| SC-011 state survives reload and restart | 7 |
| SC-012 both themes legible | 8.1 |