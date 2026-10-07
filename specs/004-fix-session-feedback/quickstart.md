# Quickstart: Session Feedback And Order Reveal

A run-and-verify guide for the three fixes. It links the rules to
[contracts/session-feedback.md](./contracts/session-feedback.md) and
[data-model.md](./data-model.md) rather than repeating them.

## Prerequisites

- Rust toolchain (backend) and Node (frontend), as for the other features.
- Build the exported frontend and run the backend, which serves it (Principle IV):

```powershell
npm --prefix frontend run build
cargo run --manifest-path backend/Cargo.toml --release
```

- Open `http://localhost:8080/`, create a session, and open the gamemaster link plus one player link
  (a second browser tab or profile so both stay connected).

## 1. Game sounds actually play

1. In an unmuted running session, in the player tab, select an item from the pool.
   - Expected: a slide cue plays in **both** tabs (the actor hears it once, from the reflected
     event, not twice).
2. Confirm a **correct** placement.
   - Expected: the success chime plays in both tabs.
3. Make a **wrong** placement.
   - Expected: the failure cue plays in both tabs.
4. In a Tripmine round, pick the mined item, then a safe item.
   - Expected: the explosion cue on the mine; the safe chime on the safe pick.
5. Click **Mute**, then trigger any event; click **Unmute** and trigger another.
   - Expected: nothing plays while muted; cues resume after unmuting.
6. Confirm the browser shows its tab audio indicator while a cue plays.
7. Disconnect the player (offline) while the gamemaster acts, then reconnect.
   - Expected: no replayed cue on reconnect.

Mapping and the consecutive-revision guard: [contracts/session-feedback.md](./contracts/session-feedback.md) §2.

## 2. Item border shows the choosing team's color

1. Order round, two teams. Confirm a correct placement for each team.
   - Expected: each placed item's border matches its team's scoreboard color (its `primaryHex`).
2. Select an item without placing it.
   - Expected: the preview border matches the **active** team's color.
3. Tripmine round: have each team pick and reveal an item.
   - Expected: each revealed item's border matches the picking team's color; the mine/safe signal
     stays visible through the badge, icon, and point text (not the border).
4. Reload the page mid-round.
   - Expected: borders persist (the placing team is stored with the board).

Rule: [contracts/session-feedback.md](./contracts/session-feedback.md) §4.

## 3. Order reveal arranges the whole bar

1. Order round: place some items correctly and leave others in the pool.
2. Note every team's score and hearts. Click **Reveal values**.
   - Expected: every item (placed and pooled) sits in its correct ordered position around the anchor,
     each showing its value, every border neutral. Lap the teams' scores/hearts: **nothing changed**.
3. Confirm victory/round-end conditions are the only things that score, not the reveal.
4. Tripmine round: reveal values; positions do not move (values only).

Arrangement and the no-scoring guarantee:
[data-model.md](./data-model.md) (revealed Order layout) and
[contracts/session-feedback.md](./contracts/session-feedback.md) §3.

## Automated checks

```powershell
# Backend: arrangement rule, placed_by association, view fields
cargo test --manifest-path backend/Cargo.toml

# Frontend: cue derivation, audio unlock, team-colored borders, neutral reveal
npm --prefix frontend run lint
npm --prefix frontend test

# End-to-end (backend must already be serving the built frontend)
npm --prefix frontend run build
cargo run --manifest-path backend/Cargo.toml --release   # in one terminal
npx playwright test                                       # in another
```

### Independent (no-server) checks

- `deriveSessionCue(prev, next)` unit tests feed hand-built `SessionState` pairs: selection, correct,
  wrong, mine, safe, round complete, return, finished, first-load, revision jump, and muted.
- `arrange_revealed_order` unit tests feed a legal board and assert the left/right split and
  ascending order, plus the tie and five-per-side cap cases.
- Component tests assert the team border color and the neutral reveal without a backend.
