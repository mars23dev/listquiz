# Quickstart: Tripmine Mode

A run-and-verify guide for the three findings. It links the rules to
[contracts/tripmine-mode.md](./contracts/tripmine-mode.md) and [data-model.md](./data-model.md).

## Prerequisites

Build the exported frontend and run the backend that serves it (Principle IV):

```powershell
npm --prefix frontend run build
cargo run --manifest-path backend/Cargo.toml --release
```

Open `http://localhost:8080/` and create a session with a Tripmine round and an Order round.

## 1. Tripmine has no hearts

1. Start the Tripmine round and look at the team bar.
   - Expected: no heart indicator for any team.
2. Pick items (safe, target, and mine).
   - Expected: no heart changes anywhere.
3. Start the Order round.
   - Expected: hearts are shown and lost on mistakes as before.

Rule: [contracts/tripmine-mode.md](./contracts/tripmine-mode.md) §2.

## 2. Round end plays before game end

1. Make the Tripmine round the last round and hit the mine.
   - Expected: the explosion cue plays, then the game-end cue.
2. In an Order last round, place the final item correctly.
   - Expected: the placement cue, then the game-end cue.
3. End a non-final round.
   - Expected: one cue only.

Rule: [contracts/tripmine-mode.md](./contracts/tripmine-mode.md) §3.

## 3. Random (Hard) Tripmine

1. In setup, open the Tripmine side control.
   - Expected: Lowest, Highest, and Random (labelled "Hard").
2. Choose Random and start the round.
   - Expected: the mine can be any item (not forced to the extreme); the "searching for" target is a
     random remaining item that is never the mine.
3. Pick the target, a safe item, and finally the mine.
   - Expected: the same scoring as the other sides; the mine ends the round.

Rule: [contracts/tripmine-mode.md](./contracts/tripmine-mode.md) §1.

## Automated checks

```powershell
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run lint
npm --prefix frontend test
npm --prefix frontend run build
```

### Independent (no-server) checks

- `sample_tripmine_grid` with `Random`: exactly one mine, its rank is within 1..=10 (and can vary), and
  the grid still ranks ascending.
- `next_searched_rank` with `Random`: the target is unrevealed and not the mine; with only the mine and
  one item left it is that item.
- `deriveSessionCues`: mine on the last round → `['explosion', 'bonus']`; a non-finishing round end →
  one cue; a non-mine game end → event cue then `bonus`.
- Component test: the setup offers the Random option; the Tripmine board shows no hearts.
