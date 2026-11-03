# Feature Specification: Expand Default Lists And Add Countries

**Feature Branch**: `014-default-lists-countries`

**Created**: 2026-10-07

**Status**: Draft

**Input**: User description: "- can you extend the existing lists? To have 20-50 items (depends on the list) - Also add a list for all countries using the flags as icon and as value their size in square km and their population."

## Clarifications

### Session 2026-10-07

- Q: How should the country flag icons be rendered? → A: Add the `flag-icons` package and show each country's flag as its item icon (offline, works on Windows).
- Q: How many countries should the Countries list hold? → A: All countries; the Countries list is exempt from the 100-item maximum (the limit is bypassed for this list only).
- Q: What counts as "all countries"? → A: The 195 sovereign states (193 UN members plus the two observer states, Vatican City and Palestine).
- Q: What values should each country carry? → A: Area in square kilometres, population, and date founded.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The shipped lists carry more items (Priority: P1)

The three lists the game ships with (World History, Iconic Statues, Blockbuster Cinema) currently hold
only eleven items each. Every round draws from a list, so a short list makes rounds repetitive. Each
shipped list should carry many more items (roughly twenty to fifty, depending on the list) while still
meeting the game's rules, so rounds have real variety out of the box.

**Why this priority**: Richer default lists improve every game type immediately and are the bulk of the
content work.

**Independent Test**: Start a game on each shipped list and confirm each list has well over eleven items
(within the allowed range), all with names, icons, and values, and that rounds still start.

**Acceptance Scenarios**:

1. **Given** a freshly seeded game, **When** the shipped lists are viewed, **Then** each list holds between twenty and fifty items.
2. **Given** a shipped list, **When** a round draws from it, **Then** every item has a name, an icon, and at least one value with a finite number.
3. **Given** a shipped list, **When** it is validated, **Then** its size is within the game's allowed minimum and maximum and its item ids are unique.
4. **Given** a shipped list, **When** an Order round starts, **Then** a valid anchor plus the drawn items exist so the round can begin.

---

### User Story 2 - A Countries list with flags (Priority: P2)

A new shipped list of countries should be available. Each country shows its national **flag** as its item
icon, and its values are its **area in square kilometres**, its **population**, and its **date founded**.
This gives the room a recognisable, factual list for Order, Tripmine, and Connect rounds.

**Why this priority**: A new, highly recognisable list with a visual flag cue; independent of the
existing-list expansion but part of the same request.

**Independent Test**: Open the Countries list, confirm each entry shows the country's flag and carries an
area, a population, and a date-founded value, and start a round from it.

**Acceptance Scenarios**:

1. **Given** a freshly seeded game, **When** the lists are viewed, **Then** a Countries list is present alongside the existing lists.
2. **Given** the Countries list, **When** its items are shown, **Then** each item's icon is that country's flag.
3. **Given** a country item, **When** its values are read, **Then** it has an area in square kilometres, a population, and a date founded.
4. **Given** the Countries list, **When** a round draws from it, **Then** the round can start (enough items with values) and the flags render.
5. **Given** the Countries list, **When** it is validated, **Then** it meets the same uniqueness and value rules as the other shipped lists (its size is exempt from the maximum).

---

### Edge Cases

- Every shipped list except Countries stays within the game's allowed item range (at least eleven, at most one hundred).
- The Countries list is exempt from the maximum and may hold every country; the maximum still applies to every other list (including user-created ones).
- Item ids are unique within each list, and value displays are non-empty with finite numbers.
- Adding items to an already-seeded database must not leave a stale item count or a half-updated list.
- Flags must render on the target platform (Windows), where emoji flags are not supported; the flag must not fall back to a generic "unknown" icon.
- A country item carries a flag, an area, a population, and a date founded; every value is a finite number.
- Founding dates for very old or contested states use a widely accepted year (negative for BC); each is a finite number.
- The Countries list must be usable by all game types that draw from lists.
- A list other than Countries must not exceed the maximum; the expansion must not push any of them over it.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Each of the three shipped lists (World History, Iconic Statues, Blockbuster Cinema) MUST be expanded to between twenty and fifty items.
- **FR-002**: Every shipped item MUST keep a name, an icon, and at least one value with a finite number, and all item ids within a list MUST be unique.
- **FR-003**: Every shipped list other than Countries MUST remain within the game's allowed item range (minimum eleven, maximum one hundred); the Countries list MUST be exempt from the maximum.
- **FR-004**: The system MUST ship a new Countries list alongside the existing shipped lists.
- **FR-005**: Each country item's icon MUST be that country's flag, rendered offline and visibly on Windows (no generic fallback icon).
- **FR-006**: Each country item MUST carry an area value in square kilometres, a population value, and a date-founded value.
- **FR-007**: The Countries list MUST satisfy the same uniqueness and value rules as the other shipped lists, but MUST be allowed to exceed the one-hundred-item maximum.
- **FR-008**: The Countries list MUST be selectable in setup and usable by the game types that draw from lists (Order, Tripmine, Connect).
- **FR-009**: Seeding MUST leave an already-seeded database consistent: no stale item counts and no partially updated list.
- **FR-010**: The shipped-list titles and the existing lists' behavior MUST otherwise stay unchanged.
- **FR-011**: The system MUST enforce the one-hundred-item maximum for every list except Countries, and MUST permit the Countries list to hold every country.

### Key Entities *(include if feature involves data)*

- **Shipped list**: A list seeded on first run (World History, Iconic Statues, Blockbuster Cinema, and the new Countries list).
- **Shipped item**: An entry in a shipped list with an id, a name, an icon, and one or more values.
- **Country item**: A shipped item whose icon is a national flag and whose values are area (km²), population, and date founded.
- **Item value**: A display text and a finite number for a column (e.g. area, population, year).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Each of the three expanded shipped lists holds between twenty and fifty items; the Countries list holds every country and is not capped at one hundred.
- **SC-002**: Every shipped item has a name, an icon, and a finite value; no duplicate ids exist within a list.
- **SC-003**: The Countries list is present with one flag icon, one area (km²), one population, and one date founded per country, for every one of the 195 countries.
- **SC-004**: Flags render on Windows for 100% of country items (no generic fallback icon).
- **SC-005**: A round can be started from every shipped list, including Countries.
- **SC-006**: A freshly seeded database and an already-seeded database both show correct item counts for every shipped list.

## Assumptions

- Target counts (chosen because "it depends on the list"): World History 30, Iconic Statues 25, Blockbuster Cinema 30. These stay within the allowed 11–100 range.
- The Countries list holds every country and is exempt from the 100-item maximum (the limit still applies to all other lists).
- Country values are area in square kilometres, population (recent estimates), and date founded (year of independence/establishment, widely accepted; negative for BC), all ordered low to high like the other shipped lists.
- Flags are provided offline via the `flag-icons` package (MIT) and shown as the item icon; no network access is required at runtime.
- Population and area numbers are approximate but consistent and finite.
- Extending the shipped lists may require resetting an existing local database (or an equivalent reseed step) so counts are correct; this is acceptable for shipped seed data.
