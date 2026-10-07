# Data Model: Fix Eliminated Team Turn Switch

No new persisted data. The change gates one existing action against existing data.

## Team (existing `team` row)

| Field | Type | Meaning |
|---|---|---|
| `id` | text | Team identity. |
| `score` | integer | Accumulated score. |
| `hearts` | integer | `0..=3`; zero means the team is out for the round. |

**Out condition**: `hearts <= MIN_HEARTS` (0), the same threshold `scoring::advance_turn` uses to skip a
team. Reset to `MAX_HEARTS` (3) at the start of each round.

## Session (existing `game_session` row)

| Field | Type | Meaning |
|---|---|---|
| `active_team_id` | text / null | The team currently on turn; the turn switch writes this. |
| `current_round` | integer | The round in play; used to look up its quiz type. |
| `status` | text | `setup`/`running`/`finished`. |

## Round config (existing `round_config` row)

| Field | Type | Meaning |
|---|---|---|
| `quiz_type` | text | `ORDER`, `TRIPMINE`, or `CONNECT`. ORDER and CONNECT use hearts; TRIPMINE does not. |

## Turn switch (the gated action)

| Input | Source | Rule |
|---|---|---|
| `teamId` | action request | Must exist in the session. |
| Target hearts | `team.hearts` | If the round uses hearts and hearts = 0, refuse. |
| Round uses hearts? | `round_config.quiz_type` for `current_round` | ORDER/CONNECT → restricted; TRIPMINE → not restricted. |

**Outcome**: either `active_team_id` is set to the target (allowed), or the action is refused and
`active_team_id` is unchanged (target is out).

## State transitions

| From | Event | Result |
|---|---|---|
| Round active, target has hearts | Turn switch | Target becomes active. |
| Round active (Order/Connect), target at 0 hearts | Turn switch | Refused; active team unchanged; feedback returned. |
| Round active (Tripmine) | Turn switch | Allowed regardless of stored hearts. |
| Team at 0 hearts | New round starts (hearts reset to 3) | Team is selectable again. |
| No round started | Turn switch | Not restricted by this rule. |

## Validation rules

- Refuse a turn switch to a zero-heart team in Order/Connect (FR-001, FR-002).
- Return clear feedback on refusal (FR-003).
- Do not offer (or mark unselectable) a zero-heart team in the Order/Connect turn control (FR-004).
- Keep allowing a switch to a team with hearts (FR-005).
- Never restrict Tripmine (FR-006).
- Apply equally to a team eliminated by play or by manual heart adjustment (FR-007).
- Re-enable the team when hearts reset at a new round (FR-008).
- Change nothing else: scores, hearts, matching, rotation (FR-009).
