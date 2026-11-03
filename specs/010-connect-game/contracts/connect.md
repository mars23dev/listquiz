# Contract: Connect Game And Tripmine Marker

Extends the session state in `specs/001-listquiz-game/contracts/realtime.md` and the session
creation in `specs/001-listquiz-game/contracts/rest-api.md`. Field names are camelCase in JSON.

## 1. The Connect round

`quizType` gains `"CONNECT"`. A Connect round uses one list and one column, like Order, and needs at
least ten valued items. It draws ten items: the **names** (left, with icons) and their **values**
(right, shuffled, no icons).

### State (in `GET /state`, every role)

```jsonc
{
  "connect": {
    "names":  [{ "itemId": "item_a", "name": "Alpha", "iconName": "BookOpen" }, /* ...10 */],
    "values": [{ "itemId": "item_a", "display": "1440" }, /* ...10, shuffled */],
    "matched": ["item_c"],
    "pending": { "nameItemId": "item_a", "valueItemId": "item_b" }   // or null
  }
}
```

- Values are public (Connect is a matching game); names are public; matched ids are public.
- The pending selection is public so the room sees the pair in the middle box.

### Actions

| Action | Body | Who | Effect |
|---|---|---|---|
| `pick_name` | `{ "itemId": "item_a" }` | gamemaster, or the on-turn team | sets the pending name |
| `pick_value` | `{ "itemId": "item_b" }` | gamemaster, or the on-turn team | sets the pending value |
| `confirm_connect` | `{}` | gamemaster, or the on-turn team | judges the pending pair |

- `confirm_connect` requires both a pending name and value.
- Correct pair → +10 points, the pair is matched (removed from pickable), the turn passes.
- Wrong pair → one heart lost, the pending clears, the pair stays unmatched, the turn passes.
- The turn passes after **every** confirm (correct or wrong).
- Each team starts a Connect round with three hearts; hearts reset per round like Order.
- The round is read-only when no team has hearts, and ends when all pairs are matched or no team has
  hearts; ending the last round finishes the game (feature 007).
- A timeout costs one heart and passes the turn (the Order rule).

## 2. The Tripmine mine marker

- `tripmine.mineRank` is the rank (1..=10) of the mine, present for **every** role from the start.
- The progress bar marks that rank from the beginning; it does not reveal which item is the mine.
- The marker is display-only: the mine, the search, and the scoring are unchanged.
