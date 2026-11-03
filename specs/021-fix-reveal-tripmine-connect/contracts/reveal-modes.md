# Contract: Reveal Values Per Mode

| Mode | Before reveal (player/spectator) | After reveal |
|---|---|---|
| Order | Values hidden | Values shown (unchanged) |
| Tripmine | Values, ranks, and the mine hidden | Values, ranks, and the mine shown |
| Connect | Values shown, pairing hidden | Pairing (answer key) shown |

| Guarantee | Rule |
|---|---|
| **G1** | The reveal works in Tripmine and Connect as well as Order. |
| **G2** | Tripmine shows values, ranks, and the mine after the reveal. |
| **G3** | Connect shows the name↔value pairing after the reveal. |
| **G4** | Nothing hidden is shown before the reveal. |
| **G5** | The reveal does not end the round or change scoring, hearts, the turn, or the timer. |
| **G6** | The reveal resets when the next round starts. |

## Non-goals

- No new dependency, no schema change.
- The gamemaster's existing full view is unchanged.
