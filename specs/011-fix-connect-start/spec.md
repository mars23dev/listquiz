# Feature Specification: Fix Connect Round Start Panel

**Feature Branch**: `011-fix-connect-start`

**Created**: 2026-10-07

**Status**: Draft

**Input**: User description: "bug report for the gametype connect: When GM press start the start button does not disappear and for players: "Waiting for the game to be started…" does not disappear. If gm press Start a second time the game ends."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The start panel clears once a Connect round begins (Priority: P1)

In a Connect session the room first sees the pre-round prompt: the gamemaster sees a "Start Round"
button and everyone else sees "Waiting for the game to be started…". When the gamemaster starts the
Connect round, that prompt must disappear for every role and be replaced by the Connect board, exactly
as it does in the Order and Tripmine game types. Today the prompt stays on screen after the round has
started, and pressing the leftover button again ends the game instead of starting a round.

**Why this priority**: This is the reported bug: the Connect round is unplayable because the start
prompt never goes away and the only remaining visible action accidentally finishes the whole game.

**Independent Test**: Start a Connect session, have the gamemaster press "Start Round", and confirm
the gamemaster's button and the players' waiting message both vanish and the Connect board appears;
confirm pressing the old button location again cannot end the game.

**Acceptance Scenarios**:

1. **Given** a Connect session before the first round, **When** the board is shown, **Then** the gamemaster sees the "Start Round" button and the players see "Waiting for the game to be started…".
2. **Given** a Connect session before the first round, **When** the gamemaster starts the round, **Then** the start button disappears for the gamemaster and the Connect board is shown.
3. **Given** a Connect session before the first round, **When** the gamemaster starts the round, **Then** the waiting message disappears for every player and the Connect board is shown to them.
4. **Given** the gamemaster has just started a Connect round, **When** the gamemaster cannot see the old start control any more, **Then** no further "start" action is available that would advance or end the game.
5. **Given** a Connect session, **When** the gamemaster starts the round, **Then** the game does not end and the round proceeds normally.

---

### User Story 2 - Starting a Connect round matches the other game types (Priority: P2)

Order and Tripmine already hide the pre-round prompt as soon as their round starts. Connect must behave
identically: the pre-round prompt is shown only while no game type is active. This ensures consistent
behaviour across game types and prevents the prompt from reappearing once a round is under way.

**Why this priority**: Prevents the same class of defect (leftover pre-round controls) in Connect and
keeps all three game types consistent for the room.

**Independent Test**: Compare the Order, Tripmine, and Connect game types: in each, the pre-round prompt
is visible only before the round starts and is gone once any of them is active.

**Acceptance Scenarios**:

1. **Given** an Order, Tripmine, or Connect round is active, **When** the board is shown, **Then** the pre-round start prompt is not shown for any role.
2. **Given** no game type is active in a session that is not finished, **When** the board is shown, **Then** the pre-round start prompt is shown.

---

### Edge Cases

- Connect: the pre-round prompt must behave the same at the start of a multi-round game and at the start of a later Connect round.
- Connect: if the gamemaster starts a round while a previous state is still rendering, the prompt must never briefly persist after the round is active.
- Connect: when the session is finished, the pre-round prompt must remain hidden as it is today.
- Connect: a spectator must see the waiting message disappear the same way a player does.
- Connect: starting the round once must be sufficient; a second, distinct start action must not exist to end the game.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: After the gamemaster starts a Connect round, the "Start Round" control MUST no longer be shown to the gamemaster.
- **FR-002**: After the gamemaster starts a Connect round, the "Waiting for the game to be started…" message MUST no longer be shown to players or the spectator.
- **FR-003**: After a Connect round is active, the Connect board MUST be shown for every role in place of the pre-round prompt.
- **FR-004**: Starting a Connect round MUST NOT end or finish the game.
- **FR-005**: The pre-round prompt MUST be shown only when no game type (Connect, Order, or Tripmine) is active and the session is not finished.
- **FR-006**: The pre-round prompt behaviour for Connect MUST match the behaviour already established for Order and Tripmine.
- **FR-007**: Once a Connect round is active, there MUST be no visible action that advances or ends the game by pressing the former start control's location.

### Key Entities *(include if feature involves data)*

- **Pre-round prompt**: The panel shown before any round has started; the gamemaster sees a start control and everyone else sees a waiting message.
- **Active game type**: The current Connect, Order, or Tripmine state of the session; its presence means the pre-round prompt must be hidden.
- **Connect round start**: The gamemaster action that creates the Connect board and begins play.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In a Connect session, the pre-round prompt disappears for 100% of roles within one state update after the gamemaster starts the round.
- **SC-002**: Starting a Connect round ends the game in 0% of trials.
- **SC-003**: The pre-round prompt is visible only when no game type is active, across Order, Tripmine, and Connect alike.
- **SC-004**: A Connect round can be started exactly once per round with a single gamemaster action.

## Assumptions

- The bug applies to Connect only, because the pre-round prompt condition already excludes the Order and Tripmine game types.
- "Start Round" and the waiting message are the same pre-round prompt panel; hiding one hides the other for the appropriate role.
- The game ends only through the intended finishing rules, not as a side effect of starting a round.
- No change to the Connect gameplay, scoring, hearts, or the round-end rules is required; this is a display/visibility fix.
