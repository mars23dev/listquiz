# Data Model: Value Column Labels

No new data. The label is an existing field of the round summary, shown where values already appear.

## Column name (existing `RoundSummary.columnName`)

| Field | Meaning |
|---|---|
| `columnName` | The round's value-column name, including any unit (e.g. "Area (km²)", "Population", "Year", "Box Office ($)"). |

Source: the list column chosen for the round. Already served in the session state.

## Where values are shown (and so where the label appears)

| Board | Values shown | Label placement |
|---|---|---|
| Order | The anchor value (always) and any revealed values | Middle of the card (value + column name), matching the anchor |
| Tripmine | Revealed values; the gamemaster's full view | Beside the values |
| Connect | The value column (always) and the gamemaster answer key | Heading of the value column |

**Rule**: the label is shown if and only if a value is shown. A hidden value shows neither the value nor
the label.

## Validation rules

- Every value display in Order, Tripmine, and Connect carries the column name (FR-001, FR-004–FR-006).
- Value visibility is unchanged; no label for a hidden value (FR-002).
- The label is the round's configured column name (FR-003).
- The gamemaster's value views show the label (FR-007).
- No rule or visibility change (FR-008).
