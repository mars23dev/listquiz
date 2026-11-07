# Research: Block Invalid Session Setup

## D-2401: Expose per-column valued-item counts on list summaries

**Decision**: Extend `ListSummary.columns[]` from `{ id, name }` to `{ id, name, valueCount }`.
`valueCount` is the number of items in that list carrying a value in that column. Both
`GET /api/default-lists` and `GET /api/folders/{folderId}/lists` return it.

**Rationale**: The setup page already loads summaries for every selectable list. Adding one derived count
per column lets the UI hide impossible lists without a new endpoint, without loading full list details, and
without changing folder privacy. The count is not secret: list item counts are already exposed (FR-008).

**Alternatives considered**: a new compatibility endpoint (extra surface and a second round trip); fetching
each list's full detail (N+1 requests and unnecessary item payload); trusting total `itemCount` (wrong when
a column has missing values); hiding only on the backend (the gamemaster would still see unusable options).

## D-2402: Put the capacity rule in a named `setup_capacity` module

**Decision**: Add `backend/src/modules/setup_capacity/mod.rs` with:

- `required_valued_items(quiz_type, tripmine_slots) -> usize`
- `valued_item_count(conn, list_id, column_id) -> ApiResult<usize>`
- `validate_capacity(position, quiz_type, tripmine_slots, available) -> ApiResult<()>`
- `validate_round_capacity(conn, round) -> ApiResult<()>`

The pure pieces are tested without a database; the DB wrapper is tested through session creation.

**Rationale**: The constitution requires business rules to be named, directly callable units with tests
(Principle II), and a new feature to live in its own small module (Principle V). The session module keeps
only the call site, not the rule.

**Alternatives considered**: inline SQL and conditionals in `sessions/mod.rs` (untestable rule, couples
rules to the handler); reusing the round-start sampler to detect the failure (defers the error, which is the
defect); duplicating thresholds in the frontend only (backend could still be bypassed).

## D-2403: Reuse the existing per-mode minimums

**Decision**: Order requires 11 valued items (`order_round::MIN_ITEMS_FOR_ORDER`), Connect requires 10
(`connect_round::PAIRS`), and Tripmine requires 10 or 20 according to the selected `tripmine_slots`
(default 10, matching the existing request parser). Add a named Tripmine 20 constant beside the existing
Tripmine 10 constant rather than repeating the literal in the rule.

**Rationale**: These minima are already the game rules enforced at round start. Reusing the constants keeps
one source of truth and prevents the setup gate from drifting away from gameplay.

**Alternatives considered**: a new independent threshold table (duplicates the mode modules); hardcoding
numbers in the UI (drift risk); changing the minima (out of scope).

## D-2404: Count values, not items

**Decision**: `valueCount` and the backend gate join `item` to `item_value` on the selected column. An item
with no row for that column does not count. A value row with an empty display still counts only if the
existing storage accepted it; the rule follows the same join the round builders already use.

**Rationale**: All three round builders read items through the selected column's `item_value` rows. Counting
total items would allow the same late failure the feature is fixing.

**Alternatives considered**: total item count (incorrect with missing values); counting columns with any
value (ignores the selected column); trusting the client's count (bypassable).

## D-2405: Hide by smallest size, retain a list that becomes incompatible

**Decision**: For list-picker eligibility, a list is shown when any of its columns meets the mode's smallest
size: Order 11, Connect 10, Tripmine 10. Picking a list selects its first column that meets the current
minimum, so a multi-column list is not hidden when only its first column is short. For the currently
selected list, use its current column. If a mode, size, or column change makes the current list
incompatible, keep the list selected and show an inline shortfall error. Unsupported Tripmine sizes stay
selectable so the error can name the exact shortfall. If no list qualifies, show an empty-state explanation
and block creation.

**Rationale**: This matches the clarification: impossible lists disappear, but the gamemaster is never
silently moved off a chosen list and sees exactly why a 20-item choice cannot run. Native selects keep the
controls keyboard accessible.

**Alternatives considered**: disabling unsupported sizes (contradicts the clarified behavior); auto-switching
lists (silent change, hard to test); hiding the selected list too (leaves an invisible invalid selection);
evaluating unselected lists by every column (a list would flicker based on a column the gamemaster has not
chosen).

## D-2406: Gate session creation before the transaction and keep round start unchanged

**Decision**: After the existing list/column existence and folder-access checks, run
`setup_capacity::validate_round_capacity` for every round, before opening the insert transaction. Refuse
with `400 bad_request` and a message naming the round, mode, required count, and available count. Quick game
keeps reusing `create_session`, so it inherits the gate; its `build_request` also filters default lists by
valued count in the first column.

**Rationale**: The existing `create_session` already validates every round before writing, and its single
transaction guarantees no partial session. Reusing it keeps quick game and standard setup consistent without
a second creation path. Keeping round start's sampler checks means a list edited after session creation
still fails safely rather than producing a broken board.

**Alternatives considered**: validating only in the route handler (quick game and any future caller could
skip it); deleting or rejecting existing sessions (out of scope); generating a partial session (violates
FR-004 and the existing transaction design).

## D-2407: Test the rule, the refusal, and the setup state independently

**Decision**: Add pure unit tests for the thresholds and messages; backend integration tests that plant
lists with exact, short, and missing-value counts and assert refusal plus no session row; frontend tests for
hidden lists, retained incompatible selection, size shortfall, empty state, and updated list fixtures.

**Rationale**: The constitution requires feature-local tests (Principle I) and explicit rule coverage
(Principle II). The frontend tests pin the clarified UX that caused the bug report.

**Alternatives considered**: manual-only verification (does not protect the rule); backend-only tests
(would not catch a picker that still offers impossible lists); UI-only tests (would not catch direct
requests).
