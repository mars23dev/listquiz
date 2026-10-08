# Feature Specification: Value Column Labels

**Feature Branch**: `017-value-column-labels`

**Created**: 2026-10-08

**Status**: Draft

**Input**: User description: "regardless of the game mode the name of the value column has to be displayed e.g. If i have the countries by Area in square kilometers i only get the numbers displayed but not what they are meaning no unit etc this should be handled by giving the list column a name and display that name accordingly where the values are presented. I do not want to change the current visibility of the values, but whenever they are visible it should include the column name to now the unit"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Values show their column name in every game mode (Priority: P1)

Every list column has a name (for example "Area (km²)" or "Population"). When a round shows a value, the
room currently sees only the number ("652,230") without the column name, so the unit and meaning are
unclear. Wherever a value is displayed — in Order, Tripmine, or Connect — its column's name must be shown
too, so the room knows what the number means. Value visibility itself must not change: a hidden value
stays hidden, and the column name is only shown where a value is shown.

**Why this priority**: This is the whole request; it affects all game modes and removes the unit
confusion.

**Independent Test**: Start an Order, a Tripmine, and a Connect round on the Countries list using the Area
column and confirm that wherever an area value is shown, the name "Area (km²)" is shown with it, and that
no hidden value becomes visible.

**Acceptance Scenarios**:

1. **Given** a round whose column is "Area (km²)", **When** a value is shown in any game mode, **Then** the label "Area (km²)" is shown with it.
2. **Given** an Order round, **When** the anchor's value is shown, **Then** the column name is shown with it.
3. **Given** an Order round where values are hidden from a player, **When** the player views the board, **Then** no value and no column label appear for hidden items (visibility is unchanged).
4. **Given** a Connect round, **When** the value column is shown, **Then** the column name is shown as the heading of that column.
5. **Given** a Tripmine round, **When** a value is revealed (or the gamemaster sees all values), **Then** the column name is shown with the value.
6. **Given** the gamemaster's answer key shows values, **When** it is displayed, **Then** the column name is shown there too.

---

### Edge Cases

- Values hidden (players before reveal): no value and no label for hidden items; the label appears only where a value is visible.
- The anchor value is always public: its column name is always shown with it.
- The gamemaster sees every value: the column name is shown with those values too.
- A column with a long name (e.g. "Box Office ($)") must still display without breaking the layout.
- A column name with a unit (e.g. "Height (m)", "Area (km²)") is shown exactly as the list defines it.
- The label must not reveal a hidden value or the answer.
- The column name shown must be the one for the round's configured column, not another column of the list.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Wherever a value is displayed in any game mode (Order, Tripmine, Connect), the name of the value's column MUST be displayed with it.
- **FR-002**: Value visibility MUST NOT change: hidden values stay hidden and no label is shown for a hidden value.
- **FR-003**: The label MUST be the name of the round's configured column (the list column chosen for the round).
- **FR-004**: The Order board MUST show the column name with the values it displays (including the anchor).
- **FR-005**: The Tripmine board MUST show the column name with the values it displays (revealed values and the gamemaster's full view).
- **FR-006**: The Connect board MUST show the column name with its value column.
- **FR-007**: The gamemaster's answer key (Connect) and any gamemaster-only value display MUST show the column name too.
- **FR-008**: Showing the column name MUST NOT reveal a hidden value or otherwise change the game rules.

### Key Entities *(include if feature involves data)*

- **Value column**: A named column of a list (e.g. "Area (km²)", "Population", "Year").
- **Column name**: The display name of the value column, including any unit.
- **Value display**: Any place a value is shown in a game mode; each must carry the column name.
- **Round**: The current round, which fixes which column is in play.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of value displays across Order, Tripmine, and Connect, the column name is shown with the value.
- **SC-002**: Value visibility is unchanged: a hidden value is never shown, and its label is never shown either.
- **SC-003**: The label always matches the round's configured column.
- **SC-004**: The layout still fits one screen with the label added (no new scrolling).

## Assumptions

- The column name is displayed as a short label/heading near where the values are shown (for example the heading of the Connect value column, or a label beside the Order bar / Tripmine values), so it reads as the unit/meaning of the numbers.
- The column name is not a secret; it may be shown even to players, but only where a value is already visible (hidden values keep both hidden).
- Every shipped and user list column already has a name; no new naming is required.
