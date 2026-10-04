# Feature Specification: ListQuiz

**Feature Branch**: `001-listquiz-game`

**Created**: 2026-10-04

**Status**: Draft

**Input**: User description: list-based quiz game "ListQuiz" — gamemaster-hosted, two quiz types (Order, Tripmine), password-protected list folders, dark/light theme, visual design reused 1:1 from the read-only draft frontend in `the reference frontend`.

## Design Authority

The written specification is the source of truth. Where the draft frontend in `the reference frontend` contradicts this document, this document wins and the draft is corrected. Where this document is silent on presentation or navigation — screen structure, navigation, visual language, control layout — the draft frontend's behavior is adopted unchanged.

Corrections already identified in the draft frontend and applied here:

| Area | Draft frontend | This specification |
|------|----------------|-------------------|
| Order round scoring | +100 per correct placement | +10 per correct placement |
| Round timer | 15s/20s/30s turn counter | No timer, no buzzer |
| Round count | Capped at 5 | 1 to 10 |
| Gamemaster password | Optional | Required |
| Round start player | Always restarts at player 1 | Rotates one player per round |
| Theme | Dark only | Dark and light |
| List storage | All lists directly accessible | Password-protected folders plus a fixed default folder |
| Session links | 5 links (gamemaster, P1–P4, spectator) | Same 5 links, retained |
| Tripmine side | Gamemaster may switch mid-round, which redraws the 10 items | Locked once the round starts; the 10 items never change mid-round |

## Clarifications

### Session 2026-10-04

- Q: When an Order round picks its 10 candidate items from a list with more items on one side of the middle than the other, which 10 are used? → A: Random sample of 5 below and 5 above the anchor.
- Q: If the gamemaster changes the tripmine side after a Tripmine round has started, do the 10 items on the board stay the same? → A: The tripmine side is locked once play begins; no change mid-round.
- Q: What folder management actions exist beyond create and unlock? → A: Rename, plus delete only after the folder password is re-entered as confirmation.
- Q: Should the draft's sound effects be kept, and can they be muted? → A: Keep the sounds and add a mute toggle in the gamemaster view.
- Q: After a session ends, is a way to reopen it from within the product needed, or is the bookmarked link enough? → A: Link only; no session list in the interface.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Host a quiz session (Priority: P1)

A gamemaster creates a quiz session: picks 1–10 rounds, assigns a quiz type and a source list to each
round, names up to four players/teams, and sets a gamemaster password. On completion the gamemaster
receives one link per role and starts the game.

**Why this priority**: Everything else depends on a session existing. Without it there is no board,
no scoring, and no player view.

**Independent Test**: Create a session with 2 rounds and 2 teams, verify the five role links are
produced, verify the gamemaster link rejects a wrong password and accepts the right one, and verify
the scoreboard for both teams starts at 0 points and 3 hearts.

**Acceptance Scenarios**:

1. **Given** an authenticated gamemaster on the setup wizard, **When** they choose the number of
   rounds (1–10), **Then** exactly that many round configurations are shown.
2. **Given** a round configuration, **When** the gamemaster assigns a quiz type and a source list,
   **Then** that round is marked complete and cannot start without both.
3. **Given** completed round configurations, **When** the gamemaster sets a gamemaster password,
   **Then** the session is created and five role links are shown with copy and launch actions.
4. **Given** an existing session, **When** a visitor opens the gamemaster link without the password,
   **Then** access is refused with an incorrect-password message and no host controls are shown.
5. **Given** an existing session, **When** the gamemaster launches their link, **Then** the
   gamemaster control view opens with the host dashboard active.

---

### User Story 2 - Prepare lists in password-protected folders (Priority: P1)

A gamemaster first creates a password-protected folder, then creates or imports lists inside it.
Lists are never reachable without the folder password. A fixed, password-free default folder ships
with ready-to-play lists and cannot receive user lists.

**Why this priority**: Rounds need source lists. The folder model is also the privacy boundary the
user asked for, so it must exist before any list can be created.

**Independent Test**: Create a folder with a password, add a list inside it, verify the list is
invisible without the folder password, verify the default folder cannot be written to, and verify
the sensitive-data hint appears on both the list editor and the CSV import screen.

