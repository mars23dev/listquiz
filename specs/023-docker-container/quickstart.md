# Quickstart: Run ListQuiz as a Docker Container

Manual validation guide for the local Docker image. It proves the feature end to end without a registry or
a release.

## Prerequisites

- Docker Desktop or Docker Engine with BuildKit, running.
- Repository checkout at the repository root.
- Internet access for the first build to pull base images and dependencies.
- Host port 8080 free, or an alternate host port chosen for the run command.
- No host Rust or Node.js installation is needed.

## 1. Build the image

```powershell
docker build --tag listquiz:local .
```

Expected: the command exits successfully and `docker image ls listquiz` shows `listquiz:local`. Both the
frontend stage and the backend stage must report success; a failure in either stage fails the build.

## 2. Start the container

```powershell
docker run --rm --name listquiz --publish 8080:8080 --volume listquiz-data:/app/data listquiz:local
```

Expected: the container stays running. In another terminal:

```powershell
docker ps --filter name=listquiz
docker logs listquiz
```

The log should report that the server is serving the built frontend on port 8080.

## 3. Check the HTTP surface

```powershell
Invoke-RestMethod http://localhost:8080/api/health
```

Expected: `status` is `ok` and the reported version matches the backend package version.

Open `http://localhost:8080/` in a browser.

Expected: the ListQuiz landing page loads with its styles and assets. No fallback placeholder or missing
asset error appears.

## 4. Check list import and gameplay

1. Open the list management page.
2. Export any shipped default list to CSV.
3. Create a custom folder.
4. Import the exported CSV into that folder by file upload or pasted text.
5. Confirm the import succeeds and the new list is listed with its item count.
6. Create a session using the imported list and start a round.

Expected: the import report is shown, the list is selectable in setup, and the round renders normally.

Optional export check: export the imported list and confirm a CSV download is produced.

## 5. Check persistence across container replacement

1. Note the list created in step 4.
2. Stop the container:

   ```powershell
   docker stop listquiz
   ```

3. Start a new container with the same volume:

   ```powershell
   docker run --rm --name listquiz --publish 8080:8080 --volume listquiz-data:/app/data listquiz:local
   ```

4. Reload the list management page.

Expected: the custom list, its folder, and its items are still present. Default lists are not duplicated.

## 6. Check a clean rebuild

1. Stop the container.
2. Delete or rename the local generated directories `frontend/dist`, `frontend/node_modules`,
   `backend/target`, and `backend/data` (they are ignored by Git).
3. Rebuild without the cache:

   ```powershell
   docker build --no-cache --tag listquiz:local .
   ```

4. Start a new container with a different empty volume:

   ```powershell
   docker run --rm --name listquiz-check --publish 8081:8080 --volume listquiz-check-data:/app/data listquiz:local
   ```

5. Open `http://localhost:8081/`.

Expected: the image builds from source alone, the landing page loads, and the empty volume starts with the
seeded default lists. This confirms no stale host build output was used.

Clean up the check container and volume:

```powershell
docker stop listquiz-check
docker volume rm listquiz-check-data
```

## 7. Check existing automated tests

The container feature does not change application logic, so the existing checks must still pass:

```powershell
cargo fmt --manifest-path backend/Cargo.toml --check
cargo clippy --manifest-path backend/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path backend/Cargo.toml
npm --prefix frontend run lint
npm --prefix frontend test
npm --prefix frontend run build
```

Expected: all checks pass. The portable release packaging remains untouched and can still be run with
`powershell -ExecutionPolicy Bypass -File scripts/package-release.ps1`.

## 8. Stop and clean up

```powershell
docker stop listquiz
docker volume rm listquiz-data
```

The first command removes the container because it was started with `--rm`. The second command is
destructive and deletes the persisted game data; run it only when a reset is intended.

## Failure checklist

| Symptom | Check |
|---|---|
| `port is already allocated` | Use `--publish 8090:8080` and open `http://localhost:8090/` |
| Build cannot fetch dependencies | Confirm Docker has network access; retry the build |
| Backend fails at startup | Inspect `docker logs listquiz` for the missing path or permission error |
| Landing page missing | Confirm the frontend stage completed and `/app/frontend/dist/index.html` exists |
| Data disappears after recreation | Confirm the same named volume is mounted at `/app/data` |
| Permission denied on the volume | Use the documented named volume, or fix ownership for a bind mount |
| Container exits immediately | Inspect the log; verify the data path is writable and port 8080 is free inside the image |
