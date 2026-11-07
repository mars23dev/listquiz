# Contract: Setup Capacity

This feature extends the existing list-summary and session-creation contracts. It adds no new endpoint and
no schema change. JSON fields are camelCase.

## 1. List summaries gain `valueCount`

`GET /api/default-lists` and `GET /api/folders/{folderId}/lists` keep their envelope and add one field to
every column:

```json
{
  "lists": [
    {
      "id": "default_world_history",
      "title": "World History & Inventions",
      "description": "",
      "itemCount": 12,
      "isDefault": true,
      "columns": [
        { "id": "year", "name": "Year", "valueCount": 12 }
      ]
    }
  ]
}
```

| Field | Type | Rule |
|---|---|---|
| `columns[].valueCount` | integer | Items in the list with a value in that column; `0..=itemCount`. |
| Existing fields | unchanged | No field is removed or renamed. |

The count is derived from `item` joined to `item_value` for that column. A list can have a high `itemCount`
and a low `valueCount` when values are missing.

## 2. Session creation enforces capacity

`POST /api/sessions` and `POST /api/sessions/quick` keep their request shapes. Before writing anything, the
server checks every round:

| Mode | `tripmineSlots` | Required `valueCount` |
|---|---|---|
| `ORDER` | absent | 11 |
| `CONNECT` | absent | 10 |
| `TRIPMINE` | absent or `10` | 10 |
| `TRIPMINE` | `20` | 20 |

### Refusal

| Situation | Status | Code | Result |
|---|---|---|---|
| Any round's selected column has fewer valued items than required | `400` | `bad_request` | No session, teams, round configurations, or links are written. |
| All rounds meet their minimum | `201` | n/a | Session created as today. |
| Existing invalid list/column reference | `404` | `not_found` | Existing behavior, checked before capacity. |
| Existing folder-password refusal | `400` | `unauthorized` | Existing behavior. |

Error body keeps the existing shape:

```json
{
  "error": {
    "code": "bad_request",
    "message": "Round 2: Tripmine with 20 items needs at least 20 items with values in the selected column; the list has 15."
  }
}
```

The message MUST name the round, the game mode, the required count, and the available count. One failing
round refuses the whole request; no earlier round is stored.

## 3. Quick game

`POST /api/sessions/quick` keeps its request and response. It reuses the same creation gate.

| Situation | Behavior |
|---|---|
| A default list fits the mode by valued count | That list may be chosen. |
| No default list fits the mode | Refused before a session exists; existing `not_ready` quick-game error applies. |
| Chosen default list is short at creation time | Refused by the shared capacity gate; no session is written. |

## 4. UI guarantees

| Guarantee | Rule |
|---|---|
| **G1** | The list picker offers a list when any column meets the selected mode's smallest size: Order 11, Connect 10, Tripmine 10. |
| **G2** | Picking a list selects its first column that meets the current minimum; the currently selected list is evaluated with its current column. |
| **G3** | A list that supports Tripmine 10 but not Tripmine 20 stays visible while Tripmine is selected. |
| **G4** | Selecting 20 items for a 10-only list shows an inline shortfall error and blocks creating the session. |
| **G5** | If a mode, item-count, or column change makes the current list incompatible, the list stays selected and the shortfall error is shown. |
| **G6** | Changing to a supported mode, count, column, or list clears the error and allows creation. |
| **G7** | When no loaded list supports the chosen mode, the picker shows an empty-state explanation and creation is blocked. |
| **G8** | A backend refusal is shown in the existing alert region; the page never pretends a session is ready. |
| **G9** | List, mode, item-count, column, and create controls stay keyboard operable with visible focus; errors use the existing alert region. |

## Non-goals

- No new endpoint, request field, schema migration, or dependency.
- No change to the minimums themselves: Order 11, Connect 10, Tripmine 10/20.
- No retroactive deletion or rewriting of already-created sessions.
- No change to list editing, import, export, folder access, or round-start rules.