**Acceptance Scenarios**:

1. **Given** no folder exists, **When** the gamemaster opens list management, **Then** only the
   default folder is visible and a create-folder action is offered.
2. **Given** the gamemaster creates a folder with a password, **When** they enter that password,
   **Then** the folder's lists become visible and editable.
3. **Given** a wrong folder password, **When** the gamemaster enters it, **Then** the folder stays
   locked and its list names are not revealed.
4. **Given** the default folder, **When** the gamemaster attempts any create, import, delete, or
   rename action, **Then** the action is not offered and the folder is read-only.
5. **Given** the list editor or the CSV import screen, **When** it is displayed, **Then** a visible
   hint states that the data is not protected and that sensitive data must not be added.

---

### User Story 3 - Play an Order round (Priority: P1)

In an Order round every team starts with 3 hearts. A bar holds one anchor item in the middle with
its value visible. Ten further items — five below and five above the anchor — are shown as separate
name-only boxes in random order. The team on turn picks one item and places it into the bar. A
correct placement earns 10 points and joins the bar; a wrong placement costs 1 heart, earns nothing,
and leaves the item in the pool. Turn passes either way.

**Why this priority**: Order is one of the two core game modes and the most interactive one.

**Independent Test**: Play a full Order round with two teams: verify 10 candidate items appear
without values, verify a correct placement adds 10 points and a wrong one removes a heart without
points, verify the turn passes in both cases, and verify a team at 0 hearts is skipped permanently
within the round.

**Acceptance Scenarios**:

1. **Given** a round starting, **When** the board renders, **Then** the active team starts with 3
   hearts, the anchor item sits in the middle of the bar with its value visible, and 10 items
   (5 lower, 5 higher) are shown in random order without values.
2. **Given** an item on the active team's turn, **When** the gamemaster places it on the correct
   side of the bar, **Then** the item joins the bar aligned to its neighbours, the team gains 10
   points, and the item leaves the pool.
3. **Given** an item on the active team's turn, **When** the gamemaster places it on the wrong
   side, **Then** the item stays in the pool, the team loses 1 heart, no points are awarded, and the
   turn still passes.
4. **Given** the bar holds more than one item, **When** the gamemaster places an item between two
   existing items, **Then** only left, between-two, and right placements are offered and any other
   position is not.
5. **Given** a team has 0 hearts, **When** the turn advances, **Then** the team is skipped for the
   remainder of the round and cannot score.
6. **Given** the round has ended, **When** the gamemaster presses reveal, **Then** all item values
   become visible to the player view.
7. **Given** an item is still in the pool, **When** it is placed, **Then** its position in the bar is
   the team's choice — the bar is not auto-sorted by value.

---

### User Story 4 - Play a Tripmine round (Priority: P1)

Ten random items appear in a grid without values. The gamemaster decides whether the highest or the
lowest item is the tripmine, which also fixes the direction the players search from. On each turn a
team picks one item and its position (1–10) in the ordered set is revealed. Hitting the tripmine
ends the round.

**Why this priority**: Tripmine is the second core game mode.

**Independent Test**: Play a full Tripmine round with two teams: verify the searched position
display, verify +10 for the searched item, +5 for a wrong non-tripmine item, −10 plus +10 for each
other team when the tripmine is hit, and +20 when the correct item is picked with exactly two
unrevealed items left.

**Acceptance Scenarios**:

1. **Given** a Tripmine round starting, **When** the grid renders, **Then** 10 items appear without
   values and the searched position is displayed as "searching item 1" when the tripmine is the
   highest item, or "searching item 10" when the tripmine is the lowest item.
2. **Given** the searched position, **When** the active team picks the item at that position,
   **Then** the team gains 10 points and the next unrevealed position in the search direction
   becomes the new searched position.
3. **Given** an unrevealed item that is neither the searched position nor the tripmine, **When** the
   active team picks it, **Then** the team gains 5 points, the position is revealed, and the searched
   position does not change.
4. **Given** the tripmine item, **When** the active team picks it, **Then** the team loses 10 points,
   every other team gains 10 points, all positions are revealed, and the round ends.
