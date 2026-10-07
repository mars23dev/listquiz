# Data Model: Fix Play Findings

This feature adds no table and no column. It changes one stored sequence and one view field.

## Stored changes

### `tripmine_state.items` (existing JSON column)

Unchanged shape; the **array order** now means "display order".

| Field | Change |
|---|---|
| `items` | The array is shuffled once when the round starts. Each element keeps `item`, `true_rank`, `is_tripmine`, `revealed`, `revealed_by`, `points_awarded` exactly as before. |

Rules:
- The shuffled order is written once at round start and never changes mid-round (a reload or a second
  view sees the same order).
- No rule reads a slot by its array index; scoring, the searched position, and the answer key all use
  `item_id` and `true_rank`, so the shuffle cannot affect an outcome.

## View changes

### `TripmineView` (state served to every role)

| Field | Type | Rules |
|---|---|---|
| `side` | text | unchanged |
| `searchedRank` | number | unchanged |
| `hit` / `completed` | boolean | unchanged |
| `items[]` | array | now in shuffled display order; per-cell reveal rules unchanged |
| `targetValue` | text \| null | **new.** The mined item's value for the round's column, present for every role. Null only if the round has no resolvable value. |

`targetValue` is derived from the stored mined slot's value, so it always equals the value that is
shown once the mined cell is picked (SC-007). It is the only value exposed for an unpicked item.

## Action authorization (no storage change)

The action channel now evaluates a requester identity:

| Requester | Rule |
|---|---|
| `gamemaster` | `X-Quiz-Password` must match, as today. May act for the active team and may `return_to_pool`. |
| `p1`…`p4` that owns the team on turn | May post the in-round actions for that team. No password. |
| any other player role, or a player naming a team that is not on turn | Refused with `not_ready`; no board change. |
| spectator / stage (`view`) | Never accepted. |

## Client state (frontend, no persistence)

| State | Meaning |
|---|---|
| board present vs absent | Drives the start control (host) and the waiting message (players/spectators). |
| role owns the team on turn | Makes the player board interactive for that role only; all other player views stay read-only. |
