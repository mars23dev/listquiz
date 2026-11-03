# Data Model: Fix Connect Round Start Panel

This feature has no persisted data. It concerns the visibility of the pre-round prompt, derived from
the session state already streamed to the client.

## Pre-round prompt visibility

The prompt is a derived, non-persisted value computed during render.

| Field | Source | Meaning |
|---|---|---|
| Session present | `state` | The client has session state to render. |
| Order board active | `state.order` | An Order round is on screen. |
| Tripmine board active | `state.tripmine` | A Tripmine round is on screen. |
| Connect board active | `state.connect` | A Connect round is on screen. |
| Session finished | `state.status === 'finished'` | The game is over. |

**Visibility rule**: the prompt is shown when the session is present, **no** game type is active
(Order, Tripmine, and Connect all absent), and the session is not finished. It is hidden otherwise.

**Role rendering**: when the prompt is visible, the gamemaster sees the "Start Round" control and every
other role sees the "Waiting for the game to be started…" message. When it is hidden, neither is
rendered for any role.

## State transitions

| From | Event | To |
|---|---|---|
| Session started, no game type active, not finished | Gamemaster starts a round (`next_round`) | A game-type board becomes active → prompt hidden for all roles |
| Any game-type board active | Round ends, another round begins | Prompt stays hidden while any board is active |
| Any state | Session finishes | Prompt stays hidden |

## Validation / edge rules

- Connect behaves identically to Order and Tripmine: the prompt disappears as soon as the Connect
  board is present.
- The prompt must never remain visible while a game-type board is active (the reported defect).
- Hiding the prompt removes the only control that issued `next_round`, so a started round cannot be
  ended by pressing the former control (FR-004, FR-007).
- A finished session never shows the prompt.
