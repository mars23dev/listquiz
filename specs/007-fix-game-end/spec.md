# Feature Specification: Fix Game End After The Last Round

**Feature Branch**: `007-fix-game-end`

**Created**: 2026-10-07

**Status**: Draft

**Input**: User description: "findings/improvemnets:
In quick mode the game end screen does not appear in general after last round of session the new dialog should pop up"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The game-end dialog appears when the last round ends (Priority: P1)

When the final round of a session is over, the whole game is over, so the game-end dialog should pop up for everyone. Today the dialog only appears after the gamemaster presses "Next Round" / "Finish Game", so it never appears in a quick game (which has no gamemaster) and is easily missed in a normal game. The end of the last round should finish the game by itself and show the dialog.

**Why this priority**: This is the payoff screen of the whole game; in quick mode it never appears at all, so the game has no ending.

**Independent Test**: Play a one-round session (a quick game is the simplest) to the end of its round and confirm the game-end dialog appears for every role without any gamemaster action; play a multi-round session and confirm ending a non-final round does not show it.

**Acceptance Scenarios**:

1. **Given** a session whose last round has just ended (its board is complete or no team can continue), **When** the round ends, **Then** the game is finished and the game-end dialog appears for every role.
2. **Given** a quick game (one team, one round, no gamemaster), **When** the round ends, **Then** the game-end dialog appears for the player without any further action.
3. **Given** a multi-round session, **When** a round that is not the last one ends, **Then** the game is not finished and the game-end dialog does not appear.
4. **Given** the game-end dialog has appeared, **When** it is dismissed, **Then** the finished session is cleaned up as before.

---

### Edge Cases

- The last round ends either because every item is placed or because no team has hearts left (Order), or because the Tripmine round completes; all of these must finish the game when it is the last round.
- The last round ends in an Order round and in a Tripmine round alike.
- A game finished this way still reveals values and reports the final standings as before.
- If the gamemaster had already finished the game manually, ending the round does not finish it twice or error.
- A multi-round game still lets the gamemaster move between rounds as before.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: When the last configured round of a session ends, the session MUST become finished without any gamemaster action.
- **FR-002**: When the session becomes finished this way, the game-end dialog MUST appear for every role.
- **FR-003**: A round that is not the last configured round MUST NOT finish the game when it ends.
- **FR-004**: This MUST hold for both game types (Order and Tripmine).
- **FR-005**: Finishing at the last round MUST reveal values and expose the final standings as the existing finish path does.
- **FR-006**: Dismissing the game-end dialog MUST still delete the finished session (including in quick mode).
- **FR-007**: Finishing at the last round MUST be safe when the game is already finished (no double finish, no error).

### Key Entities *(include if feature involves data)*

- **Round end**: The moment a round's board is complete or no team can continue.
- **Session finish**: The state where no further rounds remain; it drives the game-end dialog and the finished-session cleanup.
- **Last round**: The final configured round of a session; its end is what finishes the game.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A one-round quick game shows the game-end dialog after its round ends, with zero gamemaster actions.
- **SC-002**: A multi-round game does not show the dialog when a non-final round ends.
- **SC-003**: The dialog appears for every role (gamemaster, players, spectator) at game end.
- **SC-004**: Dismissing the dialog at the end of a quick game removes the session.

## Assumptions

- "The last round of the session" means the final configured round (round number equals the session's round count).
- Ending the last round is equivalent to the gamemaster finishing the game, so it uses the same finish behavior (reveal values, clear the active team, show the end dialog).
- Between-rounds behavior for non-final rounds is unchanged: the gamemaster still advances to the next round.
