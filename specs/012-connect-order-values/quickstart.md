# Quickstart: Connect Value Order And GM Answer Key

Validation for both changes. See [spec.md](./spec.md) for requirements and
[contracts/connect-board.md](./contracts/connect-board.md) for the exact rules.

## Prerequisites

- Rust and Node/npm available; the frontend and backend build.
- A Connect session configured, or a Connect quick game startable.

## Automated checks (primary)

Backend — value order and shuffled names:

```powershell
cargo test --manifest-path backend/Cargo.toml connect_round
```

Expected: `sample_connect_round` returns ten values in the list's item order and ten names in a
different (shuffled) order; existing Connect rule tests stay green.

Frontend — GM key visibility:

```powershell
npm --prefix frontend run test -- ConnectBoard Session
```

Expected:
- Rendering `ConnectBoard` with the GM flag shows each unmatched name's correct value.
- Rendering it without the flag (player/spectator) shows no pairing.
- `Session` passes the gamemaster flag (GM sees the key, `p1` and `view` do not).

Full regression:

```powershell
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run test
npm --prefix frontend run build
```

## Manual end-to-end check

1. Start a Connect round (GM view plus a player view and a spectator view).
2. Confirm the **value column** reads in the list's item order (not shuffled) and the **name column** is in a random order.
3. In the GM view, confirm each name row shows its **correct value** (answer key).
4. In the player and spectator views, confirm **no** pairing is shown.
5. Match a pair correctly; confirm the pair's key entry disappears in the GM view and the remaining order is unchanged.
6. Reload the GM view; confirm the key and the value order are unchanged.

## Out of scope

No schema change, no new dependency; the answer key is derived from the existing board payload.
