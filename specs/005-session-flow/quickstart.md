# Quickstart: Session Flow And Quick Game

A run-and-verify guide for the six findings. It links the rules to
[contracts/session-flow.md](./contracts/session-flow.md) and
[data-model.md](./data-model.md) rather than repeating them.

## Prerequisites

Build the exported frontend and run the backend that serves it (Principle IV):

```powershell
npm --prefix frontend run build
cargo run --manifest-path backend/Cargo.toml --release
```

Open `http://localhost:8080/`.

## 1. The gamemaster sees every Tripmine value

1. Create a session with a Tripmine round and open the gamemaster view and a player view.
2. Start the round.
   - Expected: the gamemaster sees each item's value and rank, including the mine, before any
     pick. The player sees no value or rank on unpicked items.
3. Pick an item as the on-turn team.
   - Expected: the gamemaster still sees everything; the player sees the picked item's value.

Rule: [contracts/session-flow.md](./contracts/session-flow.md) §5.

## 2. Turn timer

1. As gamemaster, start a short timer (e.g. 10s) on a turn.
   - Expected: a live countdown shows in every view for the current team.
2. Let it reach zero.
   - Expected: the on-turn team loses one heart, the turn passes to the next team with hearts,
     and the timer clears (the failed-turn cue plays).
3. Start another timer and click **Stop**.
   - Expected: it clears with no heart lost.
4. Start a timer, then end the turn another way (place/pick/pass).
   - Expected: the timer clears.

Rule: [contracts/session-flow.md](./contracts/session-flow.md) §1.

## 3. Game-end dialog

1. Play a game to the last round's end.
   - Expected: a pop-up dialog appears, teams ranked by points, showing **points only** (no
     hearts).
2. End only a round in a multi-round game.
   - Expected: no end dialog.

Rule: [contracts/session-flow.md](./contracts/session-flow.md) §4.

## 4. Finished session is cleaned up

1. With the end dialog open, dismiss it.
   - Expected: you return to the title screen, and the session's URL now reports not-found.
2. In a multi-round game, end one round and reload the session.
   - Expected: it still loads (only a finished game is deleted).

Rule: [contracts/session-flow.md](./contracts/session-flow.md) §2.

## 5. Quick game

1. On the title screen, choose **Quick Game**, pick Order or Tripmine.
   - Expected: a one-team game starts immediately in the player view; no gamemaster link is
     offered.
2. Play to the end.
   - Expected: the normal end dialog appears and the session is deleted when dismissed.

Rule: [contracts/session-flow.md](./contracts/session-flow.md) §3.

## 6. Dark only

1. Visit the title, lists, setup, and session screens.
   - Expected: no theme toggle anywhere; the dark theme throughout.

Rule: [contracts/session-flow.md](./contracts/session-flow.md) §6.

## Automated checks

```powershell
cargo test --manifest-path backend/Cargo.toml      # timer rules, quick-game builder, delete guard
npm --prefix frontend run lint
npm --prefix frontend test                          # countdown, end dialog, quick game, no toggle
npm --prefix frontend run build
```

### Independent (no-server) checks

- `turn_timer` unit tests: `start`, `remaining`, `is_expired`, and `expire` (one heart, next team,
  no negative hearts).
- `quick_game::build_request` unit tests: a one-team, one-round request whose default list fits
  the game type; the smallest-fits choice; an error when no list fits.
- `sessions::delete_session` unit tests: refused unless `finished`; cascade leaves no orphan rows;
  a missing id is handled.
- Component tests: the countdown reaches zero and posts `time_out`; the end dialog lists points
  and no hearts; the title screen offers quick game; no theme toggle is rendered.
