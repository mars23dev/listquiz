# Quickstart: Block Invalid Session Setup

Validation that a session cannot be created from a list that cannot supply the chosen mode's valued items,
and that setup hides or flags impossible choices. See [spec.md](./spec.md),
[contracts/setup-capacity.md](./contracts/setup-capacity.md), and [data-model.md](./data-model.md).

## Prerequisites

- Rust and Node/npm are available; the repository builds locally.
- No new dependency or database migration is required.
- The default folder and its lists are seeded.

## Automated checks (primary)

Backend - capacity rule and session refusal:

```powershell
cargo test --manifest-path backend/Cargo.toml
```

Expected:

- Order with 10 valued items is refused; with 11 it is created.
- Connect with 9 valued items is refused; with 10 it is created.
- Tripmine 10 with 9 valued items is refused; with 10 it is created.
- Tripmine 20 with 19 valued items is refused; with 20 it is created.
- A list with enough total items but a selected column below the minimum is refused.
- A refusal writes no session, teams, or round rows.
- A multi-round request with one short round is refused entirely.
- A quick game still creates a valid session; an impossible quick game is refused before creation.

Frontend - setup wizard:

```powershell
npm --prefix frontend run test
```

Expected:

- A list below the mode minimum does not appear in the list picker.
- A list supporting Tripmine 10 but not 20 stays visible.
- Selecting Tripmine 20 for that list shows the shortfall and blocks session creation.
- Changing mode, count, or column to make the current list incompatible keeps it selected with an error.
- Correcting the selection clears the error and allows creation.
- No compatible list produces an empty-state explanation and blocks creation.
- Existing valid setup and custom-folder tests still pass.

Type and production checks:

```powershell
npm --prefix frontend run lint
npm --prefix frontend run build
```

## Manual end-to-end check

1. Create a list with 15 items, each carrying a value in the value column.
2. Open the setup wizard, enter a gamemaster password, and continue to step 2.
3. Choose Tripmine for the first round.
4. Confirm the 15-item list is offered, because Tripmine 10 is supported.
5. Select 20 items.
6. Confirm an inline error names the round, Tripmine, the required 20 items, and the 15 available; confirm
   Create session is blocked.
7. Select 10 items and confirm the error clears and Create session works.
8. Return to setup and choose Order with the same 15-item list; confirm the round is valid.
9. Create a second list with 10 items and choose Order; confirm that list is absent from the Order picker.
10. Create a third list with 19 items and choose Tripmine 20; confirm creation is refused and no session
    links appear.
11. Use the quick game entry with each mode; confirm it either starts on a list that fits or is refused
    before a session exists.
12. Edit a list to remove values from the selected column, then attempt session creation; confirm the
    refusal uses the current stored state.

## Scope checks

- No session is created for any short round; no partial teams, rounds, or links exist.
- The shortfall error names the round, mode, required count, and available count.
- Unsupported lists are absent from fresh list choices; an already-selected incompatible list stays visible
  with an error.
- No list title or value beyond the existing summary counts is exposed.
- No schema change, no new dependency, and build output stays inside the repository.
