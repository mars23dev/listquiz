# Data Model: Per-Game Ordering

One schema change: the `order_direction` column is dropped.

## List (existing `quiz_list`)

| Field | Change |
|---|---|
| `id`, `folder_id`, `title`, `description`, `item_count`, `is_default`, `created_at`, `updated_at` | unchanged |
| `order_direction` | **Removed** (column dropped by migration). |

## Game mode ordering (new rule)

| Game mode | Sort order |
|---|---|
| Order | Ascending |
| Tripmine | Ascending |
| Connect | Ascending |

Defined once per game mode (`QuizType::sort_order()`), so a future mode can choose descending without a
list field.

## Round column ordering

The round's chosen column is ordered by the game mode's sort order:
- Order: ascending (left = lowest, right = highest).
- Tripmine: rank 1 = lowest … N = highest.
- Connect: value column ascending.

## Validation rules

- No list input (create/edit/import) carries a direction (FR-001, FR-002).
- Each mode applies its own order (FR-003) — all ascending today (FR-004–FR-006).
- An existing database upgrades without error; the dropped field no longer affects rounds (FR-007).
- Scoring, hearts, the Tripmine side, and matching are unchanged (FR-008).
