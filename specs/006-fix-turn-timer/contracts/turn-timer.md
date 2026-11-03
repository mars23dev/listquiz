# Contract: Turn Timer

Amends the timer contract in `specs/005-session-flow/contracts/session-flow.md` §1. No new endpoint.

> **Amended by `specs/008-fix-timer-timeout/contracts/timer-timeout.md`:** the server now schedules
> and applies the timeout itself at the end time (no client `time_out` post, no timer error bar), and
> the countdown ticks on each of the last five seconds. The consequence in §1 is unchanged.

## 1. Timeout consequence per game type

`time_out` (`POST /api/sessions/{sessionId}/actions`) now behaves by the current round's type:

| Round type | On expiry |
|---|---|
| Order | on-turn team loses exactly one heart; turn passes to the next team with hearts; existing round-end rules apply if none remain |
| Tripmine | turn passes to the next team with hearts; NO team's hearts or points change |

Both types clear the timer and set `lastMessage` to a message beginning `"Time ran out"`. `time_out`
remains accepted only when a timer runs and has expired, and is idempotent.

## 2. Timer cues (frontend contract)

The cue mapping (feature 004 §2) gains:

- a timer appearing where there was none → the **start** cue (buzzer);
- a timer clearing with the timeout message → the **timeout** cue (failure tone).

- Driven from the reflected state, for every role, and silent while the session is muted.
- An Order timeout plays exactly one cue (the timeout check precedes the heart-loss check).
- A timer cleared by any other turn end plays no cue.

## 3. Preset durations

The gamemaster starts the timer from exactly three presets — **15s**, **30s**, **60s** — each a
single action. There is no free-text duration field. Starting a preset while a timer runs replaces
it. The `start_timer` action still accepts a positive `seconds`; the frontend only sends 15, 30, or
60.
