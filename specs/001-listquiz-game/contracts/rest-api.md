# Contract: REST API

> **Amended by `specs/005-session-flow/contracts/session-flow.md`:** adds `POST /api/sessions/quick`
> (one-team quick game) and `DELETE /api/sessions/{sessionId}` (delete a finished session).
>
> **Amended by `specs/007-fix-game-end/contracts/game-end.md`:** the last configured round ending
> finishes the session automatically (no gamemaster action required).

Base URL `http://<host>:8080`. All bodies are JSON unless stated otherwise. All endpoints are
same-origin with the served frontend, so no CORS configuration is needed.

Passwords are sent in the `X-Quiz-Password` header, not in a query string, so they stay out of URLs and
browser history. They are compared on every request; there is no login endpoint, no token, and no
cookie (research D-006).

## Error shape

Every non-2xx response uses this body:

```json
{ "error": { "code": "unauthorized", "message": "Incorrect password!" } }
```

| `code` | HTTP | Meaning |
|---|---|---|
| `bad_request` | 400 | A field is missing or malformed; `message` names the field. |
| `unauthorized` | 400 | Wrong or missing folder/gamemaster password. |
| `not_found` | 404 | Unknown session, folder, list, item, or column. |
| `conflict` | 409 | Duplicate folder name, or a write aimed at the default folder. |
| `too_small` / `too_large` | 422 | Item count outside 11-100, or a CSV that cannot produce that range. |
| `not_ready` | 409 | The round cannot start yet, or a round has already ended. |

`message` strings for the gamemaster-facing cases are the draft's own copy wherever the draft had one,
for example `Incorrect password!`.

## Folders

### `POST /api/folders`

Create a password-protected folder. The default folder is seeded, not created.

Request `{ "name": "Family facts", "password": "hunter2" }` → `201`

```json
{ "id": "fld-7k2m9qx4", "name": "Family facts", "isDefault": false }
```

Errors: `bad_request` (empty name or password), `conflict` (name already used).

### `POST /api/folders/{folderId}/unlock`

Request `{ "password": "hunter2" }` → `200 { "name": "Family facts" }`

The response echoes only the folder name; list names are never returned while a folder is locked
(FR-047, SC-009).

Errors: `not_found`, `unauthorized`.

### `POST /api/folders/{folderId}/rename`

Header `X-Quiz-Password`. Request `{ "name": "Family & friends" }` → `200` with the folder.

Errors: `unauthorized`, `conflict` (duplicate name), `conflict` (target is the default folder).

### `POST /api/folders/{folderId}/delete`

Header `X-Quiz-Password`. Request `{ "confirmPassword": "hunter2" }` → `204`

The confirmation must name the folder's list count so the UI can show it before the delete is accepted
(FR-046c). Deleting cascades to every list inside and touches nothing else (FR-046d).

Errors: `unauthorized`, `not_found`.

### `GET /api/folders`

→ `200`

```json
[
  { "id": "default", "name": "Default Lists", "isDefault": true, "locked": false },
  { "id": "fld-7k2m9qx4", "name": "Family facts", "isDefault": false, "locked": true }
]
```

`locked` is the caller's own state for that folder in this browser, so the response never reveals
whether a locked folder contains anything.

## Lists

Every endpoint below except `GET /api/default-lists` requires `X-Quiz-Password` for a protected folder.

### `GET /api/default-lists`

The read-only default folder. No password (FR-050).

→ `200 { "lists": [ <list summary>, ... ] }`

A list summary is `{ "id", "title", "description", "orderDirection", "columns": [{ "id", "name" }],
"itemCount", "isDefault" }`.

### `GET /api/folders/{folderId}/lists`

→ `200 { "lists": [ <list summary>, ... ] }`

### `GET /api/lists/{listId}`

→ `200`

```json
{
  "id": "lst-3kd8x1zp",
  "folderId": "fld-7k2m9qx4",
  "title": "World History & Inventions",
  "description": "Key human milestones",
  "orderDirection": "low_to_high",
  "columns": [{ "id": "year", "name": "Year" }],
  "items": [
    {
      "id": "itm-9ab2", "name": "Gutenberg Printing Press",
      "iconName": "BookOpen", "iconImage": null, "position": 0,
      "values": { "year": { "display": "1440", "numeric": 1440, "wasDefaulted": false } }
    }
  ]
}
```

### `POST /api/folders/{folderId}/lists`

Manual creation, used by the draft's list editor and its row table.

Request: the same body as `GET /api/lists/{listId}` without `id`, plus optional `id` when duplicating.
→ `201` with the created list.

Rules: title mandatory; 1-100 value columns is not a thing — 1 or more (FR-054); `itemCount` must land
in 11-100 (FR-052); item `position` values are honoured exactly as sent (FR-055); `iconName` and
`iconImage` are mutually exclusive (FR-056). A list in the default folder is rejected with `conflict`.

### `PUT /api/lists/{listId}`

Full replace of the editable fields, same body rules as creation. → `200` with the updated list.

### `DELETE /api/lists/{listId}`

→ `204`. Never accepts a list in the default folder.

### `GET /api/lists/{listId}/export.csv`

→ `text/csv` attachment. Header row is `Icon,Name,<column names...>` in the list's column order, which
re-imports to an equivalent list including its direction (FR-058).

### `POST /api/folders/{folderId}/lists/import`

Request `{ "title": "Imported", "orderDirection": "low_to_high", "csv": "Icon,Name,Year\nRocket,Apollo 11,1969\n" }`
→ `201` with the created list plus the import report:

