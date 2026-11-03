# Quickstart: Reveal Values In Tripmine And Connect

Validation that the reveal works in all three modes. See [spec.md](./spec.md) and
[contracts/reveal-modes.md](./contracts/reveal-modes.md).

## Prerequisites

- Rust and Node/npm available; the frontend and backend build.

## Automated checks (primary)

Backend — Tripmine redaction:

```powershell
cargo test --manifest-path backend/Cargo.toml
```

Expected: with `reveal_values` set, a player's Tripmine items report revealed values, ranks, and the mine.

Frontend — Connect pairing:

```powershell
npm --prefix frontend run test
```

Expected: a player sees the Connect answer key when the reveal flag is set, and does not see it otherwise.

Full regression:

```powershell
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run test
npm --prefix frontend run build
```

## Manual end-to-end check

1. Tripmine round: press "Reveal values"; confirm every item's value, rank, and the mine show for every role.
2. Connect round: press "Reveal values"; confirm the name↔value pairing shows for every role.
3. Order round: confirm the reveal still shows the bar values.
4. Advance to the next round and confirm the reveal is cleared (feature 020).

## Out of scope

No new dependency, no schema change; scoring, hearts, the turn, and the timer are unchanged.
