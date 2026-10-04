# Contract: UI Parity with the Draft

The user requires the draft frontend in `the reference frontend` to be reused as much as possible and taken 1:1 for
design. The constitution requires `the reference frontend` to stay read-only: it is never edited, never imported, and
never a build input. This file is the reviewable list of what is copied unchanged, what is corrected,
and what is dropped, so the copy can be verified without diffing the whole tree.

## Files copied unchanged in behaviour

| Draft file | Becomes | Notes |
|---|---|---|
| `src/main.tsx` | `frontend/src/main.tsx` | Entry point, unchanged |
| `src/index.css` | `frontend/src/index.css` | Font declarations and scrollbar styling kept; colour classes replaced by theme variables (D-008) |
| `components/StageLighting.tsx` | `frontend/src/components/StageLighting.tsx` | Decorative background, unchanged |
| `components/ItemIcon.tsx` | `frontend/src/components/ItemIcon.tsx` | Icon rendering with `HelpCircle` fallback, unchanged |
| `components/IconPickerModal.tsx` | `frontend/src/components/IconPickerModal.tsx` | Searchable icon set plus image upload, unchanged |
| `components/BottomTeams.tsx` | `frontend/src/components/BottomTeams.tsx` | Team podiums with colour, score, hearts, `YOU` badge, unchanged |
| `components/TopBarOrdering.tsx` | `frontend/src/components/TopBarOrdering.tsx` | 11-slot bar layout; drag plus click-to-place; `CONFIRM PLACEMENT` / `CANCEL`; pending state |
| `components/MiddleItemGrid.tsx` | `frontend/src/components/MiddleItemGrid.tsx` | Pool grid; placed items hidden; drag or click to place |
| `components/TripmineBoard.tsx` | `frontend/src/components/TripmineBoard.tsx` | 10-item grid, searched-rank display, feedback banner, explosion and defuse states |
| `utils/audio.ts` | `frontend/src/utils/audio.ts` | All existing cues, unchanged |
| `src/types/quiz.ts` | `frontend/src/types/quiz.ts` | Domain types reused; extended where the spec adds fields |

## Files copied and corrected

| Draft file | Correction | Requirement |
|---|---|---|
| `src/App.tsx` | Keeps the draft's four-view navigation; replaces the hash router with history-based path routing matching `contracts/rest-api.md`, since the server owns `/`, `/lists`, `/setup`, `/{sessionId}/gamemaster`, `/{sessionId}/p1`-`p4`, and `/{sessionId}/view`, redirects `/{sessionId}` to `p1`, and falls back to `index.html`; replaces `localStorage`/`BroadcastChannel` state with the API client and the SSE subscription; adds the light theme attribute | FR-060, FR-061, FR-063, FR-012 |
| `components/LandingPage.tsx` | Layout, copy, `LIST`+`QUIZ` wordmark, and both cards kept; "Quick Start Sample" removed because no session list or demo generator ships | clarification 5 |
| `components/SetupQuiz.tsx` | Three-step wizard kept. Round count becomes 1-10 with rows added and removed. Gamemaster password becomes mandatory. Team count becomes 1-4. Step 2 gains the list folder unlock step. Step 3 keeps the link list with copy and launch. | FR-001, FR-004, FR-007, FR-045 |
| `components/ListManagement.tsx` | Editor form, column manager, draggable table, `Add Row`, icon picker, CSV modal, duplicate, export, and delete kept. A folder bar is added above the list grid: create folder, unlock, rename, delete with password confirmation. The default folder renders read-only. The 11-100 range is enforced with inline validation. The sensitive-data hint is added to the editor and the import modal. | FR-045 to FR-059 |
| `components/GameSessionView.tsx` | Keeps the top bar, role switcher, perspective banner, password gate with the draft's copy (`Gamemaster Access`, `Enter the moderator password to unlock host controls.`, `Incorrect password!`, `Switch to View`, `Unlock`), the waiting banner, and the board wiring. Removed: the timer block. Added: mute toggle, reveal control, manual score and heart controls, `Next Round`. Order scoring changed to +10. Turn starts rotate per round. | FR-015, FR-021, FR-024, FR-023, FR-029 |
| `src/data/quizData.ts` | Becomes seed data for the default folder, moved into `backend/src/modules/default_lists/`. The legacy `QuizCategory` shape is dropped; only the three lists survive. | FR-050 |
| `src/utils/listStorage.ts` | Not ported. Its list building and CSV parsing become the Rust `lists` and `csv_import` modules; its storage becomes SQLite. | FR-061, Principle III |
| `src/utils/sessionSync.ts` | Not ported. Its broadcast and storage-event sync becomes the SSE endpoint. | FR-061, Principle III |

## Additions not present in the draft

| Addition | Why | Requirement |
|---|---|---|
| Theme context, toggle, and `data-theme` attribute variables | The draft is dark-only | FR-063 |
| Folder bar and folder endpoints | The draft stores lists flat with no access boundary | FR-045 to FR-051 |
| Mute control | The draft plays audio with no way to silence it | FR-024b |
| Server-side value redaction and SSE client | The draft syncs state between browser tabs only | FR-060 to FR-062 |
| Keyboard operation for the pool grid and bar slots | The draft is drag-first | FR-065 |
| Not-found view for unknown sessions | The draft silently shows a blank board | FR-013 |
| 11-100 item validation with a named range | The draft's lists can hold any count | FR-052, FR-059 |

## Copy rules

1. Nothing in `frontend/` may import from, read, or reference `the reference frontend` at build time or run time. The
   draft path may appear in a code comment marking provenance and nowhere else.
2. Copy order is draft first, then correct. Correcting a screen is an edit inside `frontend/`, never an
   edit in the draft.
3. Visual decisions that the specification is silent about — spacing, radii, typography, motion, icon
   choices, copy strings — follow the draft.
4. Any visual deviation beyond the eight Design Authority corrections must be justified in the pull
   request, because the user's requirement is 1:1 parity.