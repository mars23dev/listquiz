# Feature Specification: Reveal Values In Tripmine And Connect

**Feature Branch**: `021-fix-reveal-tripmine-connect`

**Created**: 2026-10-08

**Status**: Draft

**Input**: User description: "bug if i reveal the values nothing is revealed in trip mine and connect"

## Clarifications

### Session 2026-10-08

- Q: What should "Reveal values" show in Tripmine and Connect? → A: All hidden information — Tripmine shows every item's value plus its rank/mine; Connect shows the name↔value pairing (answer key) to everyone.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Revealing values works in Tripmine and Connect (Priority: P1)

The gamemaster's "Reveal values" action reveals the hidden values in an Order round. In a Tripmine or
Connect round, pressing it does nothing: in Tripmine the values are sent but stay hidden, and in Connect
the value column is already public so the action has no effect. Revealing must reveal the hidden
information in every game mode.

**Why this priority**: It is the reported bug: the reveal action is a no-op in two of the three game types.

**Independent Test**: As the gamemaster, press "Reveal values" in a Tripmine round and a Connect round and
confirm the hidden information becomes visible to every role.

**Acceptance Scenarios**:

1. **Given** an Order round, **When** values are revealed, **Then** the values show (unchanged).
2. **Given** a Tripmine round, **When** values are revealed, **Then** the values that were hidden become visible to every role.
3. **Given** a Connect round, **When** values are revealed, **Then** the hidden information becomes visible to every role.
4. **Given** a reveal, **When** any role views the board, **Then** the revealed information is shown; nothing extra is revealed before the action.

---

### Edge Cases

- The reveal is per round and resets when the next round starts (feature 020).
- Revealing must not end the round or change scoring, hearts, the turn, or the timer.
- A player must not see the hidden information before the reveal.
- The gamemaster already sees everything; the reveal changes what players/spectator see.
- Finishing the game still reveals as it does today.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Revealing values MUST reveal the hidden information in Tripmine and Connect, as it already does in Order.
- **FR-002**: In Tripmine, revealing MUST show every item's value and its rank/mine to every role; in Connect, revealing MUST show the name↔value pairing (answer key) to every role.
- **FR-003**: Before the reveal, players and the spectator MUST NOT see the hidden information.
- **FR-004**: Revealing MUST NOT change scoring, hearts, the turn, the timer, or end the round.
- **FR-005**: The reveal reset per round (feature 020) MUST still hold.

### Key Entities *(include if feature involves data)*

- **Reveal values**: The per-round gamemaster action that unhides information for every role.
- **Tripmine hidden info**: Item values (and possibly ranks/the mine) hidden until picked.
- **Connect hidden info**: The name↔value pairing (the answer key); the values themselves are public.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Revealing shows the hidden information in Tripmine and Connect in 100% of rounds.
- **SC-002**: Before the reveal, the hidden information is never shown to players or the spectator.
- **SC-003**: Revealing changes nothing else (scoring, hearts, turn, timer, round end).
- **SC-004**: The per-round reveal reset still works.

## Assumptions

- The reveal is a per-round gamemaster action that unhides information without ending the round.
- In Connect the value column is public by design, so the reveal shows the pairing; in Tripmine the reveal shows the full answer (values, ranks, and the mine).
