# Feature Specification: Select Custom List In Session Setup

**Feature Branch**: `022-fix-custom-list-setup`

**Created**: 2026-11-03

**Status**: Draft

**Input**: User description: "I created my own list called 'Test', added names and values, saved the list, and now I am unable to select the list during setup."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Select a custom list for a round (Priority: P1)

A gamemaster creates a protected folder and saves a custom list inside it. In the setup wizard, only the shipped default lists are offered, so the custom list cannot be used. After the fix, the gamemaster can choose a protected folder, enter its password, and select any list from that folder as a round source.

**Why this priority**: This is the reported bug. Without it, every user-created list is unusable and the main authoring workflow never reaches gameplay.

**Independent Test**: Create a protected folder containing a list named "Test", open the setup wizard, unlock the folder, confirm "Test" appears in the List selector, select it with a value column, and create a session.

**Acceptance Scenarios**:

1. **Given** the setup wizard, **When** it loads, **Then** the shipped default lists remain selectable without a password.
2. **Given** at least one protected folder, **When** the setup wizard shows the folder choices, **Then** the protected folder is visible but its list titles are hidden until its password is entered.
3. **Given** a protected folder, **When** the gamemaster enters the correct folder password, **Then** every list in that folder becomes selectable as a round source with its title and item count.
4. **Given** a list with more than one value column, **When** the gamemaster selects that list, **Then** the Value Column selector offers that list's columns.
5. **Given** an unlocked folder and a selected custom list, **When** the session is created, **Then** the session uses that list and its chosen column and can start normally.
6. **Given** an unlocked folder, **When** the gamemaster configures additional rounds, **Then** the folder's lists remain available without re-entering the password.

---

### User Story 2 - Keep protected lists protected during setup (Priority: P2)

The folder password is a privacy boundary. Adding custom lists to setup must not expose their titles, item counts, or contents before the folder is unlocked, and a session must never be created from a list in a folder that was not unlocked.

**Why this priority**: A fix that makes custom lists visible without their folder password would break the existing privacy guarantee.

**Independent Test**: Open the setup wizard with a locked folder and inspect every visible control and network response; confirm no list names or item counts from that folder appear until the correct password is entered. Then attempt to create a session with a list identifier from that folder without unlocking it and confirm the request is refused.

**Acceptance Scenarios**:

1. **Given** a locked protected folder in setup, **When** the gamemaster views the page, **Then** no list titles or item counts from that folder are shown.
2. **Given** a locked protected folder, **When** the gamemaster enters a wrong password, **Then** the folder stays locked, the lists stay hidden, and a clear error is shown.
3. **Given** a locked protected folder, **When** a session request names one of its list identifiers directly, **Then** the request is refused and no session is created.
4. **Given** a folder unlocked earlier in the wizard, **When** the page is reloaded, **Then** the folder is locked again and its lists require the password again.

---

### User Story 3 - Use several folders and lists in one setup (Priority: P3)

A gamemaster may keep lists in several folders and use them across different rounds. Unlocking one folder must not unlock another, and changing a round's folder or list must leave the round in a valid state.

**Why this priority**: Multi-folder use is a natural extension of the authoring workflow, but the core bug is already fixed once one custom list can be selected.

**Independent Test**: Create two protected folders with different passwords and one list in each. Unlock both in one setup session, configure two rounds using a list from each folder, and create the session.

**Acceptance Scenarios**:

1. **Given** two protected folders, **When** the gamemaster unlocks one, **Then** only that folder's lists become visible.
2. **Given** two unlocked folders, **When** the gamemaster configures two rounds, **Then** each round can use a list from either folder.
3. **Given** a round with a custom list selected, **When** the gamemaster switches to a list with a different set of columns, **Then** the round's value column is reset to a valid choice or must be chosen again before the session can be created.
4. **Given** two folders contain lists with the same title, **When** the gamemaster chooses a list, **Then** the folder context makes the intended list distinguishable.

---

### Edge Cases

