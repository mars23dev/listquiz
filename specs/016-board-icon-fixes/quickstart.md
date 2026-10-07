# Quickstart: Board Icon And Layout Fixes

Validation for the three fixes. See [spec.md](./spec.md) and
[contracts/board-fixes.md](./contracts/board-fixes.md).

## Prerequisites

- Rust and Node/npm available; the frontend and backend build.

## Automated checks (primary)

Backend — icons carried into the views:

```powershell
cargo test --manifest-path backend/Cargo.toml
```

Expected: the Order and Tripmine views carry each item's `iconName` (a Country item reports
`flag:<cc>`), while Connect already does.

Frontend — bar and rows:

```powershell
npm --prefix frontend run test
```

Expected: a twenty-item Tripmine board renders a twenty-position bar (and ten for ten); a Connect board's
name row and value row carry the same fixed height.

Full regression:

```powershell
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run test
npm --prefix frontend run build
```

## Manual end-to-end check

1. Start an Order round, a Tripmine round, and a Connect round on the **Countries** list; confirm every
   item shows its flag in all three.
2. Repeat on another shipped list; confirm each item shows its own icon (not a generic placeholder).
3. Start a **20-item** Tripmine round; confirm the rank bar shows twenty positions and the mine marker is
   within them; repeat with 10 to confirm ten.
4. Open a Connect round; confirm the left and right rows are the same height and line up row-for-row.

## Out of scope

No new dependency, no schema change; the Countries flags stay as they are.
