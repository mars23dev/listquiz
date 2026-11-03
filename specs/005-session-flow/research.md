# Research: Session Flow And Quick Game

## D-501: Restore the gamemaster's Tripmine answer key on the board

**Decision**: The gamemaster board shows every Tripmine item's value (and rank) whether or not
it has been picked. The board takes an explicit "show values" flag from the role; the server
already sends the values to the gamemaster, so only the rendering gate changes. Player and
spectator redaction is untouched.

**Rationale**: The server's `redact` already computes `revealed = is_gamemaster || item.revealed`
and sends `value`/`true_rank` to the gamemaster for every item (app_state.rs). The regression is
that the frontend derives `isRevealed` from the picked flag and only renders the value inside the
picked branch, so the gamemaster's hidden items fall through to "Hidden". Passing the role's
"show values" intent into the board fixes the display without touching redaction (Principles II
and VII).

**Alternatives considered**: loosening redaction (it is already correct for the gamemaster);
special-casing the value in `Session.tsx` mapping (spreads the rule across two places).

## D-502: The timer stores an end time; the client ticks, the server judges

**Decision**: `game_session` gains nullable `timer_ends_at` (epoch ms) and `timer_total_seconds`.
Starting a timer records both; the view reports `timer: { totalSeconds, secondsLeft }` computed
from `ends_at` at read time. The client counts down locally and, at zero, posts a `time_out`
action; the server accepts it only when a timer is running, `now >= ends_at`, and the caller is
the gamemaster or the on-turn team, then applies the penalty and clears the timer. `start_timer`
and `stop_timer` are gamemaster actions.

**Rationale**: The server owns session state, but a per-session background task to fire the
timeout adds a scheduler, lifecycle, and restart-recovery surface the finding does not need. An
end time makes every client agree on the remaining seconds regardless of when it read; the
server still validates the timeout, so a client cannot fake one (Principle II). This mirrors the
reference draft, where the countdown is client-side and the timeout is applied once. Because the
room is always connected, a timeout is applied promptly after it lapses.

**Alternatives considered**: a tokio task per timer (a scheduler and restart recovery for a
feature the draft keeps client-side); storing a decrementing `secondsLeft` and trusting writes
(drift and race between clients).

## D-503: A timeout is a failed turn

**Decision**: On expiry the on-turn team loses exactly one heart and the turn passes to the next
team with hearts; the timer clears. This reuses the scoring module's heart change and turn
advance, and the existing round-end rule when hearts run out. Any other end-of-turn event
(placement, pick, pass, set turn, next round) clears the timer.

**Rationale**: The draft treats "ran out of time" exactly like a wrong answer: `-1 heart`, turn
advanced, feedback shown. Reusing the scoring rule keeps one definition of "a turn ended badly"
(Principle II) and guarantees the timer cannot invent a new scoring path. Clearing on every turn
change prevents a stale timer firing on the wrong team (FR-007).

**Alternatives considered**: zero points instead of a heart (the draft and the spec both use a
heart); leaving the timer running across turns (would penalise the wrong team).

## D-504: A dedicated points-only end dialog

**Decision**: When the session status is `finished`, a modal dialog (`role="dialog"`, focus
managed, dismissible by keyboard) shows the final standings ranked by points, with points only —
no hearts. It replaces the inline final-ranking section.

**Rationale**: Hearts reset each round and are meaningless at game end, so the current inline
list (which prints "pts / hearts") is misleading; a modal is the clear, single end-of-game moment
for every role (FR-009–FR-011). A real dialog with keyboard focus satisfies Principle VI.

**Alternatives considered**: keeping the inline section and hiding hearts (still not a clear end
moment); showing hearts as a tiebreaker (the finding says points only).

## D-505: Delete a finished session when the end dialog is dismissed

**Decision**: Add `sessions::delete_session(conn, id)` and `DELETE /api/sessions/{id}`. The
server deletes only when the status is `finished`; the row's `ON DELETE CASCADE` removes teams,
rounds, round state, and score events. The frontend calls it when the end dialog is dismissed,
then returns to the title screen. Deleting an already-missing session is treated as success.

**Rationale**: The finding wants finished sessions gone so they cannot be reopened, but the final
standings must stay readable first — hence deletion on dismissal, not on finish (FR-012). The
guard keeps a between-rounds or mid-round session undeletable (FR-012). Relying on the existing
foreign-key cascade means no bespoke cleanup code and no risk of orphaning rows (Principles V and
VII). Deleting the session never touches the lists it referenced (the cascade only flows from
`game_session`).

**Alternatives considered**: deleting on finish (the standings would vanish before anyone read
them); a soft-delete flag (the finding says remove from the database).

## D-506: Quick game builds a normal one-team session from a default list

**Decision**: Add `quick_game::build_request(game_type, default_lists)` returning an ordinary
`CreateSessionRequest` — one team, one round of the chosen type on a default list and column that
fits the type — and `POST /api/sessions/quick {gameType}` that creates it and returns the normal
links. The frontend offers the game type on the title screen and navigates to the session's `p1`
view; it never shows the gamemaster link.

**Rationale**: A quick game is not a new kind of session; it is the existing session with fixed,
sensible choices, so it must reuse `create_session` to inherit every rule, the end dialog, and the
finished-session cleanup (FR-016). Putting the list choice in a named Rust function keeps the
"which list suits this type" rule testable and out of the frontend (Principle II). The default
lists already exist (`default_world_history`, `default_statues_monuments`,
`default_cinema_box_office`), so no new content is needed. Not exposing the gamemaster link is a
frontend choice; the link still exists in the payload but the player path is the only route shown
(FR-015).

**Alternatives considered**: a separate "quick" session type (duplicates every rule); the
frontend picking the list (business logic in the UI, violates Principle II).

## D-507: Remove the theme toggle; dark only

**Decision**: Delete the `ThemeToggle` component and its three usages, and make the theme
provider apply the dark theme always with no toggle API. The dark theme stays as-is.

**Rationale**: The finding asks for dark only; keeping a provider that can only be dark removes
the light path without a larger refactor (Principle VII). Removing the toggle control satisfies
FR-017 and Principle VI (no control that does nothing).

**Alternatives considered**: leaving the toggle but disabling it (a dead control); deleting the
provider and hardcoding the attribute (more churn across tests for no gain).
