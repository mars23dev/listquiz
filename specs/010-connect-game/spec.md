# Feature Specification: Connect Game And Tripmine Marker

**Feature Branch**: `010-connect-game`

**Created**: 2026-10-07

**Status**: Draft

**Input**: User description: "the trip mine game has a bar that shows the current searched and the already chosen items, but it should also display the mine posistion from the beginning. The posistion does not reveal the information which item it is its just a better overview.

New Game type: Connect. Same Grphical Style as other Games. Split game screen / 2 sides on the left hand side the items represent 10 random item names and on the right hand side the items containing the values of the 10 in a random order. Turn based pick one item (name) from the left and one item value (right) Pull them into a box in the middle. When confirmed then check if these name, value pair belong together. If correct 10points if wrong lose 1 heart (3 hearths) at start. If no hearts left go to read only mode analog to the order gamemode. Icon box is only shown on name."

## Clarifications

### Session 2026-10-07

- Q: After a team confirms a correct Connect pair, does the turn pass to the next team, or does the same team keep playing? → A: The turn passes after every confirm, correct or wrong (like Order).
- Q: Should the Tripmine mine-position marker be shown to every role, or only to the gamemaster? → A: Every role sees the mine position (gamemaster, players, spectator).

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The Connect game type (Priority: P1)

A new turn-based game type, styled like the others. The screen is split: on the left, ten item
**names** (each with its icon); on the right, the ten **values** in a random order (no icons). On a
turn, the team picks one name and one value and pulls them into a box in the middle, then confirms.
If the name and value belong together the pair is correct (+10 points); if not, the team loses one
heart. Each team starts the round with three hearts. When no team has hearts left, the round goes
read-only, exactly like an Order round runs out of hearts. The icon is only ever shown on the name
side.

**Why this priority**: This is the requested new game type and the bulk of the work.

**Independent Test**: Start a Connect round, pick a matching pair (score +10), pick a mismatching pair (lose a heart), and drive a team to zero hearts to confirm read-only. See quickstart.md §1.

**Acceptance Scenarios**:

1. **Given** a Connect round starts, **When** the board is shown, **Then** the left column holds ten names (each with an icon) and the right column holds the ten values in a random order with no icons.
2. **Given** a team on turn, **When** it selects one name and one value, **Then** both appear in the middle box awaiting confirmation.
3. **Given** a name and value in the box, **When** the team confirms and they belong together, **Then** the pair is scored correct (+10 points) and the turn passes.
4. **Given** a name and value in the box, **When** the team confirms and they do not belong together, **Then** the team loses one heart and the turn passes; the pair stays unmatched.
5. **Given** a team with one heart, **When** it confirms a wrong pair, **Then** it is at zero hearts.
6. **Given** no team has hearts left, **When** the board updates, **Then** it is read-only and the round is over.
7. **Given** every pair has been matched, **When** the last pair is confirmed, **Then** the round is over.
8. **Given** the gamemaster's turn timer runs out in a Connect round, **When** it expires, **Then** the on-turn team loses one heart and the turn passes (as in Order).

---

### User Story 2 - The Tripmine bar shows the mine position (Priority: P2)

The Tripmine progress bar shows the current searched position and the already-chosen ones. It should
also mark the **position of the mine** from the start, so the room can see where the danger sits. The
marker shows only the position, never which item holds the mine.

**Why this priority**: A small, self-contained overview improvement.

**Independent Test**: Start a Tripmine round and confirm the bar marks the mine's position from the beginning while the item itself stays hidden. See quickstart.md §2.

**Acceptance Scenarios**:

1. **Given** a Tripmine round starts, **When** the bar is shown, **Then** the mine's position is marked for every role before anything is revealed.
2. **Given** the marker, **When** any role looks at the items, **Then** it does not reveal which item is the mine (the items stay hidden as before).
3. **Given** the mine position is marked, **When** a mine is later revealed, **Then** the bar still shows it as the mine.

---

### Edge Cases

- Connect: confirming with only a name or only a value selected does nothing (both are required).
- Connect: a value can be selected only once per attempt; selecting a new one replaces the pending value.
- Connect: an already-matched name or value cannot be picked again.
- Connect: a round with fewer than ten valued items cannot start.
- Connect: the wrong-pair heart loss never takes a team below zero.
- Connect: when the last round's Connect game runs out of hearts or all pairs match, the game finishes as other rounds do.
- Tripmine: the marker must not change the mine's difficulty or the search behaviour; it is display only.
- Tripmine: the marker must be consistent across the lowest, highest, and random sides.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The system MUST offer a Connect game type alongside Order and Tripmine.
- **FR-002**: A Connect round MUST show ten item names on the left (each with its icon) and the ten corresponding values on the right in a random order, without icons on the value side.
- **FR-003**: A Connect turn MUST let the team on turn select one name and one value and move them into a middle box before confirming.
- **FR-004**: Confirming a matching name/value pair MUST award 10 points and pass the turn to the next team.
- **FR-005**: Confirming a mismatching pair MUST remove one heart from the on-turn team and pass the turn; the pair stays unmatched.
- **FR-006**: Each team MUST start a Connect round with three hearts, reset per round like Order.
- **FR-007**: A Connect round MUST become read-only once no team has hearts, like an Order round.
- **FR-008**: A Connect round MUST end when every pair is matched or no team has hearts; ending the last round MUST finish the game as other rounds do.
- **FR-009**: The gamemaster turn timer MUST apply to Connect: a timeout costs one heart and passes the turn.
- **FR-010**: The icon MUST be shown only on the name side; the value side MUST NOT show an icon.
- **FR-011**: The setup screen MUST let the gamemaster configure a Connect round (list and column), and a quick game MUST be able to start a Connect game.
- **FR-014**: The turn MUST pass after every confirm, whether the pair was correct or wrong.
- **FR-012**: The Tripmine progress bar MUST mark the mine's position from the start for every role, without revealing which item is the mine.
- **FR-013**: The Tripmine marker MUST be display-only and MUST NOT change the mine, the search, or the scoring.

### Key Entities *(include if feature involves data)*

- **Connect board**: The ten names, the ten shuffled values, the matched pairs, and the pending selection (a name and a value in the middle box).
- **Connect pair**: A name and its value; matched when confirmed correctly.
- **Heart**: The per-round life indicator used by Order and Connect (three per team).
- **Tripmine mine position**: The rank/position of the mine, shown on the bar from the start; not the item's identity.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A Connect round shows ten names (with icons) and ten values (no icons) in random order.
- **SC-002**: A correct pair always awards 10 points; a wrong pair always removes exactly one heart.
- **SC-003**: A Connect round reaches read-only in 100% of trials once no team has hearts.
- **SC-004**: The Tripmine mine position is visible from the start for every role, and the mine's item identity stays hidden until revealed.
- **SC-005**: A Connect round can be configured in setup and started as a quick game.

## Assumptions

- Connect uses the same list/column model as Order: ten items drawn from the chosen list and column; the round needs at least ten valued items.
- The wrong-pair consequence matches an Order mistake: one heart, turn passes; the existing round-end and last-round finish rules apply.
- Hearts are shown and used in Connect (unlike Tripmine), matching Order.
- The middle box is a staging area: the pair is judged only on confirmation.
- The Tripmine mine position is the mine's rank in the ten-item ranking; it is shown to every role, and in the random side it exposes only the position (not the item).
- Cues reuse the existing set (a correct pair is a success cue; a wrong pair is a failure cue).
