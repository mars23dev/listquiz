# Research: Tripmine Mode

## D-901: Hearts are hidden in a Tripmine round (display only)

**Decision**: During a Tripmine round the frontend hides the heart indicator (and the heart adjust
controls); Order keeps them. No server change: the Tripmine rules already never change hearts, and the
timeout is already pass-only (feature 006).

**Rationale**: Hearts are an Order mechanic; a Tripmine mistake is the mine, which ends the game, so
showing hearts implies a life that cannot be lost. Hiding the indicator (rather than deleting hearts)
keeps the heart value available for Order and for the between-rounds reset, and needs no schema or rule
change (Principles II and VII).

**Alternatives considered**: removing hearts from the data model (touches Order, the reset, and the
schema for no gain); showing hearts but never changing them (still misleading).

## D-902: The cue derivation returns an ordered list

**Decision**: `deriveSessionCue` becomes `deriveSessionCues`, returning an ordered list. The event cue
is computed first (mine explosion, safe reveal, placement, timeout, and so on), and the game-end cue
(`bonus`) is appended last when the session finished in the same transition. The `finished` early
return is removed.

**Rationale**: Feature 007 made the last round's end finish the game, so one transition can be both a
round end and a game end. The current function returns `bonus` for the finish before it checks the mine
reveal, so the explosion is skipped — the reported bug. Ordering the event cue before the game-end cue
makes "round end first, then game end" explicit and general (any round-ending event, both game types),
and a non-finishing round end still yields a single cue (Principle II).

**Alternatives considered**: special-casing the mine (misses an Order placement that finishes the
game); a second independent "did the game end" cue played after a delay (more moving parts).

## D-903: A Random (Hard) side

**Decision**: `TripmineSide` gains `Random` (persisted as `"random"`, shown as "Random (Hard)"). In a
Random round the mine is a randomly chosen rank among the ten, and the searched target is a random
unrevealed non-mine item, re-chosen after each target hit. Scoring is unchanged.

**Rationale**: The finding wants a hard mode where the mine is not the extreme and the search is
unpredictable. Keeping the same scoring and pick flow means only how the mine and the searched target
are chosen changes, which is exactly the existing `sample_tripmine_grid` and `next_searched_rank`
responsibilities (Principles II and VII). Lowest/Highest keep their deterministic ladder.

**Alternatives considered**: a separate round type (duplicates the whole Tripmine flow); choosing the
mine at pick time (breaks the "answer key" the gamemaster sees and the deterministic setup).

## D-904: The random search re-selection uses the runtime RNG

**Decision**: The initial Random mine and search target use the RNG passed to `start_tripmine_round`;
the re-selection after a hit uses the runtime's thread RNG inside `next_searched_rank`. Tests assert
the invariants (the target is unrevealed and not the mine; the mine can be any rank) rather than a
fixed rank.

**Rationale**: Re-selection happens deep in `pick_item`, which does not take an RNG today; threading an
RNG through every caller for a non-deterministic choice adds ceremony without value. The invariants are
the real contract, so they are what the tests pin (Principles II and VII).

**Alternatives considered**: threading an RNG through `pick_item` and the route (more churn); a seeded
RNG stored on the round (new state for no benefit).
