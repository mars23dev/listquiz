# Feature Specification: Round UI And List Fixes

**Feature Branch**: `015-round-ui-and-list-fixes`

**Created**: 2026-10-07

**Status**: Draft

**Input**: User description: "1. You have expanded the lists but not in order. E.g. I have 135 million this goes up to 2.3 billion than down to 1.x billion and then rising again, should not happen. The lists are always ordered by value this is important. 2. visual bug in connect the right items had smaller boxes than the left so it looked like there are less items on the right than on the left. Make them the same size. 3. add an option to tripmine to expand the game from 10 to 20 items. Adapt the item size for 20 accordingly no scrolling should be neceessary everything needs to fit on 1 screen. 4. The view switch at the header should not be there for players or viewers only for gamemaster. If gamemaster click on it it should open the page on a new tab and stay on the gamemaster page"

## Clarifications

### Session 2026-10-07

- Q: Which value orders each shipped list? → A: Each list is ordered by its value column, which must be numeric (History: Year; Statues: Height; Cinema: Box Office; Countries: Area). Shipped values are stored as numbers; large numbers are auto-formatted on display (146000000 → "146 Million", 1300000000 → "1.3 Billion").

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Shipped lists are ordered by value (Priority: P1)

Every shipped list must be presented in ascending order of its value, so the room never sees a list jump up
and down (for example a box-office column that rises to billions, drops, then rises again). The value must
be a real number, and large numbers should be shown in a readable form (146000000 shown as "146 Million"),
so the list reads correctly and consistently.

**Why this priority**: It is a correctness bug in the shipped data the user calls out as important, and it
affects every list.

**Independent Test**: Open each shipped list and confirm the items read in ascending value order, with no
decrease anywhere in the sequence.

**Acceptance Scenarios**:

1. **Given** a shipped list, **When** its items are read top to bottom, **Then** their values are in non-decreasing (ascending) order.
2. **Given** the Cinema list, **When** the box-office values are read, **Then** they rise monotonically (no billion figure followed by a lower one).
3. **Given** the Countries list, **When** its items are read, **Then** they are in ascending order of the list's ordering value.
4. **Given** any shipped list, **When** it is reseeded, **Then** the order is the same ordered sequence.
5. **Given** a value of 146000000, **When** it is displayed, **Then** it reads as "146 Million" rather than a long digit string.

---

### User Story 2 - Connect columns look equal (Priority: P2)

On the Connect board the left (name) column and the right (value) column should be the same size. Today the
right cells are shorter than the left cells, so the value column looks like it has fewer items than the
name column. The two columns must render with equal-sized rows.

**Why this priority**: A visual bug that misleads the room about the number of items; small but confusing.

**Independent Test**: Open a Connect round and compare the two columns; the rows and the columns are the
same size.

**Acceptance Scenarios**:

1. **Given** a Connect round, **When** the board is shown, **Then** every value row is the same size as every name row.
2. **Given** a Connect round, **When** the two columns are compared, **Then** the left and right columns occupy the same height and the same number of rows is visually apparent on both sides.
3. **Given** a Connect round with a matched pair hidden, **When** the board updates, **Then** the two columns stay equal.

---

### User Story 3 - Tripmine can play 10 or 20 items (Priority: P2)

A Tripmine round should be playable with either the current ten items or an expanded twenty items. When the
twenty-item option is chosen, the board must scale so everything fits on one screen with no scrolling.

**Why this priority**: Adds a requested variation to Tripmine; independent of the other fixes.

**Independent Test**: Configure a Tripmine round with twenty items, start it, and confirm the board shows
twenty items that all fit on one screen without scrolling.

**Acceptance Scenarios**:

1. **Given** the setup for a Tripmine round, **When** the gamemaster configures it, **Then** a choice of ten or twenty items is available.
2. **Given** a ten-item Tripmine round, **When** it starts, **Then** it behaves exactly as today.
3. **Given** a twenty-item Tripmine round, **When** it starts, **Then** the board shows twenty ranked items.
4. **Given** a twenty-item Tripmine round, **When** the board is shown, **Then** all items fit on one screen with no scrolling.
5. **Given** a twenty-item round, **When** the mine is set, **Then** the mine position respects the chosen side (lowest, highest, random) across all twenty ranks.

---

### User Story 4 - The header view switch is gamemaster-only (Priority: P2)

The view switch in the header must be visible only to the gamemaster, not to players or the spectator. When
the gamemaster uses it, the chosen view opens in a new browser tab and the gamemaster stays on the
gamemaster page.

**Why this priority**: Players/spectators should not see a control that switches roles, and the gamemaster
needs to open a view without losing their own page.

**Independent Test**: Open the session as a player and as the spectator and confirm no view switch is
present; as the gamemaster confirm the switch is present and opens the chosen view in a new tab while the
gamemaster page stays.

**Acceptance Scenarios**:

