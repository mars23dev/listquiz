# Contract: Connect Board Value Order And GM Answer Key

Two UI contracts for the Connect board. Both are pure functions of the board the client already holds.

## 1. Value column order

**Inputs**: the drawn `names` and `values` for the round.

- `names` — the left column, shuffled by the backend.
- `values` — the right column, in the list's item order.

| Condition | Result |
|---|---|
| Round created | `values` is in the list's item order; `names` is random |
| Round reloaded / reconnected | The stored order is shown unchanged |
| A value is matched | It is removed; the remaining values keep their relative order |

**Guarantees**:
- **G1**: The value column is never a random shuffle and is never re-sorted (no numeric/alphabetical ordering).
- **G2**: The name column is random each round and is independent of the value order.
- **G3**: Ordering does not change scoring, hearts, matching, or turn passing.

## 2. GM answer key visibility

**Inputs**: the board, the viewer's role, and the `matched` set.

| Viewer | Pair unmatched | Pair matched |
|---|---|---|
| Gamemaster | Name row shows the correct matching value | Name row hidden (matched) |
| Player | No pairing shown | Name row hidden (matched) |
| Spectator | No pairing shown | Name row hidden (matched) |

**Guarantees**:
- **G4**: Only the gamemaster sees the name→value pairing; players and the spectator never see it.
- **G5**: The key is present for the whole round while a pair is unmatched and disappears with the pair when matched.
- **G6**: The key is correct after reload/reconnect.
- **G7**: The key changes no game rule (scoring, hearts, matching, turns) and no column order.

## Non-goals

- No backend role filtering or answer-key field.
- No schema, dependency, or scoring change.
- No change to the middle box, confirm flow, or heart/score rules.
