# Quickstart: Quick Game Connect And Random Lists

Validation for the Connect quick game and the random default list. See [spec.md](./spec.md) and
[contracts/quick-game.md](./contracts/quick-game.md).

## Prerequisites

- Rust and Node/npm available; the frontend and backend build.

## Automated checks (primary)

Backend — random eligible list:

```powershell
cargo test --manifest-path backend/Cargo.toml quick_game
```

Expected: with a seeded rng the chosen list varies across seeds (when several lists fit); with a single
eligible list that list is always chosen; with no eligible list the request is refused.

Frontend — Connect offered:

```powershell
npm --prefix frontend run test -- App
```

Expected: the landing page shows Order, Tripmine, and Connect; starting a quick game still works.

Full regression:

```powershell
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run test
npm --prefix frontend run build
```

## Manual end-to-end check

1. From the landing page, start a Connect quick game; confirm a Connect round starts immediately (one
   team, player view, no host controls).
2. Start the same quick game type several times; confirm the list/round content varies (when more than one
   list fits).
3. Confirm Order and Tripmine quick games still work.

## Out of scope

No new dependency, no schema change; the rest of the quick game behavior is unchanged.