1. **Given** a player or spectator view, **When** the header is shown, **Then** no view switch is present.
2. **Given** the gamemaster view, **When** the header is shown, **Then** the view switch is present.
3. **Given** the gamemaster, **When** a view is chosen in the switch, **Then** that view opens in a new browser tab.
4. **Given** the gamemaster chose a view, **When** the new tab opens, **Then** the gamemaster's current page remains on the gamemaster view.

---

### Edge Cases

- List ordering: ties in value keep a stable, deterministic order; ordering must not change scoring or round behavior.
- List ordering: reseeding an existing database produces the ordered sequence, not the previous order.
- List values: units live in the column name (e.g. "Height (m)", "Box Office ($)") so the value itself stays a plain number.
- List values: large numbers use magnitude suffixes (thousand, Million, Billion, Trillion); small numbers show with thousands separators; a value with no suffix is still a number.
- List values: a negative value (e.g. a BC year) orders below zero and displays with a sign.
- Connect: equal sizing must hold for a full board, a partly matched board, and when a name or value is staged.
- Tripmine twenty items: a list must have at least twenty items with values for the twenty-item option to be usable; otherwise the option is unavailable or refused.
- Tripmine twenty items: the timer, scoring, mine, and reveal rules are unchanged by the item count.
- View switch: a direct link to a player/spectator view still works even though the switch is hidden.
- View switch: opening a new tab must not navigate the gamemaster's current tab away.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every shipped list MUST be presented in ascending order of its ordering value, with no decrease in the sequence.
- **FR-002**: Each shipped list MUST be ordered ascending by its numeric value column (History: Year; Statues: Height; Cinema: Box Office; Countries: Area).
- **FR-003**: Re-seeding MUST produce the ordered sequence for every shipped list.
- **FR-012**: Every shipped value MUST be stored as a finite number; its display MUST be derived from the number rather than stored as free text.
- **FR-013**: Large numeric displays MUST use automatic magnitude formatting with thousands separators and thousand/Million/Billion/Trillion suffixes (e.g. 146000000 → "146 Million", 1300000000 → "1.3 Billion").
- **FR-004**: The Connect board's value column rows MUST be the same size as its name column rows, and the two columns MUST be visually equal.
- **FR-005**: The setup for a Tripmine round MUST offer a choice of ten or twenty items.
- **FR-006**: A Tripmine round MUST support twenty items, ranking them across all ranks, with the mine side (lowest, highest, random) applied across the chosen count.
- **FR-007**: A twenty-item Tripmine board MUST fit entirely on one screen with no scrolling, scaling item size as needed.
- **FR-008**: The ten-item Tripmine behavior MUST be unchanged.
- **FR-009**: The header view switch MUST be visible only to the gamemaster and hidden from players and the spectator.
- **FR-010**: Activating the view switch as the gamemaster MUST open the chosen view in a new browser tab and MUST leave the gamemaster's current page on the gamemaster view.
- **FR-011**: Direct links to a player or spectator view MUST keep working regardless of the hidden switch.

### Key Entities *(include if feature involves data)*

- **Shipped list**: A seeded list whose items are now ordered by value.
- **Ordering value**: The numeric value of the column that defines a list's order.
- **Value display**: The readable text derived from a numeric value (thousands separators; thousand/Million/Billion/Trillion suffixes).
- **Connect column**: The name column and the value column; their rows must be equal-sized.
- **Tripmine item count**: Ten or twenty items for a Tripmine round, configured in setup.
- **View switch**: The header control that opens another role's view; gamemaster-only.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of shipped lists the values read in ascending order with zero decreases.
- **SC-002**: Connect value rows are the same size as name rows in 100% of rounds, so both columns look equally full.
- **SC-003**: A twenty-item Tripmine round shows all twenty items on one screen with no scrolling, at the standard game window size.
- **SC-004**: Ten-item Tripmine rounds behave exactly as before.
- **SC-005**: The view switch is visible to 0% of players/spectators and 100% of gamemasters.
- **SC-006**: Using the view switch opens a new tab 100% of the time and never navigates the gamemaster's current tab away.
- **SC-007**: Large shipped values display with a magnitude suffix (e.g. "146 Million", "1.3 Billion") in 100% of cases, and every shipped value has a finite number behind its display.

## Assumptions

- "Ordered by value" means ascending (low to high), matching the lists' declared `low_to_high` direction.
- The ordering column is the list's value column: Year (History), Height (Statues), Box Office (Cinema), Area (Countries). Changing this later means re-seeding.
- Shipped values are numbers; the display text is generated from the number (thousands separators and thousand/Million/Billion/Trillion suffixes), not stored by hand.
- Units belong to the column name, so a value like a height is stored as the number with the unit in the column label.
- The twenty-item Tripmine option is configured in the setup wizard; a quick game keeps the default ten unless a way to choose is added.
- The twenty-item board scales down item size to fit one screen rather than scrolling.
- The view switch is the header control that changes the viewed role; the gamemaster's own page is not navigated when it is used.
