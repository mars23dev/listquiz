# Quickstart: Fix Connect Round Start Panel

Validation that a started Connect round clears the pre-round prompt for every role. See
[spec.md](./spec.md) for requirements and [contracts/pre-round-prompt.md](./contracts/pre-round-prompt.md)
for the exact rule.

## Prerequisites

- Node/npm available for the frontend and the backend already built and runnable.
- A Connect session configured (list + column) or a Connect quick game startable.

## Automated check (primary)

Run the frontend unit test that covers the prompt visibility rule:

```powershell
npm --prefix frontend run test -- Session
```

Expected: the suite passes, including the case that starts from a Connect session-state fixture with a
present `connect` board and asserts:

- `queryByRole('button', { name: /start round/i })` is absent, and
- `queryByText(/waiting for the game to be started/i)` is absent,

while a fixture with no `connect` board still shows the gamemaster button / player waiting message.

## Manual end-to-end check

1. Start the app and create a Connect session (or a Connect quick game).
2. Open the gamemaster view, plus a player view (and a spectator view) in separate tabs.
3. Before starting, confirm: gamemaster sees **Start Round**; player/spectator see **Waiting for the
   game to be started…**.
4. As the gamemaster, press **Start Round** once.
5. Confirm across all tabs:
   - The **Connect board** appears for every role.
   - The **Start Round** button is gone for the gamemaster.
   - The **Waiting for the game to be started…** message is gone for players and the spectator.
   - The game does **not** end, and no leftover control can advance or end the game.
6. Complete or intentionally end the Connect round and confirm normal round/game flow (hearts,
   scoring, game-end dialog) is unaffected.

## Regression check

Confirm Order and Tripmine still behave as before: the pre-round prompt is visible before their round
starts and hidden once their board is active.

## Out of scope

No backend restart or schema change is required; the Connect board is already streamed to the client.
