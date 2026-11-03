# Research: Fix Game End After The Last Round

## D-701: The last round's end finishes the game

**Decision**: Add a named `sessions::finish_if_last_round(conn, session_id)` that, when the session
is not already `finished` and its current round is the last configured round, sets the session to
`finished` with `reveal_values = 1` and no active team (the same outcome as the gamemaster's manual
finish). Call it from the handlers that can end a round: the Order confirmation when the board is
over, the Tripmine pick when the round completes, and a timeout that leaves nobody on turn.

**Rationale**: The game-end dialog keys off the `finished` status, and only `advance_round` set that
status. A quick game has no gamemaster to press "Finish Game", so its session never finished and the
dialog never appeared — exactly the finding. Finishing on the last round's end reuses the existing
finish behavior (reveal, clear the active team), so the dialog, the final standings, and the
finished-session deletion all work unchanged (Principles II and VII). Doing it in one function keeps
the rule out of each handler.

**Alternatives considered**: showing the dialog off a "round is over" signal instead of `finished`
(the dialog's semantics and the finished-session deletion are defined on `finished`, so this would
fork the end state); a frontend timer/poll that finishes the game (the client does not own status).

## D-702: Only the last round finishes the game

**Decision**: A round that is not the last configured round leaves the session running; the
gamemaster still advances to the next round as before.

**Rationale**: Multi-round games must keep their between-rounds step; only the final round's end is
terminal (FR-003). The check is simply `current_round >= round_count`.

**Alternatives considered**: auto-advancing to the next round on round end (out of scope; the spec
keeps between-rounds manual).

## D-703: No schema or frontend change

**Decision**: Reuse the existing `finished` status, `reveal_values`, and active-team columns; make no
frontend change.

**Rationale**: The end dialog already renders for every role when `status === 'finished'` (feature
005), and deletion rides the same status. The fix is purely the transition into that status
(Principle VII).

**Alternatives considered**: a new status value (unnecessary; `finished` already means "no further
rounds").

## D-704: Safe when already finished

**Decision**: `finish_if_last_round` is a no-op when the session is already `finished` or when the
current round is not the last.

**Rationale**: A gamemaster may still finish manually before the last round's board resolves; the
function must not error or finish twice (FR-007).

**Alternatives considered**: relying on the caller to check (spreads the guard across call sites).
