# Feature Specification: Fix Play Findings

**Feature Branch**: `002-fix-play-findings`

**Created**: 2026-10-07

**Status**: Draft

**Input**: User description: "Findings/Bugs that needs to be adapted:
1. In List creation the icons show the icon text instead of the icon, the porportions of the table seem of / not fitting see screenshot under ../findings/tablelist.jpg
2. In Setup when you enter a teamname after the first character entered the field loses focus you you can only enter 1 character at a time
3. The Quizgame Tripmine the selected items are not in a random order? The highest/lowest item is the tripmine but if this is an ordered sequence to the players they can just go from left to right or the other way around.
4. When opening the link nothing is there until the gamemaster clicks on next round. That has 2 issues first it must be more clear for the gamemaster that he has to start the round e.g. use the empty space in the middle for a large start button and the second to players and spectators it should display something like waiting for the game to be started in the huge empty middle area until the game starts
5. Players currently cannot play the game. Only the gamemaster page can interact with the items this should not be the case. GM behavior spectator/view as well correct, but if Team 1 is at a turn that team needs to be able to interact with it as well and the other players not."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The team on turn plays from its own player view (Priority: P1)

A player opens their team link on a phone or laptop. When it is their team's turn, they can
select and place an item in an Order round, or pick a cell in a Tripmine round, exactly as the
gamemaster currently can. When it is not their turn, their board is read-only. Spectators and the
stage view never receive controls.

**Why this priority**: Without this, the game is single-player from the host's device only; the
player views are decorative. This is the difference between a working game and a demo.

**Independent Test**: Open the gamemaster link and the link for the team that is on turn in two
browser windows. Confirm the team's own window can select and confirm a placement and that the
other player window cannot. Confirm the spectator window cannot.

**Acceptance Scenarios**:

1. **Given** an Order round where team 1 is on turn, **When** the team 1 player selects a pool item and confirms a bar slot, **Then** the placement is scored and the player's own board updates.
2. **Given** the same round, **When** the team 2 player tries to select an item, **Then** the board offers no acting control and no action is sent.
3. **Given** a Tripmine round where team 1 is on turn, **When** the team 1 player picks a cell, **Then** the pick is scored and revealed to every view.
4. **Given** any playing round, **When** the spectator/stage view is opened, **Then** it shows a read-only board with no acting controls.
5. **Given** the gamemaster view, **When** the gamemaster acts, **Then** host behaviour is unchanged and still accepted.

---

### User Story 2 - A clear start affordance and a waiting state before play begins (Priority: P1)

When the gamemaster opens a freshly created session, the large empty middle area shows a prominent
way to start the first round instead of an empty page. Players and spectators who open their links
before the round has started see a clear "waiting for the game to start" message in that same area,
which disappears the moment the round begins.

**Why this priority**: An empty screen reads as a broken link; both the host and the room need to
know what to do and what is happening.

**Independent Test**: Create a session and open the gamemaster link plus a player link. Confirm the
gamemaster sees a large, obvious start control in the middle, the player sees a waiting message, and
after the gamemaster starts the round both settle into the live board.

**Acceptance Scenarios**:

1. **Given** a session in setup with no round started, **When** the gamemaster opens their link and unlocks, **Then** a large start control is shown in the middle of the board area.
2. **Given** the same session, **When** a player or spectator opens their link, **Then** a waiting-for-start message is shown in the middle area.
3. **Given** the round has started, **When** any view is refreshed, **Then** the start control and the waiting message are gone and the live board is shown.

---

### User Story 3 - Tripmine grid is shown in random order (Priority: P2)

In a Tripmine round the ten items appear in a shuffled order, unrelated to their ranked position, so
the room cannot simply sweep the grid from one end to the other. The tripmine is still the highest
(or lowest) item and this is only knowable through play.

**Why this priority**: A grid laid out in rank order trivialises the round and removes the deduction
the quiz depends on.

