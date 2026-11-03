# Contract: Pre-Round Prompt Visibility

The session page exposes one UI contract: when the pre-round prompt is shown, and what each role sees
in it. The rule is a pure function of the streamed session state.

## Inputs

The contract reads the session state the client already subscribes to:

- `state` — null when no state is available.
- `state.order` — the active Order board, or null.
- `state.tripmine` — the active Tripmine board, or null.
- `state.connect` — the active Connect board, or null.
- `state.status` — the session status; `'finished'` means the game is over.

## Behavior

| Condition | Result |
|---|---|
| No state | Prompt not rendered. |
| `status === 'finished'` | Prompt not rendered. |
| Any of `order`, `tripmine`, `connect` is present | Prompt not rendered. |
| State present, not finished, and none of `order`, `tripmine`, `connect` present | Prompt rendered. |

When the prompt is rendered:

- The gamemaster sees a button labeled "Start Round" that issues the `next_round` action once.
- Every other role sees the text "Waiting for the game to be started…".
- After the prompt is hidden, neither element is present in the DOM for any role.

## Guarantees

- **G1**: In a Connect session, after the gamemaster starts a round, the prompt is absent for all roles
  (gamemaster, players, spectator).
- **G2**: Starting a Connect round does not finish the game; no visible control remains that would
  advance or end it from the former prompt.
- **G3**: Order, Tripmine, and Connect behave identically with respect to this contract.
- **G4**: The prompt is only ever rendered when a session exists, no game type is active, and the
  session is not finished.

## Non-goals

- No change to Connect gameplay, scoring, hearts, or round-end rules.
- No change to `next_round` server semantics.
- No new rendered control, state, or component.
