# Data Model: Board Icon And Layout Fixes

No schema change. The change carries an existing column into the session views and fixes two layout rules.

## Item icon (existing `item.icon_name`, carried into the views)

| View field | Source | Meaning |
|---|---|---|
| `OrderItem.iconName` | `item.icon_name` | The list item's icon (or `flag:<cc>`), shown by the Order board. |
| `TripmineItem.iconName` | `item.icon_name` | Same, shown by the Tripmine board. |
| `AnchorItem.iconName` | `item.icon_name` | The anchor's icon on the Order bar. |
| `ConnectName.iconName` | `item.icon_name` | Already present (feature 010); unchanged. |

**Rule**: every board renders the item's `iconName`; when it is absent the fallback icon is used. A game
type never substitutes its own icon.

## ListItemValue (round builder input)

| Field | Meaning |
|---|---|
| `item_id`, `name`, `display`, `numeric` | Existing. |
| `icon_name` | **New.** Carried from `item.icon_name` so Order/Tripmine views can emit it. |

## Tripmine rank bar

| Field | Rule |
|---|---|
| positions | One per item: `1..=items.length` (10 or 20). |
| mine marker | On the position equal to the mine's rank within the count. |

## Connect row

| Field | Rule |
|---|---|
| height | The name cell and the value cell share one fixed height. |
| alignment | Row N on the left lines up with row N on the right. |
| icon | Constrained so it does not make the name row taller. |

## Validation rules

- Every item that has an icon shows it in Order, Tripmine, and Connect (FR-001, FR-002).
- Country items keep their flags everywhere (FR-003).
- An item with no icon shows the fallback (FR-005).
- The bar shows exactly one position per item; the mine marker uses the chosen count (FR-006, FR-007).
- Connect name and value rows are equal height and aligned (FR-008, FR-009).
- No other behavior changes (FR-010).
