# Data Model: Connect Game And Tripmine Marker

One new stored board, one new view, and one new public field on the Tripmine view.

## Stored — `connect_state` (SQLite, NEW table)

| Column | Type | Notes |
|---|---|---|
| `session_id` | TEXT | primary key, references `game_session` (cascade) |
| `names` | TEXT | JSON array of `{ itemId, name, iconName }` — the left column, fixed order |
| `values` | TEXT | JSON array of `{ itemId, display }` — the right column, shuffled order |
| `matched` | TEXT | JSON array of matched `itemId`s |
| `pending` | TEXT | JSON of `{ nameItemId, valueItemId }` or `null` |

- Added with `CREATE TABLE IF NOT EXISTS` (no migration); cleared with the other round state at each
  round start.

## Domain — `connect_round`

- `ConnectBoard { names, values, matched, pending }`.
- `sample_connect_round(items) -> ApiResult<ConnectBoard>`: draws ten items (≥10 required), names in
  list order, values shuffled.
- `pick_name(board, item_id)` / `pick_value(board, item_id)`: set the pending side; reject an
  already-matched id.
- `confirm(board, teams, team_id) -> ConfirmOutcome`: matching pair → +10 points, marks it matched,
  clears pending; mismatch → one heart lost, clears pending, pair stays. Always advances the turn.
- `round_is_over(board, teams)`: true when all pairs matched or no team has hearts.
- `find`/`side` helpers as needed.

## View DTO — `SessionState` additions (Rust `app_state`, mirrored in `types/session.ts`)

| DTO | Field | Redaction |
|---|---|---|
| `ConnectView` | `names: { itemId, name, iconName }[]` | public |
| | `values: { itemId, display }[]` | public (Connect is a matching game) |
| | `matched: string[]` | public |
| | `pending: { nameItemId, valueItemId } \| null` | public |
| `TripmineView` | `mineRank: number` | **public to every role** (position only) |

## Actions (`POST /api/sessions/{id}/actions`)

| Action | Body | Who | Effect |
|---|---|---|---|
| `pick_name` | `{ itemId }` | gamemaster, or on-turn team | set the pending name |
| `pick_value` | `{ itemId }` | gamemaster, or on-turn team | set the pending value |
| `confirm_connect` | `{}` | gamemaster, or on-turn team | judge the pending pair, score, advance the turn |

A Connect timeout uses the Order rule (one heart + pass), since the timeout rule is `LoseHeart` for
every type except Tripmine.

## Frontend view

- `ConnectBoard`: left names (icon + name), right values (no icon), a middle box holding the pending
  name and value, and Confirm/Cancel; items are buttons.
- The Tripmine progress bar marks `mineRank` from the start.
