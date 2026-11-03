# Research: Fix Order And Setup

## D-301: A side is contiguous; a drop chooses a gap

**Decision**: A bar side keeps its items packed next to each other. A new item is placed into a
**gap** — between two adjacent items or beside either end — and the client sends that gap as the
item's insertion position on the side (`0..=side length`). The side remains contiguous afterwards.

**Rationale**: The bar is not a set of independent slots; it is a contiguous run of items on each side
of the anchor, exactly as the reference frontend prototype drags them. `0..=length` is precisely the set of gaps:
before the first item (`0`), between items (`i`), and after the last item (`length`). The visual
columns the player aims at are a display detail; the drop target the player chooses is a gap
(Principles II and VII). This matches the corrected model in the spec's Clarifications.

**Alternatives considered**: treating the fixed five columns as independent slots that can be refused
(the original defect); loosening the server to clamp an arbitrary index (hides the mismatch instead of
modelling gaps).

## D-302: Map the dropped gap column to the insertion position in the client

**Decision**: The client translates the column the player drops on to the corresponding gap position
on that side (`0..=length`) before sending `choose_slot`. No backend change; `order_round` keeps
rejecting a position the side does not have.

**Rationale**: The defect was a space mismatch: a fixed-column index was sent where a gap position was
expected, producing `"Slot 3 is not on the left side. The slots there are 0 to 0."`. The server rule
is already the correct neighbour-based judgement (`validate_placement` inserts at the position and
checks the side stays ordered relative to the anchor, ties allowed); the client must speak the same
space (Principles II and VII). The same column→index arithmetic the bar already uses to lay an item
into its display slot is reused so the item lands in the gap the player aimed at.

**Alternatives considered**: a defensive server clamp (two sources of truth for a valid gap); a wider
bar redesign (larger than the finding needs).

## D-303: Correctness is judged from neighbours (server rule, unchanged)

**Decision**: Keep the existing judgement: an item left of the anchor must be lower than its right
neighbour, an item right of the anchor must be higher than its left neighbour, and an item between two
items must be higher than its left neighbour and lower than its right neighbour (ties allowed). No
change to scoring or heart cost.

**Rationale**: This is exactly the intended rule from the clarification, and it already exists as a
named, tested rule; the fix is only that the client must send a gap the side has so the rule can run.

**Alternatives considered**: pre-computing a "correct spot" and filtering drops to it (the model the
clarification rejects — the drop is free, the judgement is what scores).

## D-304: Hide the tripmine side control for Order rounds

**Decision**: In the Setup round configuration, render the tripmine side control only when the round
type is Tripmine; for Order, do not render it and do not submit a side.

**Rationale**: The control is already disabled for Order rounds but still shown, which implies a
setting that does not apply. A conditional render is preferable to a disabled control for a value that
must not be sent (Principle VII). The create-session rule that an Order round must not carry a side is
unchanged.

**Alternatives considered**: leaving the control disabled (still misleading); always rendering it and
ignoring it server-side (keeps the confusing control).
