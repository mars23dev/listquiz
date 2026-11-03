# Contract: Per-Game Ordering

## 1. No list direction

| Guarantee | Rule |
|---|---|
| **G1** | List creation, editing, and CSV import expose no higher/lower choice. |
| **G2** | No list stores an order direction. |
| **G3** | An existing database with the old field upgrades without error. |

## 2. Per-game ordering

| Game mode | Order |
|---|---|
| Order | Ascending (left = lowest, right = highest) |
| Tripmine | Ascending (rank 1 = lowest … N = highest) |
| Connect | Ascending (value column low → high) |

| Guarantee | Rule |
|---|---|
| **G4** | Each game mode orders its round column by its own rule. |
| **G5** | The Order round no longer depends on any list-level direction. |
| **G6** | Tripmine and Connect ordering is unchanged. |

## Non-goals

- No change to scoring, hearts, the Tripmine side, or matching.
- No new dependency.
