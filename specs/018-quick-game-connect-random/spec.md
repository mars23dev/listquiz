# Feature Specification: Quick Game Connect And Random Lists

**Feature Branch**: `018-quick-game-connect-random`

**Created**: 2026-10-08

**Status**: Draft

**Input**: User description: "add connect also to the quickgame category. Make sure that the default lists for the quickgame modes are random so that the players have different rounds now and then and not replaying the same stuff over and over."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Connect is available as a quick game (Priority: P1)

The landing page offers quick games for Order and Tripmine but not Connect. Connect must be offered as a
quick game too, so a player can start a Connect round immediately with one team and no host, exactly like
the other quick game types.

**Why this priority**: It is the requested addition; Connect is already a supported game type but is
missing from the quick game choices.

**Independent Test**: From the landing page, start a Connect quick game and confirm a playable Connect
round begins immediately with one team and no host controls.

**Acceptance Scenarios**:

1. **Given** the landing page, **When** the quick game choices are shown, **Then** Order, Tripmine, and Connect are all offered.
2. **Given** the Connect quick game choice, **When** it is chosen, **Then** a Connect round starts immediately with one team and no host.
3. **Given** a Connect quick game, **When** it runs, **Then** it behaves like the other quick games (one team, one round, player view).

---

### User Story 2 - Quick games use a random default list (Priority: P2)

Today a quick game always picks the first default list that fits, so repeating the same quick game type
replays the same list. The default list for a quick game must be chosen at random from the lists that fit
the game type, so players see different content over time.

**Why this priority**: The user wants variety between quick games, not the same rounds every time.

**Independent Test**: Start the same quick game type several times and confirm the chosen list is not
always the same (when more than one list fits).

**Acceptance Scenarios**:

1. **Given** more than one default list fits a quick game type, **When** a quick game starts, **Then** the list is chosen at random among the fitting lists.
2. **Given** only one default list fits, **When** a quick game starts, **Then** that list is used.
3. **Given** no default list fits the game type, **When** a quick game is requested, **Then** it is refused with a clear message.
4. **Given** repeated quick games of the same type, **When** they run, **Then** they are not always on the same list.

---

### Edge Cases

- A quick Connect game needs a list with at least ten items that have values; only such lists are eligible.
- The random choice must consider only lists that fit the game type (enough items with values and at least one column).
- A list with too few items for the game type must never be chosen.
- When only one list fits, the result is deterministic (that list).
- The rest of the quick game behavior (single team, single round, immediate start, player view) is unchanged.
- Randomness must not make a quick game fail to start (a fitting list always exists when one is available).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The quick game choices MUST include Order, Tripmine, and Connect.
- **FR-002**: Starting a Connect quick game MUST begin a playable Connect round immediately (one team, one round, no host).
- **FR-003**: A quick game MUST choose its default list at random from the lists that fit the game type.
- **FR-004**: Only lists that fit the game type (enough items with values, at least one column) MUST be eligible for the random choice.
- **FR-005**: When only one list fits, that list MUST be used.
- **FR-006**: When no list fits, the quick game MUST be refused with a clear message.
- **FR-007**: Repeating a quick game of the same type MUST be able to produce different lists (variety).
- **FR-008**: The rest of the quick game behavior (single team, single round, immediate start, player view) MUST stay unchanged.

### Key Entities *(include if feature involves data)*

- **Quick game**: A one-team, one-round session of a chosen game type on a default list, started immediately.
- **Game type**: Order, Tripmine, or Connect.
- **Eligible list**: A default list that fits the game type (enough items with values and at least one column).
- **Random list choice**: The random selection among eligible lists for a quick game.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The landing page offers Connect as a quick game alongside Order and Tripmine.
- **SC-002**: A Connect quick game starts a playable Connect round 100% of the time (when a fitting list exists).
- **SC-003**: Across repeated quick games of one type (with multiple fitting lists), more than one distinct list is used.
- **SC-004**: A quick game never chooses a list that does not fit the game type.
- **SC-005**: Existing quick game behavior (one team, one round, immediate start) is unchanged.

## Assumptions

- Connect is already a supported game type; only the quick game entry point and the list choice change.
- "Random" means uniformly chosen among the eligible default lists each time a quick game starts.
- The eligible set is the shipped default lists (quick games use default lists).
- Variety is expected across separate quick games, not within a single game.