```json
{
  "list": { "...": "as returned by GET /api/lists/{listId}" },
  "report": {
    "rowCount": 11,
    "defaultedRows": [{ "position": 4, "raw": "n/a" }],
    "warnings": ["Row 5: value 'n/a' is not numeric; 4 was used instead."]
  }
}
```

An optional leading `Icon` column is detected case-insensitively; otherwise the first column is the name
column and every following column is a value column. Quoted fields containing commas survive intact
(FR-057, research D-009). Fewer than 11 or more than 100 data rows fails with `too_small` / `too_large`
and names the required range (FR-059).

## Sessions

### `POST /api/sessions`

Body:

```json
{
  "gmPassword": "letmein",
  "teamCount": 2,
  "teams": [{ "name": "Team Cyan Pulsar" }, { "name": "Team Ruby Spark" }],
  "roundCount": 3,
  "rounds": [
    { "roundNumber": 1, "quizType": "ORDER",     "listId": "lst-3kd8x1zp", "columnId": "year" },
    { "roundNumber": 2, "quizType": "TRIPMINE",  "listId": "lst-55t0pa9c", "columnId": "box_office", "tripmineSide": "highest" },
    { "roundNumber": 3, "quizType": "ORDER",     "listId": "lst-3kd8x1zp", "columnId": "year" }
  ]
}
```

→ `201`

```json
{
  "sessionId": "lq-8fj2wkd9",
  "links": {
    "gamemaster": "/lq-8fj2wkd9/gamemaster",
    "players":   [{ "role": "p1", "path": "/lq-8fj2wkd9/p1", "hasTeam": true },
                  { "role": "p2", "path": "/lq-8fj2wkd9/p2", "hasTeam": true },
                  { "role": "p3", "path": "/lq-8fj2wkd9/p3", "hasTeam": false },
                  { "role": "p4", "path": "/lq-8fj2wkd9/p4", "hasTeam": false }],
    "spectator": "/lq-8fj2wkd9/view"
  }
}
```

The draft's five role links are always produced (FR-009): one gamemaster link, all four player links
P1 to P4, and one spectator link. `hasTeam` reports whether that player slot is backed by a configured
team, so the setup screen can label the links; a player link with `hasTeam: false` still resolves and
renders the read-only board with no team badge and no turn emphasis. `gmPassword` is mandatory
(FR-007). The caller's list and column references are validated against lists reachable from a folder
the gamemaster has unlocked, so a session cannot be built on a locked list.

### `POST /api/sessions/{sessionId}/unlock`

Request `{ "password": "letmein" }` → `200 { "ok": true }`. Wrong password → `unauthorized` with
`Incorrect password!` (FR-011, the draft's own copy).

### `GET /api/sessions/{sessionId}/state?role=<role>`

`role` is one of `gamemaster`, `p1`..`p4`, `view`. Any other value, or an unknown session id, returns
`not_found` (FR-013).

→ `200 <state>` — see `realtime.md` for the shape and for exactly what each role may see.

Player and spectator roles never receive host controls or un-revealed values (FR-062); the gamemaster
role is the only one that does.

### `POST /api/sessions/{sessionId}/actions`

> **Amended by `specs/002-fix-play-findings/contracts/actions-amendment.md`:** the body accepts an
> optional `role`; a player role posts its own in-round actions for the team on turn without the
> gamemaster password.

Header `X-Quiz-Password`. Every gamemaster action is one of these:

| `action` | Fields | Effect | Requirement |
|---|---|---|---|
| `select_item` | `itemId` | Marks a pool item as chosen by the active team in Order | FR-028 |
| `choose_slot` | `side`, `slotIndex` | Left, between-two, or right of the bar | FR-028 |
| `cancel_selection` | — | Returns the item to the pool, no scoring | FR-033 |
| `return_to_pool` | `itemId` | Removes a placed item from the bar | FR-033 |
| `pick_tripitem` | `itemId` | Reveals the picked position and scores it | FR-039 to FR-044 |
| `reveal_values` | — | Reveals every item value in every view | FR-024 |
| `next_round` | — | Ends the current round and starts the next | FR-023 |
| `pass_turn` | — | Passes the turn without scoring | FR-018 |
| `set_turn` | `teamId` | Turn simulation from the gamemaster view | FR-064 |
| `adjust_score` | `teamId`, `delta` | Manual score change | FR-023 |
| `adjust_hearts` | `teamId`, `delta` | Manual heart change, clamped 0-3 | FR-023 |
| `set_muted` | `muted` | Mutes audio in every view of the session | FR-024b |

→ `200 { "revision": 42 }`. The response carries only the new revision; the client re-reads state.
Errors: `unauthorized`, `not_found`, `not_ready` (round over, wrong turn, or incomplete setup).

## Static and navigation routes

| Route | Response |
|---|---|
| `/` | The draft landing page |
| `/lists` | The draft list-management page |
| `/setup` | The draft three-step setup wizard |
| `/{sessionId}` | Redirects to `/{sessionId}/p1` (FR-012) |
| `/{sessionId}/gamemaster` | Gamemaster view; shows the password gate until unlocked |
| `/{sessionId}/p1` … `/p4` | Player view |
| `/{sessionId}/view` | Spectator view |
| anything else | `index.html`, so client routing works on reload |
| `/assets/*` | Files from the exported `frontend/dist` |

Unknown session identifiers still return `index.html`; the client then renders the not-found view from
the `not_found` error on `GET /api/sessions/{id}/state` (FR-013).

There is no timer endpoint, because the product has no timer (FR-021).