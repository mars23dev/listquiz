# Data Model: ListQuiz

SQLite schema, entity rules, and state transitions. Types are given as SQLite storage classes with the
Rust-side intent noted where it matters. Every rule here traces to a functional requirement or a
clarification in `spec.md`.

## Tables

### folder

| Column | Type | Rules |
|---|---|---|
| `id` | TEXT PK | `default` for the built-in folder, otherwise `fld-<8 random alphanumerics>` |
| `name` | TEXT NOT NULL | trimmed, 1-60 chars, unique among non-default folders (FR-046) |
| `password` | TEXT NOT NULL | stored as entered, never hashed (FR-068). Empty only for `id = 'default'` (FR-050) |
| `is_default` | INTEGER NOT NULL | 1 for the built-in folder; enforces the read-only rule at the data layer (FR-049, FR-050) |
| `created_at` | INTEGER NOT NULL | epoch milliseconds |

### list_item_column

| Column | Type | Rules |
|---|---|---|
| `id` | TEXT PK | `col_<slug>` derived from the column name at creation |
| `list_id` | TEXT NOT NULL → `quiz_list.id` | ON DELETE CASCADE |
| `name` | TEXT NOT NULL | 1-40 chars (FR-054) |
| `position` | INTEGER NOT NULL | 0-based display order; `columns[0]` is the default value column (FR-054) |

### quiz_list

| Column | Type | Rules |
|---|---|---|
| `id` | TEXT PK | `lst-<8 random alphanumerics>` |
| `folder_id` | TEXT NOT NULL → `folder.id` | ON DELETE CASCADE (FR-046d) |
| `title` | TEXT NOT NULL | 1-80 chars, mandatory (FR-052) |
| `description` | TEXT | optional |
| `order_direction` | TEXT NOT NULL | `low_to_high` or `high_to_low`; defines which end of the list is the low end (FR-053) |
| `item_count` | INTEGER NOT NULL | maintained by the `lists` module; 11-100 enforced on every write path (FR-052, FR-059) |
| `is_default` | INTEGER NOT NULL | mirrors the owning folder's `is_default` (FR-049) |
| `created_at` / `updated_at` | INTEGER NOT NULL | epoch milliseconds |

### item

| Column | Type | Rules |
|---|---|---|
| `id` | TEXT PK | `itm-<8 random alphanumerics>` |
| `list_id` | TEXT NOT NULL → `quiz_list.id` | ON DELETE CASCADE |
| `name` | TEXT NOT NULL | mandatory, trimmed (FR-052) |
| `position` | INTEGER NOT NULL | the gamemaster's drag order, preserved exactly (FR-055) |
| `icon_name` | TEXT | icon chosen from the product icon set, or null (FR-056) |
| `icon_image` | BLOB / TEXT | uploaded image, or null; mutually exclusive with `icon_name` (FR-056) |

### item_value

One row per (item, column). Keeping values in their own table lets a list carry several columns and lets
one round read exactly one of them (FR-054).

| Column | Type | Rules |
|---|---|---|
| `item_id` | TEXT NOT NULL → `item.id` | ON DELETE CASCADE |
| `column_id` | TEXT NOT NULL → `item_item_column.id` | ON DELETE CASCADE |
| `display` | TEXT NOT NULL | what players see after reveal, e.g. `$2.26 Billion` |
| `numeric` | REAL NOT NULL | the comparable value used for sorting and placement; falls back to row position when unparseable (FR-057, D-011) |
| `was_defaulted` | INTEGER NOT NULL | 1 when `numeric` was derived rather than parsed; reported by import (FR-057) |
| PK | (`item_id`, `column_id`) | |

### game_session

| Column | Type | Rules |
|---|---|---|
| `id` | TEXT PK | `lq-` + 8 random lowercase alphanumerics (D-007) |
| `gm_password` | TEXT NOT NULL | non-empty, stored as entered, never hashed (FR-007, FR-068) |
| `team_count` | INTEGER NOT NULL | 1-4 (FR-004) |
| `round_count` | INTEGER NOT NULL | 1-10 (FR-001) |
| `current_round` | INTEGER NOT NULL | 1-based index of the round in play; 0 before the first round starts |
| `status` | TEXT NOT NULL | `setup` → `running` → `finished` |
| `muted` | INTEGER NOT NULL | 0/1; applies to every view of the session (FR-024b) |
| `revision` | INTEGER NOT NULL | bumped on every state change; the value pushed over SSE (D-005) |
| `created_at` | INTEGER NOT NULL | epoch milliseconds |

