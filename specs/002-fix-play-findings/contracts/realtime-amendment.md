# Contract Amendment: Realtime State for Tripmine and Player Controls

Amends `specs/001-listquiz-game/contracts/realtime.md`.

## `GET /api/sessions/{sessionId}/state?role=<role>`

### Tripmine view

Two changes, both visible to every role:

```json
"tripmine": {
  "side": "highest",
  "searchedRank": 1,
  "hit": false,
  "completed": false,
  "targetValue": "$2.92 Billion",
  "items": [ { "itemId": "itm-5", "name": "Titanic", "revealed": false, "isTripmine": false, "pointsAwarded": null } ]
}
```

| Field | Gamemaster | Player / spectator |
|---|---|---|
| `targetValue` | present | present — the intended public clue |
| `tripmine.items[]` order | shuffled display order | shuffled display order (identical to the host's) |
| `tripmine.items[].value` | always present | still `null` until the item is picked or values are revealed |
| `tripmine.items[].isTripmine` | always present | still only once that item is revealed |

`targetValue` equals the mined item's value for the round's column; it is the one value shown for an
unpicked item, on purpose (finding 6).

### Player and spectator controls

The previous rule "player and spectator roles never receive host controls (FR-062)" is narrowed:

- A player role whose team is on turn **may** post in-round actions (see
  `contracts/actions-amendment.md`) and its board is interactive.
- A player role not on turn, and every spectator/stage view, remains read-only with no acting
  controls.
- The gamemaster keeps all host controls.

The state payload itself is unchanged for controls (controls are a client concept); only the
redaction table's `targetValue` row is added.

## Redaction rules, enumerated (delta)

| Field | Gamemaster | Player / spectator |
|---|---|---|
| `tripmine.targetValue` | Present | Present |
| all previously enumerated fields | unchanged | unchanged |
