# Data Model: Fix Order And Setup

No table, column, or stored field changes. This documents the two concepts the fixes rely on.

## Order bar (existing)

The bar is the anchor plus a left side and a right side. Each side stores its items in placement
order and keeps them **contiguous** — the items are packed next to each other, not separated by slots.

| Concept | Rule |
|---|---|
| Side length | The number of items currently on that side (`0` to `5`). |
| Gap | An insertion point: before the first item, between two adjacent items, or after the last item. A side with `n` items has `n + 1` gaps, numbered `0..=n`. |
| Empty side | Has exactly one gap, `0`. |
| Full side | Has no new gap (the side already holds the maximum). |

A drop chooses a gap; the client sends the gap number as the item's insertion position. After a
correct placement the side is still contiguous.

## Neighbour judgement (existing, unchanged)

An item placed at a gap is judged from its neighbours:

| Where | Correct when |
|---|---|
| Left of the anchor | lower than the item on its right |
| Right of the anchor | higher than the item on its left |
| Between two items | higher than its left neighbour and lower than its right neighbour |

Ties are allowed. A correct placement scores the existing points; an incorrect one costs the existing
heart and leaves the item in the pool.

## Round configuration (existing)

| Field | Rule |
|---|---|
| `quizType` | `ORDER` or `TRIPMINE`. |
| `tripmineSide` | Required for a `TRIPMINE` round, absent for an `ORDER` round. In Setup, its control is shown only while the round type is Tripmine. |

## Client state (frontend)

| State | Meaning |
|---|---|
| chosen side + gap | The gap the player dropped on for the pending item. |
| chosen position | The gap mapped to the side's insertion position (`0..=side length`), which is what is sent on confirm. |
