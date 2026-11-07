# Data Model: Block Invalid Session Setup

No database schema change. The feature adds one derived count to the list-summary response and one
compatibility rule over existing entities.

## Existing entities (unchanged)

### List (`quiz_list`)

| Field | Meaning |
|---|---|
| `id` | List id referenced by a round. |
| `folder_id` | Owning folder; unchanged access rules apply. |
| `title` | Display title. |
| `item_count` | Total items in the list. |
| `is_default` | True for seeded default lists. |

### Value column (`list_item_column`)

| Field | Meaning |
|---|---|
| `list_id` | Owning list. |
| `id` | Column id referenced by a round. |
| `name` | Display name. |
| `position` | Display order; the first column is selected when a list is picked. |

### Item (`item`) and item value (`item_value`)

| Field | Meaning |
|---|---|
| `item.id`, `item.list_id`, `item.name` | An item and its list. |
| `item_value.item_id`, `item_value.column_id` | A value for one item in one column. |
| `item_value.display`, `item_value.numeric` | The stored value. |

An item counts toward a round only when it has an `item_value` row for the round's selected
`column_id`. Total `item_count` is not sufficient.

## Derived field: `valueCount`

`ListSummary.columns[]` gains `valueCount`:

```json
{
  "id": "default_world_history",
  "title": "World History & Inventions",
  "itemCount": 12,
  "isDefault": true,
  "columns": [
    { "id": "year", "name": "Year", "valueCount": 12 }
  ]
}
```

| Rule | Behavior |
|---|---|
| Definition | Number of items in the list with a value in that column. |
| Range | `0..=itemCount`. |
| Availability | Returned by both default-list and folder-list endpoints. |
| Storage | Derived at read time; nothing is persisted. |

## Round compatibility rules

| Mode | Selected size | Required valued items | Refusal when short |
|---|---|---|---|
| Order | n/a | 11 | Round cannot be created. |
| Connect | n/a | 10 | Round cannot be created. |
| Tripmine | 10 (default) | 10 | Round cannot be created. |
| Tripmine | 20 | 20 | Round cannot be created. |

- The selected column's `valueCount` is compared with the required count.
- Exact equality with the minimum is valid.
- A session is created only when every round passes.
- One failing round refuses the whole request; nothing is written.

## Session-creation request (unchanged shape)

`POST /api/sessions` and `POST /api/sessions/quick` keep their existing request shapes. The rule adds no
field; it evaluates the existing `listId`, `columnId`, `quizType`, and `tripmineSlots`.

## Ephemeral setup state (frontend)

| State | Meaning |
|---|---|
| List summary `columns[].valueCount` | Per-column valued-item counts used to hide impossible lists. |
| Round draft | Existing `quizType`, `listId`, `columnId`, `tripmineSlots`. |
| Round error | Shortfall message for a retained incompatible selection; clears when corrected. |
| Empty-state message | Shown when no loaded list supports the chosen mode. |

### Eligibility rule used by the setup page

- A list is offered when any column meets the mode's smallest size:
  Order 11, Connect 10, Tripmine 10.
- Picking a list selects its first column that meets the current minimum; a list
  with a qualifying later column is not hidden.
- The currently selected list is evaluated with its current column.
- If a mode, size, or column change makes the current list incompatible, the list stays selected and an
  error names the shortfall.
- Unsupported Tripmine sizes stay selectable; selecting one shows the error.
- If no list qualifies, the picker shows an empty-state explanation.

### State transitions

```text
valid selection --(mode/size/column change makes it short)--> retained selection + error, create blocked
retained + error --(supported mode/size/column or list)-----> valid selection, error cleared
list picker      --(no list meets mode minimum)-------------> empty-state message, create blocked
valid selection  --(create session)-------------------------> backend re-checks every round, then writes
```

## Validation rules

- Empty gamemaster password, 1-10 rounds, 1-4 teams, and list/column existence keep their existing rules.
- A round's list must belong to the default folder or supply the correct folder password (existing rule).
- A round's selected column must have at least the required valued-item count (new rule).
- The backend uses stored data at creation time, so a list or column edited after setup opened is judged by
  its current state.
- The frontend mirrors the rule for usability; the backend refusal is authoritative.
