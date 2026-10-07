# Research: Round UI And List Fixes

## D-1501: Shipped lists are ordered and numeric, with a magnitude display formatter

**Decision**: Reorder every shipped list ascending by its ordering column (History Year, Statues Height,
Cinema Box Office, Countries Area). Store the true numeric value and derive each display with one
`format_value(n: f64) -> String` helper: `>= 1e12` → "… Trillion", `>= 1e9` → "… Billion", `>= 1e6` →
"… Million", else a thousands-separated number; trim trailing zeros. Cinema numerics become real dollars
(e.g. `1440000000.0`). The seed writes the formatted display and, via the item upsert, refreshes item
positions so an existing database is reordered.

**Rationale**: The user requires the lists read in ascending value order and large numbers shown as
"146 Million"/"1.3 Billion". Because years, heights, and small areas are below one million, a single
threshold-based formatter leaves them as plain numbers while population/box-office/area get suffixes —
no per-column configuration (Principle VII). Deriving the display from the number makes "only numeric
values" true and keeps display and ordering consistent (FR-001, FR-012, FR-013).

**Alternatives considered**: hand-authored display strings (fragile, can disagree with the number, and the
user asked for automatic exchange); per-column format metadata (more moving parts than the threshold
rule needs).

## D-1502: The seed upserts item position, not just values

**Decision**: In `seed_default_lists`, upsert `item` (`ON CONFLICT(id) DO UPDATE SET name, position,
icon_name`) in addition to the existing `quiz_list` upsert and `item_value` replace.

**Rationale**: `INSERT OR IGNORE` left an existing database's item positions untouched, so reordering the
constants would not reorder an existing list. Default lists are read-only, so overwriting name/position is
safe (FR-003, FR-009 from feature 014).

**Alternatives considered**: a data migration (heavier than an upsert for seed data); reordering only at
read time (would not persist and would fight the stored positions).

## D-1503: Connect name and value cells share one size

**Decision**: Give both the name cell and the value cell the same min-height and padding in
`ConnectBoard` (the name cell currently grows taller because of its icon), so the two columns are equal.

**Rationale**: The value cells render text only while the name cells include an icon, so the value column
looks shorter. One shared min-height (with vertical centering) makes both columns equal without changing
content (FR-004). Simplest fix that satisfies the requirement (Principle VII).

**Alternatives considered**: adding a placeholder icon to the value cells (misleading); changing the grid
(not needed — the rows just need equal height).

## D-1504: Tripmine item count is a per-round setting

**Decision**: Add an optional `tripmineSlots` (10 or 20, default 10) to the round input, store it in
`round_config.tripmine_slots` via an additive `ALTER TABLE` migration, thread it through `ValidRound`, and
make `sample_tripmine_grid(items, side, slots, rng)` sample and rank `1..=slots` with the mine side applied
across the chosen count. The board uses a denser grid and smaller cells for twenty items so it fits one
screen without scrolling.

**Rationale**: The item count belongs to the round configuration, like `tripmine_side`, so it is stored
and validated the same way (FR-005, FR-006). The additive column matches the existing migration pattern
and keeps old rows valid (absent → ten) (Principle VII). A twenty-item grid of 5×4 with reduced cell size
fits the standard window (FR-007), and ten-item behavior is unchanged (FR-008).

**Alternatives considered**: a global constant (cannot vary per round); a separate table (unnecessary for
one value).

## D-1505: The header view switch is gamemaster-only and opens a new tab

**Decision**: Render the header role switch only when the viewer is the gamemaster; when the gamemaster
activates a target, open `/{sessionId}/{rolePath(target)}` in a new browser tab and do not navigate the
current page. Direct links to a player/spectator view keep working.

**Rationale**: Players and the spectator should not see a role-switching control (FR-009), and the
gamemaster needs to open a view without losing their page (FR-010). Opening a new tab and leaving the
current route untouched satisfies both; the routes themselves are unchanged so direct links still work
(FR-011). One conditional and one `window.open` (Principle VII).

**Alternatives considered**: disabling the switch for non-gamemasters (still shows it); navigating in the
same tab (loses the gamemaster page).

## D-1506: Tests cover each fix

**Decision**: Add/adjust tests: a formatter unit test and a per-list ascending-order test in
`default_lists`; a Tripmine slot/rank test; `ConnectBoard` equal-height assertion; `TripmineBoard`
twenty-item render; `Session` view switch hidden for `p1`/`view` and present for the gamemaster. Update any
existing tests that assert old displays or positions.

**Rationale**: Each rule is business logic the constitution requires to be tested (Principles I, II).

**Alternatives considered**: relying on manual checks only (rejected — the rules are unit-testable).
