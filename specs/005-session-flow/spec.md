# Feature Specification: Session Flow And Quick Game

**Feature Branch**: `005-session-flow`

**Created**: 2026-10-07

**Status**: Draft

**Input**: User description: "new findings/bugs/improvements
1. Hidden Values are no longer displayed to the GM for Tripmine fix it GM always needs to see.
2. Add a new pop up dialog for the GameEnd instead of displaying it in the current dialog only show points not hearths (hearts reset for each round so it make no sense to display the hearts at the end screen)
3. When Game is finished and GameEnd dialog is displayed the game session can be deleted from db no need to access again. (When Game is finished not round)
4. Add a quickgame mode option on title screen, Where you only select the game type you want to play and than instantly start the game (1 Team) in the p1 view no access to gamemaster view.
5. Remove the light/dark mode toggle only the current dark mode
6. Game Master had the option in the the reference frontend to set a custom time counter to the player at turn which forces a time end if not answered until the time run up (to manually prevent players overthinking)."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - The gamemaster always sees every Tripmine value (Priority: P1)

The gamemaster needs the full answer key during a Tripmine round — every item's hidden value, its true rank, and which item is the mine — whether or not it has been picked, so they can run the round. Today a hidden item's value is no longer shown to the gamemaster on the board. It must always be visible to the gamemaster and never to a player who has not revealed it.

**Why this priority**: This is a regression that removes information the host needs to run the game. It is the most urgent of the findings.

**Independent Test**: Start a Tripmine round and, as the gamemaster, confirm every item shows its value (and rank) before and after being picked; as a player, confirm hidden items still show no value.

**Acceptance Scenarios**:

1. **Given** a Tripmine round with unpicked items, **When** the gamemaster views the board, **Then** every item shows its value and rank, including the mine.
2. **Given** the same round, **When** a player views the board, **Then** unpicked items still hide their value and rank.
3. **Given** an item the gamemaster revealed, **When** any role views it, **Then** the value and rank are shown as before.

---

### User Story 2 - Turn timer (Priority: P2)

The gamemaster can put a countdown on the team currently on turn so a stalling player cannot think forever. The gamemaster chooses the duration and starts it; everyone sees the countdown; when it reaches zero the team on turn loses a heart and the turn passes on, exactly as if they had failed to act. The gamemaster can stop the timer early, and starting a new turn clears it.

**Why this priority**: A requested host tool that directly shapes live play; it prevents one team from holding the room hostage.

**Independent Test**: As the gamemaster start a short timer on a turn; watch it count down in every view; let it expire and confirm the on-turn team loses a heart and the turn passes; start another and stop it early to confirm it clears.

**Acceptance Scenarios**:

1. **Given** a running round, **When** the gamemaster starts a timer of a chosen length, **Then** every view shows a live countdown for the current team.
2. **Given** a running timer, **When** it reaches zero, **Then** the on-turn team loses one heart, the turn passes to the next team with hearts, and the timer clears.
3. **Given** a running timer, **When** the gamemaster stops it, **Then** it clears with no heart lost.
4. **Given** a running timer, **When** the turn ends another way (a placement, a pick, a pass, or a round change), **Then** the timer clears.

---

### User Story 3 - A game-end dialog that shows points only (Priority: P3)

When the whole game finishes, a dedicated pop-up dialog presents the final standings. It ranks teams by points and shows only points — hearts are left out because they reset each round and mean nothing at the end. The dialog is the clear end-of-game moment for every role.

**Why this priority**: The current inline end display mixes in hearts, which is meaningless at game end; a dedicated dialog is the correct payoff screen.

**Independent Test**: Finish a game and confirm a pop-up dialog appears showing every team ranked by points with no heart counts.

**Acceptance Scenarios**:

1. **Given** the last round has ended and the game is finished, **When** the game-end dialog appears, **Then** it lists every team ordered by points, highest first.
2. **Given** the dialog, **When** any team is inspected, **Then** only its points are shown; no hearts appear.
3. **Given** a game that is merely between rounds, **When** a round ends, **Then** the game-end dialog does not appear.

---

### User Story 4 - A finished game's session is cleaned up (Priority: P4)

Once the game is finished and the end dialog has been shown, the session has served its purpose and is deleted from storage, so finished sessions do not accumulate. Only a finished **game** is deleted this way — a session that is still between rounds or mid-round is never deleted.

**Why this priority**: Keeps storage tidy without any user action; lower risk than the live-play features.

**Independent Test**: Finish a game, dismiss the end dialog, and confirm the session no longer loads (its links report not-found). Confirm a session that has only finished a round still loads.

**Acceptance Scenarios**:

1. **Given** a game that is finished and its end dialog has been shown, **When** the dialog is dismissed, **Then** the session is deleted and any later access reports it as not found.
2. **Given** a session that has finished only a round (game not over), **When** the round ends, **Then** the session is not deleted.
3. **Given** the end dialog is still open, **When** the player is reading it, **Then** the standings remain readable until it is dismissed.

---

### User Story 5 - Quick game from the title screen (Priority: P5)

The title screen offers a quick game: the user picks only the game type they want, and a one-team game starts immediately in the player view. No setup wizard, no gamemaster view, no link management — just play. This suits a solo or casual run.

**Why this priority**: A new, self-contained entry path; valuable but not required for the core fixes.

**Independent Test**: From the title screen, choose a game type and confirm a one-team game starts straight in the player view with no gamemaster access.

**Acceptance Scenarios**:

1. **Given** the title screen, **When** the user opens quick game, **Then** they choose between the game types (Order, Tripmine).
2. **Given** a chosen game type, **When** the user starts, **Then** a one-team game begins immediately in the player view.
3. **Given** a quick game, **When** the user looks for gamemaster access, **Then** none is offered.
4. **Given** a quick game, **When** the game finishes, **Then** the game-end dialog and cleanup behave as in a normal game.

---

### User Story 6 - Dark mode only (Priority: P6)

The light/dark toggle is removed. The app keeps the current dark theme everywhere and no longer offers a theme choice.

**Why this priority**: A small, purely cosmetic cleanup with no dependencies.

**Independent Test**: Load the app and confirm there is no theme toggle on any screen and the dark theme is always used.

**Acceptance Scenarios**:

1. **Given** any screen, **When** the user looks for a theme toggle, **Then** none is shown.
2. **Given** a fresh load, **When** the app renders, **Then** it uses the dark theme.

---

### Edge Cases

- The gamemaster's own reveal action still reveals values to players, but never hides anything from the gamemaster.
- A timer expiring on a team with one heart leaves that team at zero hearts and the round ends if no team has hearts (existing round-end rules apply).
- A timer set, then the turn ended by any other action, clears the timer so it cannot fire on the wrong team.
- Two timers cannot run at once: starting a new one replaces the current countdown.
- Deleting the session must not delete the quiz lists it used, only the session.
- If the end dialog is shown while another role is mid-read, each role deletes/reads independently without erroring.
- Quick game with a list too small for the chosen game type: the game must not start silently broken (a clear message instead).
- Quick game always uses one team; the player view is the only access.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The gamemaster MUST see every Tripmine item's value, true rank, and mine status at all times, whether or not the item has been picked.
- **FR-002**: A player or spectator MUST NOT see an unpicked Tripmine item's value or rank; existing redaction for other roles is unchanged.
- **FR-003**: The gamemaster MUST be able to start a countdown timer for the team currently on turn, choosing its duration.
- **FR-004**: A running timer MUST count down live and be visible to every role.
- **FR-005**: When a timer reaches zero, the on-turn team MUST lose exactly one heart and the turn MUST pass to the next team with hearts; the timer MUST clear.
- **FR-006**: The gamemaster MUST be able to stop a running timer; stopping loses no heart.
- **FR-007**: Any other end-of-turn event (placement, pick, pass, or round change) MUST clear a running timer.
- **FR-008**: A timer MUST NOT take a team below zero hearts; existing round-end rules still apply when hearts run out.
- **FR-009**: When the whole game finishes, a dedicated pop-up dialog MUST present the final standings.
- **FR-010**: The game-end dialog MUST rank teams by points, highest first, and MUST show points only — never hearts.
- **FR-011**: The game-end dialog MUST appear for every role when the game is finished, and MUST NOT appear when only a round has ended.
- **FR-012**: When the game is finished and the end dialog has been shown, dismissing it MUST delete the session from storage; only a finished game may be deleted, never a session that is between rounds or mid-round.
- **FR-013**: After a finished session is deleted, any further access to it MUST report it as not found.
- **FR-014**: The title screen MUST offer a quick game that asks only for the game type and then starts a one-team game immediately in the player view.
- **FR-015**: A quick game MUST NOT expose a gamemaster view or link.
- **FR-016**: A quick game MUST reuse the standard game flow, including the game-end dialog and finished-session cleanup.
- **FR-017**: The application MUST NOT offer a light/dark theme toggle and MUST render the dark theme everywhere.

### Key Entities *(include if feature involves data)*

- **Tripmine item**: An item in the Tripmine grid with a hidden value, true rank, mine status, and revealed state; the gamemaster's view always carries the answer key.
- **Turn timer**: A countdown attached to the current turn, with a total duration and a remaining time, running or stopped; owned by the session and cleared when the turn ends.
- **Game-end dialog**: The end-of-game pop-up, listing teams by points.
- **Session**: A running or finished game; a finished one is deleted once its end dialog is dismissed.
- **Quick game**: A one-team session started from the title screen by choosing only a game type, reachable through the player view alone.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The gamemaster can read every Tripmine value and rank before any pick, with zero hidden values in their view.
- **SC-002**: A player sees zero unpicked Tripmine values.
- **SC-003**: A timer expiring removes exactly one heart from the on-turn team and advances the turn in 100% of trials.
- **SC-004**: The game-end dialog shows points and zero hearts for every team.
- **SC-005**: A finished session is unreachable after its end dialog is dismissed; a between-rounds session remains reachable.
- **SC-006**: A quick game reaches active play within a few seconds of choosing a game type, with no gamemaster access offered.
- **SC-007**: No screen presents a theme toggle.

## Assumptions

- The Tripmine answer key (value, rank, mine) is the same information the gamemaster already sees; only its display on the board was lost and is restored.
- The timer is gamemaster-started per turn (matching the reference draft), with a duration the gamemaster chooses; the reference draft's 15/20/30-second presets are a reasonable starting set and a custom duration is allowed.
- Timer expiry is treated like a failed turn: one heart lost, turn passed, timer cleared.
- "Game finished" means the whole session is over (no further rounds), distinct from a single round ending.
- The session is deleted when the end dialog is dismissed, so the final standings stay readable first; deleting the session never deletes the quiz lists it referenced.
- Quick game uses one team and the standard rules for the chosen game type, with sensible default list/column choices made for the player.
- Removing the theme toggle leaves the dark theme as the only theme; no light theme is maintained.
