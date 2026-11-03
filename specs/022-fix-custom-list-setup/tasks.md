---

description: "Task list for custom-list setup access"
---

# Tasks: Select Custom List In Session Setup

**Input**: Design documents from `/specs/022-fix-custom-list-setup/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/setup-list-access.md, quickstart.md

**Tests**: Included because the project constitution requires every business rule to have a test and the plan defines automated backend/frontend checks. Write the tests first and confirm they fail before implementation.

**Organization**: Tasks are grouped by user story so each increment is independently testable.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: User story label for story-phase tasks only
- Every task names the exact repository path it changes

## Path Conventions

- Web application: `backend/src/`, `backend/tests/`, `frontend/src/`
- Feature documents: `specs/022-fix-custom-list-setup/`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Confirm the change stays inside the existing project shape before editing.

- [X] T001 [P] Confirm no dependency or schema change is needed; keep `backend/Cargo.toml`, `frontend/package.json`, and `backend/src/db.rs` unchanged.
- [X] T002 [P] Record baseline test status with `cargo test --manifest-path backend/Cargo.toml` and `npm --prefix frontend run test`; note any pre-existing failures before implementation.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Shared request field, list-folder lookup, and access rule needed by every story.

- [X] T003 [P] Add `pub fn folder_id(conn: &Connection, list_id: &str) -> ApiResult<String>` in `backend/src/modules/lists/mod.rs` with the existing not-found message, and update `backend/src/routes/lists.rs` to reuse it instead of duplicating the lookup.
- [X] T004 In `backend/src/modules/sessions/mod.rs`, add `use std::collections::HashMap;` and `#[serde(default)] pub folder_passwords: HashMap<String, String>` to `CreateSessionRequest`, preserving the container's camelCase serde naming so the wire field is `folderPasswords`.
- [X] T005 In `backend/src/modules/sessions/mod.rs`, add the required `use crate::modules::{folders, lists};` import and implement `validate_round_folder_access(conn: &Connection, round: &ValidRound, folder_passwords: &HashMap<String, String>) -> ApiResult<()>`; resolve the list's folder through `lists::folder_id`, compare with `folders::password_matches`, allow the default folder without an entry, and return `ApiError::unauthorized()` for a missing or wrong password without echoing a list title.
- [X] T006 Call `validate_round_folder_access` for every round in `create_session` in `backend/src/modules/sessions/mod.rs` after `validate_round_references` and before the insert transaction.
- [X] T007 [P] Add `folderPasswords?: Record<string, string>` to the `CreateSessionRequest` interface in `frontend/src/api/client.ts` so the setup page can send the access map.

**Checkpoint**: The backend can validate list-folder access and the frontend type can carry the proof. No user story UI exists yet.

---

## Phase 3: User Story 1 - Select a custom list for a round (Priority: P1) - MVP

**Goal**: A gamemaster unlocks a protected folder in setup, sees its list in the round selector, and creates a session from it.

**Independent Test**: Create a protected folder with a list named `Test`, open setup, unlock the folder, select `Test`, create the session, and confirm the round uses `Test` and its chosen column.

### Tests for User Story 1

> Write these first and confirm they fail.

- [X] T008 [P] [US1] Add a backend integration test in `backend/tests/session_create.rs` that creates a protected folder and list, then posts a session body with the correct `folderPasswords` entry and asserts `201`, a session id, and a `round_config` row pointing at the custom list and column.
- [X] T009 [P] [US1] Update the fetch mock in `frontend/src/test/SetupQuiz.test.tsx` for `GET /api/folders`, `POST /api/folders/{folderId}/unlock`, and `GET /api/folders/{folderId}/lists`, then add a test that unlocks a folder, sees the custom list, selects it, and asserts `POST /api/sessions` includes only `folderPasswords: { [folderId]: password }`.

### Implementation for User Story 1

