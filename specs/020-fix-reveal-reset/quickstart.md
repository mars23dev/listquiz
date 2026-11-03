# Quickstart: Fix Reveal Reset On Next Round

Validation that advancing a round clears the reveal. See [spec.md](./spec.md) and
[contracts/reveal-reset.md](./contracts/reveal-reset.md).

## Prerequisites

- Rust available; the backend builds. (No frontend change.)

## Automated checks (primary)

```powershell
cargo test --manifest-path backend/Cargo.toml
```

Expected: revealing then advancing clears the reveal flag; finishing at the last round keeps it; other
suites stay green.

Full regression:

```powershell
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run test
npm --prefix frontend run build
```

## Manual end-to-end check

1. As the gamemaster in a multi-round game, press "Reveal values"; the control reads "Values revealed".
2. Press "Next Round"; confirm the control reads "Reveal values" and the new round's values are hidden for
   every role.
3. Play to the last round, reveal, and finish; confirm the game-end view still shows the values.

## Out of scope

No new dependency, no schema change; the frontend is unchanged.
