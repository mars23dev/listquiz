# Quickstart: Fix Eliminated Team Turn Switch

Validation that an eliminated team cannot be put back on turn. See [spec.md](./spec.md) for
requirements and [contracts/turn-switch.md](./contracts/turn-switch.md) for the exact rule.

## Prerequisites

- Rust and Node/npm available; the frontend and backend build.
- An Order or Connect session with at least two teams.

## Automated checks (primary)

Backend — the turn switch refuses an out team:

```powershell
cargo test --manifest-path backend/Cargo.toml
```

Expected: a new test drives a team to zero hearts, posts `set_turn` for that team, and asserts the
action is refused and `active_team_id` is unchanged; a switch to a team with hearts still succeeds; the
same switch in a Tripmine round is allowed. Existing suites stay green.

Frontend — the out team is unselectable:

```powershell
npm --prefix frontend run test -- Session
```

Expected: in a heart-using round the gamemaster turn `<select>` renders the zero-heart team's option as
`disabled`; a team with hearts stays enabled.

Full regression:

```powershell
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run test
npm --prefix frontend run build
```

## Manual end-to-end check

1. Start an Order or Connect round with two or more teams.
2. Play (or use `-1♥`) until one team is at zero hearts.
3. In the gamemaster view, open the turn control: confirm the out team's option is unselectable.
4. If a direct switch to the out team is attempted, confirm it is refused with feedback and the turn stays where it was.
5. Switch the turn to a team that still has hearts; confirm it becomes active.
6. Advance to the next round (hearts reset) and confirm the previously out team is selectable again.
7. In a Tripmine round, confirm every team remains selectable.

## Out of scope

No schema change, no new dependency; automatic turn rotation already skips zero-heart teams.