### team

| Column | Type | Rules |
|---|---|---|
| `id` | TEXT PK | `team_1` .. `team_4` |
| `session_id` | TEXT NOT NULL → `game_session.id` | ON DELETE CASCADE |
| `name` | TEXT NOT NULL | gamemaster-supplied or the default for that slot (FR-004) |
| `color` | TEXT NOT NULL | `cyan` / `rose` / `amber` / `emerald`, unique within the session (FR-005) |
| `primary_hex` | TEXT NOT NULL | `#06b6d4`, `#f43f5e`, `#f59e0b`, `#10b981` — fixed in both themes (D-008) |
| `accent_glow` | TEXT NOT NULL | the draft's per-team glow value |
| `score` | INTEGER NOT NULL | carries across rounds, never reset (FR-017) |
| `hearts` | INTEGER NOT NULL | 0-3, reset to 3 at the start of every round (FR-016) |
| `turn_index` | INTEGER NOT NULL | position in the rotation; the first turn of round N belongs to `team_{((N-1) mod count) + 1}` (FR-015) |

### round_config

| Column | Type | Rules |
|---|---|---|
| `session_id` | TEXT NOT NULL → `game_session.id` | ON DELETE CASCADE |
| `round_number` | INTEGER NOT NULL | 1-10 (FR-001) |
| `quiz_type` | TEXT NOT NULL | `ORDER` or `TRIPMINE` (FR-002) |
| `tripmine_side` | TEXT | `highest` or `lowest`, required for Tripmine, null for Order; immutable once the round starts (FR-036, FR-036a) |
| `list_id` | TEXT NOT NULL → `quiz_list.id` | the round's source list (FR-002) |
| `column_id` | TEXT NOT NULL → `item_item_column.id` | the one value column this round uses (FR-002, FR-054) |
| PK | (`session_id`, `round_number`) | a round with no list or no type cannot start (FR-014) |

### order_state

Written when an Order round starts; exactly one row per session at a time.

| Column | Type | Rules |
|---|---|---|
| `session_id` | TEXT PK → `game_session.id` | replaced at each round start |
| `anchor_item_id` | TEXT NOT NULL → `item.id` | the middle item, rendered in its own colour (FR-026) |
| `anchor_value` | REAL NOT NULL | the comparison value used for every placement decision |
| `pool` | TEXT NOT NULL | JSON array of item ids: 5 sampled below and 5 sampled above the anchor (FR-027, clarification 1) |
| `left_ids` | TEXT NOT NULL | JSON array, ordered exactly as placed — never re-sorted by value (FR-029) |
| `right_ids` | TEXT NOT NULL | same, for the high side |
| `pending` | TEXT | JSON `{item_id, side, slot_index, team_id}` or null; exists only between choosing and confirming (FR-028) |

### tripmine_state

Written when a Tripmine round starts; exactly one row per session at a time.

| Column | Type | Rules |
|---|---|---|
| `session_id` | TEXT PK → `game_session.id` | replaced at each round start |
| `side` | TEXT NOT NULL | `highest` or `lowest`, copied from `round_config.tripmine_side` and never changed mid-round (FR-036a) |
| `searched_rank` | INTEGER NOT NULL | the position players are hunting; 1 when the tripmine is highest, 10 when it is lowest (FR-037) |
| `items` | TEXT NOT NULL | JSON array of 10 objects `{item_id, true_rank, is_revealed, revealed_by, points}` (FR-035) |
| `hit` | INTEGER NOT NULL | 1 once the tripmine has been picked (FR-042) |
| `hit_by` | TEXT | the team that hit it |
| `completed` | INTEGER NOT NULL | 1 when the round has ended for any reason |

### score_event

Append-only audit of every point and heart change. Keeps the end-of-round scorecard and makes the
cross-round carry-over checkable (SC-005).

| Column | Type | Rules |
|---|---|---|
| `id` | TEXT PK | monotonic sequence per session |
| `session_id` | TEXT NOT NULL → `game_session.id` | ON DELETE CASCADE |
| `round_number` | INTEGER NOT NULL | the round in which it happened |
| `team_id` | TEXT NOT NULL → `team.id` | |
| `kind` | TEXT NOT NULL | `points_awarded`, `points_removed`, `heart_lost`, `heart_restored`, `manual_adjustment` |
| `amount` | INTEGER NOT NULL | signed |
| `reason` | TEXT NOT NULL | `correct_placement`, `tripmine_hit`, `searched_item_found`, `safe_pick`, `final_item`, `zero_hearts`, `game_master` |

