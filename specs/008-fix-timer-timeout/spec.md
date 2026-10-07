# Feature Specification: Fix Timer Timeout

**Feature Branch**: `008-fix-timer-timeout`

**Created**: 2026-10-07

**Status**: Draft

**Input**: User description: "bugs:
-\"The timer has not run out yet.\" bar is not specified and should not be displayed only the timer
- Nothing happens if the timer runs down to 0s, Expected turn end -> next players turn and counts as miss/error in order
- Dispite the start sound i also expect a sound for each of the last five seconds to highlight that the time is running up."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Reaching zero actually ends the turn (Priority: P1)

When the countdown reaches 0 seconds the turn must end, as it is meant to. In an Order round that
counts as a miss/error: the on-turn team loses one heart and the turn passes to the next team. In a
Tripmine round the turn simply passes. Today nothing happens at 0 — the countdown sits at zero and
play does not move on.

**Why this priority**: The timer's entire purpose is to end a stalling turn; a countdown that reaches
zero and does nothing is broken.

**Independent Test**: Start a short timer on a turn and let it reach 0; confirm the turn ends
(Order: one heart lost and the turn passes; Tripmine: the turn passes) with no further action.

**Acceptance Scenarios**:

1. **Given** a running Order timer, **When** it reaches 0, **Then** the on-turn team loses exactly one heart and the turn passes to the next team with hearts.
2. **Given** a running Tripmine timer, **When** it reaches 0, **Then** the turn passes to the next team and no hearts or points change.
3. **Given** the turn has ended by the timeout, **When** the board updates, **Then** the timer is cleared.
4. **Given** a timeout ends the last team's turn in an Order round, **When** no team has hearts, **Then** the existing round-end rules apply.

---

### User Story 2 - No stray error bar for the timer (Priority: P2)

While the countdown runs, the only timer feedback is the countdown itself. A message like "The timer
has not run out yet." must never appear to the player; the countdown is the whole timer UI.

**Why this priority**: An internal, premature-rejection message is confusing and makes the timer look
broken; it is a visible defect.

**Independent Test**: Let a timer run to zero and watch for any error/notice bar; confirm none appears
besides the countdown.

**Acceptance Scenarios**:

1. **Given** a running timer, **When** the countdown reaches 0, **Then** no error or notice bar appears; the turn just ends.
2. **Given** any role watching, **When** the timer runs, **Then** the countdown is the only timer indicator.

---

### User Story 3 - A tick for each of the last five seconds (Priority: P3)

Besides the start cue, each of the last five seconds of the countdown plays a short tick, so the room
hears the time running out. When the timer reaches zero, the timeout cue plays as before.

**Why this priority**: A final warning makes the deadline felt; it is an enhancement on top of the
working timer.

**Independent Test**: Start a timer of at least five seconds and listen: a tick on each of the last
five seconds (5, 4, 3, 2, 1), then the timeout cue at zero.

**Acceptance Scenarios**:

1. **Given** a running timer with five or more seconds left, **When** each of the last five seconds is reached, **Then** a tick plays for every role.
2. **Given** a muted session, **When** the last five seconds pass, **Then** no tick plays.
3. **Given** the countdown reaches 0, **When** the timeout happens, **Then** the timeout cue plays after the ticks.

---

### Edge Cases

- A timer shorter than five seconds ticks on each of its remaining seconds (e.g., a 3s timer ticks at 3, 2, 1).
- The tick must not double-fire when the state is refetched mid-countdown.
- Reaching zero must end the turn exactly once, even if more than one role is watching.
- The timeout cue and the ticks obey mute together.
- An Order timeout still costs exactly one heart, not one per tick.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: When a running timer reaches 0, the turn MUST end without any further user action.
- **FR-002**: In an Order round a timeout MUST count as a miss: the on-turn team loses exactly one heart and the turn passes to the next team with hearts.
- **FR-003**: In a Tripmine round a timeout MUST pass the turn and change no team's hearts or points.
- **FR-004**: A timeout MUST clear the timer and MUST NOT require the gamemaster to act.
- **FR-005**: No error or notice bar for the timer MUST be shown to any role; the countdown is the only timer indicator.
- **FR-006**: A short tick sound MUST play on each of the last five seconds of a running countdown.
- **FR-007**: A timer shorter than five seconds MUST tick on each of its remaining seconds.
- **FR-008**: The timeout cue MUST still play when the countdown reaches 0.
- **FR-009**: The ticks and the timeout cue MUST be silent while the session is muted.
- **FR-010**: Reaching 0 MUST end the turn exactly once, regardless of how many roles are connected.

### Key Entities *(include if feature involves data)*

- **Turn timer**: The countdown attached to the current turn; reaching zero triggers the timeout.
- **Timeout**: The end-of-turn event at zero; Order loses one heart and passes the turn, Tripmine passes the turn.
- **Tick cue**: A short sound on each of the last five seconds of the countdown.
- **Countdown UI**: The only visible timer indicator; no error or notice bar accompanies it.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A timer reaching 0 ends the turn in 100% of trials (Order: one heart + pass; Tripmine: pass).
- **SC-002**: No timer-related error or notice bar is shown at any point.
- **SC-003**: A countdown of five or more seconds plays exactly five ticks, one per last second.
- **SC-004**: The timeout cue plays once at zero, after the ticks.
- **SC-005**: With sound muted, neither the ticks nor the timeout cue play.

## Assumptions

- "The last five seconds" means the seconds 5, 4, 3, 2, 1 before zero; a timer shorter than five seconds ticks on each remaining second.
- The Order miss consequence is the same as feature 006's timeout rule (one heart, turn passes); the Tripmine timeout remains score-neutral.
- The tick is a short cue from the existing effect set; no audio files are added.
- The countdown remains the client-side display of a server-owned end time; the turn-ending action is still judged by the server.
