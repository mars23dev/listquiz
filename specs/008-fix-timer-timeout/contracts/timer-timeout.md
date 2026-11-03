# Contract: Timer Timeout

Amends the timer contract in `specs/005-session-flow/contracts/session-flow.md` §1 and
`specs/006-fix-turn-timer/contracts/turn-timer.md`.

## 1. The server owns the deadline

- `start_timer` records the timer's end time AND schedules the timeout to be applied at that time.
- When the deadline passes, the server applies the timeout for that session and bumps the revision,
  so every connected client sees the new turn. No client action is required.
- The scheduled application is a no-op if the timer was stopped, replaced, the round ended, or the
  session was deleted.

## 2. The timeout consequence (unchanged from feature 006)

| Round type | On expiry |
|---|---|
| Order | on-turn team loses exactly one heart; turn passes to the next team with hearts; existing round-end rules apply if none remain |
| Tripmine | turn passes to the next team with hearts; no heart or point change |

Both clear the timer and set a `"Time ran out…"` message. The `time_out` action still exists and
applies the same rule when a timer runs and has expired.

## 3. No client post, no timer error bar

- The client does not post `time_out`; reaching 0 in the countdown is not an action.
- No error or notice bar for the timer is shown to any role; the countdown is the only timer
  indicator.

## 4. Last-five-seconds ticks

- The client plays a short tick on each of the last five seconds of a running countdown (5, 4, 3, 2,
  1; a shorter timer ticks on each remaining second).
- Ticks obey mute; the timeout cue still plays at zero, after the ticks.
