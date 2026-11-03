# Quickstart: Fix Game End After The Last Round

A run-and-verify guide for the finding. It links the transition to
[contracts/game-end.md](./contracts/game-end.md) and [data-model.md](./data-model.md).

## Prerequisites

Build the exported frontend and run the backend that serves it (Principle IV):

```powershell
npm --prefix frontend run build
cargo run --manifest-path backend/Cargo.toml --release
```

Open `http://localhost:8080/`.

## 1. Quick game ends and shows the dialog

1. On the title screen, choose **Quick Game → Order** (or Tripmine). The one-team game starts in the
   player view.
2. Play the single round to its end (place every item, or run the Tripmine round to completion).
   - Expected: the game-end dialog pops up on its own, with no gamemaster action.
3. Dismiss the dialog.
   - Expected: you return to the title screen and the session's URL now reports not-found.

Rule: [contracts/game-end.md](./contracts/game-end.md).

## 2. Normal game: last round vs earlier rounds

1. Create a multi-round session and open the gamemaster and a player view.
2. Finish a round that is **not** the last.
   - Expected: no game-end dialog; the gamemaster can start the next round.
3. Finish the **last** round.
   - Expected: the game-end dialog appears for every role, without pressing "Finish Game".

Rule: [contracts/game-end.md](./contracts/game-end.md).

## 3. Both game types

1. Repeat §1 with an Order quick game and a Tripmine quick game.
   - Expected: the dialog appears at the end of each.

## Automated checks

```powershell
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run lint
npm --prefix frontend test
npm --prefix frontend run build
```

### Independent (no-server) checks

- Backend tests: ending the last Order round (board full / no hearts) and the last Tripmine round
  sets `finished`; ending a non-last round does not; a quick game ends without any gamemaster action;
  finishing an already-finished session is a no-op.
- The existing game-end dialog component test still passes (unchanged).
