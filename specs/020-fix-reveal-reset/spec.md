# Feature Specification: Fix Reveal Reset On Next Round

**Feature Branch**: `020-fix-reveal-reset`

**Created**: 2026-10-08

**Status**: Draft

**Input**: User description: "bug if i reveal the values as gamemaster and the go to next round the button stays 'values revealed'"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Revealing values is per round; the next round starts hidden (Priority: P1)

The gamemaster can reveal the round's values. That reveal belongs to the round. When the gamemaster moves
to the next round, the new round must start with its values hidden and the reveal control back to its
normal state, instead of staying on "Values revealed".

**Why this priority**: It is the reported bug: the reveal state leaks from one round into the next, so the
new round looks revealed when it is not.

**Independent Test**: As the gamemaster, reveal the values, go to the next round, and confirm the control
reads "Reveal values" again and the new round's values are hidden.

**Acceptance Scenarios**:

1. **Given** the gamemaster has revealed the values in a round, **When** the gamemaster moves to the next round, **Then** the reveal control reads "Reveal values" (not "Values revealed").
2. **Given** a new round has just started, **When** any role views the board, **Then** the round's values are hidden (as a fresh round).
3. **Given** the last round is crossed and the game finishes, **When** the game-end state is shown, **Then** the values are revealed for the final standings (unchanged).
4. **Given** the gamemaster has not revealed, **When** the next round starts, **Then** the control is unchanged ("Reveal values").

---

### Edge Cases

- Revealing in a non-final round then advancing must reset the reveal for the new round.
- Revealing in the final round then finishing the game keeps the reveal (the end dialog shows values).
- Advancing a round must not change scores, hearts, the turn, or the timer behavior beyond the reveal reset.
- A player's view of the new round is hidden even if the gamemaster had revealed the previous round.
- The reveal reset must hold whether the gamemaster revealed via the control or the game reached the reveal another way.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Starting a new round (advancing) MUST reset the reveal-values state so the new round starts hidden.
- **FR-002**: After advancing, the gamemaster's reveal control MUST read "Reveal values".
- **FR-003**: Every role's view of a freshly started round MUST show hidden values.
- **FR-004**: Finishing the game at the last round MUST still reveal the values (unchanged).
- **FR-005**: Advancing MUST NOT otherwise change round behavior (scores, hearts, turn, timer).

### Key Entities *(include if feature involves data)*

- **Reveal values**: The per-round flag that unlocks hidden values; reset when a new round starts.
- **Round advance**: Moving from the current round to the next configured round.
- **Game finish**: Crossing the last round, which reveals values for the final standings.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: After advancing, the reveal control reads "Reveal values" in 100% of rounds.
- **SC-002**: A freshly started round shows hidden values to every role.
- **SC-003**: Finishing at the last round still reveals the values.
- **SC-004**: No change to scores, hearts, the turn, or the timer from the reset.

## Assumptions

- The reveal flag is stored per session and applies to the current round; it is set by the gamemaster's reveal action and by finishing the game.
- "Advancing" is the gamemaster's next-round action; the last-round finish path is separate and keeps its reveal.