5. **Given** exactly two unrevealed items remain (the tripmine and the last valid item), **When** the
   active team picks the valid one, **Then** the team gains 20 points instead of 10 and the round ends.
6. **Given** any pick, **When** it resolves, **Then** the turn passes to the next eligible team.

---

### User Story 5 - Watch and steer from separate views (Priority: P2)

Players in the room watch one shared player view on a second device and tell the gamemaster what to
select. The gamemaster performs every action and sees values, feedback, and answers; players see
only what the gamemaster reveals.

**Why this priority**: This is the hosting arrangement, but it needs the game modes (P1) to exist.

**Independent Test**: Open the gamemaster link and a player link side by side, place an item as
gamemaster, and verify the player view reflects the placement without exposing hidden values.

**Acceptance Scenarios**:

1. **Given** an active session, **When** a player opens a player link, **Then** the board renders in
   read-only form with no host controls and no hidden values.
2. **Given** a player view open on a second device, **When** the gamemaster changes the board,
   **Then** the player view reflects the change without a manual reload.
3. **Given** a player view, **When** the gamemaster acts on behalf of a team, **Then** the player
   view shows the resulting feedback message and score change.
4. **Given** a player view, **When** the gamemaster reveals values, **Then** all previously hidden
   values become visible.
5. **Given** a spectator link, **When** a spectator opens it, **Then** the board renders in read-only
   form with no team-specific turn emphasis.

---

### User Story 6 - Manage list content and ordering (Priority: P2)

Lists hold 11–100 items, each with a name and one or more value columns, and the gamemaster decides
which direction "low to high" means for that list. Lists can be typed manually, imported from CSV,
duplicated, exported, deleted, and reordered by dragging rows.

**Why this priority**: Needed before rounds can be configured, but folder access (US2) is the gating
step.

**Independent Test**: Create an 11-item list by hand, import an equivalent CSV, verify both produce
the same usable round, verify a 10-item list is rejected, and verify row drag changes item order.

**Acceptance Scenarios**:

1. **Given** the list editor, **When** the gamemaster adds rows, renames columns, adds or removes a
   value column, and assigns icons, **Then** the list saves with the gamemaster's row order preserved.
2. **Given** a CSV with an optional icon column, a name column, and one or more value columns,
   **When** it is imported, **Then** the list is created with matching columns and items.
3. **Given** a CSV with fewer than 11 or more than 100 data rows, **When** import is attempted,
   **Then** the import is rejected with a message stating the required row count.
4. **Given** a list, **When** the gamemaster sets its direction to high-to-low, **Then** the round
   treats the largest value as the low end of the bar.
5. **Given** a non-default list, **When** the gamemaster duplicates, exports, or deletes it, **Then**
   only that list is affected and other lists in the folder are untouched.

---

### User Story 7 - Carry scores across rounds and finish the game (Priority: P2)

Points accumulate across all rounds rather than resetting. Each round starts with the next team in
rotation, and the game ends when the configured rounds are done.

**Why this priority**: Determines final scoring but is a thin layer over completed rounds.

**Independent Test**: Play two rounds and verify round two starts from round one's totals, that the
starting team rotates, and that the final screen shows the cross-round ranking.

**Acceptance Scenarios**:

1. **Given** a team finished round one with 60 points, **When** round two starts, **Then** that team
   enters round two on 60 points.
2. **Given** round one started with team 1, **When** round two starts, **Then** team 2 has the first
   turn, skipping any team with 0 hearts.
3. **Given** the last configured round has ended, **When** the game concludes, **Then** the final
   ranking across all rounds is shown and all values are revealed.

---

### User Story 8 - Use the interface in either theme and without a mouse (Priority: P3)

The whole product works in a dark and a light theme, keeps every action reachable by keyboard alone,
and stays responsive while the game runs.

**Why this priority**: A cross-cutting quality bar from the project's governing principles, verified
across every other story rather than delivered on its own.

**Independent Test**: Complete session setup, a round of each quiz type, and list management using
only the keyboard in each theme, with no input freeze or full-page reload for a local change.

**Acceptance Scenarios**:

