# Research: Per-Game Ordering

## D-1901: The per-list direction is removed everywhere

**Decision**: Remove `order_direction` from `ListInput`, `NewList`, `ListDetail`, `ParsedList` and
`parse_csv`, the list import request, the `quiz_list` schema and seed, and the frontend list editor/payloads.
A guarded migration drops the column.

**Rationale**: The field is a per-list value but lists now have multiple columns and rounds pick one, so a
single direction cannot fit (the user's point). Removing it from the data model and UI is the requested
change (FR-001, FR-002). Existing databases upgrade via a guarded `DROP COLUMN` (FR-007; Principle VII).

**Alternatives considered**: keep the column and ignore it (leaves a dead, confusing field); keep it as a
per-column value (more scope than requested).

## D-1902: Ordering is a property of the game mode

**Decision**: Add `QuizType::sort_order()` returning a small `SortOrder` (`Ascending`/`Descending`). For
now Order, Tripmine, and Connect all return `Ascending`. The Order round uses this instead of the list's
direction; Tripmine and Connect keep their current ascending behavior.

**Rationale**: The user wants the ordering to depend on the game mode. A named per-mode rule makes that
explicit and testable (FR-003; Principle II). All three modes currently ascend, so behavior is unchanged
(FR-004, FR-005, FR-006).

**Alternatives considered**: a boolean per call site (no single owner of the rule); a per-column direction
(not requested).

## D-1903: The Order round orders ascending regardless of any old direction

**Decision**: `round_state` builds the Order board from the round column ordered ascending (the game
mode's order), independent of any stored list direction.

**Rationale**: With the field gone, the Order round must not depend on it (FR-004). Ascending matches the
board's "lowest left, highest right" layout. A list previously set high-to-low now runs ascending, which
the user accepted.

**Alternatives considered**: keep reading a direction from somewhere (contradicts the removal).

## D-1904: No change to scoring, hearts, side, or matching

**Decision**: Only the direction field and the ordering source change.

**Rationale**: The removal is a data/UI change; the round rules are untouched (FR-008; Principle VII).

**Alternatives considered**: none needed.

## D-1905: Tests cover the removal and the per-mode order

**Decision**: Update `cargo test` (lists, csv_import, default_lists, round_state) to create/import without
a direction and assert the per-mode order; update `vitest` (ListManagement, SetupQuiz, client, App) to
drop the direction field.

**Rationale**: The rules are business logic the constitution requires to be tested (Principles I, II).

**Alternatives considered**: manual checks only (rejected).
