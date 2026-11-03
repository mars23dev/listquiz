# Research: Session Feedback And Order Reveal

## D-401: Cues are derived by diffing consecutive session states

**Decision**: The frontend plays a cue from the **reflected session state**, not from a local
interaction. A pure function `deriveSessionCue(prev, next)` compares the previously rendered
`SessionState` with the newly fetched one and returns exactly one cue for that transition. A cue is
emitted only when `next.revision === prev.revision + 1` (a genuinely consecutive, live update);
`prev` being `null` (first load) or a revision jump (reconnect, heartbeat refetch) yields no cue.

**Rationale**: The stream deliberately carries only a revision and every visible value comes from a
fresh `GET /state` (contracts/realtime.md); there is no server event channel to subscribe to, and
adding one would change the realtime contract for a purely presentational concern. Diffing the state
the room already sees keeps a single source of truth and — with the `revision + 1` guard — honours the
existing promise that "a reconnect cannot replay a scoring sound". This is why the acting team also
hears its own event once: the local-interaction cue path is removed rather than kept (spec
Clarifications, FR-002).

**Alternatives considered**: a backend `lastEvent` field on `SessionState` (a realtime-contract change
and still needs an event id to avoid replay on reconnect); keeping the local `playSlideSound` calls in
the components (double cues for the actor, two sources of truth).

## D-402: A fixed event → cue mapping, matched to the reference draft

**Decision**: One named mapping, in priority order, from a detected transition to one cue:

| Detected transition | Cue |
|---|---|
| a pending item appears/changes (selection) | slide |
| an item is moved onto the bar (drop/confirm) | slide |
| a team's score rises and a bar side grows (correct placement) | chime |
| a team's hearts fall (wrong placement) | failure |
| a Tripmine item goes hidden → revealed and is the mine | explosion |
| a Tripmine item goes hidden → revealed and is safe | chime (safe) |
| a Tripmine round becomes completed | chime (round complete) |
| a placed item leaves a side for the pool (return) | remove |
| the session becomes `finished` | bonus chime |

**Rationale**: These are the cues the reference draft plays for the same events; the effect set
already exists in `frontend/src/utils/audio.ts`. Correct-vs-wrong is readable from public data (score
up / heart down), so no hidden value is consulted (FR-002 works for every role). The mapping stays a
small, testable table (Principles II and VII).

**Alternatives considered**: inferring correctness from the bar alone (ambiguous — a wrong placement
leaves the item in the pool, which a correct one does not, but both are state changes with no local
marker); synthesizing new effects (the spec pins the reference draft's set).

## D-403: Unlock the audio context on the first user gesture

**Decision**: Add an explicit `unlockAudio()` to `audio.ts` that creates and resumes the
`AudioContext`, and call it once from the session view on the first `pointerdown` **or** `keydown`.
Cue functions keep resuming the context as a fallback.

**Rationale**: Browsers refuse to start an `AudioContext` outside a user gesture; cues now fire from
the reflected event, which is not inside the gesture that caused it. Without an explicit unlock the
context stays suspended, which is exactly the observed bug (no sound, no browser tab audio
indicator). Listening for both pointer and key interaction keeps the fix keyboard-reachable
(Principle VI).

**Alternatives considered**: relying on the existing lazy `resume()` inside each play call (the play
call is not in a gesture, so it does not help the first cue); a visible "enable sound" button
(unnecessary friction, and the mute control already exists).

## D-404: Record the placing team on the Order bar; expose the Tripmine picker

**Decision**: Add `placed_by` to the Order board as a map from item id to team id, persisted in a new
additive `order_state.placed_by` JSON column, written when a correct placement joins a side, and
exposed as `OrderItem.placedBy`. Expose the already-stored Tripmine `revealed_by` as
`TripmineItem.pickedBy`. The pending preview is colored by the **active team** (the team on turn),
which is already public, so no redaction change is needed for players or spectators.

**Rationale**: Tripmine already stores which team revealed each item (`TripmineSlot.revealed_by`), so
its border can be colored with no new state. Order stores only item ids per side, so the placing team
must be recorded; a small id→team map keeps `ListItemValue` untouched and survives the reload that
every state read performs. Coloring the pending preview from the on-turn team avoids loosening the
rule that hides another team's proposed `pending.teamId`. All three sources are public in a running
round (the room watched the action).

**Alternatives considered**: adding a team field to `ListItemValue` (lost — the board is persisted as
bare id arrays); deriving the team from `score_event` (that table records no item); making
`pending.teamId` public (weakens an existing redaction rule for no gain).

## D-405: The border takes the acting team's color

**Decision**: Replace the Order higher/lower side tint and the Tripmine mine/safe border with the
acting team's color, applied from the team's existing `primaryHex` through one small helper
(`teamBorderStyle`). Mine/safe and higher/lower stay visible through the existing non-border cues
(the rank/label badge, the bomb/check icons, the point text, and feedback messages). When an Order
round is revealed, borders go neutral instead (D-406).

**Rationale**: The team already has a color on the scoreboard; reusing it makes the border name the
responsible team at a glance (FR-005–FR-008) and keeps one color vocabulary across both games.
`primaryHex` is exact and avoids a second color→class table (Principle VII). Keeping mine/safe
information off the border but on the card satisfies FR-009.

**Alternatives considered**: a Tailwind color-name → class switch (duplicates a palette and drifts
from `primaryHex`); a border plus a separate team chip (more chrome than the finding asks for).

## D-406: Reveal arranges the Order bar as a view-only presentation

**Decision**: When the session's reveal flag is set, the Order **view** is rebuilt into the correct
layout before redaction: a pure `arrange_revealed_order(anchor, items)` sorts every non-anchor item
into a side — values `<=` the anchor ascending on the left, values `>` the anchor ascending on the
right (matching the bar invariant) — and the view then shows left, anchor, right with the pool
emptied. The stored board is not modified, no placement is scored, and no heart moves.

**Rationale**: Arrangement depends on numeric values, which only the backend holds; doing it in the
view layer keeps the leak closed (no numeric in the DTO) and guarantees it cannot affect scoring,
because the scoring path reads the stored board, never the view. The sort mirrors
`side_stays_ordered`, so a revealed bar is exactly a bar that would have been legal, and it is
deterministic (stable sort on `(numeric, original index)`). FR-010–FR-014 all fall out of one pure
function (Principle II). Pool items are included because the reveal shows the round's full answer.

**Alternatives considered**: mutating the stored board on reveal (risks scoring and durability — a
reveal that awards points by accident is exactly what FR-012 forbids); arranging in the frontend
(needs numeric in the payload, weakening the value redaction).

## D-407: Add the column with an idempotent migration

**Decision**: `db.rs` keeps `CREATE TABLE IF NOT EXISTS` and adds a guarded step: inspect
`PRAGMA table_info(order_state)` and, when `placed_by` is absent, run
`ALTER TABLE order_state ADD COLUMN placed_by TEXT`. Reads treat a missing/`null` value as an empty
map.

**Rationale**: A plain `CREATE TABLE IF NOT EXISTS` never adds a column to an existing database, so
an existing session store must be upgraded in place; the guard makes the step safe to run on every
open, matching the existing idempotent-schema approach (Principle VII).

**Alternatives considered**: changing `left_ids`/`right_ids` to objects (a silent format change that
would drop the bar on an old database unless extra back-compat parsing is added).
