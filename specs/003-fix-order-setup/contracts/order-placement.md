# Contract Amendment: Order Placement Gap and Setup Round Control

Amends `specs/001-listquiz-game/contracts/rest-api.md` (the `choose_slot` action) and the Setup
round-configuration UI.

## `choose_slot` position is a gap

An item is dropped into a **gap** on the chosen side: between two items already on that side, or
beside either end. The side's items stay contiguous, so the side with `n` items has exactly `n + 1`
gaps. The `slotIndex` a client sends is the gap number, in `0..=side length`.

| Side state | Gaps (valid `slotIndex`) |
|---|---|
| Empty | `0` |
| Holds _n_ items | `0`…`n` |
| Full (5 items) | none; the side accepts no placement |

A client that renders a fixed set of columns MUST translate the column the player drops on to the
corresponding gap before sending; it MUST NOT send a column index the side does not have. The server
keeps judging the placement from the item's neighbours and rejects a gap the side does not have with
`not_ready`/`bad_request`; the client surfaces that as "choose a gap again" rather than the raw range
text.

Example (left side, empty): a drop on the left side sends
`{ "action": "choose_slot", "side": "left", "slotIndex": 0, "teamId": "team_1" }`.

Example (left side already holding two items): the three gaps are `0` (beside the outer end), `1`
(between the two items), and `2` (beside the anchor).

## Setup round configuration

| Round type | Tripmine side control | `tripmineSide` sent |
|---|---|---|
| `TRIPMINE` | shown | the chosen value |
| `ORDER` | not shown | omitted |

The create-session contract is unchanged: an `ORDER` round with a `tripmineSide` is still refused.

## Unchanged

Neighbour-based correctness, scoring, heart cost, the `choose_slot` response shape (`{ "revision" }`),
and every other endpoint are unchanged.
