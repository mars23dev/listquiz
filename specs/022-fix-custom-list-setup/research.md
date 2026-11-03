# Research: Select Custom List In Session Setup

## D-2201: Reuse the existing folder endpoints; do not add an all-lists endpoint

**Decision**: Load folders with `GET /api/folders`, validate a password with
`POST /api/folders/{folderId}/unlock`, and load a protected folder's lists with
`GET /api/folders/{folderId}/lists` using the existing `X-Quiz-Password` header. Keep
`GET /api/default-lists` for the password-free default folder.

**Rationale**: These endpoints already own folder discovery, password comparison, and list privacy
(FR-001, FR-002, FR-007). Reusing them keeps the locked-folder boundary intact and adds no new server
surface (Principle VII). A new all-lists endpoint would either bypass the password rule or duplicate it.

**Alternatives considered**: a single all-lists endpoint (breaks the locked-folder privacy rule); fetching
every folder's lists eagerly (leaks names before unlock); adding a new folder-picker endpoint (duplicates an
existing API for no gain).

## D-2202: Prove folder access on session creation with a request-scoped password map

**Decision**: Extend the `POST /api/sessions` body with an optional
`folderPasswords: { [folderId]: password }` map. For every round, resolve the list's folder; the default
folder needs no entry, and a custom folder must have a matching entry. A missing or wrong entry is refused
with `unauthorized` before a session is written.

**Rationale**: Folder passwords are stateless by design: there is no login, token, or cookie. The setup
wizard may use lists from several folders, so one header cannot carry every proof. A map is the smallest
request-scoped proof that preserves the existing model (FR-014, FR-016; Principle VII).

**Alternatives considered**: a server-side unlock token or cache (adds expiry, storage, and lifecycle rules
for no present-day need); a single password header (cannot represent multiple folders); a password query
parameter (violates the no-passwords-in-URLs rule); trusting the client's prior unlock (would let a direct
request use a locked list).

## D-2203: Enforce a named list-folder access rule in the sessions module

**Decision**: Add `validate_round_folder_access(conn, round, folder_passwords)` in
`backend/src/modules/sessions/mod.rs`. It resolves the list's folder through a narrow
`lists::folder_id(conn, list_id)` helper and delegates the comparison to the existing
`folders::password_matches`. Keep the existing list/column existence check separate and run both before the
insert transaction.

**Rationale**: The rule is business logic and must be a named, directly testable unit (Principle II). The
lookup is shared with the list routes, so a small helper avoids two copies of the same SQL (Principle V).
The default folder stores an empty password, so the existing comparison naturally permits it with no entry
in the map.

**Alternatives considered**: inlining the query in the route handler (untestable business rule); duplicating
the list lookup in `sessions` (two maintenance points); treating any list id as valid (the current bug's
security gap).

## D-2204: Put folder access inline in step 2, not behind a new page or modal

**Decision**: In `SetupQuiz.tsx`, add a folder access panel showing the default folder and every custom
folder. A custom folder starts locked; selecting it reveals an inline password input and `Unlock` button.
A successful unlock loads that folder's lists and keeps them available for the rest of the wizard. Wrong
passwords show the existing `Incorrect password!` copy and reveal nothing.

**Rationale**: The setup wizard already has a three-step flow; adding a fourth step would disrupt the tested
navigation contract. An inline form avoids a new dialog component, focus trap, and duplicated modal markup
(Principle VII) while remaining keyboard accessible (Principle VI).

**Alternatives considered**: a separate folder step (more navigation and more tests); a modal unlock dialog
(more focus-management code for no user benefit); one folder per round (repeats the password and contradicts
the per-wizard unlock rule).

## D-2205: Combine available lists and qualify labels with the folder name

**Decision**: The round's List selector contains the default lists plus every list from unlocked custom
folders. Options read `{title} ({itemCount} items)` for default lists and
`{folderName} / {title} ({itemCount} items)` for custom lists. Selecting a different list resets `columnId`
to that list's first column; a single-column list needs no column choice.

**Rationale**: One list selector keeps the existing round form and column behavior. Folder-qualified labels
make duplicate titles distinguishable without a second selector (FR-009, FR-012). Locked folders contribute
no options, so their names never appear (FR-002).

**Alternatives considered**: one list selector per folder (clutters the round form); unqualified titles
(ambiguous when two folders contain the same list title); a separate list modal (unnecessary interaction).

## D-2206: Keep unlock state in the setup page memory only

**Decision**: Store entered folder passwords in a `Record<folderId, password>` state and clear it when the
page unmounts or reloads. Send only the passwords for folders referenced by the configured rounds. Never put
a password in a URL, the DOM text, or persistent storage.

**Rationale**: This matches the existing stateless password model (research D-006 in feature 001) and
satisfies FR-016 and FR-017. Sending only referenced passwords limits unnecessary credential exposure.

**Alternatives considered**: `sessionStorage`/`localStorage` (persists a password beyond the wizard);
sending every unlocked password (sends credentials the request does not need); a cookie/token (a new auth
mechanism the product explicitly does not have).

## D-2207: Test the access rule and the setup flow independently

**Decision**: Add backend integration tests for a custom list with no password map, a wrong password, and
the correct password, plus the unchanged default-list path. Add frontend tests that a locked folder's list
endpoint is not called, a correct unlock exposes the list, a wrong password leaves it hidden, and session
creation sends only the referenced `folderPasswords`. Update the existing setup mocks to answer
`GET /api/folders`.

**Rationale**: The rule is business logic (Principle II) and must be exercised without the full UI
(Principle I). The frontend tests pin the privacy and request-shape behavior that caused the bug report.

**Alternatives considered**: manual-only verification (does not protect the access rule); backend-only
tests (would not catch a setup page that still loads only default lists).
