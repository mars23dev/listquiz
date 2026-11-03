# Feature Specification: Per-Game Ordering

**Feature Branch**: `019-per-game-ordering`

**Created**: 2026-10-08

**Status**: Draft

**Input**: User description: "Remove it make the ordering per game ASC, DESC depending what is required for game mode"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Lists no longer carry a direction; each game mode orders (Priority: P1)

Today a list has a single "higher to lower / lower to higher" choice, chosen when the list is created or
imported and used to order an Order round. With multiple value columns and per-round column choice, one
list-level direction cannot fit every column. The direction choice should be removed, and each game mode
should define the order it needs (ascending or descending) itself.

**Why this priority**: It removes a now-meaningless choice and makes ordering a property of the game mode,
which is the user's request.

**Independent Test**: Create or import a list and confirm there is no direction choice; run an Order, a
Tripmine, and a Connect round and confirm each orders as its game mode defines (all ascending today).

**Acceptance Scenarios**:

1. **Given** list creation, **When** the form is shown, **Then** there is no higher/lower direction choice.
2. **Given** CSV import, **When** a list is imported, **Then** no direction is required or stored.
3. **Given** a list, **When** it is viewed or edited, **Then** no direction field is shown.
4. **Given** an Order round, **When** the board is built, **Then** the items are ordered by the game mode's direction (ascending: left is lowest, right is highest).
5. **Given** a Tripmine round, **When** the grid is ranked, **Then** ranks go ascending (1 lowest … N highest).
6. **Given** a Connect round, **When** the value column is shown, **Then** values read ascending.
7. **Given** an existing list that had a direction, **When** a round runs, **Then** it uses the game mode's direction (the stored direction is gone).

---

### Edge Cases

- Existing databases that still have the old direction field must upgrade without error.
- No game mode's behavior is otherwise changed: scoring, hearts, the Tripmine side (highest/lowest/random), and matching stay the same.
- The Order board still shows the lowest value on the left and the highest on the right (ascending).
- A list with multiple columns has no single direction stored; the round's column is ordered by the game mode.
- The direction is defined in one place per game mode, so a future mode can choose descending without a list field.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The list-level order direction (higher/lower) MUST be removed from list creation, list editing, and CSV import.
- **FR-002**: The system MUST NOT require or store an order direction on a list.
- **FR-003**: Each game mode MUST define the ordering direction it uses: Order ascending, Tripmine ascending, Connect ascending.
- **FR-004**: Order rounds MUST order the round's column ascending (left = lowest, right = highest), regardless of the old list direction.
- **FR-005**: Tripmine rounds MUST rank ascending (1 = lowest … N = highest), unchanged.
- **FR-006**: Connect rounds MUST show the value column ascending, unchanged.
- **FR-007**: An existing database with the old direction field MUST upgrade cleanly (no error), and the field MUST no longer affect rounds.
- **FR-008**: Removing the direction MUST NOT change scoring, hearts, the Tripmine side, or matching.

### Key Entities *(include if feature involves data)*

- **List**: No longer has an order direction.
- **Game mode ordering**: The ascending/descending order a game mode applies to the round's column (a property of the game mode).
- **Round column**: The list column chosen for the round; ordered by the game mode.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: No direction choice appears in list creation, editing, or import (0 occurrences).
- **SC-002**: Order, Tripmine, and Connect rounds order their column as the game mode defines (all ascending today).
- **SC-003**: An existing database upgrades without error and rounds behave the same as with the old ascending lists.
- **SC-004**: No change to scoring, hearts, the Tripmine side, or matching.

## Assumptions

- All three game modes currently require ascending order (Order left-to-right low-to-high; Tripmine rank 1 lowest; Connect values low-to-high), so the per-mode direction is ascending for each; defining it per mode lets a future mode choose descending.
- Removing the choice means a list previously set to high-to-low now runs ascending in Order; this is accepted.
- The old direction field is dropped from the schema via a migration.
