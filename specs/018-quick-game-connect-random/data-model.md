# Data Model: Quick Game Connect And Random Lists

No schema change. This describes the quick game's inputs and the list choice.

## Quick game request (existing `POST /api/sessions/quick`)

| Field | Meaning |
|---|---|
| `gameType` | `ORDER`, `TRIPMINE`, or `CONNECT` (Connect now reachable from the UI). |

## Game type → required items

| Game type | Minimum items on the list |
|---|---|
| Order | anchor + ten candidates |
| Tripmine | ten slots |
| Connect | ten pairs |

## Eligible list

A default list is eligible for a game type when it has at least one column and at least the required
number of items with values for that column.

## Random list choice

| Situation | Result |
|---|---|
| More than one eligible list | One chosen uniformly at random. |
| Exactly one eligible list | That list is used. |
| No eligible list | Refused with a clear message. |

## Validation rules

- The quick game offer includes Order, Tripmine, and Connect (FR-001).
- A Connect quick game starts immediately and is playable (FR-002).
- The list is chosen at random among eligible lists (FR-003, FR-004).
- Single-fit is deterministic; no-fit is refused (FR-005, FR-006).
- Repeats can produce different lists (FR-007).
- One team, one round, immediate start, player view are unchanged (FR-008).
