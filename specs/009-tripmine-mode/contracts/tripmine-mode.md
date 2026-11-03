# Contract: Tripmine Mode

Amends the Tripmine behaviour in `specs/001-listquiz-game/contracts/realtime.md` and the session
creation in `specs/001-listquiz-game/contracts/rest-api.md`. No new endpoint.

## 1. Tripmine sides

The `tripmineSide` value a round carries is one of:

| Value | Label | Mine | Searched target |
|---|---|---|---|
| `lowest` | Lowest | rank 1 | the next unrevealed rank below the last hit |
| `highest` | Highest | rank 10 | the next unrevealed rank above the last hit |
| `random` | Random (Hard) | a random rank | a random unrevealed item that is not the mine, re-chosen after each hit |

- The setup screen offers all three; Random is labelled "Hard".
- The side is fixed for the round and shown on the board.
- Scoring is the same for all three sides (target hit, safe pick, mine penalty, final-item bonus).

## 2. Hearts

- Hearts are an Order mechanic. During a Tripmine round no heart indicator is shown, and no Tripmine
  event (pick, safe reveal, timeout) changes any team's hearts.
- Order rounds are unchanged.

## 3. Cue order at the end

- The cue derivation returns an ordered list. The round-ending cue plays first; if the same transition
  also finishes the game, the game-end cue plays after it.
- A game ending on the mine plays the explosion, then the game-end cue.
- A round ending without finishing the game plays exactly one cue.
- All cues remain silent while the session is muted.
