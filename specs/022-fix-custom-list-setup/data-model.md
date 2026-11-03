# Data Model: Select Custom List In Session Setup

No database schema change. The feature adds one request-scoped wire structure and one ephemeral frontend
state.

## Existing entities (unchanged)

### Folder

| Field | Meaning |
|---|---|
| `id` | Folder id; `default` is the built-in folder. |
| `name` | Display name; visible before unlock. |
| `password` | Stored password; empty only for the default folder. Never returned. |
| `is_default` | True for the read-only default folder. |

### List (`quiz_list`)

| Field | Meaning |
|---|---|
| `id` | List id used by a round. |
| `folder_id` | The folder that owns the list and gates access. |
| `title` | List title; protected while the folder is locked. |
| `item_count` | Number of items shown in the selector. |
| `is_default` | True for seeded default lists. |

Columns and items are unchanged. A round still references one list and one column on that list.

## Request-scoped structure: folder password map

`POST /api/sessions` gains an optional field:

```json
{
  "gmPassword": "letmein",
  "rounds": [
    { "roundNumber": 1, "quizType": "ORDER", "listId": "lst-test", "columnId": "value" }
  ],
  "folderPasswords": {
    "fld-family1": "hunter2"
  }
}
```

| Rule | Behavior |
|---|---|
| Default folder | No map entry is needed; the stored password is empty, so the existing comparison passes. |
| Protected folder | The map must contain the list's `folder_id` with the exact password. |
| Missing or wrong entry | The request is refused with `unauthorized`; no session is written. |
| Extra map entries | Ignored; only folders referenced by a round are checked. |
| Source of truth | The list's stored `folder_id` decides which password is required; the client cannot claim another folder. |
| Lifetime | Request only. Nothing is written to SQLite, a cookie, or browser storage. |

## Ephemeral setup state (frontend)

| State | Meaning |
|---|---|
| `folders` | Result of `GET /api/folders`: default folder plus every custom folder and its locked flag. |
| `folderPasswords` | Passwords entered during this wizard visit, keyed by folder id. |
| `folderLists` | Lists loaded per unlocked custom folder. |
| `pendingFolderId` / `passwordDraft` | The locked folder currently showing the inline password form. |
| `availableLists` | Default lists plus lists from every unlocked custom folder, each carrying its folder id and name. |

### State transitions

```text
locked --(wrong/empty password)--> locked + error, no list data
locked --(correct password)------> unlocking -> unlocked + lists added
unlocked --(page reload/unmount)-> locked (state discarded)
unlocked --(session created)------> passwords sent only for folders referenced by rounds
```

## Validation rules

- A round's list must exist and its column must belong to it (existing rule).
- A round's list folder must be the default folder or have a matching password entry (FR-014).
- A locked folder's list titles, item counts, and columns are not fetched or rendered (FR-002).
- Changing a round's list resets the value column to a valid choice for the new list (FR-012).
- The existing 1-10 rounds, 1-4 teams, and non-empty gamemaster password rules are unchanged.
