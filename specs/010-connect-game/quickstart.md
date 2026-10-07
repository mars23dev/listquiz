# Quickstart: Connect Game And Tripmine Marker

A run-and-verify guide for the two findings. It links the rules to
[contracts/connect.md](./contracts/connect.md) and [data-model.md](./data-model.md).

## Prerequisites

Build the exported frontend and run the backend that serves it (Principle IV):

```powershell
npm --prefix frontend run build
cargo run --manifest-path backend/Cargo.toml --release
```

Open `http://localhost:8080/`.

## 1. The Connect game

1. Create a session with a Connect round (setup now offers it), or start a Connect quick game.
2. Open the board.
   - Expected: ten names on the left (each with an icon), ten values on the right in a random order
     with no icons.
3. On a turn, pick one name and one value; both appear in the middle box.
   - Expected: the pair waits in the box until confirmed.
4. Confirm a matching pair.
   - Expected: +10 points, the pair is matched, the turn passes to the next team.
5. Confirm a mismatching pair.
   - Expected: the on-turn team loses one heart, the pair stays, the turn passes.
6. Keep making mistakes until a team has no hearts.
   - Expected: the round goes read-only (like Order) and is over.
7. Let the gamemaster's timer run out in a Connect round.
   - Expected: the on-turn team loses one heart and the turn passes.

Rule: [contracts/connect.md](./contracts/connect.md) §1.

## 2. Tripmine mine marker

1. Start a Tripmine round (any side) and look at the progress bar.
   - Expected: the mine's position is marked from the start, for every role.
2. Look at the items.
   - Expected: the mine's item is still hidden; only the position is shown.
3. Reveal the mine later.
   - Expected: the bar still shows it as the mine.

Rule: [contracts/connect.md](./contracts/connect.md) §2.

## Automated checks

```powershell
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run lint
npm --prefix frontend test
npm --prefix frontend run build
```

### Independent (no-server) checks

- `sample_connect_round`: ten names and ten shuffled values; a list under ten items is refused.
- `pick_name`/`pick_value`/`confirm`: a matching pair scores +10 and matches; a mismatch loses one
  heart; the turn always advances; a matched id cannot be picked again.
- `round_is_over`: true when all matched or no hearts.
- Tripmine view: `mineRank` is present for every role; the board marks it; the item stays hidden.