- [X] T010 [US1] In `frontend/src/pages/SetupQuiz.tsx`, load `fetchFolders()` alongside `fetchDefaultLists()`; store the default folder plus custom folders with `id`, `name`, `isDefault`, and `locked`, and preserve the existing default-list error behavior.
- [X] T011 [US1] In `frontend/src/pages/SetupQuiz.tsx`, add an inline protected-folder access panel showing each custom folder's locked state, a password input, an `Unlock` action, and the `Incorrect password!` error path.
- [X] T012 [US1] In `frontend/src/pages/SetupQuiz.tsx`, call `unlockFolder(folderId, password)` on submit, then `fetchFolderLists(folderId, password)` only after success; store the lists by folder id and never request a locked folder's lists.
- [X] T013 [US1] In `frontend/src/pages/SetupQuiz.tsx`, build combined source lists from the default lists plus every unlocked custom folder's lists, carry each entry's `folderId` and `folderName`, render them in the round List selector with item counts, and keep `pickList` resetting `columnId` to the selected list's first column.
- [X] T014 [US1] In `frontend/src/pages/SetupQuiz.tsx`, build `folderPasswords` from only the custom folders referenced by configured rounds and pass it to `createSession` in `createSessionAndShowLinks`.
- [X] T015 [US1] In `frontend/src/pages/SetupQuiz.tsx`, make the unlock panel keyboard operable with associated labels, visible focus, and Enter-submit behavior without blocking the page during requests.

**Checkpoint**: User Story 1 works end to end: custom folder unlock, custom list selection, session creation, and correct data in the round.

---

## Phase 4: User Story 2 - Keep protected lists protected during setup (Priority: P2)

**Goal**: Locked folders reveal no list information, wrong passwords fail clearly, and direct session requests cannot bypass folder access.

**Independent Test**: With a locked folder, inspect the page and network requests; no list title or item count appears. Post a session directly with that list id and no password map and confirm refusal.

### Tests for User Story 2

> Write these first and confirm they fail.

- [X] T016 [P] [US2] Add backend tests in `backend/tests/session_create.rs` for a custom list with no `folderPasswords` entry and with a wrong password, asserting `400 unauthorized`, `Incorrect password!`, and no `game_session` row.
- [X] T017 [P] [US2] Add frontend tests in `frontend/src/test/SetupQuiz.test.tsx` proving a locked folder's list endpoint is never called, its title is absent, a wrong password leaves it absent and shows the error, and the default-list flow sends no unnecessary folder password.

### Implementation for User Story 2

- [X] T018 [US2] Harden the unlock failure path in `frontend/src/pages/SetupQuiz.tsx`: on a wrong or empty password keep the folder locked, clear the password draft, keep its lists out of `availableLists`, and show the existing `Incorrect password!` copy.

**Checkpoint**: User Story 2 is independently testable: privacy and direct-request refusal hold even without using the full setup flow.

---

## Phase 5: User Story 3 - Use several folders and lists in one setup (Priority: P3)

**Goal**: Multiple protected folders can be unlocked independently and used across rounds, including duplicate list titles.

**Independent Test**: Create two protected folders with different passwords and one list in each. Unlock both in one setup visit, configure two rounds using a list from each folder, and create the session.

### Tests for User Story 3

> Write these first and confirm they fail.

- [X] T019 [US3] Add a backend test in `backend/tests/session_create.rs` that creates two protected folders with different passwords, asserts a session with both correct entries succeeds, and asserts omitting one folder's entry refuses the request.
- [X] T020 [US3] Add a frontend test in `frontend/src/test/SetupQuiz.test.tsx` that unlocks two folders, distinguishes duplicate list titles by folder label, reuses both folders across two rounds, and changes a list without leaving an invalid column.

### Implementation for User Story 3

- [X] T021 [US3] In `frontend/src/pages/SetupQuiz.tsx`, qualify every custom list option with its folder name so duplicate titles are distinguishable, while keeping default-list labels unchanged.

**Checkpoint**: All three stories work independently and together.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Final verification and repository hygiene.