1. **Given** either theme is active, **When** any screen is opened, **Then** text, borders, and board
   surfaces keep readable contrast and every element matches the draft frontend's visual language.
2. **Given** the theme preference is changed, **When** the change is applied, **Then** it persists and
   no screen loses its current state.
3. **Given** a user is on any screen, **When** they navigate and act using only the keyboard,
   **Then** every control is reachable, operable, and visibly focused.
4. **Given** a local change such as placing an item or toggling a control, **When** it is made,
   **Then** the update appears without blocking input or reloading the page.

---

### Edge Cases

- A list with duplicate values: two items with equal numeric values cannot be ordered against each
  other; placement between them is accepted as correct if the side is correct.
- A CSV where a value cell is empty or non-numeric: the value falls back to the row position so the
  list remains playable, and the gamemaster is told which rows were defaulted.
- A CSV containing quoted fields with commas: the quote marks are stripped and the field is kept
  whole.
- A folder whose password is forgotten: the folder and its lists are permanently unreachable; there
  is no recovery.
- A folder is deleted: every list inside it is gone, a session already configured with one of those
  lists keeps running with the items it drew, and later rounds using a deleted list cannot start and
  must report that the list is gone.
- A gamemaster password is forgotten: the session cannot be controlled; the player and spectator
  views remain reachable.
- The gamemaster opens several views at once: all views converge on the same board state and the same
  turn.
- A team reaches 0 hearts mid-round: it is skipped immediately and for the rest of that round, and
  its score freezes.
- All remaining teams reach 0 hearts in an Order round: the round ends immediately with the pool
  unplaced and the reveal available.
- An Order round whose list has more items on one side of the anchor than the other: the shorter side
  is topped up from the longer side, and the anchor itself is never a candidate.
- A list of exactly 11 items used in an Order round: all eleven items are used, one anchor plus ten
  candidates, and the two sampled groups cannot overlap.
- The selected list contains fewer than 10 distinct values after import: the round cannot start and
  the gamemaster is told to fix the list.
- A Tripmine round where the searched position was already revealed by a previous wrong pick: the
  search skips it and targets the next unrevealed position.
- A Tripmine round with only the tripmine and one valid item unrevealed, where the active team picks
  the tripmine: normal tripmine scoring applies and the round ends.
- The gamemaster closes the tab mid-round: reopening the gamemaster link restores the exact board,
  scores, hearts, and turn.
- A player opens a link for a session that does not exist: a clear not-found message, not a blank board.
- The gamemaster reveals values mid-round: values stay visible for the remainder of the round.

## Requirements *(mandatory)*

### Session Setup

- **FR-001**: The gamemaster MUST be able to choose a round count between 1 and 10 inclusive.
- **FR-002**: Each round configuration MUST contain a quiz type, a source list, and — when the list
  exposes more than one value column — a chosen value column.
- **FR-003**: Each round configuration of type Tripmine MUST contain a tripmine side, either highest
  or lowest.
- **FR-004**: The gamemaster MUST be able to add between 1 and 4 teams and give each a name; a team
  that is given no name MUST receive a default name.
- **FR-005**: Each team MUST be assigned a distinct colour, drawn in order from the colour set in the
  draft frontend, and MUST be re-assigned automatically if a team is removed.
- **FR-006**: Every team MUST start the game with 3 hearts and 0 points.
- **FR-007**: The gamemaster MUST be required to set a non-empty gamemaster password before the
  session is created.
- **FR-008**: Creating a session MUST produce a session identifier that is unique and usable directly
  in a URL.
- **FR-009**: The session MUST expose the draft's five role links regardless of how many teams are
  configured: one gamemaster link, all four player links P1 to P4, and one public spectator link. A
  player link whose slot is beyond the configured team count MUST still resolve, rendering the
  read-only board with no team badge and no turn emphasis.
- **FR-010**: Each role link MUST offer a copy action and a launch action.
- **FR-011**: The gamemaster link MUST reject any request that does not supply the session password
  and MUST NOT expose host controls before the password is accepted.
- **FR-012**: The session base link without a role MUST resolve to a player-facing view of the same
  session.
