# Feature Specification: Tripmine Mode

**Feature Branch**: `009-tripmine-mode`

**Created**: 2026-10-07

**Status**: Draft

**Input**: User description: "findings/improvements:
1. Tripmine is not suppossed to have hearts this is an order specific feature that might also be used in further future gametypes but not in tripmine because error = game end
2. The tripmine sound does not play when game ends with the mine. The game ends sequence seem to skip the round end sequence its supposed to be round end first than gameend
3. Tripmine additionally to lowest/highest add random with a hint that this is \"Hard\" difficulty. Change Mine is not necessary the highest or lowest but a random item from the list. The searched item is also always random from the remaining except the mine."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Tripmine has no hearts (Priority: P1)

Hearts are an Order mechanic. A Tripmine round must not use or show hearts at all: a mistake is the
mine, which ends the game outright, not a heart loss. During a Tripmine round the heart indicator is
not shown, and no Tripmine event changes a team's hearts.

**Why this priority**: Hearts in a game type that cannot lose them is misleading and breaks the model
the player is taught.

**Independent Test**: Play a Tripmine round and confirm no hearts are shown and no event changes them;
play an Order round and confirm hearts still work.

**Acceptance Scenarios**:

1. **Given** a Tripmine round, **When** any role views the board, **Then** no heart indicator is shown for any team.
2. **Given** a Tripmine round, **When** any pick, safe reveal, or timeout happens, **Then** no team's hearts change.
3. **Given** an Order round, **When** it is played, **Then** hearts behave exactly as before (shown and lost on mistakes).

---

### User Story 2 - Round end plays before game end (Priority: P2)

When a round ends and the same moment finishes the game, the sounds must play in order: the
round-ending cue first (for Tripmine, the mine explosion or the safe reveal), then the game-end cue.
Today a game that ends on the mine skips the round-end sound and plays only the game-end sound.

**Why this priority**: The dramatic payoff of hitting the mine is lost when its sound is skipped.

**Independent Test**: Hit the mine on the last round and listen: the explosion plays, then the
game-end cue.

**Acceptance Scenarios**:

1. **Given** the last round ends with the mine, **When** the state updates, **Then** the explosion cue plays, followed by the game-end cue.
2. **Given** the last round ends with a correct Order placement, **When** the state updates, **Then** the placement cue plays, followed by the game-end cue.
3. **Given** a round ends without finishing the game, **When** the state updates, **Then** only the round cue plays.
4. **Given** a muted session, **When** the game ends, **Then** no cue plays.

---

### User Story 3 - Random (Hard) Tripmine mode (Priority: P3)

Besides lowest and highest, a Tripmine round can be set to **Random (Hard)**: the mine is not
necessarily the lowest or highest item but a random one, and the item being searched for is always a
random one among the remaining (never the mine). The setup screen labels this choice as "Hard"
difficulty.

**Why this priority**: A new difficulty that adds replay value; it builds on the existing Tripmine
round rather than changing it.

**Independent Test**: Start a Random Tripmine round, confirm the mine can be any item and the search
target is a random remaining item each turn; confirm the setup shows "Hard".

**Acceptance Scenarios**:

1. **Given** the setup screen, **When** the gamemaster chooses the Tripmine side, **Then** the choices are Lowest, Highest, and Random (Hard).
2. **Given** a Random round, **When** it starts, **Then** the mine is a randomly chosen item, not tied to the lowest or highest.
3. **Given** a Random round, **When** a pick happens, **Then** the next searched item is a random one among the remaining items, excluding the mine.
4. **Given** a Random round, **When** the searched item is picked, **Then** it scores as a target hit (as in the other sides); a safe pick and a mine hit behave as usual.
5. **Given** the lowest or highest side, **When** it is played, **Then** its behavior is unchanged.

---

### Edge Cases

- A Random round with only the mine and one other item left: the searched target is the remaining non-mine item.
- A Random round where the searched item is picked with exactly two items left still scores the final-item bonus and ends the round.
- The searched target is never the mine, and never an already-revealed item.
- Tripmine heart hiding must not affect the game-end dialog (which already shows points only).
- The cue sequence must remain one cue when a round ends without finishing the game.
- Choosing Random must be persisted and shown on the board like the other sides.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A Tripmine round MUST NOT show a heart indicator for any team.
- **FR-002**: No Tripmine event (pick, safe reveal, or timeout) MUST change any team's hearts.
- **FR-003**: Order rounds MUST keep hearts exactly as they are today.
- **FR-004**: When a round's end also finishes the game, the round-ending cue MUST play before the game-end cue.
- **FR-005**: A game ending on the mine MUST play the explosion cue, then the game-end cue.
- **FR-006**: A round ending without finishing the game MUST play only its round cue.
- **FR-007**: All cues MUST remain silent while the session is muted.
- **FR-008**: The Tripmine side choice MUST offer Lowest, Highest, and Random (Hard).
- **FR-009**: In a Random round the mine MUST be a randomly chosen item, not required to be the lowest or highest.
- **FR-010**: In a Random round the searched target MUST be a random remaining item, never the mine and never already revealed, re-chosen after each target hit.
- **FR-011**: A Random round MUST use the same scoring as the other sides (target hit, safe pick, mine penalty, final-item bonus).
- **FR-012**: Lowest and Highest rounds MUST be unchanged.
- **FR-013**: The setup screen MUST label the Random choice as "Hard" difficulty.

### Key Entities *(include if feature involves data)*

- **Tripmine side**: The round's difficulty/direction — Lowest, Highest, or Random (Hard).
- **Mine**: The single losing item; in Random it is any item, in the other sides the lowest or highest.
- **Searched target**: The item worth the target bonus this turn; in Random it is a random remaining non-mine item.
- **Heart**: The Order-specific life indicator, not used by Tripmine.
- **Cue sequence**: The ordered sounds for one state change; a round end that finishes the game plays the round cue then the game-end cue.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A Tripmine round shows zero hearts and changes zero hearts across a full round.
- **SC-002**: Ending the game on the mine plays two cues in order (explosion, then game end) 100% of the time.
- **SC-003**: A round ending without a game end plays exactly one cue.
- **SC-004**: The setup offers exactly three Tripmine sides, with Random labelled "Hard".
- **SC-005**: In a Random round the mine is uniformly any item, and the searched target is always a remaining non-mine item.

## Assumptions

- "No hearts in Tripmine" means the indicator is hidden during a Tripmine round and no Tripmine event changes hearts; the underlying heart value may still be reset between rounds but is not used by Tripmine.
- The game-end dialog already shows points only, so hiding hearts does not change it.
- Random difficulty reuses the existing Tripmine scoring and flow; only how the mine and the searched target are chosen changes.
- The lowest/highest sides keep their deterministic search ladder; Random replaces the ladder with a random remaining target.
