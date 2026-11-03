# Quickstart: Verify the Play Findings

Validation guide. It proves each finding is fixed end to end. It contains no implementation code.

## Prerequisites

Same as the main quickstart: Rust 1.99+, Node.js 24+, npm 11+. Build and run:

```bash
npm --prefix frontend run build
cargo run --manifest-path backend/Cargo.toml --release
```

Open `http://localhost:8080/`. Two browser contexts (or two devices) are needed for findings 4 and 5.

## Automated checks

```bash
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run lint
npm --prefix frontend test
```

## Finding 4 — start affordance and waiting state

1. Create a session in `/setup` (any 2-team, 1-round setup).
2. Open the gamemaster link in context A and unlock.
   - Expected: a large start control is shown in the middle of the board area, not an empty page.
3. Open a player link in context B and the spectator link in a third tab.
   - Expected: both show a waiting-for-start message in the middle area.
4. Start the round from context A.
   - Expected: within one second the start control and the waiting messages are replaced by the live
     board in every open view.

## Finding 5 — the team on turn plays from its own view

1. In an Order round where team 1 is on turn, use the team 1 player link in context B.
   - Expected: the pool items and bar slots are interactive for team 1; a placement can be selected,
     a slot chosen, and confirmed; the score and turn update in every view.
2. Use the team 2 player link.
   - Expected: the board is read-only; no acting controls; no action is sent.
3. Use the spectator link.
   - Expected: read-only, no controls, no turn emphasis.
4. Repeat a Tripmine pick from the active team's player link.
   - Expected: the pick is scored and revealed to every view.
5. As gamemaster, act for the active team.
   - Expected: host behaviour unchanged.

## Finding 3 — Tripmine grid order

1. Play the same list as a Tripmine round twice (restart or a second session).
   - Expected: the on-screen order of the ten items differs between the two runs.
2. Inspect a single grid.
   - Expected: the order does not follow the value ranking from one end to the other.
3. Pick cells.
   - Expected: the revealed position, the searched position, and every score behave exactly as before.

## Finding 6 — Tripmine target value in the hint

1. Open a Tripmine round.
   - Expected: the hint reads that the tripmine is the highest (or lowest) **and** shows the mined
     item's value next to that text.
2. Look at an unpicked cell.
   - Expected: its value stays hidden.
3. Continue until the tripmine is picked.
   - Expected: the value shown in the hint matches the picked tripmine's value.

## Finding 2 — team name typing in Setup

1. Open `/setup`, type a multi-character name such as `Team Cyan Pulsar` into a team field.
   - Expected: the full name is entered; focus is not lost after the first character.

## Finding 1 — list editor icons and table

1. Open `/lists`, unlock a non-default folder, create or edit a list.
   - Expected: each row shows an icon control (a glyph or an empty icon affordance), never the literal
     word "Icon".
2. Narrow the window.
   - Expected: name and value fields sit under their headings with even spacing and no oversized empty
     column.

## Regression guard

- Confirm `cargo test` and `npm --prefix frontend test` pass, including the existing Tripmine, Order,
  and role-view tests, which must be unchanged in outcome.
