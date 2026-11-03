# Data Model: Session Flow And Quick Game

No new persisted entity except two nullable columns on `game_session`. One new derived view
value (the timer), one new request shape (quick game), and one guard on deletion.

## Stored — `game_session` (SQLite)

| Column | Type | Change | Notes |
|---|---|---|---|
| `timer_ends_at` | INTEGER | **NEW** | epoch ms when the running timer expires; `NULL` = no timer |
| `timer_total_seconds` | INTEGER | **NEW** | the duration the gamemaster chose; `NULL` = no timer |

**Validation / rules**

- Both columns are `NULL` together; a timer exists only when both are set.
- Added by an idempotent migration (same guarded `ALTER TABLE` pattern as feature 004); an
  existing database reads `NULL` (no timer) until one is started.
- Deletion: `DELETE FROM game_session WHERE id = ?1`; the schema's `ON DELETE CASCADE` removes
  `team`, `round_config`, `order_state`, `tripmine_state`, and `score_event` rows for the
  session. `quiz_list` is never touched.

## Domain — `TurnTimer` (Rust, `turn_timer`)

| Field | Type | Notes |
|---|---|---|
| `ends_at` | `i64` | epoch ms |
| `total_seconds` | `u32` | the chosen duration, for the ring/label |

**Rules (pure, tested)**

- `start(now, seconds) -> TurnTimer`: `ends_at = now + seconds*1000`; seconds must be > 0.
- `remaining(ends_at, now) -> u32`: `ceil((ends_at - now)/1000)`, floored at 0.
- `is_expired(ends_at, now) -> bool`: `now >= ends_at`.
- `expire(teams, active_team_id) -> (teams, next_team_id)`: on-turn team loses exactly one
  heart (never below 0), turn passes to the next team with hearts; reuses the scoring module.

**State transitions**

- `start_timer(seconds)` → set both columns (gamemaster only).
- `stop_timer` → clear both columns (gamemaster only).
- `time_out` → accepted only when a timer runs, `now >= ends_at`, and the caller is the
  gamemaster or the on-turn team; applies `expire` and clears the timer (idempotent: a second
  call finds no timer and is refused).
- Any other end-of-turn event (placement confirmed, Tripmine pick, `pass_turn`, `set_turn`,
  `next_round`) → clear both columns.

## View DTO — `SessionState` addition (Rust `app_state`, mirrored in `types/session.ts`)

| Field | Type | Redaction |
|---|---|---|
| `timer` | `{ totalSeconds: number, secondsLeft: number } \| null` | public to every role; `null` when no timer runs |

`secondsLeft` is computed at read time from `timer_ends_at`; the client counts down locally
between reads and posts `time_out` at zero.

## View — Tripmine visibility (unchanged redaction, changed rendering)

The server already sends the gamemaster every item's `value` and `true_rank` and the players
only revealed ones. The board must render the gamemaster's value for **every** item (picked or
not) and keep the player's value hidden until revealed.

## Request — quick game (`quick_game`)

`build_request(game_type, default_lists) -> CreateSessionRequest`

- One team; one round; `status` starts the round as a normal `next_round` would.
- List/column chosen from the default lists so the type fits: an Order round needs at least
  eleven valued items, a Tripmine round ten. The chosen default list is the first that fits.
- `gm_password` is set to an unused value (quick games expose no gamemaster access).
- Produces an ordinary session: it inherits the end dialog and the finished-session deletion.

## Deletion guard

`delete_session(conn, id)`:

- `not_found` when the session does not exist (also treated as success by the endpoint).
- `not_ready` when the status is not `finished`.
- Otherwise delete; the cascade removes the session's rows.
