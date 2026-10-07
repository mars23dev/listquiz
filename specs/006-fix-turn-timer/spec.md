# Feature Specification: Fix Turn Timer

**Feature Branch**: `006-fix-turn-timer`

**Created**: 2026-10-07

**Status**: Draft

**Input**: User description: "findings/improvemnets:
1. Timer has no sound
2. Timer should not be custom input but pre defined values 15s,30s,60s
3. If timer runs up counts as turn is over -> In Order loses hearth and next player / In Tripmine next players turn"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - A timeout ends the turn correctly for each game type (Priority: P1)

When the gamemaster's timer runs out, it counts as the turn being over. In an Order round that means the on-turn team loses one heart and the turn passes to the next team, exactly like a wrong placement. In a Tripmine round there are no hearts, so the timeout simply passes the turn to the next team. Today a timeout always removes a heart, which is wrong for Tripmine.

**Why this priority**: The consequence of a timeout is the whole point of the timer; getting it wrong penalises the wrong team in one of the two game types.

**Independent Test**: Run an Order round and a Tripmine round; start a short timer on a turn and let it expire in each; confirm Order loses one heart and passes the turn, and Tripmine passes the turn with no heart change.

**Acceptance Scenarios**:

1. **Given** an Order round with a running timer, **When** the timer expires, **Then** the on-turn team loses exactly one heart and the turn passes to the next team with hearts.
2. **Given** a Tripmine round with a running timer, **When** the timer expires, **Then** the turn passes to the next team and no team's hearts change.
3. **Given** a timer that expires on the last team with hearts in an Order round, **When** it expires, **Then** the existing round-end rules apply.
4. **Given** any round, **When** the timer expires, **Then** the timer clears and a timeout message is shown.

---

### User Story 2 - The timer makes a sound (Priority: P2)

Starting the timer and letting it run out are both silent today. Starting the timer should play a cue, and a timer running out should play a cue, for every role watching. This mirrors the reference draft, where starting the timer plays a buzzer and the timeout plays the failure cue.

**Why this priority**: A countdown the room cannot hear defeats the purpose of warning a stalling player; it is a clear, self-contained defect.

**Independent Test**: In an unmuted session, start a timer and confirm a cue plays in every view; let it expire and confirm the timeout cue plays; with the session muted, confirm neither plays.

**Acceptance Scenarios**:

1. **Given** an unmuted session, **When** the gamemaster starts the timer, **Then** a start cue plays for every role.
2. **Given** an unmuted session, **When** the timer expires, **Then** a timeout cue plays for every role.
3. **Given** a muted session, **When** the timer starts or expires, **Then** no cue plays.
4. **Given** an Order timeout that also costs a heart, **When** it expires, **Then** the timeout cue plays once, not twice.

---

### User Story 3 - Fixed timer durations (Priority: P3)

The gamemaster no longer types a duration. The timer is started with one of three preset durations — 15 seconds, 30 seconds, or 60 seconds — chosen with a single action each.

**Why this priority**: A small usability change; the current free-text input invites invalid or absurd durations.

**Independent Test**: As gamemaster, confirm the timer offers exactly 15s, 30s, and 60s, and that starting with one runs a countdown of that length.

**Acceptance Scenarios**:

1. **Given** the gamemaster's timer controls, **When** they look at them, **Then** they see three choices: 15s, 30s, 60s — and no free-text duration field.
2. **Given** the gamemaster picks 30s, **When** the timer starts, **Then** the countdown runs from 30 seconds.
3. **Given** a choice is made, **When** the timer is already running, **Then** starting again replaces the running timer with the new duration.

---

### Edge Cases

- A timeout in Tripmine must never change any team's hearts or points.
- An Order timeout on the last team with hearts ends the round by the existing rules; the timer still clears.
- A timer already running is replaced by a new preset rather than stacking.
- Starting the timer is gamemaster-only; a timeout is reported by the acting side and judged by the server as before.
- Muting mid-countdown silences the expiry cue too.
- A preset of 60s must be accepted even though it is longer than the other presets.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: When a timer expires in an Order round, the on-turn team MUST lose exactly one heart and the turn MUST pass to the next team with hearts.
- **FR-002**: When a timer expires in a Tripmine round, the turn MUST pass to the next team with hearts and NO team's hearts or points MUST change.
- **FR-003**: A timer expiry MUST clear the timer and show a timeout message, in both game types.
- **FR-004**: A timer expiry MUST respect the existing round-end rules when no team has hearts (Order).
- **FR-005**: Starting a timer MUST play a start cue for every role.
- **FR-006**: A timer expiry MUST play a timeout cue for every role.
- **FR-007**: The start and timeout cues MUST be silent while the session is muted.
- **FR-008**: An Order timeout MUST play exactly one cue (no double cue from the heart loss).
- **FR-009**: The gamemaster MUST start the timer from exactly three preset durations: 15 seconds, 30 seconds, and 60 seconds.
- **FR-010**: The gamemaster timer controls MUST NOT offer a free-text duration input.
- **FR-011**: Starting a timer while one runs MUST replace it with the chosen preset.

### Key Entities *(include if feature involves data)*

- **Turn timer**: A countdown attached to the current turn, with a total duration and a remaining time; it carries the round's game type to decide the timeout's consequence.
- **Timeout**: The end-of-turn event when a timer runs out; its consequence depends on the game type (Order: heart lost + turn passed; Tripmine: turn passed only).
- **Timer cue**: The start sound and the timeout sound, part of the existing cue set.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: An Order timeout removes exactly one heart and advances the turn in 100% of trials.
- **SC-002**: A Tripmine timeout changes hearts and points by exactly zero and advances the turn.
- **SC-003**: With sound unmuted, both the start and the timeout of a timer are audible; with sound muted, neither is.
- **SC-004**: The timer offers exactly three durations and no free-text field.
- **SC-005**: An Order timeout plays exactly one cue.

## Assumptions

- Tripmine has no heart mechanic for a timeout (hearts belong to Order); a Tripmine timeout only ends the turn, matching the finding.
- The cue set is the existing synthesized effects (from the reference draft): a start cue and a timeout cue, reusing the draft's choice of a buzzer on start and a failure tone on timeout.
- Timeouts remain server-judged (a running, expired timer), reported by the acting side, as implemented.
- The three presets are fixed at 15/30/60 seconds; no other values are offered.
