# Research: Reveal Values In Tripmine And Connect

## D-2101: Tripmine items reveal under the session flag

**Decision**: In `app_state::redact`, compute a player's Tripmine item reveal as
`revealed = is_gamemaster || item.revealed || reveal`. This makes the value, rank, and mine show when the
gamemaster reveals.

**Rationale**: The redaction already sends the value when `reveal` is set, but leaves `revealed` false, and
the board keys visibility on `revealed`, so nothing appeared (FR-001, FR-002). Tying `revealed` to the
reveal flag is the smallest change and reuses the existing board rendering (Principle VII). The gamemaster
already sees everything, so the reveal only affects players/spectator (FR-003).

**Alternatives considered**: a separate "reveal values only" path (more code for no visible difference);
a frontend-only change (the client does not own the redaction).

## D-2102: Connect reveals the pairing via the existing answer-key prop

**Decision**: In `Session.tsx`, pass `showAnswerKey={isGamemaster || (state?.revealValues ?? false)}` to
`ConnectBoard`. The Connect values are already public, so the reveal shows the name↔value pairing.

**Rationale**: Connect's only hidden information is the pairing, which the board can already render as an
answer key (feature 012). Reusing that prop for the reveal is the smallest change (FR-002; Principle VII).

**Alternatives considered**: a new backend field for the pairing (the client can derive it from the shared
item id); leaving Connect a no-op (contradicts the request).

## D-2103: No reveal before the action; reset per round

**Decision**: The reveal only applies when the session flag is set; the per-round reset (feature 020)
still clears it.

**Rationale**: Players must not see the hidden info early (FR-003), and the reset keeps the flag per round
(FR-005).

**Alternatives considered**: none needed.

## D-2104: No other behavior changes

**Decision**: Revealing does not end the round or change scoring, hearts, the turn, or the timer.

**Rationale**: The reveal is a display flag (FR-004; Principle VII).

**Alternatives considered**: none needed.

## D-2105: Tests cover both modes

**Decision**: Add a `cargo test` case (Tripmine redaction reveals items under the flag) and a `vitest` case
(the Connect answer key shows to a player when the reveal flag is set and is hidden otherwise).

**Rationale**: The rules are business logic the constitution requires to be tested (Principles I, II).

**Alternatives considered**: manual checks only (rejected).
