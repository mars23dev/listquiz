# Research: Expand Default Lists And Add Countries

## D-1401: Flags render via `flag-icons` and a `flag:<cc>` icon-name convention

**Decision**: Add the `flag-icons` package (MIT) to the frontend, import its CSS once in the app entry,
and teach `ItemIcon` to render a flag when `icon_name` starts with `flag:` (the remainder is an
ISO 3166-1 alpha-2 code, e.g. `flag:de` → `<span class="fi fi-de" />`). Country seed items set
`icon_name = "flag:<cc>"`.

**Rationale**: The app's `ItemIcon` only maps lucide names (unknown → `HelpCircle`) or an uploaded image
BLOB; there is no flag source. Emoji flags do not render on the target platform (Windows lacks the flag
glyph), so an offline SVG set is required (FR-005). A prefixed name keeps flags from colliding with the
existing lucide names and needs one small parse in one component (Principle VII).

**Alternatives considered**: emoji flags in `icon_name` (broken on Windows); vendoring ~195 SVGs (more
files and upkeep than a small MIT package); storing flag images in `item.icon_image` (BLOB seeding is
awkward and duplicates the package).

## D-1402: The Countries list is exempt from the cap with no rule change

**Decision**: Do not change `validate_list_size` or `MAX_ITEMS`. Seed the 195-country list directly.

**Rationale**: Default lists are read-only (`lists::reject_default_list`), so the only enforcement point
(`lists::mod.rs:206`) never runs for them. The 100-item maximum therefore already applies to every
editable list and never to the seeded Countries list, satisfying FR-003 and FR-011 with no code change
(Principle VII).

**Alternatives considered**: special-casing `validate_list_size` by list id (unnecessary — the seeded
list never passes through it); raising `MAX_ITEMS` globally (would wrongly allow huge user lists).

## D-1403: Seeding upserts so an existing database becomes consistent

**Decision**: In `seed_default_lists`, upsert the `quiz_list` row (`INSERT ... ON CONFLICT(id) DO UPDATE
SET title, description, order_direction, item_count`) instead of `INSERT OR IGNORE`; keep `item` as
insert-or-ignore for new ids and `item_value` as insert-or-replace.

**Rationale**: Seeding runs on every startup (`main.rs:23`). With `INSERT OR IGNORE`, an existing
database kept the old `item_count`/metadata, so an extended list showed a stale count (FR-009). Default
lists are read-only, so overwriting their metadata is safe. New items are added by the item insert and
their values refreshed by the value replace.

**Alternatives considered**: a migration (heavier for seed data); deleting and recreating the default
folder (would drop nothing valuable but is more disruptive than an upsert).

## D-1404: Countries data shape

**Decision**: One country per sovereign state (195: 193 UN members + Vatican City + Palestine), id
`c_<cc>`, `icon_name = "flag:<cc>"`, columns `area_km2` "Area (km²)", `population` "Population",
`founded` "Founded"; `order_direction = "low_to_high"`. Area and population are recent estimates;
`founded` is the widely accepted independence/establishment year (negative for BC).

**Rationale**: Matches the spec (195 states; area, population, founded; low to high). A stable id per
ISO code makes the list reproducible and the flag name trivial.

**Alternatives considered**: including dependent territories (~250) — excluded by the clarification;
full dates for `founded` (a year keeps the numeric value meaningful for ordering).

## D-1405: Existing lists keep their rules while growing

**Decision**: Expand World History to 30, Iconic Statues to 25, Blockbuster Cinema to 30, keeping the
existing columns, `order_direction`, icons, and unique-id/name/value rules.

**Rationale**: The spec's default counts; each stays inside 11–100 and gives rounds variety (FR-001).

**Alternatives considered**: filling each to 50 (more authoring without a stated need).

## D-1406: Tests are updated to the new reality

**Decision**: Update `default_lists/tests.rs`: expect four shipped lists; assert each of the three
expanded lists is within 20–50; assert the Countries list has 195 items, every item's `icon_name` starts
with `flag:`, and every item has area/population/founded with finite numbers; keep the unique-id and
value checks. Add a `flag:` render test to `ItemIcon`.

**Rationale**: The current tests hard-code three lists and exactly eleven items, which the feature
changes; the new assertions encode the spec's success criteria (Principles I and II).

**Alternatives considered**: loosening the assertions to `>= MIN_ITEMS` only (would not test the new
guarantees).
