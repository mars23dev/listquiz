# Contract: Reveal Reset On Next Round

| Guarantee | Rule |
|---|---|
| **G1** | Starting the next round clears the reveal flag (new round starts hidden). |
| **G2** | After advancing, the gamemaster's reveal control reads "Reveal values". |
| **G3** | Every role sees a freshly started round's values hidden. |
| **G4** | Finishing the game at the last round still reveals the values. |
| **G5** | No change to scores, hearts, the turn, or the timer. |

## Non-goals

- No new dependency, no schema change.
- The reveal action and the finish path are otherwise unchanged.
