# Contract: Realtime Updates and Role Redaction

> **Amended by `specs/002-fix-play-findings/contracts/realtime-amendment.md`:** the rule below that
> "player and spectator views have no controls" is narrowed — the player role that owns the team on
> turn may act, and the Tripmine view gains `targetValue`.
>
> **Amended by `specs/004-fix-session-feedback/contracts/session-feedback.md`:** the Order view gains
> `order[].placedBy` and the Tripmine view gains `tripmine.items[].pickedBy` (the acting team, public);
> when `revealValues` is set the Order bar is sent in its correct arrangement with an empty pool; and
> cues are driven from the reflected event (the client diffs consecutive revisions).
>
> **Amended by `specs/005-session-flow/contracts/session-flow.md`:** the state gains `timer`
> (`{ totalSeconds, secondsLeft }` or `null`), driven by the `start_timer`/`stop_timer`/`time_out`
> actions; a finished session can be deleted; and quick games are created through
> `POST /api/sessions/quick`. The Tripmine gamemaster view always carries the answer key.
>
> **Amended by `specs/007-fix-game-end/contracts/game-end.md`:** the last configured round ending
> finishes the session by itself (no gamemaster action), which is what makes the game-end dialog
> appear in a gamemaster-less quick game.
>
> **Amended by `specs/006-fix-turn-timer/contracts/turn-timer.md`:** a timeout costs a heart only in
> an Order round (a Tripmine timeout just passes the turn), and the timer plays a start cue and a
> timeout cue.
>
> **Amended by `specs/008-fix-timer-timeout/contracts/timer-timeout.md`:** the server schedules the
> timer's timeout at its end time and applies it itself, so no client posts `time_out`; the client
> shows the countdown, ticks the last five seconds, and shows no timer error.
>
> **Amended by `specs/009-tripmine-mode/contracts/tripmine-mode.md`:** Tripmine adds a Random (Hard)
> side, hides hearts (hearts are Order-only), and a round end that also finishes the game plays the
> round cue before the game-end cue.
>
> **Amended by `specs/010-connect-game/contracts/connect.md`:** adds a Connect game type
> (`quizType: "CONNECT"`, a `connect` state with public names/values/matched/pending, and
> `pick_name`/`pick_value`/`confirm_connect` actions) and a public `tripmine.mineRank`.

## Why revisions instead of pushed state

Every mutation in ListQuiz comes from the gamemaster view; player and spectator views have no controls
(FR-062). The stream therefore carries one signal — "something changed" — and each client re-reads
state for its own role. Hidden item values are redacted in exactly one function on the server, so there
is no code path by which a player view can render a value the gamemaster has not revealed. This is
research decision D-005 and it is the reason the product does not use a bidirectional protocol.

## `GET /api/sessions/{sessionId}/events?role=<role>`

Server-sent events. `role` is validated the same way as on `GET /state`; an unknown session or role
returns `not_found` before the stream opens.

Response headers: `Content-Type: text/event-stream`, `Cache-Control: no-store`.

### Events

| `event` | `data` | When |
|---|---|---|
| `state` | `{"revision": 42}` | After every accepted gamemaster action, round transition, or mute change |
| `heartbeat` | `{"revision": 42}` | Every 15 s while idle, so proxies and phones do not drop the stream |

The client compares `revision` with what it already holds and skips the refetch when nothing moved. The
revision is monotonic per session and survives a restart because it lives on `game_session.revision`.

Example:

```text
event: state
data: {"revision": 43}
```

## `GET /api/sessions/{sessionId}/state?role=<role>`

Returns the full state for that role.

### Gamemaster role

The only role that receives un-revealed values and controls.

