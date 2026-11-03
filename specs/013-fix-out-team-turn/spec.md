# Feature Specification: Fix Eliminated Team Turn Switch

**Feature Branch**: `013-fix-out-team-turn`

**Created**: 2026-10-07

**Status**: Draft

**Input**: User description: "bug if a team has no hearts (order or connect) left he is out of the game and only the other teams can continue, you can break that function when game master switches the turn to a team that is out of the game. They can continue playing dispite being out which should not happen."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - An eliminated team cannot be put back on turn (Priority: P1)

In an Order or Connect round a team that has lost all its hearts is out for the rest of the round; only
the teams that still hold hearts keep playing. The turn normally rotates past an eliminated team by
itself, but the gamemaster can still hand the turn to that team with the turn switch, which lets an
eliminated team keep playing. The system must refuse to put an eliminated team on turn so this cannot
happen.

**Why this priority**: This is the reported bug: an eliminated team can rejoin play through the
gamemaster's turn control, breaking the round's elimination rule.

**Independent Test**: In an Order or Connect round, drive a team to zero hearts and have the gamemaster
try to switch the turn to it; confirm the team is not put on turn and the round continues with a team
that still has hearts.

**Acceptance Scenarios**:

1. **Given** an Order or Connect round where a team has zero hearts, **When** the gamemaster tries to switch the turn to that team, **Then** the team is not put on turn and the current turn is unchanged.
2. **Given** an Order or Connect round where every team still has hearts, **When** the gamemaster switches the turn to a team, **Then** that team is put on turn as before.
3. **Given** an Order or Connect round where a team has zero hearts, **When** the gamemaster views the turn-switch options, **Then** the eliminated team is not offered as a target (or is clearly shown as out and unselectable).
4. **Given** the gamemaster tries to put an eliminated team on turn, **When** the switch is refused, **Then** the gamemaster receives clear feedback that the team is out.
5. **Given** an eliminated team, **When** the next round starts and hearts reset, **Then** the team is back in and can be put on turn again.

---

### Edge Cases

- The rule applies to Order and Connect (both use hearts); Tripmine has no hearts, so every team remains selectable there.
- A team taken to zero hearts by the gamemaster's manual heart adjustment is out the same way as one that lost its last heart by playing.
- If the team currently on turn loses its last heart, the turn advances as it does today; the gamemaster must not be able to hand it back.
- A refused turn switch must not change the active team, the scores, the hearts, or the round.
- Before any round has started (no hearts in play), the turn switch is not restricted by this rule.
- Reconnecting or reloading must not restore the ability to select an eliminated team.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: In an Order or Connect round, the system MUST NOT put a team with zero hearts on turn.
- **FR-002**: A gamemaster turn switch to a team with zero hearts MUST be refused, leaving the current turn unchanged.
- **FR-003**: A refused turn switch MUST give the gamemaster clear feedback that the team is out.
- **FR-004**: The gamemaster turn-switch control MUST NOT offer (or MUST show as unselectable) a team with zero hearts in Order and Connect.
- **FR-005**: A gamemaster turn switch to a team that still has hearts MUST keep working as today.
- **FR-006**: In Tripmine (no hearts), every team MUST remain selectable.
- **FR-007**: The rule MUST apply to a team that reached zero hearts either by playing or by a manual heart adjustment.
- **FR-008**: Starting a new round (hearts reset) MUST make the team selectable again.
- **FR-009**: The rule MUST NOT change scores, hearts, matching, or the automatic turn rotation.

### Key Entities *(include if feature involves data)*

- **Team standing**: A team's score and hearts; a team with zero hearts is out for the round.
- **Active team**: The team currently on turn.
- **Turn switch**: The gamemaster action that names the active team.
- **Eliminated team**: A team at zero hearts during an Order or Connect round.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In Order and Connect, a team with zero hearts is put on turn in 0% of gamemaster turn switches.
- **SC-002**: A refused turn switch leaves the active team unchanged in 100% of attempts.
- **SC-003**: A turn switch to a team with hearts still succeeds in 100% of attempts.
- **SC-004**: After hearts reset at a new round, a previously eliminated team is selectable again in 100% of rounds.
- **SC-005**: The automatic turn rotation and all scoring behavior are unchanged.

## Assumptions

- A team is "out" for the round when its hearts are zero; this is the same condition the automatic turn rotation already uses to skip it.
- Refusing the switch (with feedback) is the chosen behavior; the turn is left on the current team rather than silently jumping to another.
- The gamemaster turn switch is the only manual path that can set an active team; automatic rotation already skips eliminated teams.
- Tripmine rounds do not use hearts, so this restriction does not apply to them.
