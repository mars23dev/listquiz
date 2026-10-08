# Data Model: Reveal Values In Tripmine And Connect

No schema change. The reveal flag drives what each mode unhides.

## Reveal flag (existing `game_session.reveal_values`)

| Value | Meaning |
|---|---|
| 0 | Hidden for the current round. |
| 1 | Revealed for the current round (gamemaster action); reset when the next round starts (feature 020). |

## What each mode unhides when the flag is set

| Mode | Hidden by default | Revealed |
|---|---|---|
| Order | Bar/pool values | Values (unchanged) |
| Tripmine | Item values, ranks, the mine | All items' values, ranks, and the mine |
| Connect | The name↔value pairing (values are public) | The pairing (answer key) |

## Validation rules

- The reveal works in all three modes (FR-001, FR-002).
- Before the reveal, players and the spectator see none of the hidden info (FR-003).
- Revealing changes nothing else and does not end the round (FR-004).
- The per-round reset still holds (FR-005).
