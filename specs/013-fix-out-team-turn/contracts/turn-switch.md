# Contract: Gamemaster Turn Switch

The gamemaster turn switch names the active team. This contract states when it is accepted or refused.

## Inputs

- `teamId` — the team to put on turn.
- The session's `current_round` and that round's `quiz_type` (`round_config`).
- The target team's `hearts`.

## Behavior

| Round quiz type | Target hearts | Result |
|---|---|---|
| ORDER or CONNECT | > 0 | Accepted: the target becomes the active team. |
| ORDER or CONNECT | 0 | Refused: active team unchanged; clear feedback returned. |
| TRIPMINE | any | Accepted: the target becomes the active team. |
| No round started / not a heart-using round | any | Accepted (not restricted by this rule). |

## Guarantees

- **G1**: In Order and Connect, a team with zero hearts is never made the active team by a turn switch.
- **G2**: A refused switch leaves `active_team_id`, scores, hearts, and the round unchanged.
- **G3**: A switch to a team with hearts behaves exactly as before.
- **G4**: Tripmine turn switches are unrestricted.
- **G5**: A team eliminated by play or by a manual heart adjustment is refused the same way.
- **G6**: After a new round resets hearts, the team is switchable again.

## UI mirror

- The gamemaster turn control marks a zero-heart team's option `disabled` when the round uses hearts
  (Order/Connect). The backend remains the authority; the UI only reflects G1.

## Non-goals

- No change to automatic turn rotation, scoring, hearts, or matching.
- No schema, dependency, or new state.
