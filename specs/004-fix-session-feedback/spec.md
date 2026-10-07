# Feature Specification: Session Feedback And Order Reveal

**Feature Branch**: `004-fix-session-feedback`

**Created**: 2026-10-07

**Status**: Draft

**Input**: User description: "further findings/improvements/bugs
1. No sound its supposed to be there you even added a mute/unmute button but there is currently no sound in the game. The same sound as used from the the reference frontend
2. Current behavior in order if item is higher than initial border color is red if item is lower border color is blue. Target: Regardless of the game the Border of a selected (tripmine) or sorted item (order) should become the team color that correctly choose that item.
3. Reveal currently only displays the hidden value this is fine in tripmine but in order it should not just reveal but also place all items in the correct position in the bar leaving the border color neutral (black/gray/just not a team color). Make sure that this does not award any points by accident."

## Clarifications

### Session 2026-10-07

- Q: When a team acts, should that team's own device play the cue immediately on the interaction while everyone else hears it when the server reflects the event, or should every cue play only from the reflected server event so nobody hears it twice? → A: Server-driven only: every cue (slide included) plays once from the reflected event; no local interaction cue.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Game sounds actually play (Priority: P1)

During a running session the game is silent even though a mute/unmute control exists. Players, the gamemaster, and spectators expect the same set of cue sounds the reference draft plays: a slide/whoosh when an item is picked up or moved, a chime when a placement is correct or a round completes, a buzzer/failure tone when a placement is wrong, an explosion when a Tripmine is triggered, and a removal tone when an item goes back to the pool. Muting silences every cue; unmuting brings them back.

**Why this priority**: The mute/unmute control currently implies audio that never happens. A show game without its cue sounds is a visible, day-one defect, and every other fix in this feature is judged against a board that already feels wrong.

**Independent Test**: Run a session, trigger each event (select, correct placement, wrong placement, Tripmine hit, round end, return to pool) and confirm each plays its cue while unmuted and none plays while muted.

**Acceptance Scenarios**:

1. **Given** an unmuted running session, **When** a team selects an item and moves it, **Then** the slide cue plays for every connected role at the moment the move is reflected.
2. **Given** an unmuted running session, **When** a placement is judged correct, **Then** the success chime plays; **When** it is judged wrong, **Then** the failure cue plays.
3. **Given** an unmuted Tripmine round, **When** a mine is triggered, **Then** the explosion cue plays; **When** a safe item is revealed, **Then** the safe cue plays.
4. **Given** the session is muted, **When** any of the above events occur, **Then** no cue plays; **When** the session is unmuted again, **Then** later events play cues again.
5. **Given** a user who has just interacted with the page, **When** the first event occurs, **Then** the cue is audible without requiring a second, separate audio-enabling gesture.

---

### User Story 2 - Item border shows the choosing team's color (Priority: P2)

Today an Order item's border color tells the player whether the item is lower (one color) or higher (another) than the anchor, and a Tripmine item's border tells whether it was a mine or safe. Players instead want to look at a placed (Order) or picked (Tripmine) item and read which team was responsible, from the item's border color. Each team already has a color on the scoreboard; that same color must frame every item the team has placed or picked.

**Why this priority**: The border is the most prominent per-item signal during play. Re-purposing it to name the acting team makes it possible to see at a glance who contributed what, which is the core social read of the game.

**Independent Test**: Play an Order round and a Tripmine round with more than one team; confirm each item's border matches the color of the team that placed or picked it, and that the higher/lower and mine/safe meaning is gone from the border.

**Acceptance Scenarios**:

1. **Given** an Order round, **When** a team confirms a correct placement, **Then** that item's border takes the placing team's color.
2. **Given** an Order round, **When** an item is selected but not yet placed, **Then** its preview border takes the selecting team's color.
3. **Given** a Tripmine round, **When** a team picks an item and it is revealed, **Then** that item's border takes the picking team's color.
4. **Given** items placed or picked by different teams, **When** viewed by any role, **Then** each item's border matches its own team, and the same team color appears in both the Order and Tripmine boards.

---

### User Story 3 - Order reveal arranges the whole bar (Priority: P3)

When values are revealed during an Order round, the hidden numbers appear but the items stay wherever they were placed. A revealed Order round should instead finish the story: every item sorts into its correct position in the bar (lower values on the correct side of the anchor, in true order), each item shows its value, and the borders go neutral — no team color, no higher/lower color. Revealing must be a presentation of the answer only and must never change a team's score or hearts, so it cannot award points by accident. Tripmine reveal is unchanged.

**Why this priority**: The reveal is the payoff moment of a round. Showing correct positions completes it; leaving mistakes scattered makes the reveal look unfinished. It is lower priority than sound and team colors because it only matters once the other signals exist.

**Independent Test**: Play an Order round, place some items correctly and some not, trigger reveal, and confirm every item sits in its correct ordered position with a neutral border and each team's score/hearts are unchanged.

**Acceptance Scenarios**:

1. **Given** an Order round in progress, **When** the gamemaster reveals values, **Then** all items — placed and still in the pool — appear in the bar in their correct order relative to the anchor with values shown.
2. **Given** the reveal has happened, **When** any item is inspected, **Then** its border is neutral (not a team color and not a higher/lower color).
3. **Given** the reveal has happened, **When** scores and hearts are compared to before the reveal, **Then** none have changed.
4. **Given** a Tripmine round, **When** values are revealed, **Then** behavior is unchanged: hidden values appear and item positions do not move.

---

### Edge Cases

- The browser blocks audio until the page has been interacted with: the first cue after interaction must play; if an event occurs before any interaction, that cue may be lost without error.
- Muting mid-round, then unmuting: cues resume for subsequent events; no backlog plays on unmute.
- An event is reflected for several roles at once: each role, including the team that acted, hears the cue exactly once from the reflected event; a local interaction never adds a second cue.
- Order reveal while some items are still in the pool: pool items are arranged into the bar too, and no placement points are awarded for them.
- Order reveal after a wrong placement already cost a heart: the reveal does not refund or add anything.
- Reveal toggled more than once: arranging and value display are idempotent; no repeated point award.
- An item returned to the pool by the gamemaster: its team-colored border is gone with it.
- Fewer teams than available colors: each team still shows a distinct color; unused colors never appear on items.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST play an audio cue for each key game event — item selected, item moved, correct placement, wrong placement, Tripmine triggered, safe item revealed, round completed, and item returned to the pool — using the same cue set as the reference draft.
- **FR-002**: System MUST drive every cue from the reflected session event as the single source, so each role (gamemaster, players, spectators, including the acting team) hears one cue per event; no cue plays from a local interaction, so the acting team never hears a doubled cue.
- **FR-003**: System MUST silence every cue while the session is muted and MUST play cues again after unmuting.
- **FR-004**: System MUST NOT require a separate, second gesture to enable audio once the user has interacted with the page.
- **FR-005**: In an Order round, a placed (sorted) item's border MUST take the color of the team that placed it.
- **FR-006**: In an Order round, the currently selected (pending) item's preview border MUST take the color of the team that selected it.
- **FR-007**: In a Tripmine round, a revealed item's border MUST take the color of the team that picked it.
- **FR-008**: The team color used for a border MUST be the team's configured color and MUST be the same color shown for that team elsewhere in the session.
- **FR-009**: The item border MUST NOT encode higher/lower (Order) or mine/safe (Tripmine); that information MUST remain available through the existing non-border cues (rank/label, icons, point text, feedback messages).
- **FR-010**: When values are revealed in an Order round, system MUST arrange every item — placed items, pool items, and the anchor — into its correct position in the bar relative to the anchor, in true value order.
- **FR-011**: When values are revealed in an Order round, every arranged item's border MUST be neutral (neither a team color nor a higher/lower color).
- **FR-012**: Revealing an Order round MUST NOT change any team's score, hearts, or the round's progress; it MUST award no placement points.
- **FR-013**: Revealing values in a Tripmine round MUST remain unchanged: hidden values appear and item positions do not move.
- **FR-014**: When values are revealed in an Order round, each arranged item MUST display its value.

### Key Entities *(include if feature involves data)*

- **Team**: A competing team with an identity, a name, and a color (plus its richer color values) shown on the scoreboard; the color is reused to frame items the team acted on.
- **Order bar item**: An item placed on the bar, with its value, its side of the anchor, its position, and the team that placed it (new association needed so the border can be colored).
- **Tripmine item**: An item in the Tripmine grid, with its hidden value, revealed state, and the team that picked it (already tracked) so its border can be colored.
- **Game event**: A meaningful in-game occurrence (select, move, correct, wrong, mine, safe, round complete, return to pool) that maps to exactly one audio cue.
- **Cue set**: The fixed collection of sound effects used by the reference draft, reused here without adding audio files.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: With sound unmuted, 100% of the defined game events produce an audible cue; with sound muted, 0% do.
- **SC-002**: A listener can identify which team placed or picked a given item from its border color alone, with no misreads across a full round.
- **SC-003**: After an Order reveal, 100% of items sit in their correct ordered position in the bar and every border is neutral.
- **SC-004**: Revealing an Order round changes the sum of all team scores and hearts by exactly zero.
- **SC-005**: A Tripmine reveal still shows the previously hidden values with item positions unchanged.

## Assumptions

- The cue set and event-to-cue mapping follow the reference draft (`the reference frontend`); no new audio files are added — the existing synthesized effects are reused.
- The reference draft is read-only, per the constitution; its audio behavior is reimplemented inside this repository.
- Team colors are those already assigned to teams (a distinct color per team), and team identity/color is available to every session view.
- The Order reveal is triggered by the existing gamemaster reveal action and at session end; it is presentational only and does not mutate the scored board.
- Only the enumerated key events get cues; incidental state refreshes do not.
- Cues are emitted from the reflected session event only; there is no separate local-interaction cue path.
- Roles that connect after an event has occurred do not receive a retroactive cue for it.