- No protected folders exist: setup behaves exactly as before, offering the default folder's lists.
- A protected folder exists but contains no lists: it can be unlocked but contributes no selectable lists.
- A custom list has one value column: that column is selected automatically and no empty column picker is required.
- A custom list has several value columns: the gamemaster must choose exactly one for the round.
- The password is wrong, empty, or only whitespace: the folder stays locked and no list information is revealed.
- The same password is used for two folders: each folder must still be unlocked separately.
- The gamemaster unlocks a folder, then deletes or renames the list in another tab before creating the session: session creation is refused with a clear message rather than silently substituting another list.
- The gamemaster unlocks a folder, then the folder or list is deleted before session creation: the request fails with a clear error.
- The gamemaster reloads or leaves the setup wizard: every folder locks again and passwords are forgotten.
- A custom list contains fewer than 11 or more than 100 items: existing list validation still applies.
- Existing default-list selection, round configuration, team setup, and link generation continue to work unchanged.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The setup wizard MUST offer source lists from the default folder and from every custom folder, not from the default folder alone.
- **FR-002**: A protected folder MUST remain locked, and its list titles, item counts, columns, and contents MUST NOT be exposed, until the correct folder password is entered.
- **FR-003**: The gamemaster MUST be able to choose a protected folder and enter its password from within the setup wizard.
- **FR-004**: A correct folder password MUST unlock that folder and make all of its lists selectable as round sources.
- **FR-005**: A wrong, empty, or whitespace-only password MUST leave the folder locked, reveal no list information, and show a clear error.
- **FR-006**: The default folder MUST remain selectable without a password and MUST remain read-only.
- **FR-007**: Password checks MUST be scoped to one folder: entering the password for one folder MUST NOT unlock any other folder.
- **FR-008**: After a folder is unlocked, its lists MUST remain available to every round in the current setup session without re-entering that folder's password.
- **FR-009**: Each selectable list MUST show enough information for the gamemaster to identify it, including its title and item count, and its folder when titles repeat.
- **FR-010**: When a selected list exposes more than one value column, the gamemaster MUST choose exactly one value column for the round.
- **FR-011**: When the selected list exposes one value column, that column MUST be selected automatically.
- **FR-012**: Changing a round's list MUST NOT leave an invalid value column attached to that round.
- **FR-013**: The existing rule that no session can be created without a list and a value column for every round MUST remain enforced.
- **FR-014**: Session creation MUST refuse a list whose folder was not unlocked as part of that setup, even when the request names a valid list identifier directly.
- **FR-015**: Session creation MUST fail with a clear error if the selected folder, list, or column no longer exists.
- **FR-016**: Unlocking a folder in the setup wizard MUST NOT persist its password beyond the current wizard session; a reload or a new setup session MUST require the password again.
- **FR-017**: Folder passwords MUST NOT appear in URLs, browser history, or user-visible text.
- **FR-018**: Unlocking folders and selecting custom lists MUST NOT modify list data, list ownership, or folder permissions.
- **FR-019**: All new controls MUST be reachable and operable by keyboard alone with visible focus, and the setup page MUST remain responsive during password checks and list loading.
- **FR-020**: A session created from a custom list MUST draw its round items and values from that list exactly as a session created from a default list does.

### Key Entities *(include if feature involves data)*

- **Folder**: A container of lists. The default folder is password-free and read-only; custom folders are password-protected.
- **List**: An ordered set of 11-100 items with a title and one or more value columns, belonging to exactly one folder.
- **Setup session**: The temporary state of the setup wizard, including which folders the gamemaster has unlocked during that visit. It is not persisted.
- **Round configuration**: One round's quiz type, source list, and chosen value column, plus the game-specific options for that type.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 100% of lists in a correctly unlocked custom folder can be selected as round sources in setup.
- **SC-002**: Before a folder is unlocked, 0 list titles or item counts from that folder are visible in the setup wizard or returned to it.
- **SC-003**: A wrong folder password keeps the folder locked in 100% of attempts and shows a clear error.
- **SC-004**: A session created from a custom list uses the selected list and column in 100% of rounds.
- **SC-005**: The existing default-folder setup flow passes unchanged, including round count, team count, and link generation.
- **SC-006**: A gamemaster can unlock one folder once and use its lists across every configured round without re-entering the password.
- **SC-007**: No protected list information is exposed through direct session-creation requests without a successful folder unlock.

## Assumptions

- Custom lists live only in protected folders; the default folder cannot receive user lists.
- The gamemaster knows the folder password. There is no password recovery, consistent with the existing folder model.
- Folder unlock state is per setup wizard session and is not shared with List Management or persisted across reloads.
- Folder names may be listed before unlock, but list names and contents are protected.
- The existing list and folder data, validation, and permission rules are reused; this feature only changes which lists setup can reach.
- Out of scope: moving lists between folders, changing folder passwords, password recovery, and changing which folder is the default.