**Independent Test**: Start a Tripmine round several times and confirm the on-screen order of the ten
items differs between rounds and does not follow the value ranking, while the hidden tripmine and the
searched position behave as before.

**Acceptance Scenarios**:

1. **Given** the same list played as a Tripmine round twice, **When** the two grids are compared, **Then** the item order differs.
2. **Given** a Tripmine grid, **When** it is displayed, **Then** its order does not follow the ranked values.
3. **Given** a Tripmine round, **When** a cell is picked, **Then** the revealed computed position is still correct and the searched position advances as before.

---

### User Story 4 - Team names can be typed in Setup (Priority: P2)

While configuring a session, the gamemaster types a multi-character team name into a team field and
the whole name is entered; the field keeps focus until they leave it.

**Why this priority**: Team naming is a normal setup step; being unable to type more than one
character makes the field unusable.

**Independent Test**: In the setup wizard, type a multi-character name into a team field and confirm
the full name is retained and another field can then be focused.

**Acceptance Scenarios**:

1. **Given** the participants step, **When** the gamemaster types "Team Cyan Pulsar" into a team field, **Then** the complete name is shown and retained.
2. **Given** a name is partially typed, **When** the gamemaster continues typing, **Then** each keystroke extends the same field without losing focus.

---

### User Story 5 - List editor shows real icons and fits its table (Priority: P3)

In the list editor each row's icon control shows an icon rather than the word "Icon", and the row
table lays out name and value fields next to their column headings without large empty gaps,
regardless of window width.

**Why this priority**: It is a visual/usability defect that misrepresents the control and makes the
editor awkward, but it does not block creating or saving a list.

**Independent Test**: Open the list editor and confirm each row shows an icon control (a glyph, not
the literal word), and that the row fields line up under their headings without a large gap, down to
a narrow window.

**Acceptance Scenarios**:

1. **Given** the list editor with rows present, **When** a row is shown, **Then** its icon control displays an icon (or a clearly empty icon affordance), never the literal text "Icon".
2. **Given** the list editor, **When** it is viewed at common widths, **Then** the name and value fields sit under their headings with even spacing and no oversized empty column.

---

### User Story 6 - Tripmine hint names the target value (Priority: P2)

In a Tripmine round the hint that says the tripmine is the highest (or lowest) item also shows the
tripmine's value next to that text — the value, not the item's name — so the room knows exactly which
number they are hunting while every grid cell's value stays hidden until it is picked.

**Why this priority**: The hint currently gives only a direction ("highest"), which is a weak clue;
naming the target value turns the round into a clear "find this value" hunt without revealing which
cell holds it.

**Independent Test**: Start a Tripmine round and confirm the hint shows the highest (or lowest) item's
value next to the direction text, that no grid cell value is shown, and that the displayed value
matches the value of the tripmine item.

**Acceptance Scenarios**:

1. **Given** a Tripmine round hunting the highest item, **When** the board is shown, **Then** the hint reads that the tripmine is the highest and shows that item's value next to it.
2. **Given** the same round, **When** a grid cell has not been picked, **Then** its value stays hidden even though the target value is shown in the hint.
3. **Given** the hint value, **When** the tripmine item is eventually picked, **Then** the value that was shown matches the picked tripmine's value.

---

### Edge Cases

