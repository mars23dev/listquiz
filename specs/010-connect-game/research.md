# Research: Connect Game And Tripmine Marker

## D-1001: Connect is a new self-contained round module

**Decision**: Add `connect_round`, mirroring `order_round`: plain board structs and pure functions
(`sample_connect_round`, `pick_name`, `pick_value`, `confirm`, `round_is_over`), with its own stored
board and view. It reuses the shared scoring (points + hearts), team standings, and the round-state
helpers.

**Rationale**: The constitution's Principle V wants a feature addable without editing other feature
modules. Connect shares no rule with Order or Tripmine (names vs. values, not a bar or a grid), so it
is its own module, not a mode of another. Reusing scoring and teams keeps one definition of points and
hearts (Principles II and VII).

**Alternatives considered**: a mode inside `order_round` (couples two unrelated games); a generic
"matching" framework (a speculative abstraction for one game, against Principle VII).

## D-1002: The board is ten names and ten shuffled values; values are public

**Decision**: A round draws ten items and their chosen-column values. The left column is the ten
names (with icons); the right column is the ten values shuffled. Both are visible to every role — a
player cannot match without seeing the values.

**Rationale**: Unlike Order (where values are hidden until reveal), Connect is a matching game, so
hiding the values would make it unplayable. This is a deliberate departure from the Order redaction,
documented in the contract (Principle II).

**Alternatives considered**: hiding values until reveal (breaks the game); hiding names (the names
are the puzzle's left side).

## D-1003: A confirm scores points or a heart; the turn always passes

**Decision**: Confirming a matching pair awards 10 points; a mismatch removes one heart. Either way
the turn passes (the clarification). The pair is judged only on confirm; a matched pair leaves the
board (cannot be picked again). The round ends when all pairs are matched or no team has hearts.

**Rationale**: This is the requested rule and matches Order's rhythm (one action per turn, hearts as
the mistake currency, read-only when nobody has hearts). Reusing `apply_points`/`apply_hearts` and
the existing turn advance keeps one scoring path (Principle II). The mid-turn selection is a staging
area, so the judgement happens once, on confirm.

**Alternatives considered**: the same team keeps playing on a correct pair (rejected in the
clarification); losing a heart per wrong value (one heart per wrong pair is the spec).

## D-1004: Connect uses hearts; Tripmine does not

**Decision**: Connect resets hearts per round and uses them like Order (shown, lost on a wrong pair,
read-only at zero). The existing `showHearts` rule (`quizType !== 'TRIPMINE'`) already covers Connect,
so hearts show in Connect automatically.

**Rationale**: The spec gives Connect three hearts; feature 009 made hearts Order-only by hiding them
only for Tripmine. Connect is an Order-like game, so it keeps hearts (Principles II and VII).

**Alternatives considered**: hearts hidden in Connect (contradicts the spec's three hearts).

## D-1005: The Tripmine mine position is a public rank

**Decision**: Add `mineRank` to the Tripmine view (the rank of the mine), always present for every
role, and mark it on the progress bar from the start. The item's identity stays hidden until revealed.

**Rationale**: The request is a positional overview for the whole room (clarification: every role).
The mine's rank is not its identity, and the displayed order is already shuffled, so the marker adds
overview without naming the item (Principle II). It is display-only, so no rule changes.

**Alternatives considered**: revealing the item (the request explicitly does not want that); a
gamemaster-only marker (rejected in the clarification).

## D-1006: Connect cues reuse the existing set

**Decision**: A correct pair plays the success cue, a wrong pair the failure cue. The cue derivation
gains a Connect case: a matched pair growing while the score rises is a success; a heart drop is the
failure (already covered).

**Rationale**: The cue set is fixed (feature 004); Connect has no new sound. Reading the matched-pair
count mirrors the Order bar-growth check (Principle VII).

**Alternatives considered**: a new Connect sound (unnecessary); no cue (the room expects feedback).
