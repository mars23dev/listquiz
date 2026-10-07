# Contract: Session Feedback And Order Reveal

Extends the realtime contract in `specs/001-listquiz-game/contracts/realtime.md`. The stream still
carries only a revision; these are additions to the per-role `GET /state` payload and the rendering
rules the frontend must follow.

## 1. Session state additions

Every role receives the same bar and the same acting-team associations (all are public in a running
round). Field names are camelCase in JSON.

### `order.left[]`, `order.right[]`, `order.pool[]` → `OrderItem`

```jsonc
{
  "itemId": "item_a",
  "name": "Alpha",
  "value": "12.4",        // null for players until revealValues (unchanged)
  "revealed": false,      // unchanged
  "placedBy": "team_2"    // NEW: team that placed the item, or null (pool / not yet placed)
}
```

- `placedBy` is non-null exactly for items currently on a side (`left` or `right`).
- `placedBy` is `null` for pool items.

### `tripmine.items[]` → `TripmineItem`

```jsonc
{
  "itemId": "item_a",
  "name": "Alpha",
  "value": "12.4",        // null for players until revealed (unchanged)
  "revealed": true,       // unchanged
  "trueRank": 3,          // null for players until revealed (unchanged)
  "isTripmine": false,    // unchanged
  "pointsAwarded": 5,     // unchanged
  "pickedBy": "team_1"    // NEW: team that picked it, or null while hidden
}
```

- `pickedBy` is non-null exactly when `revealed` is true (the room saw the pick).
- Hidden items carry `pickedBy: null`.

### Reveal flag

`revealValues` is unchanged. When it is `true` for an Order round, the server sends the bar in its
**correct arrangement** (see §3): `order.left` and `order.right` hold every non-anchor item and
`order.pool` is empty. When it is `false`, the bar is the placement order (unchanged).

## 2. Cue mapping (frontend contract)

Cues are driven by the reflected event only; there is no local-interaction cue.

- Emit at most one cue per state update, and only when `prev !== null` **and**
  `next.revision === prev.revision + 1`.
- Map the transition with this table (first match wins):

| Transition | Cue |
|---|---|
| `order.pending` appears or changes | `slide` |
| `left.length + right.length` increases | `slide` (item moved) then, if a score rose, `chime`; a correct placement plays `chime` |
| some team's `hearts` decreased | `failure` |
| a Tripmine item `revealed` false→true, `isTripmine` true | `explosion` |
| a Tripmine item `revealed` false→true, `isTripmine` false | `chime` |
| `tripmine.completed` false→true | `chime` |
| `order.pool.length` increases while a side shrinks | `remove` |
| `status` becomes `finished` | `bonus` |

- No cue plays for: initial load (`prev === null`), reconnect, heartbeat refetch, revision jumps, or
  a `muted`/`revealValues`/turn-only change.
- `playSlides` etc. respect the session mute flag; a muted session plays nothing.

## 3. Reveal rendering rule (Order)

- When `revealValues` is true, the bar renders the server's arranged `left`/`right` with values shown
  and **neutral borders** (no team color, no higher/lower color). The Lowest/Highest side tints go
  neutral.
- Reveal is presentational on the server: it does not change any score, heart, turn, or the stored
  board. A reveal MUST NOT award points (FR-012).
- Tripmine reveal is unchanged: values appear, positions do not move.

## 4. Border color rule

- A placed (Order) or revealed (Tripmine) item's border takes the color of the team recorded in its
  `placedBy` / `pickedBy` (`primaryHex`), consistent with that team's scoreboard color.
- The pending Order preview is framed with the **active team's** color.
- The border no longer encodes higher/lower or mine/safe; those remain on the existing badges, icons,
  point text, and feedback messages.
- When an Order round is revealed, the neutral rule of §3 overrides the team color.