## Entity relationships

```text
folder 1--n quiz_list 1--n item 1--n item_value n--1 item_item_column
folder (is_default) 1--n quiz_list (seeded only, read-only)
game_session 1--n team
game_session 1--n round_config n--1 quiz_list n--1 item_item_column
game_session 1--1 order_state        (Order rounds only)
game_session 1--1 tripmine_state     (Tripmine rounds only)
game_session 1--n score_event n--1 team
```

## Derived values, never stored

- **Item rank in a Tripmine round** (`true_rank`, 1 lowest to 10 highest): computed by sorting the ten
  sampled items by the round column's `numeric` ascending. Recomputable from `tripmine_state.items`, so
  it is stored only as part of that JSON.
- **Display text of a value**: `item_value.display`, stored verbatim because it is what players read.
- **Whether a player view may see a value**: not stored. Computed per request from the role and the
  reveal state (see `contracts/realtime.md`).
- **Rank history for the final scorecard**: aggregated from `score_event`.

## State transitions

### Session

```text
setup --create session--> running --last round ends--> finished
                              |                              ^
                              +--restart / reload--> running (no state change)
```

A reload or a backend restart is not a transition: all state is read back from SQLite, so a session
resumes with identical scores, hearts, turn, and board (FR-067, SC-011).

### Order round

```text
start -> awaiting_placement -> pending -> (confirm correct) -> bar updated, +10, next turn
                              |        -> (confirm wrong)   -> item stays in pool, -1 heart, next turn
                              |        -> (cancel)          -> awaiting_placement, no change
                              |        -> (return to pool)  -> awaiting_placement, no change
                              +--all items placed--> round_over
                              +--no team has hearts--> round_over
```

`round_over` triggers the reveal control and hands control back to the gamemaster for the next round
(FR-024, FR-032).

### Tripmine round

```text
start -> awaiting_pick -> (pick) -> reveal position -> advance searched_rank
                                   -> next turn
                             -> (pick = searched)   -> +10 (or +20 if 2 unrevealed left), end if final
                             -> (pick = tripmine)    -> -10 for the picker, +10 for everyone else, end
                             -> (pick = other)       -> +5, searched_rank unchanged
```

The tripmine and the ten items are fixed at round start and never re-drawn (FR-036a, clarification 2).

## Validation rules

| Rule | Enforced by | Requirement |
|---|---|---|
| Folder name unique, 1-60 chars | `folders` | FR-046 |
| Folder password non-empty except default | `folders` | FR-046, FR-050 |
| Folder delete requires the correct password re-entered | `folders` | FR-046c |
| Default folder rejects every write | `folders` + `is_default` column | FR-049, FR-050 |
| List belongs to exactly one unlocked folder | `lists` | FR-045, FR-048 |
| List title mandatory, 1-80 chars | `lists` | FR-052 |
| Item count 11-100 on create, edit, import | `lists`, `csv_import` | FR-052, FR-059 |
| Item name mandatory | `lists`, `csv_import` | FR-052 |
| Item order preserved exactly as supplied | `lists` | FR-055 |
| At least one value column per list | `lists` | FR-054 |
| A round uses exactly one column of its list | `sessions` | FR-002, FR-054 |
| Round count 1-10 | `sessions` | FR-001 |
| Team count 1-4, colours unique | `teams` | FR-004, FR-005 |
| Gamemaster password non-empty | `sessions` | FR-007 |
| A round cannot start without a type and a list | `sessions` | FR-014 |
| Tripmine rounds require a side; side immutable after start | `sessions`, `tripmine_round` | FR-036, FR-036a |
| First turn rotates one team per round, skipping 0-heart teams | `scoring` | FR-015, FR-019 |
| Hearts reset to 3 each round; scores never reset | `scoring` | FR-016, FR-017 |
| Placement accepted only left, between-two, right | `order_round` | FR-028 |
| Equal values count as correctly placed on the correct side | `order_round` | FR-034 |
| Pick refused for a team that is not on turn | `tripmine_round` | FR-044 |
| Every action requires the gamemaster password | `sessions` | FR-011 |

## Scale assumptions

One SQLite file holds a handful of folders, tens of lists of up to 100 items, and every session played
on the machine. Sessions are never deleted by the product, so `score_event` is the only table that grows
without bound; at roughly 6 events per round and 10 rounds per session it adds well under a hundred rows
per game. No partitioning or archival is warranted (Principle VII).