# Data Model: Fix Game End After The Last Round

No new entity and no schema change. One status transition and its guards.

## Session status transition

| From | Trigger | To | Side effects |
|---|---|---|---|
| `running` (or `setup`), current round is the last configured round, that round ends | `finish_if_last_round` | `finished` | `reveal_values = 1`, `active_team_id = NULL`, `last_message = "Game over"` |
| `finished` | `finish_if_last_round` | `finished` (unchanged) | none (no double finish) |
| `running`, current round is not the last | `finish_if_last_round` | `running` (unchanged) | none |

**Rules (named, tested)**

- `finish_if_last_round(conn, session_id) -> ApiResult<bool>` returns whether it finished the game.
- Finishes only when `status != 'finished'` AND `current_round >= round_count`.
- A round "ends" when the Order board is over (every item placed, or no team has hearts) or the
  Tripmine round completes; the call sites are the handlers that end a round.

## Derived — the client view (unchanged shape)

`SessionState.status` becomes `"finished"` for every role, which the frontend already turns into the
game-end dialog (feature 005). No new fields.

## Relationships

- **Session** owns the round count and the current round; the finish check compares them.
- **Game-end dialog** and **finished-session deletion** both key off `status === 'finished'`; neither
  changes.
