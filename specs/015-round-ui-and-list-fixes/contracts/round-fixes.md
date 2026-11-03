# Contract: Round Fixes

Interfaces for the four fixes: value display, list ordering, the Tripmine slot count, and the view switch.

## 1. Value display

`format_value(n: f64) -> String`:

| Input | Output |
|---|---|
| `1440000000` | `1.44 Billion` |
| `146000000` | `146 Million` |
| `2920000000` | `2.92 Billion` |
| `1969` | `1969` |
| `48` | `48` |
| `-2560` | `-2,560` |
| `0.49` | `0.49` |

| Guarantee | Rule |
|---|---|
| **G1** | Every shipped value has a finite number; its display equals `format_value(numeric)`. |
| **G2** | Large numbers show a magnitude suffix; small numbers show with thousands separators. |

## 2. List ordering

| Guarantee | Rule |
|---|---|
| **G3** | Each shipped list is non-decreasing by its ordering column (Year/Height/Box Office/Area). |
| **G4** | Re-seeding produces the ordered sequence for an existing database too. |

## 3. Tripmine item count

| Input | Rule |
|---|---|
| `tripmineSlots` | Optional, `10` or `20`; absent means `10`; only valid on a Tripmine round. |
| List size | Must have at least the slot count of items with values. |

| Guarantee | Rule |
|---|---|
| **G5** | A Tripmine round samples and ranks exactly the configured slot count (`1..=N`). |
| **G6** | The mine side (`highest`/`lowest`/`random`) applies across the chosen count. |
| **G7** | A twenty-item board fits one screen with no scrolling; a ten-item round is unchanged. |

## 4. View switch

| Guarantee | Rule |
|---|---|
| **G8** | The header view switch is rendered only for the gamemaster (hidden for players and spectator). |
| **G9** | Activating it opens the target view in a new browser tab and leaves the current page unchanged. |
| **G10** | Direct links to any role's view still resolve. |

## Non-goals

- No new dependency, no schema change beyond the additive `tripmine_slots` column.
- No change to scoring, hearts, timers, or the ten-item Tripmine behavior.
