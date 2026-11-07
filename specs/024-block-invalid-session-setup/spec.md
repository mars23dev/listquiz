# Feature Specification: Block Invalid Session Setup

**Feature Branch**: `024-block-invalid-session-setup`

**Created**: 2026-11-07

**Status**: Draft

**Input**: User description: "Block session setup whenever selected list cannot support the chosen game mode and item count (e.g. Tripmine 20 needs >=20 valued items). Invalid selection must prevent session creation, not fail at round start."

## Clarifications

### Session 2026-11-07

- Q: When a selected list cannot support a game mode or item count, how should setup show that choice? → A: Hide lists that cannot support the chosen game mode at all; keep a list visible when it supports at least one item-count variant, and show an error when the gamemaster selects an unsupported variant.
- Q: When changing the game mode, item count, or column makes the current list incompatible, what should setup do with that round's list selection? → A: Keep the incompatible selection, show an error, and block continuing until the gamemaster changes the mode, item count, column, or list.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Prevent creating a session from an undersized list (Priority: P1)

A gamemaster configures a round with a list that cannot supply enough valued items for the selected game mode and item count. The setup flow refuses the invalid selection, so no session is created and no player links are issued.

**Why this priority**: This is the reported defect and the core safety rule. A session that exists but cannot start traps the gamemaster and every invited player in a dead game.

**Independent Test**: Select a list with fewer than 20 valued items, choose Tripmine with 20 items, and attempt to create the session. Confirm the attempt is refused, no session appears, and no player links are produced.

**Acceptance Scenarios**:

1. **Given** a list with 19 valued items in the selected column, **When** the gamemaster selects Tripmine with 20 items, **Then** setup shows an error that 20 items are required and session creation is blocked.
2. **Given** the same list, **When** the gamemaster selects Tripmine with 10 items, **Then** setup remains valid because 10 items are available.
3. **Given** an invalid round selection, **When** the gamemaster tries to proceed, **Then** no session is created and no gamemaster or player links are shown.
4. **Given** a valid round selection at the exact minimum for its mode, **When** the gamemaster creates the session, **Then** the session is created as it is today.

---

### User Story 2 - Reject invalid setup through any entry point (Priority: P1)

Any session-creation path, including a standard setup submission or a quick game, refuses a round whose list cannot supply the required number of valued items. Validation happens before any session, team, or round configuration is stored.

**Why this priority**: Guarding only the visible setup screen leaves the invalid state reachable through direct or alternate requests. The rule must hold wherever sessions are created.

**Independent Test**: Submit a session-creation request containing one round whose list is one item short. Confirm the request is refused, no session record exists afterward, and the response identifies the failing round and the shortfall.

**Acceptance Scenarios**:

1. **Given** a session-creation attempt with one undersized round, **When** it is submitted, **Then** it is refused with an error naming the round, game mode, required count, and available valued-item count.
2. **Given** a multi-round setup where only the third round is invalid, **When** the gamemaster submits, **Then** the whole attempt is refused and no earlier round is stored.
3. **Given** a quick game whose chosen default list is too small for its mode, **When** the quick game starts, **Then** the invalid session is refused before it is created.
4. **Given** a refused attempt, **When** the gamemaster corrects the list or the item count, **Then** the corrected setup can create a session normally.

---

### User Story 3 - Hide unsupported lists and flag unsupported sizes (Priority: P2)

While configuring a round, the list picker only offers lists that can support the chosen game mode at its smallest item count. A list that supports one Tripmine size but not the other stays visible; choosing the unsupported size shows an error and blocks further progress. If a configuration change makes an already-selected list incompatible, that list stays selected and the same error blocks progress until the gamemaster corrects the configuration.

**Why this priority**: Preventing submission is mandatory; hiding impossible lists and explaining rejected sizes avoids repeated failed attempts and makes the rule understandable.

**Independent Test**: Choose Order with a list of 10 valued items and confirm the list is hidden. With a list of 15 valued items and Tripmine selected, confirm the list stays visible, selecting 20 items shows an error, and selecting 10 items works.

**Acceptance Scenarios**:

1. **Given** a list with 10 valued items in the selected column, **When** the gamemaster configures an Order round, **Then** that list is hidden from the list picker.
2. **Given** a list with 15 valued items in the selected column, **When** the gamemaster configures a Tripmine round, **Then** the list stays visible because the 10-item size is supported.
3. **Given** that visible list and Tripmine selected, **When** the gamemaster selects the 20-item size, **Then** setup shows an error explaining the shortfall and blocks continuing.
4. **Given** a list that cannot support any size of the chosen game mode, **When** the gamemaster configures that mode, **Then** the list is hidden.
5. **Given** the shortfall error is shown, **When** the gamemaster selects a supported size or a list that supports the chosen size, **Then** the error clears and continuing is allowed.
6. **Given** a compatible list is selected, **When** a mode, item-count, or column change makes that list incompatible, **Then** the list stays selected, an error explains the shortfall, and continuing is blocked until the configuration is corrected.
7. **Given** no list can support the chosen game mode, **When** the gamemaster views the list picker, **Then** setup shows that no compatible list exists and blocks continuing.

---

### Edge Cases

