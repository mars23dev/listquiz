# Research: Fix Eliminated Team Turn Switch

## D-1301: The turn switch refuses a zero-heart team when the round uses hearts

**Decision**: In `apply_set_turn` (`backend/src/routes/sessions.rs`), after confirming the target team
exists, determine whether the current round uses hearts (quiz type ORDER or CONNECT) and load the
teams. If the round uses hearts and the target team's hearts are zero, return a refusal error with a
clear message instead of updating `active_team_id`.

**Rationale**: This is the one manual path that can set the active team; the automatic rotation already
skips zero-heart teams (`scoring::advance_turn`). Enforcing the same threshold in the handler closes the
gap (FR-001, FR-002) with the smallest change (Principle VII). Refusing — rather than silently jumping
to another team — keeps the gamemaster's intent explicit and gives feedback (FR-003).

**Alternatives considered**: silently redirecting to the next team with hearts (hides the gamemaster's
mistake and changes the active team unexpectedly); only disabling the option in the UI (a direct action
call would still bypass it — the backend must own the rule, Principle II/III).

## D-1302: "Uses hearts" comes from the current round's quiz type

**Decision**: Read the current round's quiz type with the same query `sessions::apply_turn_timeout`
already uses (`round_config` for the session's `current_round`); treat ORDER and CONNECT as
heart-using, TRIPMINE as not.

**Rationale**: Tripmine has no hearts, so the restriction must not apply there (FR-006). Reusing the
established quiz-type lookup keeps one source of truth and needs no new state (Principle VII).

**Alternatives considered**: inferring hearts from the team rows alone (Tripmine teams still carry
hearts, so this would wrongly restrict Tripmine); a new stored flag (unnecessary).

## D-1303: The "out" condition is the same threshold the rotation uses

**Decision**: A team is out when `hearts <= scoring::MIN_HEARTS` (zero); this matches
`advance_turn`'s `candidate.hearts > MIN_HEARTS` test.

**Rationale**: One definition of "out" keeps the manual switch and the automatic rotation consistent
(FR-001, FR-007, FR-009). It also covers a team taken to zero by a manual heart adjustment, since that
writes the same `hearts` column.

**Alternatives considered**: a separate "eliminated" flag (would need to be kept in sync with hearts;
not justified).

## D-1304: The gamemaster UI marks the out team unselectable

**Decision**: In the turn `<select>` in `frontend/src/pages/Session.tsx`, add `disabled` to an option
whose team has zero hearts when the round shows hearts (`showHearts`). The control stays a native
`<select>`.

**Rationale**: The spec requires the out team not be offered (FR-004). Disabling the option (rather than
removing it) shows the team still exists but is out, and keeps the control keyboard accessible
(Principle VI). The backend remains the authority (D-1301), so the UI is a convenience only.

**Alternatives considered**: removing the option entirely (loses the information that the team is out);
a custom listbox (a needless abstraction over a native select).

## D-1305: No schema, dependency, or scoring change

**Decision**: No migration, no new dependency; `advance_turn`, scoring, hearts, and turn rotation are
untouched.

**Rationale**: The rule only gates one handler and one option; nothing else changes (FR-009;
Principle VII).

**Alternatives considered**: none needed.
