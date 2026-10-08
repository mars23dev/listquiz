# Research: Value Column Labels

## D-1701: The label is the round's column name, already served

**Decision**: Use `state.round.columnName` (the `RoundSummary` already in the session state) as the label
and pass it to the boards as a prop.

**Rationale**: The column name is the unit/meaning the user is missing, and it is already part of the
state the page holds, so no backend change or new data is needed (FR-003; Principle VII).

**Alternatives considered**: adding a new field or endpoint (unnecessary — the name is served).

## D-1702: The label is rendered where values are shown, and nowhere else

**Decision**: Render the column name only alongside a visible value: the Connect value column heading, a
label beside the Order bar/anchor values, and a label beside the Tripmine values (revealed values and the
gamemaster's full view). No label appears for a hidden value.

**Rationale**: The user asked to keep value visibility unchanged and to add the name only where values are
visible (FR-001, FR-002). Tying the label to the same "is this value shown" condition the board already
uses keeps the two in lockstep (Principle VII).

**Alternatives considered**: always showing the label (would label hidden values and hint at them);
appending the unit to every value string (changes the value text and risks layout).

## D-1703: Presentation per board

**Decision**: A short, muted label near the values — a heading above the Connect value column, and a small
label beside the Order and Tripmine values — so it reads as the unit without dominating the board.

**Rationale**: One consistent, minimal treatment across modes (FR-004, FR-005, FR-006). The label is
non-interactive and small so it does not add scrolling (SC-004; Principle VI).

**Alternatives considered**: a large banner per mode (visual noise); a tooltip only (not always visible).

## D-1704: The gamemaster answer key shows the label too

**Decision**: Where the gamemaster sees values the room does not (Connect answer key, full Tripmine view),
the label is shown with those values.

**Rationale**: The label is part of the value display wherever it appears (FR-007). It reveals no hidden
value because the gamemaster already sees the value.

**Alternatives considered**: gamemaster-only labels omitted (inconsistent).

## D-1705: No schema, dependency, or rule change

**Decision**: No migration, no new dependency; visibility, scoring, and rules are untouched.

**Rationale**: Presentation only (FR-008; Principle VII).

**Alternatives considered**: none needed.
