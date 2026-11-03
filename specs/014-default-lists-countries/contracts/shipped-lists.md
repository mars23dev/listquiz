# Contract: Shipped Lists, Flags, And The Size Limit

The interfaces this feature exposes: the seed data shape, the flag icon-name convention, and the list
size rule.

## 1. Shipped-list seed shape

A shipped list provides `id`, `title`, `description`, `order_direction`, `columns`, and `items`. Each
item provides `id`, `name`, `icon_name`, and one `(column_id, display, numeric)` per column.

| Guarantee | Rule |
|---|---|
| **G1** | Four shipped lists exist: World History, Iconic Statues, Blockbuster Cinema, Countries. |
| **G2** | The three expanded lists hold 20–50 items each; ids are unique per list. |
| **G3** | Every item has a name, an icon, and at least one finite value. |

## 2. Flag icon-name convention

`icon_name = "flag:<cc>"` where `<cc>` is an ISO 3166-1 alpha-2 code (lowercase).

| Input | Rendered |
|---|---|
| `flag:de` | The German flag |
| `flag:us` | The United States flag |
| A lucide name (e.g. `Film`) | That lucide icon |
| Anything else | `HelpCircle` |

| Guarantee | Rule |
|---|---|
| **G4** | Every country item's `icon_name` starts with `flag:` and renders its flag offline on Windows. |
| **G5** | A flag never falls back to the generic `HelpCircle` icon. |

## 3. Size limit

| List | Cap |
|---|---|
| Every editable list (user-created or edited) | Minimum 11, maximum 100 |
| Countries (read-only default list, seeded) | None — holds all 195 sovereign states |

| Guarantee | Rule |
|---|---|
| **G6** | The 100-item maximum still applies to every editable list. |
| **G7** | The Countries list holds 195 items and is never rejected by the size rule. |

## 4. Country values

| Guarantee | Rule |
|---|---|
| **G8** | Every country item has area (km²), population, and founded (year), all finite numbers. |
| **G9** | The Countries list uses `low_to_high` and is selectable in setup and usable by Order, Tripmine, and Connect. |

## 5. Seeding

| Guarantee | Rule |
|---|---|
| **G10** | Re-running the seed updates `quiz_list` metadata and `item_count`, adds missing items, and refreshes values, leaving no stale counts. |

## Non-goals

- No schema change, no backend dependency, no change to the 100-item rule for editable lists.
- No change to the existing lists' columns, titles, or behavior.