```json
{
  "revision": 43,
  "sessionId": "lq-8fj2wkd9",
  "status": "running",
  "muted": false,
  "currentRound": 2,
  "roundCount": 3,
  "round": { "roundNumber": 2, "quizType": "TRIPMINE", "tripmineSide": "highest", "listTitle": "Blockbuster Cinema Box Office", "columnName": "Box Office ($)" },
  "teams": [
    { "id": "team_1", "name": "Team Cyan Pulsar", "color": "cyan", "primaryHex": "#06b6d4", "score": 60, "hearts": 3, "onTurn": true },
    { "id": "team_2", "name": "Team Ruby Spark", "color": "rose", "primaryHex": "#f43f5e", "score": 40, "hearts": 2, "onTurn": false }
  ],
  "order": {
    "anchor": { "itemId": "itm-9ab2", "name": "Eiffel Tower Completed", "value": "330 m" },
    "pool": [ { "itemId": "itm-1", "name": "Colosseum", "value": "48 m", "revealed": true } ],
    "left":  [ { "itemId": "itm-4", "name": "Tokyo Skytree", "value": "634 m", "revealed": true } ],
    "right": [ { "itemId": "itm-7", "name": "Burj Khalifa", "value": "828 m", "revealed": true } ],
    "pending": { "itemId": "itm-2", "name": "Big Ben", "side": "left", "slotIndex": 0, "teamId": "team_1" }
  },
  "tripmine": {
    "side": "highest",
    "searchedRank": 4,
    "hit": false,
    "completed": false,
    "items": [ { "itemId": "itm-5", "name": "Titanic", "revealed": false, "isTripmine": false, "pointsAwarded": null } ]
  },
  "revealValues": false,
  "lastMessage": "Team Cyan Pulsar found Rank #4! (+10 pts)"
}
```

For the gamemaster, `revealed` is always true in effect: `revealValues` is reported but the field
carries the real value regardless, which is how the draft already shows values to the host.

### Player and spectator roles

Identical shape minus controls, minus the answer keys. Two rules are enforced server-side:

1. If the round is an Order round and values have not been revealed, every item's `value` is `null`.
2. If the round is a Tripmine round, `isTripmine` is `true` only for items already revealed.

A player view never receives `pending.teamId` as anything other than the acting team, and never
receives the searched-rank answer — only the searched rank number, which is public by design (FR-038).

```json
{
  "revision": 43,
  "sessionId": "lq-8fj2wkd9",
  "status": "running",
  "currentRound": 2,
  "round": { "roundNumber": 2, "quizType": "TRIPMINE", "columnName": "Box Office ($)" },
  "teams": [ { "id": "team_1", "name": "Team Cyan Pulsar", "color": "cyan", "score": 60, "hearts": 3, "onTurn": true } ],
  "tripmine": { "searchedRank": 4, "items": [ { "itemId": "itm-5", "name": "Titanic", "revealed": false, "value": null } ] },
  "revealValues": false,
  "lastMessage": "Team Cyan Pulsar found Rank #4! (+10 pts)"
}
```

The spectator role differs from a player role in one way: it has no "your turn" emphasis and no team
badge (FR acceptance scenario US5-5). It receives the same redaction.

### Redaction rules, enumerated

| Field | Gamemaster | Player / spectator |
|---|---|---|
| `order.pool[].value`, `order.left[].value`, `order.right[].value` | Always present | `null` until `revealValues` is true |
| `order.anchor.value` | Present | Present — the anchor's value is public by design (FR-026) |
| `tripmine.items[].value` | Always present | `null` until revealed or `revealValues` is true |
| `tripmine.items[].isTripmine` | Always present | Only when that item's `revealed` is true |
| `teams[].score`, `teams[].hearts` | All teams | All teams |
| `round.tripmineSide` | Present | Present |
| Any host action control | Present | Absent |

## Client behaviour

- On mount, a view fetches state once, then opens the stream.
- On `state`, if `revision` is newer, it refetches state and re-renders. No optimistic updates for
  scoring: every visible score comes from the server, so the gamemaster and the room can never disagree.
- On `heartbeat`, nothing is refetched unless the revision has moved.
- On stream error, the client retries with backoff and shows a single reconnecting notice. It never
  falls back to a full-page reload (FR-007 in the quality checklist, FR-066 in the spec).
- Audio cues fire from the client when the refetched state shows a new score event, never from the
  stream message itself, so a reconnect cannot replay a sound.
- A read-only view polls nothing when the stream is healthy; the polling fallback is explicitly absent.