- A list has enough total items but too few items carrying a value in the selected column: only valued items count toward the requirement, and the list is hidden for that column.
- A list supports Tripmine 10 but not Tripmine 20: the list stays visible, selecting 20 shows an error, and the 10-item choice remains usable.
- A list supports Tripmine 20 but the selected column has fewer than 20 values: the selected column, not the list total, decides.
- Multiple rounds are configured and only one is invalid: the whole session creation is refused.
- The list or column changes after setup loads but before submission: creation is refused using the current stored data.
- A list or column is deleted after setup loads but before submission: creation is refused as invalid.
- A list has zero valued items in the selected column: the list is hidden for every game mode.
- A quick game has no default list large enough for the requested mode: the quick game is refused before a session exists.
- A list meets the exact minimum: the list is visible and the session is allowed.
- Changing the selected column can hide or reveal a list because valued-item counts differ per column.
- A selected list becomes incompatible after changing the mode, item count, or column: the list stays selected with an error and continuing is blocked.
- No list is compatible with the chosen game mode: setup shows that no compatible list exists and blocks continuing.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST determine the minimum number of valued items each game mode needs: Order needs 11, Connect needs 10, Tripmine needs 10 when the 10-item option is chosen, and Tripmine needs 20 when the 20-item option is chosen.
- **FR-002**: The system MUST count only items that have a value in the selected round column when judging whether a list is large enough.
- **FR-003**: Session setup MUST prevent creating a session when any configured round's selected list and column cannot supply the required number of valued items for that round's game mode and item count.
- **FR-004**: A session-creation attempt containing at least one invalid round MUST be refused entirely; no session, teams, round configuration, or player links may be stored or issued.
- **FR-005**: A refusal MUST identify the failing round and state the game mode, required item count, and available valued-item count.
- **FR-006**: The rule MUST apply to every path that creates a session, including standard setup and quick game.
- **FR-007**: The rule MUST be enforced using current stored list and column data at the moment of session creation, not only data loaded when setup opened.
- **FR-008**: When offering list choices, setup MUST hide every list that cannot supply the minimum valued-item count for the chosen game mode's smallest item-count variant, using the selected column: Order 11, Connect 10, Tripmine 10.
- **FR-009**: Setup MUST keep a list visible when it supports at least one item-count variant of the chosen game mode.
- **FR-010**: When the gamemaster selects an item-count variant that a visible list cannot support, setup MUST display an error explaining the shortfall.
- **FR-011**: Setup MUST block continuing while any selected round is unsupported.
- **FR-012**: A valid setup at or above every minimum MUST create a session exactly as it does today.
- **FR-013**: An invalid selection MUST be detected during session setup and MUST NOT be deferred to round start.
- **FR-014**: When a list or column changes so that a previously valid selection becomes invalid, the next session-creation attempt MUST be refused.
- **FR-015**: When a mode, item-count, or column change makes the currently selected list incompatible, setup MUST keep that list selected, show the shortfall error, and block continuing until the gamemaster changes the mode, item count, column, or list.
- **FR-016**: When no list can support the chosen game mode, setup MUST show that no compatible list exists and block continuing.
- **FR-017**: Setup MUST keep the list, game-mode, item-count, and column controls operable by keyboard and MUST expose validation errors to assistive technology.

### Key Entities

- **List**: A named collection of items with one or more value columns and a count of items.
- **Item Value**: One item's value in one column; only items carrying a value in the selected column can be used by a round.
- **Round Configuration**: The game mode, list, column, and for Tripmine the item count selected for one round.
- **Session**: The created game containing teams, round configurations, and role links; it exists only after every round passes validation.
- **Game Mode Requirement**: The minimum valued-item count a mode needs, including the Tripmine 10 and Tripmine 20 variants.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of verification attempts, a setup containing a round with too few valued items is refused and no session is created.
- **SC-002**: Zero invalid sessions reach the round-start step in verification runs; every invalid selection is stopped during setup.
- **SC-003**: 100% of refusal messages identify the failing round and the required versus available valued-item count.
- **SC-004**: 100% of valid setups at or above the minimum for each mode create a session successfully, with no change to existing valid behavior.
- **SC-005**: The list picker only offers lists that can support the chosen game mode, and 100% of incompatible selections or configuration changes show a shortfall error that blocks continuing.
- **SC-006**: A quick game that lacks a large enough default list is refused before any session exists in 100% of verification runs.

## Assumptions

- The minimums are the existing game rules: Order 11 valued items, Connect 10 valued items, Tripmine 10 or 20 valued items according to the chosen item count.
- A list has one selected column per round, and only values in that column count toward the requirement.
- Lists and columns can be edited between opening setup and submitting; the stored data at submission time is authoritative.
- Lists that shrink after a session already exists are outside this feature; existing round-start behavior remains for that separate situation.
- List-picker eligibility uses the smallest item-count variant of the chosen game mode; a larger Tripmine size is validated after it is selected.
- Unsupported item-count variants remain selectable so the gamemaster can see the specific shortfall error; continuing stays blocked until the selection is corrected.
- A list selected before it became incompatible stays visible and selected with an error, even though the picker would not offer that list for a fresh selection.
- If no list is compatible with the chosen game mode, setup shows an empty-state explanation instead of an empty picker.
- All session-creation paths share the same underlying validation rule and error behavior.
