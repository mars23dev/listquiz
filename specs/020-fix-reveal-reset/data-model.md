# Data Model: Fix Reveal Reset On Next Round

No schema change. The change resets an existing flag at a round boundary.

## Session reveal flag (existing `game_session.reveal_values`)

| Field | Meaning |
|---|---|
| `reveal_values` | 0 = values hidden for the current round; 1 = revealed. |

## Transitions

| From | Event | `reveal_values` after |
|---|---|---|
| Any round | Gamemaster reveals | 1 |
| Round active | Advance to the next round | **0** (reset) |
| Last round | Cross the last round (finish) | 1 (unchanged) |

## Validation rules

- Advancing resets the flag so the next round starts hidden (FR-001–FR-003).
- Finishing the game keeps the reveal (FR-004).
- No other round behavior changes (FR-005).