- **FR-013**: Requesting an unknown session identifier MUST show a not-found message.

### Rounds and Turns

- **FR-014**: A round MUST NOT start until every configured round has a quiz type and a source list.
- **FR-015**: Round one MUST begin with team 1 holding the first turn; each subsequent round MUST
  begin with the next team in rotation, skipping any team at 0 hearts.
- **FR-016**: Hearts MUST reset to 3 at the start of every round.
- **FR-017**: Points MUST accumulate across rounds and MUST NOT reset between rounds.
- **FR-018**: After every scored action the turn MUST pass to the next team that still has hearts.
- **FR-019**: A team at 0 hearts MUST be skipped for the remainder of the round and MUST NOT be able
  to gain points in it.
- **FR-020**: The game MUST end after the last configured round completes, and the final ranking
  across all rounds MUST be shown.
- **FR-021**: No timer, countdown, or buzzer MUST be presented in any view.
- **FR-022**: The active team MUST be visibly highlighted in every view.
- **FR-023**: The gamemaster MUST be able to end a round and advance to the next one manually, and
  MUST be able to adjust a team's score and hearts directly.
- **FR-024**: For every quiz type the gamemaster MUST have a control that reveals all item values to
  the player and spectator views once the round is over.
- **FR-024a**: The product MUST play the draft frontend's audio cues on correct and incorrect Order
  placements, on a Tripmine hit, and on a Tripmine find.
- **FR-024b**: The gamemaster view MUST offer a mute control that stops all audio cues, and the muted
  state MUST apply to every view of that session.

### Order Round

- **FR-025**: An Order round MUST display a bar with five slots left of a centre anchor slot and five
  slots right of it; the left side represents lower values and the right side higher values.
- **FR-026**: The anchor item MUST be taken from the middle of the list in the list's chosen
  direction, MUST be shown in a distinct colour, and MUST show its value.
- **FR-027**: Ten candidate items MUST be drawn from the list — five sampled at random from the items
  below the anchor and five sampled at random from the items above it — MUST be shown in random order
  as individual boxes, and MUST NOT show their values until revealed.
- **FR-028**: A placement MUST be offered only as left of the bar, between two existing items, or
  right of the bar.
- **FR-029**: A correct placement MUST add the item to the bar aligned with its neighbours without
  re-sorting, MUST award 10 points to the active team, and MUST remove the item from the pool.
- **FR-030**: A wrong placement MUST leave the item in the pool, MUST award no points, MUST remove 1
  heart from the active team, and MUST still pass the turn.
- **FR-031**: Items MUST remain in the pool until every item has been placed correctly.
- **FR-032**: The round MUST end when all items are placed, or when no team retains hearts.
- **FR-033**: The gamemaster MUST be able to return a placed item to the pool.
- **FR-034**: Items with equal values MUST be treated as correctly placed when placed on the correct
  side of the bar.

### Tripmine Round

- **FR-035**: A Tripmine round MUST select 10 random items from the round's list and display them in a
  grid without values.
- **FR-036**: The gamemaster MUST choose whether the tripmine is the lowest or the highest item of the
  ten before the round starts, and MUST NOT be able to change that choice once the round has begun.
- **FR-036a**: A round that has already started MUST keep the same ten items and the same revealed
  positions for its whole duration; no action may substitute a different set of items.
- **FR-037**: When the tripmine is the highest item, the searched position MUST be position 1 and the
  search MUST advance upward; when the tripmine is the lowest item, the searched position MUST be
  position 10 and the search MUST advance downward.
- **FR-038**: The player view MUST display the currently searched position.
- **FR-039**: Any pick MUST reveal that item's position (1–10) in the ordered set of ten, whether or
  not the pick was correct.
- **FR-040**: Picking the item at the searched position MUST award 10 points and MUST advance the
  searched position to the next unrevealed position in the search direction.
- **FR-041**: Picking a wrong item that is not the tripmine MUST award 5 points, MUST reveal only that
  position, and MUST NOT advance the searched position.
- **FR-042**: Picking the tripmine MUST subtract 10 points from the active team, MUST award 10 points
  to every other team, MUST reveal all remaining positions, and MUST end the round.
