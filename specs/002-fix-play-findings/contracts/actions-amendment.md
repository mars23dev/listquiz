# Contract Amendment: Session Actions

Amends `specs/001-listquiz-game/contracts/rest-api.md` (the `POST /api/sessions/{sessionId}/actions`
section) and `contracts/realtime.md`'s line "every mutation comes from the gamemaster view".

## `POST /api/sessions/{sessionId}/actions`

The request body gains an optional `role`.

### Request

```json
{ "action": "select_item", "role": "p2", "itemId": "itm-9ab2", "teamId": "team_2" }
```

| Field | Gamemaster | Player |
|---|---|---|
| `role` | omitted or `"gamemaster"` | `"p1"` … `"p4"` |
| `X-Quiz-Password` header | required | not sent |
| `teamId` | the active team | the player's own team |

### Authorization

| Caller | Result |
|---|---|
| gamemaster with the right password | accepted; may act for the active team and may `return_to_pool` |
| player role that owns the team on turn | accepted for `select_item`, `choose_slot`, `cancel_selection`, `pick_tripitem` |
| player role that does not own the team on turn | `not_ready`, no board change |
| player role naming another team | `not_ready`, no board change |
| player role calling `return_to_pool`, `next_round`, or any host action | refused |
| spectator/stage | no channel access |

Errors keep the existing shape: `bad_request`, `unauthorized` (gamemaster password), `not_found`,
`conflict`, `not_ready`.

### Response

Unchanged: `200 { "revision": 42 }`. Every accepted action bumps the revision, so an open player view
re-reads and converges (contracts/realtime.md).

## Unchanged

All other endpoints, error codes, and the SSE stream are unchanged. Player writes do not add a new
endpoint and do not add a password or token concept.
