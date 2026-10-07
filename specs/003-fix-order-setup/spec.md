# Feature Specification: Fix Order And Setup

**Feature Branch**: `003-fix-order-setup`

**Created**: 2026-10-07

**Status**: Draft

**Input**: User description: "further findings:
1. When selecting Order why is the tripmine setting lowest/highest is displayed? This should only be shown when tripmine is selected.
2. Order is not working i select an item drop it to the right confirm and get this error: "Slot 3 is not on the left side. The slots there are 0 to 0." No correct no false just this error if i drop it to the other side its return false so i cannot play this game."

## Clarifications

### Session 2026-10-07

- Q: How is an item placed on the bar and how is correctness decided? → A: The player drops the new item into the gap between two items already on that side, or beside either end of the side; the side's items stay packed next to each other (contiguous). The game then judges the item from its neighbours: on the left of the anchor it must be lower than the item on its right; on the right of the anchor it must be higher than the item on its left; between two items it must be higher than its left neighbour and lower than its right neighbour. There is no pre-computed "correct spot" and no independent "slot" that can be refused — every gap is an allowed drop and is scored, matching the reference frontend prototype's drag handling.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Order placement works for the active team (Priority: P1)

A team on turn selects an item and drops it into a gap on the bar — between two items already on that
side, or beside either end of the side. The side's items stay packed next to each other. The game then
judges the item from its neighbours: correct placements score, wrong ones cost a heart, and a valid
drop never produces an internal "slot is not on this side" error. Right now an offered drop is
rejected, which makes the round unplayable.

**Why this priority**: The Order round cannot be played at all if a normal placement errors out; this
is a total blocker for the core game.

**Independent Test**: Start an Order round, select a pool item, drop it into a gap (between two items
or beside an end) on either side, and confirm. The result is +10 or a lost heart — never an error —
and the side stays contiguous.

**Acceptance Scenarios**:

1. **Given** an Order round on turn, **When** the team drops the item into a gap and confirms, **Then** the placement is judged correct or incorrect from its neighbours and the turn passes, with no error.
2. **Given** an empty side, **When** the team places the first item on that side, **Then** the drop is accepted and the item is the only one on that side.
3. **Given** a side that already holds items, **When** the team drops between two adjacent items or beside an end, **Then** the item is inserted at that gap and the side stays contiguous.
4. **Given** the team confirms a placement, **When** the item is on the wrong side of the anchor, **Then** the team loses one heart and the item returns to the pool (an incorrect result, not an error).
5. **Given** the gamemaster or the active-team player, **When** either places an item, **Then** both get the same scored outcome.


---

### User Story 2 - Tripmine side only appears for Tripmine rounds (Priority: P2)

While configuring rounds in Setup, the lowest/highest tripmine setting is shown only when the round
type is Tripmine. For an Order round it is not shown at all.

**Why this priority**: It is a misleading control that suggests a setting applies to Order rounds,
but it is not a functional blocker.

**Independent Test**: In Setup, switch a round between Order and Tripmine and confirm the tripmine
side control appears only for Tripmine.

**Acceptance Scenarios**:

1. **Given** a round set to Order, **When** the round configuration is shown, **Then** no tripmine side control is displayed.
2. **Given** a round set to Tripmine, **When** the round configuration is shown, **Then** the tripmine side control is displayed.
3. **Given** a round set to Tripmine with a chosen side, **When** the type is switched to Order, **Then** the control disappears and no tripmine side is submitted for that Order round.

---

### Edge Cases

- A gap is chosen, the team confirms, and the board has moved on in the meantime: the action is judged against the current board and returns a scored result or a not-ready outcome, never an internal slot-range error.
- A side already holds the maximum number of items: no further gap is offered on that side.
- The team selects an item and changes the chosen gap more than once before confirming: the last chosen gap is the one judged.
- A gap is chosen but the side's length changes before confirm: the team is told to choose again rather than shown an internal error.
- An item dropped between two neighbours is judged by those neighbours: higher than the left one and lower than the right one (or the anchor on the relevant side).
- The gamemaster acts on behalf of the active team exactly as that team would.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: A new item MUST be placed into a gap: between two items already on that side, or beside either end of the side. Each side's items MUST stay contiguous (packed next to each other), and the bar MUST offer exactly these gaps as drop targets.
- **FR-002**: Confirming a drop MUST judge the item from its neighbours — on the left of the anchor it must be lower than its right neighbour; on the right of the anchor it must be higher than its left neighbour; between two items it must be higher than its left neighbour and lower than its right neighbour — and MUST award points for a correct placement or cost a heart for an incorrect one. It MUST NOT produce an internal slot-range error for any gap the bar offered.
- **FR-003**: A correct placement MUST award the existing points and add the item to the bar; an incorrect placement MUST cost the existing heart and leave the item in the pool.
- **FR-004**: Both the gamemaster and the active-team player MUST be able to complete a placement and MUST receive the same outcome.
- **FR-005**: When the sequence changes between choosing a gap and confirming it (for example the side's length changed), the team MUST be prompted to choose again rather than shown an internal-range error.
- **FR-006**: The Setup round configuration MUST display the tripmine side control only when the round type is Tripmine.
- **FR-007**: An Order round MUST NOT submit a tripmine side, and the control's value MUST NOT leak into an Order round.
- **FR-008**: All changed controls MUST remain keyboard-operable with visible focus, and no change may freeze input or trigger a full-page reload.

### Key Entities

- **Order bar**: the anchor with a left side and a right side. Each side keeps its items contiguous; its drop targets are the gaps before the first item, between adjacent items, and after the last item, capped at the side limit.
- **Placement**: the selected item plus the chosen gap; judged correct or incorrect from its neighbours (left neighbour lower, right neighbour higher, relative to the anchor).
- **Round configuration**: a round's type (Order or Tripmine) and, only for Tripmine, its mined side.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A team can complete an Order placement from an offered gap in 100% of attempts and receive a scored result.
- **SC-002**: 0 internal slot-range errors are shown for any gap the bar offered.
- **SC-003**: In Setup, the tripmine side control is shown for 100% of Tripmine rounds and 0% of Order rounds.
- **SC-004**: A complete Order round (select, drop into a gap, confirm) can be played through by a team without any error, and the side stays contiguous after each correct placement.

## Assumptions

- The bar keeps each side's items contiguous; the drop targets are the gaps between adjacent items and at the ends, matching the reference frontend prototype's drag handling.
- Correct/incorrect judgement uses the item's neighbours and the anchor; the points awarded and the heart cost are unchanged from the current rules.
- The observed error `"Slot 3 is not on the left side. The slots there are 0 to 0."` came from treating the visual columns as independent slots and sending a column the side did not have; the fix sends the chosen gap as a valid position on a contiguous side.
- "Tripmine setting" means the lowest/highest (mined side) selector in the Setup round configuration.
- This is a correctness fix; no new rounds, scoring rules, or session types are introduced.