- **FR-043**: When exactly two unrevealed items remain and the active team picks the valid one, the
  team MUST be awarded 20 points instead of 10, all positions MUST be revealed, and the round MUST
  end.
- **FR-044**: A pick MUST be refused for any team other than the active team, except that the
  gamemaster MAY act on behalf of the active team.

### Lists and Folders

- **FR-045**: List management MUST present lists only inside folders; no list MUST be reachable
  without first entering a folder.
- **FR-046**: The gamemaster MUST be able to create a folder protected by a non-empty password.
- **FR-046b**: The gamemaster MUST be able to rename a folder.
- **FR-046c**: Deleting a folder MUST require the folder password to be re-entered as confirmation, and
  the confirmation MUST state how many lists the folder holds.
- **FR-046d**: Deleting a folder MUST delete every list inside it and MUST NOT affect the default
  folder or any other folder.
- **FR-047**: A protected folder MUST stay locked, and MUST NOT reveal its list names or contents,
  until the correct password is entered.
- **FR-048**: The gamemaster MUST be able to create, import, edit, duplicate, export, reorder, and
  delete lists inside an unlocked folder.
- **FR-049**: The gamemaster MUST NOT be offered any option to create a list without a folder or to
  add a list to the default folder.
- **FR-050**: A built-in default folder MUST exist, MUST require no password, MUST be read-only, and
  MUST contain the ready-to-play lists supplied with the product.
- **FR-051**: The list editor and the CSV import screen MUST each display a visible hint that the
  stored data is not protected and that sensitive data must not be added.
- **FR-052**: A list MUST contain between 11 and 100 items inclusive, each with a name and at least one
  value.
- **FR-053**: A list MUST store a direction, low-to-high or high-to-low, that defines which end of the
  list is the low end for rounds.
- **FR-054**: A list MAY contain multiple value columns; rounds MUST use exactly one chosen column.
- **FR-055**: A list MUST preserve the gamemaster's item order, and MUST allow items to be rearranged
  by drag and drop.
- **FR-056**: Each item MUST be able to carry an icon chosen from the product's icon set or an uploaded
  image.
- **FR-057**: CSV import MUST accept an optional leading icon column, a name column, and one or more
  value columns, MUST strip surrounding quotes, and MUST report rows whose values could not be read.
- **FR-058**: Export MUST produce a CSV that re-imports to an equivalent list, including the direction.
- **FR-059**: Import and creation MUST be rejected when the resulting item count would fall outside
  11–100.

### Views, Theme, and Access

- **FR-060**: All five role views MUST render the same board state for the same session.
- **FR-061**: A change made in one view MUST appear in the other open views without a manual reload.
- **FR-062**: Player and spectator views MUST NOT expose host controls or hidden values.
- **FR-063**: The product MUST offer a dark and a light theme, and the selected theme MUST persist
  across reloads and views.
- **FR-064**: The gamemaster MUST be able to switch the active team for turn simulation from the
  gamemaster view.
- **FR-065**: Every interactive control MUST be reachable and operable by keyboard alone and MUST
  show a visible focus indicator.
- **FR-066**: No local action — placing an item, picking an item, revealing values, changing a
  setting — MUST block input or trigger a full-page reload.
- **FR-067**: Sessions, teams, scores, hearts, rounds, and board state MUST survive a page reload and a
  backend restart.
- **FR-068**: Session passwords MUST be stored as entered, without hashing, and the product MUST state
  that this is not a protection mechanism.

### Key Entities

- **Session**: one hosted quiz game. Holds a unique identifier, the gamemaster password, the team
  list, the ordered round configurations, the current round, and the accumulated scores.
- **Team**: a player or team in a session. Holds a name, a unique colour, a score that carries across
  rounds, hearts for the current round, and a turn indicator.
- **Round**: one play sequence. Holds its ordinal, quiz type, tripmine side, source list, chosen value
  column, and its own play state.
- **List**: an ordered set of 11–100 items with a title, a direction, and one or more value columns.
  Belongs to exactly one folder.
- **List Item**: one entry in a list. Holds a name, an icon, and a display and comparable value per
  column.
