# Research: Fix Reveal Reset On Next Round

## D-2001: Advancing a round resets the reveal flag

**Decision**: In `sessions::advance_round`, when the next configured round is started, also set
`reveal_values = 0` in the same session update that writes `last_message`.

**Rationale**: The reveal belongs to the round; the next round must start hidden (FR-001–FR-003). The
finish path already sets `reveal_values = 1` and is left alone (FR-004). Reusing the existing column in the
existing update is the smallest change (Principle VII).

**Alternatives considered**: a separate update statement (unnecessary); a frontend reset (the client does
not own the flag).

## D-2002: The last-round finish keeps its reveal

**Decision**: Do not touch the finish branch of `advance_round` (it already sets `reveal_values = 1`).

**Rationale**: Finishing reveals the final standings; only starting a new round resets the flag (FR-004).

**Alternatives considered**: resetting on finish (would hide the final standings — wrong).

## D-2003: No other round behavior changes

**Decision**: Only the reveal flag is reset; scores, hearts, the turn, and the timer are untouched.

**Rationale**: The bug is a single stale flag (FR-005; Principle VII).

**Alternatives considered**: none needed.

## D-2004: Tests cover the reset and the finish

**Decision**: Add `cargo test` cases: reveal, advance, assert the flag is cleared; and finish at the last
round, assert the flag stays set.

**Rationale**: The rule is business logic the constitution requires to be tested (Principles I, II).

**Alternatives considered**: manual checks only (rejected).
