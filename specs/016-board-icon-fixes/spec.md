# Feature Specification: Board Icon And Layout Fixes

**Feature Branch**: `016-board-icon-fixes`

**Created**: 2026-10-07

**Status**: Draft

**Input**: User description: "1. The icons for the countries where not shown in all gamemodes the icons are part of the list and should be loaded by the gametype the gametype should not override the icon, but use what is added there. If there was some overriding done (which is an assumption) you can add these icons to the list (do not do it for the countries this shall stay the flags) 2. Tripmine you have extended to 20 items but not the bar which still shows 10 even though you've selected 20. 3. Connect the items on the left still not have the same size / alignment as the items on the right this was already reported previously"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Every game type shows each item's own icon (Priority: P1)

An item's icon belongs to the list. Every game type — Order, Tripmine, and Connect — must show the icon
that was added to the item, and must not replace it with its own icon. Today Order and Tripmine show a
generic placeholder instead of the item's icon, while Connect shows the real icon (the flags), so the
same list looks different depending on the game type. Country items must keep showing their flags.

**Why this priority**: It is a visible correctness bug across two of the three game types and makes the
flags disappear.

**Independent Test**: Start an Order round, a Tripmine round, and a Connect round on the Countries list
and confirm every item shows its flag in all three; repeat on another shipped list and confirm each item
shows its own icon.

**Acceptance Scenarios**:

1. **Given** a list whose items carry icons, **When** an Order round runs, **Then** each item shows its own icon.
2. **Given** a list whose items carry icons, **When** a Tripmine round runs, **Then** each item shows its own icon.
3. **Given** the Countries list, **When** any of the three game types runs, **Then** each country shows its flag.
4. **Given** an item with no icon, **When** it is shown, **Then** the existing fallback icon is used (no error).
5. **Given** an item that has an icon, **When** a game type would otherwise draw its own icon, **Then** the item's icon is used instead.

---

### User Story 2 - The Tripmine bar matches the item count (Priority: P2)

When a Tripmine round is set to twenty items, the rank bar must show twenty positions, not ten. Today the
bar always shows ten positions regardless of the chosen count, so a twenty-item round has a bar that does
not match the board.

**Why this priority**: A direct mismatch between the chosen setting and what the room sees.

**Independent Test**: Start a twenty-item Tripmine round and confirm the rank bar shows twenty positions;
start a ten-item round and confirm it shows ten.

**Acceptance Scenarios**:

1. **Given** a twenty-item Tripmine round, **When** the board is shown, **Then** the rank bar has twenty positions (1–20).
2. **Given** a ten-item Tripmine round, **When** the board is shown, **Then** the rank bar has ten positions (1–10).
3. **Given** a twenty-item round, **When** the mine position is marked, **Then** it is marked on the correct position within the twenty.

---

### User Story 3 - Connect columns are the same size and aligned (Priority: P2)

On the Connect board the name column and the value column must be exactly the same size, with rows aligned
one-to-one. Today the left (name) rows are taller than the right (value) rows, so the columns look
mismatched and misaligned (see the reported screenshot).

**Why this priority**: A repeated visual defect that makes the board look broken.

**Independent Test**: Open a Connect round and confirm each name row is the same height as the value row beside it and that the rows line up across both columns.

**Acceptance Scenarios**:

1. **Given** a Connect round, **When** the board is shown, **Then** every name row is the same height as the corresponding value row.
2. **Given** a Connect round, **When** the two columns are compared, **Then** row N on the left lines up with row N on the right.
3. **Given** a Connect round with an icon on the name side, **When** the rows render, **Then** the icon does not make the name row taller than the value row.
4. **Given** a Connect round as the gamemaster (with the answer key), **When** the rows render, **Then** the rows stay equal and aligned.

---

### Edge Cases

- Icons: an item that has both no icon name and no uploaded image still shows the fallback, in every game type.
- Icons: an uploaded icon image is used when present, exactly as an icon name is.
- Icons: adding icons to shipped items must not change the Countries flags (they stay flags).
- Icons: the Order anchor item and any staged/pending item should show their icon too, consistently with the pool items.
- Tripmine bar: with twenty items the bar must still fit one screen (no horizontal scrolling) alongside the twenty-item board.
- Tripmine bar: a round that has not started yet shows no bar or an empty bar consistently.
- Connect: equal sizing must hold for a full board, a partly matched board, and with a staged name/value.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Every game type (Order, Tripmine, Connect) MUST show each item's own icon from the list.
- **FR-002**: A game type MUST NOT replace an item's icon with its own icon when the item has one.
- **FR-003**: Country items MUST keep showing their flags in every game type.
- **FR-004**: If a shipped list item has no icon, one MUST be added to the list (Countries excepted — their flags stay).
- **FR-005**: An item with no icon MUST still render the existing fallback icon, in every game type.
- **FR-006**: The Tripmine rank bar MUST show exactly one position per item: ten for a ten-item round and twenty for a twenty-item round.
- **FR-007**: The Tripmine mine position MUST be marked on the correct position within the chosen count.
- **FR-008**: The Connect name row and value row MUST be the same height, and the rows MUST align one-to-one across the two columns.
- **FR-009**: A Connect item's icon MUST NOT make its row taller than the value row.
- **FR-010**: The icon fix MUST NOT change any other board content, scoring, or rules.

### Key Entities *(include if feature involves data)*

- **Item icon**: The icon (or flag) defined on the list item; shown by every game type.
- **Game board**: The Order, Tripmine, or Connect view that renders items and their icons.
- **Tripmine rank bar**: The progress ladder with one position per item (10 or 20).
- **Connect row**: A name row and its matching value row, which must be equal height and aligned.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In Order, Tripmine, and Connect, 100% of items that have an icon show that icon (no generic placeholder).
- **SC-002**: Country flags show in 100% of the three game types.
- **SC-003**: A twenty-item Tripmine round shows a twenty-position bar; a ten-item round shows ten.
- **SC-004**: Connect name and value rows are the same height in 100% of rounds, with rows aligned.
- **SC-005**: No change to scoring, hearts, matching, or turn rules.

## Assumptions

- The "icon not shown" bug affects Order and Tripmine because their item data does not carry the list's icon; Connect already carries it (the flags show there).
- Every shipped item already has an icon name (or a flag), so "add icons" is a safety net for any item missing one; the Countries list is never changed (flags stay).
- The Tripmine bar should fit one screen for twenty items, like the board.
- Equal Connect rows means an identical fixed row height for both columns, with content vertically centered.
