# Contract: Setup List Access

This feature extends the existing folder and session contracts. It adds no new list endpoint and no schema
change.

## Folder discovery and unlock (reused)

| Step | Request | Answer |
|---|---|---|
| Discover folders | `GET /api/folders` | `200` array of `{ id, name, isDefault, locked }`. Protected folders read `locked: true`. |
| Unlock a folder | `POST /api/folders/{folderId}/unlock` with `{ "password": "..." }` | `200 { "name": "..." }`; wrong password is `400 unauthorized` with `Incorrect password!`. |
| Load default lists | `GET /api/default-lists` | `200 { "lists": [...] }`, no password. |
| Load protected lists | `GET /api/folders/{folderId}/lists` with `X-Quiz-Password` | `200 { "lists": [...] }`; wrong/missing password is `400 unauthorized`. |

A locked folder's list endpoint MUST NOT be called before a successful unlock.

## Creating a session with custom lists

### Request

`POST /api/sessions` keeps its existing body and gains an optional map:

```json
{
  "gmPassword": "letmein",
  "teamCount": 2,
  "teams": [{ "name": "Team Cyan Pulsar" }, { "name": "Team Ruby Spark" }],
  "roundCount": 1,
  "rounds": [
    {
      "roundNumber": 1,
      "quizType": "ORDER",
      "listId": "lst-3kd8x1zp",
      "columnId": "value"
    }
  ],
  "folderPasswords": {
    "fld-family1": "hunter2"
  }
}
```

| Field | Required | Rule |
|---|---|---|
| `folderPasswords` | No | Map of folder id to password. Required only for folders referenced by a round's list; the default folder never needs an entry. |
| Existing round fields | Yes | Unchanged; `listId` and `columnId` must still be present and valid. |

### Access behavior

| Situation | Answer |
|---|---|
| List belongs to the default folder | Allowed without a map entry. |
| List belongs to a protected folder and the map has the correct password | Allowed. |
| List belongs to a protected folder and the map is missing the folder or has a wrong password | `400 unauthorized`, `Incorrect password!`; no session row, teams, or rounds are written. |
| List or column does not exist | `404 not_found` (existing behavior). |
| Map contains folders not referenced by any round | Ignored. |

The list's stored `folder_id` is authoritative. A client cannot supply a different folder id to bypass the
check.

## UI contract

| Guarantee | Rule |
|---|---|
| **G1** | The setup wizard offers default lists without a password. |
| **G2** | Every custom folder is visible by name with a locked state; its list titles and item counts are hidden until unlock. |
| **G3** | A correct password unlocks that folder and adds its lists to the round source selector. |
| **G4** | A wrong, empty, or whitespace-only password leaves the folder locked and shows a clear error. |
| **G5** | Each folder is unlocked independently; one password never unlocks another folder. |
| **G6** | An unlocked folder stays available for every round in the current wizard visit; a reload locks it again. |
| **G7** | Custom list options are folder-qualified so duplicate titles are distinguishable. |
| **G8** | Selecting a list updates the value-column choices; no session is created without a valid list and column for every round. |
| **G9** | Folder passwords never appear in a URL, browser history, or visible text. |
| **G10** | New controls are keyboard reachable and operable with visible focus. |

## Non-goals

- No new folder, list, or session endpoint.
- No persistent unlock token, cookie, or stored credential.
- No change to list ownership, folder permissions, or the read-only default folder.
- No schema migration.
