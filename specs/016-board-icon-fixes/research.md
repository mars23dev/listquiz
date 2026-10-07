# Research: Board Icon And Layout Fixes

## D-1601: The item's icon is data and is carried to every view

**Decision**: Add `icon_name` (and the uploaded image where already available) to the item views: carry
it on `ListItemValue`, and on the `OrderItem`, `TripmineItem`, and `AnchorItem` DTOs; load
`i.icon_name` in the Order and Tripmine queries; map it through the frontend `OrderItem`/`TripmineItem`
types and `toQuizItem`/`toTripmineItem`. The boards already call `ItemIcon` with the item's icon name, so
they then render the real icon (flags included).

**Rationale**: The Connect view already carries the icon (the flags show there) and looks correct; Order
and Tripmine drop it in their DTOs, so they fall back to the generic icon. The icon belongs to the item,
so the fix is to carry the existing field through, not to have a game type choose an icon (FR-001,
FR-002). No new data is invented (Principle VII).

**Alternatives considered**: mapping a generic icon per game type (exactly the bug); adding a new
icon channel (the field already exists on `item`).

## D-1602: The Countries flags are not touched

**Decision**: Do not change the Countries items' `icon_name` (they stay `flag:<cc>`); the carried icon is
what makes the flags appear in Order and Tripmine too.

**Rationale**: The user explicitly said the countries must keep their flags. Since every shipped item
already has an icon name, "add icons to items" is only a safety net for any item missing one (FR-003,
FR-004).

**Alternatives considered**: replacing flags with a game icon (rejected by the user).

## D-1603: The Tripmine bar is built from the item count

**Decision**: Build the rank ladder from `items.length` (or the round's slot count) instead of a
hardcoded `1..=10`, so a twenty-item round shows twenty positions and a ten-item round shows ten.

**Rationale**: The bar currently hardcodes ten positions, so it disagrees with a twenty-item board
(FR-006, FR-007). Deriving the range from the actual items keeps the bar and board consistent with no new
state (Principle VII). The bar already uses `flex-1` positions, so twenty still fit one screen.

**Alternatives considered**: a separate stored slot count on the client (the items already carry it).

## D-1604: Connect rows use one fixed height

**Decision**: Give both the name cell and the value cell a single fixed height (e.g. `h-12`), with content
vertically centered and the icon constrained to a fixed box, so the two columns are identical and align
row-for-row.

**Rationale**: A minimum height still lets the icon-bearing name row grow past the value row. A single
fixed height (not a minimum) removes the difference (FR-008, FR-009). This is the smallest change that
guarantees equality (Principle VII).

**Alternatives considered**: removing the icons (loses information); a separate grid alignment (the rows
just need one height).

## D-1605: No schema, dependency, or rule change

**Decision**: No migration, no new dependency; scoring, hearts, matching, and turns are untouched.

**Rationale**: The icon field and the components already exist; only plumbing and two local layout fixes
change (FR-010; Principle VII).

**Alternatives considered**: none needed.
