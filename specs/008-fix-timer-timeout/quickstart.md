# Quickstart: Fix Timer Timeout

A run-and-verify guide for the three findings. It links the rules to
[contracts/timer-timeout.md](./contracts/timer-timeout.md) and [data-model.md](./data-model.md).

## Prerequisites

Build the exported frontend and run the backend that serves it (Principle IV):

```powershell
npm --prefix frontend run build
cargo run --manifest-path backend/Cargo.toml --release
```

Open `http://localhost:8080/` and create a session with an Order round and a Tripmine round.

## 1. Reaching 0 ends the turn

1. As gamemaster, start a 15s timer on a turn and let it reach 0.
   - Expected (Order): the on-turn team loses one heart and the turn passes to the next team.
   - Expected (Tripmine): the turn passes with no heart or point change.
2. Leave the tab in the background for the whole countdown, then return.
   - Expected: the turn has already ended — no client action was needed.

Rule: [contracts/timer-timeout.md](./contracts/timer-timeout.md) §1–2.

## 2. No stray error bar

1. Watch a timer run to 0.
   - Expected: no "The timer has not run out yet." bar (or any timer error); only the countdown.

Rule: [contracts/timer-timeout.md](./contracts/timer-timeout.md) §3.

## 3. Last-five-seconds ticks

1. Start a timer of at least five seconds and listen.
   - Expected: a tick on each of the last five seconds (5, 4, 3, 2, 1), then the timeout cue at 0.
2. Mute the session and repeat.
   - Expected: no ticks, no timeout cue.

Rule: [contracts/timer-timeout.md](./contracts/timer-timeout.md) §4.

## Automated checks

```powershell
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run lint
npm --prefix frontend test
npm --prefix frontend run build
```

### Independent (no-server) checks

- `apply_turn_timeout` tests: no timer → no-op; Order → one heart + turn passes; Tripmine → turn
  passes; clears the timer; finishes the last round when nobody is left.
- HTTP test: a 1s timer ends the turn on its own (no `time_out` action posted).
- Frontend test: the countdown reaching 0 posts nothing; a tick fires on each of the last five
  seconds; the start and timeout cues still play.
