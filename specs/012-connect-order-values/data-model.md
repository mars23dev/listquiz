# Data Model: Connect Value Order And GM Answer Key

No new persisted data. The change is a board ordering rule (backend) and a derived GM view (frontend).

## Connect board (existing, in `connect_state`)

| Field | Type | Meaning |
|---|---|---|
| `names` | list of `{ itemId, name, iconName }` | The left column; **shuffled** each round. |
| `values` | list of `{ itemId, display }` | The right column; **kept in list order**. |
| `matched` | list of item ids | Pairs already matched. |
| `pending` | `{ nameItemId, valueItemId }` | The staged pair in the middle box. |

**Pairing rule**: a name and its value share the same `itemId`. This is what links a row to its
partner and is the basis of the answer key. No explicit pair table is stored.

**Ordering rule**: `values` is emitted in the list's item order (ascending source position) for the ten
drawn pairs; `names` is emitted in a random order. The two orders are independent (FR-006).

## GM answer key (derived, gamemaster view only)

| Field | Source | Meaning |
|---|---|---|
| Name item id | `names[i].itemId` | The name row. |
| Matching value | `values.find(v => v.itemId === names[i].itemId)?.display` | The correct value shown to the gamemaster. |
| Shown? | role is gamemaster AND pair not in `matched` | Visibility and lifetime. |

The key is computed per render; nothing is stored. For players and the spectator it is never rendered
(FR-010).

## State transitions

| From | Event | Effect on value order | Effect on key |
|---|---|---|---|
| Round created | `sample_connect_round` | Values fixed in list order; names shuffled | Key derivable immediately |
| Round active | reload / reconnect | Values unchanged (stored board reloaded) | Key unchanged |
| Pair matched | confirm (correct) | Value removed as today; remaining order preserved | That pair's entry disappears |
| Round over | all matched or no hearts | Value order unchanged | No entries remain (all matched) |

## Validation rules

- Values MUST be in list order and MUST NOT be shuffled (FR-001, FR-003, FR-004).
- Names MUST be shuffled (FR-002).
- Value order MUST survive reload/reconnect (FR-007).
- Removing a matched value MUST NOT reorder the remaining values (FR-008).
- The answer key MUST render only for the gamemaster and only for unmatched pairs (FR-009, FR-010, FR-012).
- The answer key and value order MUST NOT change scoring, hearts, matching, or turn passing (FR-005, FR-011).
