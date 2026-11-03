# Contract: Game End After The Last Round

Amends the session lifecycle in `specs/001-listquiz-game/contracts/rest-api.md` and the state in
`specs/001-listquiz-game/contracts/realtime.md`. No new endpoint; a change to when the session
reaches `finished`.

## When a session becomes `finished`

- **Before**: only the gamemaster action `next_round`, when the current round is the last, set
  `status = "finished"`.
- **Now**: the last configured round finishing on its own also sets `status = "finished"`.

A round finishes when:

- an Order round's board is over (every candidate placed, or no team has hearts left), or
- a Tripmine round completes (the mine is hit, or every position is revealed), or
- a timeout leaves no team on turn (Order).

When that happens on the last configured round (`currentRound >= roundCount`) and the session is not
already `finished`, the session becomes `finished` with:

- `revealValues = true`,
- no active team (`activeTeamId = null`),
- `lastMessage = "Game over"`.

This is the same end state the manual finish produced, so nothing downstream changes.

## Client-visible effects (unchanged shape)

- `GET /state` reports `status: "finished"` for every role; the client shows the game-end dialog
  (feature 005) with points only.
- Dismissing the dialog deletes the finished session (feature 005).
- A session whose round is not the last stays `running`; the gamemaster advances it as before.

## Idempotence

- If the session is already `finished`, finishing again is a no-op (no error, no double finish).
- If the current round is not the last, the call changes nothing.
