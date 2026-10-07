# Data Model: Fix Timer Timeout

No new entity and no schema change. One rule reused and one scheduling behavior.

## Domain — `sessions::apply_turn_timeout`

`apply_turn_timeout(conn, session_id) -> ApiResult<bool>`

- No timer running → no-op, returns `false` (idempotent).
- Timer running → applies the timeout and returns `true`:
  - Order: on-turn team loses exactly one heart; turn passes to the next team with hearts.
  - Tripmine: turn passes; no heart or point change.
  - Clears the timer; sets a `"Time ran out…"` message; if no team is left and it is the last round,
    finishes the game (feature 007).
- The rule is the same one the manual `time_out` action uses.

## Scheduling

- `start_timer` schedules the deadline: after the chosen seconds, the server calls
  `apply_turn_timeout` for that session and bumps the revision.
- The scheduled call is a no-op unless the session's stored timer still has the same end time
  (stopped/replaced/ended/deleted → nothing).
- No client action is required to end the turn.

## Derived — countdown UI (frontend, not persisted)

| Transition | Behavior |
|---|---|
| timer appears | start cue (buzzer), from the reflected state (unchanged) |
| each of the last five seconds | a short tick (client-side) |
| countdown reaches 0 | nothing posted by the client; the server has already applied (or will apply) the timeout, and the timeout cue plays from the reflected state (unchanged) |

The countdown stays the only visible timer indicator; there is no error or notice bar for the timer.
