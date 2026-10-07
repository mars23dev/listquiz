# Data Model: Fix Turn Timer

No new entity and no schema change. One rule parameter and one cue addition.

## Domain — `turn_timer::expire`

`expire(teams: &[TeamStanding], active_team_id: &str, rule: TimeoutRule) -> (Vec<TeamStanding>, Vec<ScoreEvent>, Option<String>)`

| `TimeoutRule` | Effect | Next team |
|---|---|---|
| `LoseHeart` (Order) | on-turn team loses exactly one heart (never below 0) | next team with hearts, or `None` |
| `PassOnly` (Tripmine) | no heart or point change | next team with hearts, or `None` |

**Rules (pure, tested)**

- The rule is chosen from the current round's quiz type by the `time_out` handler.
- `PassOnly` emits no score events.
- `None` as the next team means the round is over (the existing round-end rules apply).
- The timer always clears and a timeout message is set, in both types.

## Derived — cue mapping (frontend, not persisted)

`deriveSessionCue(prev, next): SessionCue | null` gains two transitions, checked before the
heart-loss rule:

| Transition | Cue |
|---|---|
| `prev.timer === null` and `next.timer !== null` | `buzzer` (timer started) |
| `prev.timer !== null` and `next.timer === null` and `next.lastMessage` starts with `"Time ran out"` | `failure` (timeout) |

- An Order timeout drops a heart too, but the timer check runs first, so exactly one cue plays.
- A timer cleared by any other turn end (placement, pass, next round) plays no cue.

## UI — timer controls

The gamemaster sees three preset buttons (15s, 30s, 60s) and a Stop control; there is no free-text
duration field. Each button posts `start_timer` with its fixed seconds.
