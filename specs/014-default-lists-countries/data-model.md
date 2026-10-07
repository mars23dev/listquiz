# Data Model: Expand Default Lists And Add Countries

No schema change. This describes the shipped seed data and the flag name convention.

## Shipped list (seed constant → `quiz_list` row)

| Field | Meaning |
|---|---|
| `id` | Stable list id (`default_world_history`, `default_statues_monuments`, `default_cinema_box_office`, `default_countries`). |
| `title` / `description` | Shown in setup. |
| `order_direction` | `low_to_high` for every shipped list. |
| `columns` | The value columns (see below). |
| `items` | The shipped items. |
| `is_default` | Always `1` for shipped lists (read-only). |

**Size rule**: 11–100 for editable lists. The three expanded lists stay in 20–50. The Countries list is
seeded directly and, being a read-only default list, is never checked against the cap (so it holds 195).

## Shipped item (seed constant → `item` row)

| Field | Meaning |
|---|---|
| `id` | Unique within the list. Existing prefixes `h_`, `s_`, `m_`; countries use `c_<cc>`. |
| `name` | Display name. |
| `icon_name` | A lucide name, or `flag:<cc>` for a country flag. |
| `values` | One entry per column: `(column_id, display, numeric)`, all finite. |

## Columns

| List | Columns |
|---|---|
| World History | `year` "Year" |
| Iconic Statues | `height` "Height (m)", `year_built` "Year Built" |
| Blockbuster Cinema | `box_office` "Box Office ($)", `release_year` "Release Year" |
| Countries | `area_km2` "Area (km²)", `population` "Population", `founded` "Founded" |

## Country item

| Field | Rule |
|---|---|
| id | `c_<cc>` for ISO 3166-1 alpha-2 `<cc>`. |
| name | Common English country name. |
| icon_name | `flag:<cc>` (lowercase). |
| area_km2 | Area in square kilometres, finite. |
| population | Recent estimate, finite. |
| founded | Independence/establishment year, finite; negative for BC. |

Coverage: 195 sovereign states (193 UN members + Vatican City + Palestine), one item each, unique ids.

## Flag name convention

`icon_name = "flag:<cc>"` → the UI renders the flag for `<cc>`. Any other value is treated as a lucide
name (unknown → `HelpCircle`). This is the only new interface between seed data and the icon renderer.

## Seeding behavior (idempotent upsert)

| Table | On existing row | On new row |
|---|---|---|
| `quiz_list` | Update title, description, order_direction, item_count | Insert |
| `list_item_column` | Replace | Insert |
| `item` | Leave (already present) | Insert |
| `item_value` | Replace | Insert |

Result: an already-seeded database gets correct item counts and metadata plus any new items.

## Validation rules

- Every shipped item keeps a name, an icon, and at least one finite value; ids are unique per list
  (FR-002).
- The three expanded lists stay within 20–50 and within 11–100 (FR-001, FR-003).
- The Countries list holds all 195 states with flag + area + population + founded; it is exempt from the
  100 cap (FR-004 to FR-007, FR-011).
- Countries is selectable and usable by Order, Tripmine, and Connect (FR-008).
- Seeding leaves an already-seeded database consistent (FR-009).
