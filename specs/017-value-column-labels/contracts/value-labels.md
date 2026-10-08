# Contract: Value Column Labels

## Label visibility

| Condition | Result |
|---|---|
| A value is shown (Order anchor/revealed, Tripmine revealed or gamemaster, Connect value column) | The round's column name is shown with it. |
| A value is hidden (player before reveal) | Neither the value nor the label is shown. |
| Gamemaster value view (answer key, full Tripmine) | The label is shown with the values. |

| Guarantee | Rule |
|---|---|
| **G1** | Every displayed value carries its column name. |
| **G2** | Value visibility is unchanged; hidden values have no label. |
| **G3** | The label is the round's configured column name (with its unit). |
| **G4** | The label is non-interactive and does not add scrolling. |
| **G5** | No scoring, hearts, matching, or turn rule changes. |

## Non-goals

- No new dependency, no backend/schema change.
- The value text itself is unchanged (the label is separate).