- [X] T022 [P] Run `cargo fmt --manifest-path backend/Cargo.toml --check` and `cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings`.
- [X] T023 [P] Run `npm --prefix frontend run lint` and `npm --prefix frontend run build`.
- [X] T024 [P] Inspect `frontend/src/pages/SetupQuiz.tsx` and `frontend/src/api/client.ts` to confirm folder passwords appear only in the JSON body or `X-Quiz-Password` header, never in a URL, query string, browser storage, or visible text.
- [X] T025 Run the full regression suite with `cargo test --manifest-path backend/Cargo.toml` and `npm --prefix frontend run test`, then execute the scenarios in `specs/022-fix-custom-list-setup/quickstart.md`.
- [X] T026 [P] Confirm `backend/src/db.rs`, `backend/Cargo.toml`, and `frontend/package.json` stayed unchanged and no build output became tracked.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: Starts immediately.
- **Foundational (Phase 2)**: Depends on Setup; blocks all user stories.
- **User Story 1 (Phase 3)**: Depends on Foundational. This is the MVP.
- **User Story 2 (Phase 4)**: Depends on Foundational for backend enforcement; its UI checks depend on the setup panel from US1.
- **User Story 3 (Phase 5)**: Depends on Foundational and the setup list selector from US1.
- **Polish (Phase 6)**: Depends on all desired stories.

### User Story Dependencies

- **US1 (P1)**: No dependency on US2/US3. Delivers the complete custom-list selection path.
- **US2 (P2)**: Backend privacy rule is already in Foundational; its frontend test path relies on US1's unlock panel.
- **US3 (P3)**: Builds on US1's combined list selector; no backend change beyond tests.

### Within Each User Story

- Write and fail the listed tests before implementation.
- Foundational backend helper and request field precede the access rule.
- Access rule precedes `create_session` wiring.
- Setup folder discovery and unlock precede combined list selection.
- Combined list selection precedes building `folderPasswords`.

### Parallel Opportunities

- T001 and T002 can run in parallel.
- T003 and T007 touch different files and can run in parallel.
- T008 (backend test) and T009 (frontend test) can run in parallel.
- T016 (backend tests) and T017 (frontend tests) can run in parallel.
- T022, T023, T024, and T026 can run in parallel after implementation.
- T004, T005, and T006 all edit `backend/src/modules/sessions/mod.rs` and must stay sequential.
- T010 through T015 all edit `frontend/src/pages/SetupQuiz.tsx` and must stay sequential.
- T016, T019, and T008 all edit `backend/tests/session_create.rs`; do not run them concurrently.
- T017 and T020 edit `frontend/src/test/SetupQuiz.test.tsx`; do not run them concurrently.

---

## Parallel Example: User Story 1 Tests

```bash
Task: "Add correct-password custom-list session test in backend/tests/session_create.rs"
Task: "Add setup unlock/select/create test in frontend/src/test/SetupQuiz.test.tsx"
```

## Parallel Example: User Story 2 Tests

```bash
Task: "Add missing/wrong password backend refusal tests in backend/tests/session_create.rs"
Task: "Add locked-folder privacy tests in frontend/src/test/SetupQuiz.test.tsx"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1 setup checks.
2. Complete Phase 2 foundational request field, lookup, and access rule.
3. Complete Phase 3 US1 tests and implementation.
4. Stop and validate: create a protected folder with `Test`, unlock it in setup, create a session, and confirm the round uses it.
5. Demo or commit the MVP increment.

### Incremental Delivery

1. Foundation ready: backend accepts `folderPasswords` and validates folder access.
2. US1: custom list selection works end to end.
3. US2: privacy and direct-request refusal are proven.
4. US3: multiple folders and duplicate titles are usable.
5. Polish: formatting, linting, build, and quickstart validation.

---

## Notes

- Tests are included because the constitution requires business rules to be tested and the plan defines automated checks.
- Do not persist folder passwords or add a token/cookie mechanism.
- Do not add a new endpoint, dependency, or schema migration.
- Keep locked-folder list names out of requests, responses, and the DOM until unlock succeeds.
- Commit after each logical task group, never with a skipped or failing required check.
