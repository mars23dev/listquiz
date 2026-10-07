# Quickstart: Expand Default Lists And Add Countries

Validation for the expanded lists, the Countries list, and flag rendering. See [spec.md](./spec.md) and
[contracts/shipped-lists.md](./contracts/shipped-lists.md).

## Prerequisites

- Rust and Node/npm available; the frontend and backend build.
- `flag-icons` installed (`npm --prefix frontend install`).

## Automated checks (primary)

Backend — shipped-list rules:

```powershell
cargo test --manifest-path backend/Cargo.toml default_lists
```

Expected: four shipped lists; the three expanded lists are 20–50 items with unique ids and finite
values; the Countries list has 195 items, each `icon_name` starting with `flag:` and carrying
area/population/founded; `validate_list_size` still accepts 100 and rejects 101.

Frontend — flag rendering:

```powershell
npm --prefix frontend run test -- ItemIcon
```

Expected: `ItemIcon` with `flag:de` renders the German flag (not `HelpCircle`); a lucide name still
renders that icon; an unknown name still falls back to `HelpCircle`.

Full regression:

```powershell
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run test
npm --prefix frontend run build
```

## Manual end-to-end check

1. Delete `backend/data/listquiz.db` (or start fresh) and run the backend; open the app.
2. In setup, confirm the shipped lists: World History, Iconic Statues, Blockbuster Cinema (each well over
   11 items) and a Countries list.
3. Open the Countries list and confirm each entry shows a **flag**, and has **Area (km²)**, **Population**,
   and **Founded** values.
4. Start an Order round from Countries and confirm flags render on the board; repeat quickly for a
   Tripmine or Connect round.
5. Restart the backend against the **same** database and confirm the shipped list item counts are still
   correct (no stale counts).

## Out of scope

No schema change, no backend dependency; flags are bundled (no runtime network).
