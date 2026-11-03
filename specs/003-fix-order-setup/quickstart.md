# Quickstart: Verify Fix Order And Setup

Validation guide. It proves both findings are fixed end to end. It contains no implementation code.

## Prerequisites

Same as the main quickstart: Rust 1.99+, Node.js 24+, npm 11+. Build and run:

```bash
npm --prefix frontend run build
cargo run --manifest-path backend/Cargo.toml --release
```

Open `http://localhost:8080/`, create a session whose round 1 is Order, and open the gamemaster link
(and, to prove the player path, the link for the team on turn).

## Automated checks

```bash
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run lint
npm --prefix frontend test
```

## Finding 2 — Order placement works

1. Start the Order round.
2. Select a pool item and drop it into a gap on the correct side (beside an end, or between two items
   once the side has some), then confirm.
   - Expected: the placement is scored (+10 or a lost heart). No `"Slot N is not on the ... side"`
     error appears.
3. After each correct placement, look at the side.
   - Expected: the items are packed next to each other (contiguous), and the next drop offers the gaps
     between them and at the ends.
4. Place a wrong-side item and confirm.
   - Expected: the team loses one heart and the item returns to the pool — an incorrect result, not an
     error.
5. Fill a side to its limit.
   - Expected: that side no longer offers a new gap.
6. Repeat as the active-team player (finding from feature 002).
   - Expected: the same scored outcome as the gamemaster.

## Finding 1 — Tripmine side only for Tripmine rounds

1. Open `/setup`, go to the round configuration.
2. Set a round to Order.
   - Expected: no tripmine side (lowest/highest) control is shown.
3. Switch that round to Tripmine.
   - Expected: the tripmine side control appears.
4. Switch it back to Order.
   - Expected: the control disappears and no tripmine side is sent for that round.

## Regression guard

- Confirm `cargo test` and `npm --prefix frontend test` pass, including the existing Order round,
  scoring, and Setup tests.
