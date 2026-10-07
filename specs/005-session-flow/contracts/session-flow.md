# Contract: Session Flow And Quick Game

Extends the realtime contract in `specs/001-listquiz-game/contracts/realtime.md` and the REST
contract in `specs/001-listquiz-game/contracts/rest-api.md`. Field names are camelCase in JSON.

## 1. Turn timer

> **Amended by `specs/006-fix-turn-timer/contracts/turn-timer.md`:** the timeout's consequence
> depends on the round type — an Order timeout costs one heart, a Tripmine timeout only passes the
> turn — and the timer now plays a start cue and a timeout cue. The gamemaster starts it from the
> fixed presets 15s / 30s / 60s.
>
> **Amended by `specs/008-fix-timer-timeout/contracts/timer-timeout.md`:** the server owns the
> deadline — starting a timer schedules the timeout, which fires with no client action — and the
> client no longer posts `time_out` (so no timer error bar). Each of the last five seconds ticks.

### State (in `GET /state`, every role)

```jsonc
{
  "timer": { "totalSeconds": 20, "secondsLeft": 14 }   // or null when no timer runs
}
```

- `secondsLeft` is computed by the server from the timer's end time at read time; the client
  counts down locally between reads and posts `time_out` when it reaches 0.
- Public to every role.

### Actions (`POST /api/sessions/{sessionId}/actions`)

| Action | Body | Who | Effect |
|---|---|---|---|
| `start_timer` | `{ "seconds": 20 }` | gamemaster (password) | records the end time; `seconds` > 0 |
| `stop_timer` | `{}` | gamemaster (password) | clears the timer; no heart lost |
| `time_out` | `{ "teamId": "team_2" }` | gamemaster, or the player role that owns the on-turn team | accepted only when a timer runs and `now >= endsAt`; the on-turn team loses one heart, the turn passes, the timer clears |

- `time_out` is idempotent: once the timer is cleared, a second call is refused (`not_ready`).
- Any end-of-turn event (placement confirmed, Tripmine pick, `pass_turn`, `set_turn`,
  `next_round`) clears the timer.
- A timeout is a failed turn: one heart, then the normal turn advance; existing round-end rules
  apply when hearts run out.

## 2. Delete a finished session

`DELETE /api/sessions/{sessionId}`

- Allowed only when the session's status is `finished`; otherwise `not_ready`.
- Deletes the session and, through the schema cascade, its teams, rounds, round state, and score
  events. Quiz lists are never affected.
- Deleting an already-missing session returns success (idempotent).
- After deletion, `GET /state` and the event stream report `not_found`.

The frontend calls this when the game-end dialog is dismissed, then returns to the title screen.

## 3. Quick game

`POST /api/sessions/quick`

```jsonc
// request
{ "gameType": "ORDER" }   // or "TRIPMINE"
// response: the normal CreateSessionResponse
{ "sessionId": "lq-abcd1234", "links": { "gamemaster": "...", "players": [...], "spectator": "..." } }
```

- Creates an ordinary one-team, one-round session of the chosen type on a default list that fits
  the type, with the round already started.
- The frontend shows only the `p1` player view and never presents the gamemaster link.
- The session behaves like any other: the end dialog appears at the end, and the finished session
  is deleted when that dialog is dismissed.

## 4. Game-end dialog (UI)

- Shown for every role when `status` is `finished`; never when only a round has ended.
- A modal dialog (`role="dialog"`, focus moved into it, dismissible by keyboard) listing every
  team ordered by points, highest first, showing points only — no hearts.
- Dismissing it triggers the session deletion in §2 and returns to the title screen.

## 5. Tripmine gamemaster visibility

- The gamemaster's board shows every item's value and rank, picked or not.
- Player and spectator boards show a value and rank only for a revealed item; unchanged otherwise.

## 6. Theme

- No screen offers a light/dark toggle; the application renders the dark theme only.
