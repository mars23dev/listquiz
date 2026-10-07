# Contract: Board Icon And Layout Fixes

## 1. Item icon source

| Guarantee | Rule |
|---|---|
| **G1** | Order, Tripmine, and Connect render each item's own `iconName` (or flag). |
| **G2** | A game type never replaces an item's icon with its own. |
| **G3** | Countries show their flags in all three game types. |
| **G4** | An item with no icon shows the existing fallback. |

## 2. Tripmine rank bar

| Guarantee | Rule |
|---|---|
| **G5** | The bar shows one position per item: 10 for a ten-item round, 20 for a twenty-item round. |
| **G6** | The mine marker sits on the correct position within the chosen count. |
| **G7** | The twenty-position bar fits one screen (no horizontal scrolling). |

## 3. Connect rows

| Guarantee | Rule |
|---|---|
| **G8** | A name row and its value row are the same height. |
| **G9** | Row N on the left aligns with row N on the right. |
| **G10** | An item icon does not make its row taller than the value row. |

## Non-goals

- No new dependency, no schema change, no change to scoring, hearts, matching, or turns.
- The Countries list content (flags) is unchanged.
