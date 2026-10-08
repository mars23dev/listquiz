# Quickstart: Value Column Labels

Validation that the round's column name is shown wherever a value is shown. See [spec.md](./spec.md) and
[contracts/value-labels.md](./contracts/value-labels.md).

## Prerequisites

- Node/npm available; the frontend builds. (No backend change.)

## Automated checks (primary)

```powershell
npm --prefix frontend run test
```

Expected: `ConnectBoard` shows the column name as the value-column heading; `TopBarOrdering` shows it with
the Order values; `TripmineBoard` shows it with the values; and a board does not show the label for a
hidden value.

Full regression:

```powershell
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run test
npm --prefix frontend run build
```

## Manual end-to-end check

1. Start an Order round on Countries / Area. Confirm the anchor and any revealed values show "Area (km²)".
1b. Reveal the Order values and confirm each item's value and column name appear in the middle of its card (like the anchor), not in the footer.
2. As a player before reveal, confirm hidden values show neither the number nor the label.
3. Start a Tripmine round; confirm revealed values (and the gamemaster's view) show the column name.
4. Start a Connect round on Countries / Area; confirm the value column is headed "Area (km²)".
5. Repeat with a different column (e.g. Population) and confirm the label matches.

## Out of scope

No new dependency, no backend/schema change; value text is unchanged.
