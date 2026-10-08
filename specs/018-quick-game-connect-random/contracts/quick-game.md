# Contract: Quick Game

## 1. Offered game types

| Guarantee | Rule |
|---|---|
| **G1** | The landing page offers Order, Tripmine, and Connect quick games. |
| **G2** | Choosing Connect starts a playable Connect round immediately (one team, one round, no host). |

## 2. Eligible list

| Guarantee | Rule |
|---|---|
| **G3** | A list is eligible only if it has at least one column and enough items with values for the game type. |
| **G4** | An ineligible list is never chosen. |

## 3. List choice

| Guarantee | Rule |
|---|---|
| **G5** | Among eligible lists the choice is uniformly random. |
| **G6** | With exactly one eligible list, that list is used. |
| **G7** | With no eligible list, the request is refused with a clear message. |
| **G8** | Repeating a quick game of one type can use a different list. |

## Non-goals

- No new dependency, no schema change.
- One team, one round, immediate start, and the player view are unchanged.
