# Quickstart: Per-Game Ordering

Validation that the list direction is gone and each game mode orders. See [spec.md](./spec.md) and
[contracts/ordering.md](./contracts/ordering.md).

## Prerequisites

- Rust and Node/npm available; the frontend and backend build.

## Automated checks (primary)

Backend:

```powershell
cargo test --manifest-path backend/Cargo.toml
```

Expected: lists can be created and CSV-imported without a direction; `QuizType::sort_order()` is
ascending for Order, Tripmine, and Connect; the Order round orders ascending; the migration drops the
column without error.

Frontend:

```powershell
npm --prefix frontend run test
```

Expected: the list editor offers no direction selector; the create/import payloads carry no
`orderDirection`.

Full regression:

```powershell
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run test
npm --prefix frontend run build
```

## Manual end-to-end check

1. Create a list and import a CSV; confirm no higher/lower choice anywhere.
2. Run an Order round; confirm left is the lowest and right is the highest (ascending).
3. Run a Tripmine round; confirm ranks ascend (1 lowest … N highest).
4. Run a Connect round; confirm the value column reads ascending.
5. Start against an existing database; confirm it opens without error and rounds behave the same.

## Out of scope

No new dependency; scoring, hearts, the Tripmine side, and matching are unchanged.
