# Quickstart: Select Custom List In Session Setup

Validation that a list in a protected folder can be unlocked and selected in the setup wizard, and that a
session cannot be created from a locked list. See [spec.md](./spec.md),
[contracts/setup-list-access.md](./contracts/setup-list-access.md), and
[data-model.md](./data-model.md).

## Prerequisites

- Rust and Node/npm are available; the repository builds locally.
- No new dependency or database migration is required.
- The existing default folder and its lists are seeded.

## Automated checks (primary)

Backend — session access rules:

```powershell
cargo test --manifest-path backend/Cargo.toml
```

Expected:

- A default-list session still succeeds without `folderPasswords`.
- A custom-list session without the folder password is refused with `unauthorized`.
- A custom-list session with a wrong folder password is refused with `unauthorized`.
- A custom-list session with the correct folder password is created, and its round points at that list.
- An unknown list or a column from another list is still `not_found`.

Frontend — setup wizard:

```powershell
npm --prefix frontend run test
```

Expected:

- The setup wizard loads folders without requesting a locked folder's lists.
- A locked custom folder hides its list title and item count.
- The correct password reveals the list in the round selector.
- A wrong password keeps it hidden and shows `Incorrect password!`.
- Creating a session with a custom list sends only the referenced folder passwords.
- The existing default-list flow still passes.

Type and production checks:

```powershell
npm --prefix frontend run lint
npm --prefix frontend run build
```

## Manual end-to-end check

1. In List Management, create a protected folder and save a list named `Test` with 11 or more items.
2. Open the setup wizard, enter a gamemaster password, and continue to step 2.
3. Confirm the default lists are selectable immediately.
4. Confirm the custom folder appears locked and that `Test` is not visible.
5. Enter the wrong folder password: the folder stays locked and an error appears.
6. Enter the correct folder password: `Test` appears in the List selector with its item count.
7. Select `Test`, choose its value column if more than one is offered, and create the session.
8. Open the gamemaster link and start the round; confirm the round uses `Test` and its values.
9. Add a second round using another list from the same folder without re-entering the password.
10. Reload the setup wizard and confirm the folder is locked again.

## Scope checks

- No list title or item count from a locked folder appears in the page or in a network response before unlock.
- No folder password appears in a URL, browser history, or visible text.
- The default folder remains password-free and read-only.
- No schema change, no new dependency, and build output stays inside the repository.
