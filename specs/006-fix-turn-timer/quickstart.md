# Quickstart: Fix Turn Timer

A run-and-verify guide for the three findings. It links the rules to
[contracts/turn-timer.md](./contracts/turn-timer.md) and [data-model.md](./data-model.md).

## Prerequisites

Build the exported frontend and run the backend that serves it (Principle IV):

```powershell
npm --prefix frontend run build
cargo run --manifest-path backend/Cargo.toml --release
```

Open `http://localhost:8080/` and create a session with an Order round and a Tripmine round.

## 1. Timeout consequence by game type

1. In an **Order** round, start a 15s timer on a turn and let it expire.
   - Expected: the on-turn team loses one heart and the turn passes; a timeout message shows.
2. In a **Tripmine** round, start a 15s timer and let it expire.
   - Expected: the turn passes to the next team and **no** team's hearts or points change.

Rule: [contracts/turn-timer.md](./contracts/turn-timer.md) §1.

## 2. Timer sounds

1. In an unmuted session, start the timer.
   - Expected: a start cue plays in every view.
2. Let it expire.
   - Expected: a timeout cue plays (in Order, exactly one cue even though a heart is lost).
3. Mute the session and repeat.
   - Expected: neither cue plays.

Rule: [contracts/turn-timer.md](./contracts/turn-timer.md) §2.

## 3. Preset durations

1. As gamemaster, look at the timer controls.
   - Expected: three choices — 15s, 30s, 60s — and no free-text duration field.
2. Pick 30s.
   - Expected: the countdown runs from 30 seconds.
3. Pick 15s while one runs.
   - Expected: the running timer is replaced.

Rule: [contracts/turn-timer.md](./contracts/turn-timer.md) §3.

## Automated checks

```powershell
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run lint
npm --prefix frontend test
npm --prefix frontend run build
```

### Independent (no-server) checks

- `turn_timer` unit tests: `LoseHeart` removes one heart; `PassOnly` changes nothing; both advance the
  turn.
- HTTP test: an Order timeout costs a heart; a Tripmine timeout does not; both advance the turn.
- `deriveSessionCue` tests: a timer appearing → start cue; a timer clearing with the timeout message →
  timeout cue; a placement clearing the timer → no cue; an Order timeout → one cue.
- Component test: the three presets are present and no duration input exists.
