# Feature Specification: Connect Value Order And GM Answer Key

**Feature Branch**: `012-connect-order-values`

**Created**: 2026-10-07

**Status**: Draft

**Input**: User description: "connect game improvemnet. The values should be ordered only the names should be random order other wise its too confusing."

## Clarifications

### Session 2026-10-07

- Q: How should the Connect value column be ordered? → A: Keep the values in the same order the list already defines; do not sort or invent a new ordering. Only the name column is shuffled/randomized.
- Q: How should the gamemaster see which names and values belong together? → A: A persistent GM-only answer key: in the gamemaster view, each name row shows its correct matching value, visible for the whole round; never shown to players or the spectator.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Values are readable, names stay random (Priority: P1)

In the Connect game the left column shows ten item names and the right column shows the ten
corresponding values. Today both columns are shuffled, which makes the right column hard to scan. The
names should stay in a random order (so the puzzle is not solved by position), but the values should
stay in the order the list already defines, so the room can read them in the list's own sequence. No
new ordering rule is introduced.

**Why this priority**: This is the requested improvement and the whole of the change; the random value
column is what makes the board confusing.

**Independent Test**: Start a Connect round and confirm the left (name) column order is random while
the right (value) column is shown in the same order as the list's items (never shuffled).

**Acceptance Scenarios**:

1. **Given** a Connect round starts, **When** the board is shown, **Then** the value column is shown in the same order as the list's items, not a random shuffle.
2. **Given** the same Connect round, **When** the values are compared across two starts with the same drawn items, **Then** the value order matches the list's order both times.
3. **Given** a Connect round, **When** the board is shown, **Then** the name column remains in a random order.
4. **Given** the names and values change columns independently, **When** a team reads the board, **Then** the pairing between a name and its value is not revealed by their positions.
5. **Given** a value has been matched, **When** it is removed, **Then** the remaining values stay in the list's relative order.

---

### User Story 2 - The gamemaster sees the answer key (Priority: P2)

The gamemaster hosts and referees the round, so they need to know which name belongs to which value.
In the gamemaster view, each name row shows its correct matching value as a persistent answer key for
the whole round. Players and the spectator never see this pairing; the room still has to work it out.

**Why this priority**: Hosting and confirming the round requires the mapping; without it the gamemaster
cannot referee or verify scoring. It is secondary to the core ordering fix but part of the same change.

**Independent Test**: Open a Connect round in the gamemaster view and confirm each name shows its
correct value; open the same round as a player and as the spectator and confirm no pairing is shown.

**Acceptance Scenarios**:

1. **Given** a Connect round in the gamemaster view, **When** the board is shown, **Then** each name displays its correct matching value.
2. **Given** the same round viewed by a player or the spectator, **When** the board is shown, **Then** no name↔value pairing is revealed beyond what the base game already shows.
3. **Given** a key entry for an unmatched pair, **When** that pair is matched, **Then** the entry disappears along with the matched items.
4. **Given** the gamemaster view, **When** the page reloads or reconnects, **Then** the answer key is still correct.
5. **Given** the answer key is shown, **When** the gamemaster stages and confirms pairs, **Then** scoring, hearts, matching, and turn passing are unchanged.

---

### Edge Cases

- The value column MUST follow the list's existing order; no numeric sorting, alphabetical sorting, or other new ordering rule is introduced.
- Only ten items are drawn from the list; among the drawn values the list's relative order is preserved.
- The value order must not change the Connect scoring, hearts, matching, or turn rules.
- The name column must remain random; the two columns must not be tied to the same ordering.
- An already-started Connect round keeps its order when reloaded or reconnected (no re-shuffle and no re-sort on refresh).
- The GM answer key must never be revealed to players or the spectator, on any screen or reload.
- The GM answer key must not affect the name randomness or the value list order; it is display-only.
- A matched pair's key entry must not linger, and when every pair is matched no key entries remain.
- Reconnecting to the gamemaster view must not alter the answer key.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The Connect value column MUST keep the order the list already defines; it MUST NOT be randomly shuffled.
- **FR-002**: The Connect name column MUST be randomly shuffled.
- **FR-003**: The value column order MUST come solely from the list's existing order and MUST be deterministic; it MUST NOT depend on chance.
- **FR-004**: The system MUST NOT introduce a new ordering rule (no numeric or alphabetical sorting); the value column simply preserves the list's order.
- **FR-005**: Keeping the value column in list order MUST NOT change a pair's correctness, scoring, hearts, or turn passing.
- **FR-006**: The name column ordering MUST be independent of the value column ordering (shuffling the names MUST NOT reorder the values).
- **FR-007**: The order of an active round MUST persist across reload/reconnect (the value column MUST NOT reorder while a round is in progress).
- **FR-008**: Matched values MUST be removed/hidden without disturbing the list-relative order of the remaining values.
- **FR-009**: In the gamemaster view, each name row MUST show its correct matching value as a persistent answer key.
- **FR-010**: The answer key MUST be visible only to the gamemaster; players and the spectator MUST NOT see it.
- **FR-011**: The answer key MUST NOT change the name randomness, the value list order, scoring, hearts, matching, or turn passing.
- **FR-012**: A key entry MUST disappear once its pair is matched; no stale key entry may remain.
- **FR-013**: The answer key MUST remain correct across reload and reconnect in the gamemaster view.

### Key Entities *(include if feature involves data)*

- **Connect board**: The ten names and ten values of an active round, plus the matched pairs and pending selection.
- **Name column**: The left side; randomly shuffled each round.
- **Value column**: The right side; kept in the list's existing order.
- **List order**: The order the list already defines; the value column preserves it and no separate ordering is computed.
- **GM answer key**: The name→value mapping shown only in the gamemaster view, present for each unmatched pair for the whole round.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: In 100% of Connect rounds the value column preserves the list's order (never a random shuffle).
- **SC-002**: Across repeated starts with the same drawn items the value column always follows the list's order (deterministic, never random).
- **SC-003**: The name column is random in 100% of rounds.
- **SC-004**: No new ordering rule is introduced; scoring, hearts, matching, and turn rules are unchanged by this improvement.
- **SC-005**: Reconnecting to an active Connect round never reorders the value column.
- **SC-006**: The gamemaster sees every unmatched name's correct value in 100% of Connect rounds.
- **SC-007**: Players and the spectator see the name↔value pairing in 0% of Connect rounds.
- **SC-008**: Scoring, hearts, matching, and turn rules are unchanged by the answer key.

## Assumptions

- The Connect board otherwise behaves exactly as defined in feature 010; only the value column's presentation order changes.
- The list already defines an order (its item order); the value column keeps that order. No sorting or new ordering rule is introduced (FR-004).
- Only the name column is shuffled; the value column is left in the list's order.
- The change is presentation only and does not alter the items drawn or the pair matching.
- The GM answer key is display-only, derived from the same pairs the round already holds; "persistent" means visible for the whole round while the pair is unmatched.
- The gamemaster is identified by their role (the gamemaster link), the same distinction already used elsewhere in the session.
