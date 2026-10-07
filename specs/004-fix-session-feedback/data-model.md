# Data Model: Session Feedback And Order Reveal

No new entity. One additive field on the Order board's storage, two additive fields on the session
view DTOs, and one derived, transient value (the cue).

## Stored — `order_state` (SQLite)

| Column | Type | Change | Notes |
|---|---|---|---|
| `session_id` | TEXT | — | primary key |
| `anchor_item_id` | TEXT | — | |
| `anchor_value` | REAL | — | used by the arrangement |
| `pool` | TEXT | — | JSON array of item ids |
| `left_ids` | TEXT | — | JSON array of item ids, placement order |
| `right_ids` | TEXT | — | JSON array of item ids, placement order |
| `pending` | TEXT | — | JSON of the pending placement, or `null` |
| `placed_by` | TEXT | **NEW** | JSON object `{ "<item_id>": "<team_id>" }`; missing/`null` reads as `{}` |

**Validation / rules**

- `placed_by` is written only when a **correct** placement joins a side, using the pending
  placement's `team_id`; a wrong placement never adds an entry (the item stays in the pool).
- An item leaving a side (return to pool) drops its `placed_by` entry.
- The column is added by an idempotent migration (research D-407); an existing database keeps its
  bar and reads an empty map until the next placement.

## Domain — `OrderBoard` (Rust, `order_round`)

| Field | Type | Change | Notes |
|---|---|---|---|
| `anchor` | `ListItemValue` | — | carries `numeric` for arrangement |
| `pool` / `left` / `right` | `Vec<ListItemValue>` | — | |
| `pending` | `Option<PendingPlacement>` | — | already carries `team_id` |
| `placed_by` | `BTreeMap<String, String>` | **NEW** | item id → placing team id; `#[serde(default)]` |

**State transitions**

- `confirm_placement` (correct) → insert item into the side **and** record
  `placed_by[item_id] = pending.team_id`.
- `return_to_pool` → remove the item from the side **and** remove its `placed_by` entry.
- Any load with no stored map → `{}`.

## View DTO — `SessionState` additions (Rust `app_state`, mirrored in `types/session.ts`)

| DTO | Field | Type | Redaction |
|---|---|---|---|
| `OrderItem` | `placedBy` | `string \| null` | public (the room watched the placement) |
| `TripmineItem` | `pickedBy` | `string \| null` | public when the item is revealed; `null` while hidden (read from the existing `revealed_by`) |

The `PendingPlacement.team_id` redaction is unchanged; the pending preview is colored from the
**active team** (`teams[].onTurn`), which is already public, so no field is added there.

## Derived — the cue (frontend, not persisted)

`deriveSessionCue(prev: SessionState | null, next: SessionState): SessionCue | null`

| `SessionCue` | Emitted on |
|---|---|
| `slide` | a pending item appears/changes, or an item joins a side |
| `chime` | score rises with a side growth (correct), a safe Tripmine reveal, or a Tripmine round completes |
| `failure` | a team's hearts fall |
| `explosion` | the Tripmine item is revealed |
| `remove` | a placed item returns to the pool |
| `bonus` | the session becomes `finished` |
| `null` | first load, reconnect, heartbeat, mute toggle, or any non-consecutive revision |

**Rules**

- Emit only when `prev !== null` and `next.revision === prev.revision + 1`.
- At most one cue per transition, chosen by the fixed priority table (research D-402).
- Reads public data only (pending, side lengths, pool length, team scores/hearts, Tripmine revealed
  flags, session status); never a hidden value.
- `setSoundMuted(state.muted)` gates playback; a muted session plays nothing.

## Derived — the revealed Order layout (backend view, not persisted)

`arrange_revealed_order(anchor: &ListItemValue, items: &[ListItemValue]) -> (Vec<ListItemValue>, Vec<ListItemValue>)`

- Left = items with `numeric <= anchor.numeric`, sorted ascending by `(numeric, original index)`.
- Right = items with `numeric > anchor.numeric`, sorted ascending by `(numeric, original index)`.
- Applied only when the session's reveal flag is set; the stored board and scores are untouched.
- With a legal round this yields exactly five per side (the round deals five below and five above the
  anchor); the function still clamps to the five-per-side bar cap defensively.
