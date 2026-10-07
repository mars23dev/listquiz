# Research: Connect Value Order And GM Answer Key

## D-1201: Values keep the list order; only names are shuffled

**Decision**: Change `sample_connect_round` so the ten drawn pairs are first selected, then the
**values** are emitted in the list's item order (ascending original index / `ORDER BY i.position`) while
the **names** are shuffled. Concretely: shuffle the candidate indices, take ten, sort those ten indices
ascending for the values, and shuffle the names list separately.

**Rationale**: The list already defines an order and `load_pairs` already returns it (`ORDER BY
i.position`). The only reason both columns were random was the explicit feature-010 shuffle. Removing
the value shuffle and leaving the name shuffle satisfies the spec (FR-001–FR-004) with the smallest
change and no sorting algorithm (Principle VII). The persisted board stores the arrays as-is, so an
active round keeps its order across reload/reconnect for free (FR-007).

**Alternatives considered**: a client-side re-sort of `view.values` (the backend should own the drawn
order and the stored board, per feature 010's storage model, so a client sort would drift from stored
state and re-sort on every render); numeric/alphabetical sorting (explicitly rejected by the spec —
the list order is the order).

## D-1202: The GM answer key is a client display rule, not a backend role channel

**Decision**: Add a boolean prop (e.g. `showAnswerKey`) to `ConnectBoard`; `Session` passes
`isGamemaster`. For each unmatched name the component renders the matching value, found in the same
view by shared `itemId`. Players and the spectator get no prop value, so nothing extra renders.

**Rationale**: A name and its value already carry the same `itemId` in the payload, so the pairing is
already available to the client for all roles; the feature only controls what the UI shows. Adding a
backend role filter or a separate answer-key field would add a channel the client does not need
(Principle VII) and cannot remove the ids anyway (players must send value ids to pick). The GM key is
therefore presentational, which is exactly how the spec words it (FR-009–FR-013).

**Alternatives considered**: a backend `answer_key` field sent only to the gamemaster (a redundant
channel; the client can derive it); hiding value `itemId` from players (would break `pick_value`, which
sends the value's id).

## D-1203: Matched pairs drop their key entries

**Decision**: The key entry for a name is rendered only while that pair is unmatched; once the shared
`itemId` is in `view.matched`, the name row (and its key) is hidden as today (FR-012).

**Rationale**: The board already hides matched items; keying off the same `matched` set keeps the GM
view consistent with the base game and prevents stale entries (FR-012).

**Alternatives considered**: a separate key list that persists after matching (would show stale pairs
and diverge from the board).

## D-1204: No schema, dependency, or scoring change

**Decision**: No migration, no new dependency; the `connect_state` columns and the scoring/hearts/turn
functions are untouched.

**Rationale**: The board is already persisted as JSON and the answer key is derived; scoring does not
depend on column order (Principle VII; FR-005, FR-011).

**Alternatives considered**: none needed.
