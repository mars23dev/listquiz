# Quickstart: Round UI And List Fixes

Validation for the four fixes. See [spec.md](./spec.md) and
[contracts/round-fixes.md](./contracts/round-fixes.md).

## Prerequisites

- Rust and Node/npm available; the frontend and backend build.

## Automated checks (primary)

Backend — ordering, formatting, Tripmine slots:

```powershell
cargo test --manifest-path backend/Cargo.toml
```

Expected: `format_value` matches the contract examples; every shipped list is non-decreasing by its
ordering column; a Tripmine round configured with 20 samples/ranks 20 items with the mine side applied,
and 10 still works; the additive migration adds `tripmine_slots` without breaking existing rows.

Frontend — components:

```powershell
npm --prefix frontend run test
```

Expected: `ConnectBoard` name and value rows are equal height; `TripmineBoard` renders twenty items; the
`Session` header view switch is absent for `p1`/`view` and present for the gamemaster, and activating it
opens a new tab without navigating the current page.

Full regression:

```powershell
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run test
npm --prefix frontend run build
```

## Manual end-to-end check

1. Open each shipped list and confirm the values read in ascending order with large numbers shown as
   "… Million"/"… Billion".
2. Open a Connect round and confirm the left and right columns look equally full (equal row size).
3. Configure a Tripmine round with 20 items, start it, and confirm all 20 fit on one screen with no
   scrolling; repeat with 10 to confirm no change.
4. As a player and as the spectator, confirm the header has no view switch; as the gamemaster, use the
   switch and confirm the view opens in a new tab while the gamemaster page stays.

## Out of scope

No new dependency; only the additive `tripmine_slots` column.
