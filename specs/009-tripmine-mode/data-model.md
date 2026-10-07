# Data Model: Tripmine Mode

No new entity and no schema change. One enum variant, one selection rule, and one ordered cue list.

## Domain — `TripmineSide`

| Variant | Persisted | Meaning |
|---|---|---|
| `Lowest` | `"lowest"` | mine is rank 1; search climbs from the highest down |
| `Highest` | `"highest"` | mine is rank 10; search climbs from the lowest up |
| `Random` | `"random"` | mine is a random rank; search is a random remaining non-mine item (label "Hard") |

## Domain — mine and searched target

| Side | Mine rank | Searched target |
|---|---|---|
| Lowest | 1 | next unrevealed rank below the last hit (existing ladder) |
| Highest | 10 | next unrevealed rank above the last hit (existing ladder) |
| Random | uniform 1..=10 | a random unrevealed item that is not the mine, re-chosen after each target hit |

**Rules (pure, tested)**

- Random's mine is any rank; Random's target is never the mine and never revealed.
- When only the mine and one item remain, the target is that remaining item.
- A target hit with exactly two items left still scores the final-item bonus and ends the round.
- Scoring is identical across sides (target hit, safe pick, mine penalty, final bonus).

## Derived — cue list (frontend, not persisted)

`deriveSessionCues(prev, next): SessionCue[]`

- The event cue first: mine `explosion` / safe `chime` / placement `chime` / wrong `failure` / timeout
  `failure` / selection `slide` / timer start `buzzer`.
- If the session finished in this transition, `bonus` is appended last.
- A round end that does not finish the game yields exactly one cue.
- Example: the mine on the last round → `['explosion', 'bonus']`.

## View — hearts

- `BottomTeams` gains a `showHearts` flag; the session sets it false for a Tripmine round and true for
  Order. No DTO change.
