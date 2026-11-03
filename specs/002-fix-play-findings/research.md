# Research: Fix Play Findings

## D-201: Player actions reuse the action channel with a role field

**Decision**: The existing `POST /api/sessions/{id}/actions` channel accepts an optional `role`.
When `role` is `gamemaster`, the `X-Quiz-Password` header is required as today. When `role` is `p1`…
`p4`, no password is required, and the server accepts the action only if that role owns the team on
turn; any other player role or team is refused with `not_ready`. `return_to_pool` stays gamemaster-only.

**Rationale**: One channel keeps the write path, revision bump, and redaction reasoning in one place
(Principle V and VII). The role is already derivable from the link the client holds, and the server
already has `role_has_turn`, so authorization is one named check. This amends the original contract
line "every mutation comes from the gamemaster view" — a player link is now a legitimate writer for its
own turn, which is what finding 5 requires.

**Alternatives considered**: a separate player endpoint (duplicates every action and its tests);
a per-session token (adds an auth concept the product deliberately does not have).

## D-202: Shuffle the stored Tripmine grid order at round start

**Decision**: When a Tripmine round starts, shuffle the `items` array before storing it, so the stored
sequence is the display order. Each slot keeps its `true_rank`, `is_tripmine`, and reveal state.

**Rationale**: Every Tripmine rule looks items up by `item_id`/`true_rank`, never by array index, so
changing the order changes nothing about scoring, the searched position, or the answer key. Storing
the shuffled order makes the display stable across reloads and identical for every view without extra
state (Principle VII). Reuses the round's existing Fisher–Yates shuffle.

**Alternatives considered**: shuffling in the read layer or the frontend (unstable across reloads and
views, and pushes game presentation into transport); adding a separate `display_order` column (a new
field with no other use).

## D-203: The start affordance and waiting state are driven by "no board yet"

**Decision**: A session with no round started (no Order and no Tripmine board present, and not
finished) shows the gamemaster a large start control that starts the first round, and shows players
and spectators a waiting-for-start message in the same board area. The control and message disappear
as soon as a board exists.

**Rationale**: The state already distinguishes "no round" from "round in play", so no new session
field is needed. Starting the first round is the existing `next_round` transition, so the control only
changes how obvious it is (Principle VII).

**Alternatives considered**: a new "started" flag on the session (redundant with the board's
presence); a separate start endpoint (duplicates the round-transition logic).

## D-204: The setup field loses focus because its React key is the team name

**Decision**: Key each team-name input by its stable index, not by the team name.

**Rationale**: The inputs are currently keyed by `team.name`, which is derived from the typed value;
every keystroke changes the key, so React unmounts and remounts the input and focus is lost after one
character. A stable key fixes it with no state change.

**Alternatives considered**: uncontrolled inputs (harder to reflect the chosen count); debouncing the
name (hides the bug rather than fixing it).

## D-205: The list-editor icon control shows a real icon and the table is fixed-width

**Decision**: The row icon control renders the item's chosen icon through the existing icon component,
falling back to a neutral icon affordance (never the literal word). The editor table uses constrained,
fixed column widths so name and value fields sit under their headings without an oversized empty
column.

**Rationale**: The control currently shows the item's `iconName` string (or the word "Icon" when
empty), so it reads as text; the table stretches the value column to the right, leaving a large gap.
Both are presentation fixes using existing components (Principle VII). The chosen icon stays
discoverable by keyboard and labelled for assistive technology (Principle VI).

**Alternatives considered**: a new icon widget (unnecessary — the icon component exists); a
responsive card layout (a broader redesign than the finding asks for).

## D-206: The Tripmine hint names the target value as a redaction-safe field

**Decision**: Add one field to the Tripmine state view: the target value (the mined item's value for
the round's column), present for every role. The board's hint shows it next to the highest/lowest
text. Per-cell values remain hidden until picked.

**Rationale**: The target value is a deliberate, public clue (finding 6): the grid hides values, so
naming the value to hunt does not reveal which cell holds it. Exposing exactly one derived field — the
mined item's display value — is narrower than loosening redaction, and it is computed on the server
from the stored slot, so it cannot drift from the item.

**Alternatives considered**: revealing the mined item's row to players (leaks the cell); deriving the
value in the frontend from a name (the value, not the name, is required).
