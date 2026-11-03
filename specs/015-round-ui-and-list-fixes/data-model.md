# Data Model: Round UI And List Fixes

No schema change beyond one additive column. This describes the affected data.

## Shipped item value (seed constant → `item_value` row)

| Field | Meaning |
|---|---|
| `column_id` | Which column the value belongs to. |
| `display` | Now derived from `numeric` by the magnitude formatter, not hand-written. |
| `numeric` | The true value (finite). Cinema is stored in real dollars; years are the year; heights the metres. |

**Ordering**: each shipped list's items are emitted in ascending `numeric` order of its ordering column.

**Formatter** (`format_value(n)`): `|n| >= 1e12` → Trillion; `>= 1e9` → Billion; `>= 1e6` → Million; else
thousands-separated. Examples: `146000000` → "146 Million"; `1440000000` → "1.44 Billion"; `1969` →
"1969"; `48` → "48"; `-2560` → "-2,560".

## Shipped list (ordering column)

| List | Ordering column |
|---|---|
| World History | Year |
| Iconic Statues | Height |
| Blockbuster Cinema | Box Office |
| Countries | Area |

## Round config (existing `round_config`, + one column)

| Field | Type | Meaning |
|---|---|---|
| `tripmine_side` | text | `highest` / `lowest` / `random` (existing). |
| `tripmine_slots` | integer, nullable | **New.** 10 or 20; absent means 10. Only for Tripmine rounds. |

## Tripmine round

| Field | Meaning |
|---|---|
| slot count | 10 or 20, from the round config. |
| ranks | `1..=slot_count` (1 = lowest value, N = highest). |
| mine rank | `N` (highest), `1` (lowest), or a random rank in `1..=N` (random). |
| list requirement | the list needs at least the slot count of items with values, else the round is not ready. |

## View switch (UI, gamemaster only)

| Field | Meaning |
|---|---|
| visible | gamemaster only; hidden for players and spectator. |
| target | the role to open (gamemaster, p1..p4, view). |
| action | open the target's route in a new browser tab; the current page stays. |

## Validation rules

- Shipped lists are non-decreasing by their ordering column (FR-001, FR-002).
- Every shipped value has a finite `numeric` and a display derived from it (FR-012, FR-013).
- Re-seeding reorders an existing database (FR-003).
- Connect name and value cells are equal size (FR-004).
- Tripmine accepts 10 or 20 items; ranking and mine side span the chosen count (FR-005, FR-006).
- A twenty-item board fits one screen, no scrolling (FR-007); ten-item behavior is unchanged (FR-008).
- The view switch is gamemaster-only and opens a new tab without navigating the current page (FR-009, FR-010); direct links still work (FR-011).