- **Folder**: a password-protected container of lists. The built-in default folder has no password and
  is read-only.
- **Board State**: the live state of the current round — for Order the anchor, pool, bar sides, and
  pending placement; for Tripmine the ten items, the searched position, and the reveal history.
- **Score Event**: an auditable point or heart change with its cause and the team it affected.

## Success Criteria

### Measurable Outcomes

- **SC-001**: A first-time gamemaster can go from opening the product to a running session with links
  in hand in under 5 minutes without external instructions.
- **SC-002**: 100% of host actions performed in the gamemaster view appear in an open player view
  within one second without a reload.
- **SC-003**: A player view never displays a hidden item value before the gamemaster reveals it, in
  100% of checked states across both quiz types.
- **SC-004**: Every scoring outcome in both quiz types matches the specified values in 100% of played
  rounds, verified against a per-round scorecard.
- **SC-005**: Points are identical between round two's opening scoreboard and round one's closing
  scoreboard for every team in every session tested.
- **SC-006**: Every screen is fully operable by keyboard alone, with no control reachable only by
  pointer, in both themes.
- **SC-007**: No local interaction freezes input or forces a full-page reload; 0 occurrences across a
  full 10-round session.
- **SC-008**: Lists with fewer than 11 or more than 100 items are rejected at creation and at import,
  with a message naming the required range.
- **SC-009**: A protected folder's list names are never rendered while the folder is locked.
- **SC-010**: 90% of gamemasters in a usability test complete list import and round configuration
  without external help on the first attempt.
- **SC-011**: A session reopened after a backend restart resumes with identical scores, hearts, turn,
  and board state.
- **SC-012**: Both themes pass a contrast review with no body text below the readable-contrast
  threshold on any screen.

## Assumptions

- The draft frontend in `the reference frontend` remains a read-only design reference. Its screens, navigation,
  control layout, copy, colour set, fonts, icon set, and audio cues are adopted as-is unless this
  document overrides them; its client-side state handling is replaced by real session handling.
- Where the draft frontend contradicts this document, this document wins. The known contradictions are
  listed in the Design Authority table and are treated as defects to correct during planning.
- Every screen the draft frontend implements — the landing page, list management, the three-step setup
  wizard, and the session view with its order bar, item grid, team podiums, stage lighting, and icon
  picker — is in scope and keeps its structure.
- Scores and hearts are small integers; the gamemaster's manual adjustment is the only way to change
  them outside scoring rules.
- Item values are comparable numerically; a value that cannot be read as a number falls back to the
  item's position in the list, and the gamemaster is told.
- The default folder ships with the three lists present in the draft frontend: World History &
  Inventions, Iconic Statues & Architectural Marvels, and Blockbuster Cinema Box Office.
- The minimum list size is 11 items rather than the 10 first described, because an Order round needs
  one anchor plus ten candidates; this was confirmed during specification.
- Team count is 1 to 4 rather than the draft's 2 to 4; a single-team session is valid.
- The five role links from the draft frontend are retained in preference to the two links first
  described; this was confirmed during specification. The bare session link still resolves to a
  player-facing view.
- The turn timer and its countdown controls in the draft frontend are removed; this was confirmed
  during specification.
- Light theme is derived from the draft frontend's dark theme by inverting surfaces while keeping the
  same accent and team colours; no separate light design is authored.
- Theme follows the operating system preference on first visit and is overridable by the gamemaster.
- Lists, folders, sessions, and their contents are stored in the project tree, per the project
  constitution; the constitution's storage and build-locality rules govern implementation.
- Passwords protect folders and the gamemaster view from casual access only. They are stored as
  entered, the product says so, and no encryption or hashing is claimed.
- The audience is a group in one physical room: the gamemaster operates one device and players share
  one or more second devices in the same room.
- This specification defines a single-feature product; session history, exporting a whole game, and
  any cross-session leaderboard are out of scope.
- No session index, recent-session list, or session search is part of the interface. A stored session
  is reopened only by opening its saved link, so the gamemaster is responsible for keeping it.
- There is no user account, login, or per-user identity. The gamemaster password identifies the
  session for control purposes only; the product does not attribute actions to a named person.