- A player whose team has run out of hearts tries to act: the board is read-only for them and no action is sent.
- Two views of the same active team act at almost the same time: the second action reaches a board that has already moved on and is rejected without corrupting state; the views re-read and converge.
- A player view is opened for a role with no team behind it: it is read-only and shows no turn emphasis.
- The round starts while a player is mid-selection: the board re-reads and the selection is cleared.
- A Tripmine round is started twice on the same list: the two shuffles differ, and a shuffle that happens to match the ranked order is still accepted.
- The start control is used twice quickly: only one round starts.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The active team's own player view MUST allow that team to perform the same in-round actions as the gamemaster (Order: select item, choose slot, confirm, cancel; Tripmine: pick a cell).
- **FR-002**: A player view that is NOT the team on turn MUST be read-only and MUST NOT send or offer acting controls.
- **FR-003**: Spectator and stage views MUST remain read-only with no acting controls and no turn emphasis.
- **FR-004**: The gamemaster view MUST keep all existing host controls and behaviour, including acting on behalf of the active team.
- **FR-005**: The server MUST accept an in-round action only from the gamemaster or from the player role that owns the team on turn, and MUST reject any other role or team.
- **FR-006**: When a session has no round started yet, the gamemaster view MUST show a prominent start control in the board area that starts the first round.
- **FR-007**: When a session has no round started yet, player and spectator views MUST show a waiting-for-start message in the board area.
- **FR-008**: Once a round has started, the start control and the waiting message MUST NOT be shown.
- **FR-009**: A Tripmine round's ten items MUST be displayed in an order that is shuffled independently of their ranked values.
- **FR-010**: The shuffled display MUST NOT change which item is the tripmine, the ranked position of any item, the searched position, or any scored outcome.
- **FR-011**: The setup wizard team-name fields MUST retain focus while typing and MUST keep the full entered name.
- **FR-012**: Each list-editor row MUST show an icon control that displays an icon rather than the literal word used as a placeholder.
- **FR-013**: The list-editor row table MUST align each field under its column heading without an oversized empty column at common window widths.
- **FR-014**: All changed interactive controls MUST remain reachable and operable by keyboard alone with visible focus, and no change may freeze input or trigger a full-page reload.
- **FR-015**: The Tripmine hint MUST show the tripmine item's value (not its name) next to the text that states whether the tripmine is the highest or the lowest item.
- **FR-016**: Showing the target value in the hint MUST NOT reveal the value of any grid cell that has not yet been picked.

### Key Entities *(include if applicable)*

- **Active team**: the team whose turn it is; the only team that may act during a round.
- **Player role**: one of the four player links; the role that owns the active team may act, the others may not.
- **Session status**: whether a round is in play; drives the start control versus the waiting message.
- **Tripmine grid**: the ten items of a round; their display order is independent of their ranked positions.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: A team on turn can complete a placement from its own player view without the gamemaster acting, in 100% of attempts.
- **SC-002**: Non-active players and spectators can initiate zero scoring actions; any attempt is refused and changes no board state.
- **SC-003**: On a freshly created session, both the gamemaster start control and the player waiting message are visible without any prior interaction.
- **SC-004**: Starting the same Tripmine list twice yields two different on-screen item orders, and each round's scored outcomes remain correct.
- **SC-005**: A multi-character team name can be entered in one go in the setup wizard, with no character lost.
- **SC-006**: In the list editor, no row shows the literal placeholder word as its icon and no row has an empty column wider than one value field at common window widths.
- **SC-007**: In a Tripmine round, 100% of views show the tripmine's value in the hint, and it equals the shown value when the tripmine is eventually picked.

## Assumptions

- Player links are trusted identification for this single-room game; there is no separate login. The server enforces that only the active team's role may act, and it accepts player input without a password, consistent with the existing unprotected-password model.
- The gamemaster keeps the ability to act on behalf of the active team; this is unchanged.
- "Start round" and the existing "next round" advancement are the same session transition; the new affordance only makes starting the first round obvious.
- The Tripmine shuffle applies to on-screen display order only; the internal ranked positions and the reveal of a picked position are unchanged.
- The list-editor icon control chooses from the product's existing icon set or an uploaded image, as already supported.
- Showing the tripmine's value in the hint is intentional: the grid's values stay hidden until picked, so the hint tells the room the value to hunt for without saying which cell holds it.
- The findings are visual and behavioural fixes; no new rounds, scoring rules, or session types are introduced.
