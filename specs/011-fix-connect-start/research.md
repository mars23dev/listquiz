# Research: Fix Connect Round Start Panel

## D-1101: The pre-round prompt is hidden when any game type is active

**Decision**: Extend the pre-round prompt guard in `frontend/src/pages/Session.tsx` so the prompt is
shown only when no game type is active — Connect, Order, or Tripmine — and the session is not
finished. Concretely, the existing guard (`state && !order && !tripmine && status !== 'finished'`)
gains the Connect board: `state && !connect && !order && !tripmine && status !== 'finished'`.

**Rationale**: The prompt was introduced for pre-round states and already excludes the Order and
Tripmine boards. Connect was added later (feature 010) without extending this guard, so a live Connect
round still reports no Order and no Tripmine board and the prompt persists. Adding the same exclusion
for Connect restores the intended invariant (FR-001 to FR-006) and is the smallest change that does so
(Principle VII). Because the prompt is hidden once `connect` is truthy, the second `next_round` action
is no longer reachable, so starting a round cannot finish the game (FR-004, FR-007).

**Alternatives considered**: testing `state.round.quizType` against `'CONNECT'` instead of the board
object (the guard is already driven by the presence of board objects — order/tripmine — so using the
Connect board keeps one consistent rule and does not depend on quiz-type naming); disabling the
`next_round` path in the backend when a round is already active (the bug is purely a stale client
panel; blocking the server action would change round-advance semantics elsewhere).

## D-1102: No new helper or state

**Decision**: Change the guard expression in place; do not introduce a derived flag or helper.

**Rationale**: The predicate already exists and is a single boolean over `state`; a named helper would
be indirection without a second caller (Principle VII). The render expression is the one place the
prompt is decided.

**Alternatives considered**: a `showPreRoundPrompt` derived constant (adds a name for a one-use value;
not justified under Principle VII).

## D-1103: Verify with a frontend test, no backend change

**Decision**: Add a case in `frontend/src/test/Session.test.tsx` using a Connect session-state fixture
that asserts the start button and waiting message are absent once a Connect board exists, alongside the
existing Order-based visibility test.

**Rationale**: The rule is client-side visibility over stubbed state, so it is directly testable in
isolation (Principle I) with the existing Vitest/Testing-Library setup. The backend already emits the
Connect board; nothing server-side changes (Principles III and IV unchanged).

**Alternatives considered**: an end-to-end Playwright test only (slower and requires the full stack;
kept as an optional manual check in quickstart.md instead).
