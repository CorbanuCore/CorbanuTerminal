The dashboard now publishes the integration branch's latest commit, and the four uncommitted files are left out. That holds for future publishes too, because the publish wrapper now exports committed files only. The other session's four files were never staged, committed or modified.

**What the export reads:** Before this change it copied the files as they sit on disk, not as committed, so a dirty checkout didn't stop it. The 06:00Z publish on the server (commit `66a5b66840`) was made that way.

**How the publish was run:**
1. **One-off clean publish.** I made a temporary checkout of the branch at `444f7f8678` (`git worktree add --force`) and ran the committed `sync-source.sh` under the wrapper's usual nonblocking lock. It succeeded: `444f7f8678`, digest `e93dc602e619…`, 451 files, 380 documents.
2. **Fix and normal publish.** After committing the fix, I ran the normal wrapper `initiative-control.oGQGyA/sync-source.sh` from the main checkout. Exit 0: `21162588cd`, digest `43b0917b20ea…`, collected 2026-10-03T17:55:00Z.

**Verification:**
- **Server check:** `sync_check.py check --expected-commit 21162588cd… --expected-digest 43b0917b…` printed "Verified publication".
- **Server timers:** the publish timer was disabled and inactive before and after both publishes; the web service stayed active.
- **The four files:** in both exports, each file's published hash matches the committed version, not the edited copy on disk.
- **Pages:** `http://127.0.0.1:8769/` `index.html`, `facilities.html` and `health.json` all returned 200; `health.json` shows ok, generation `build-3b39mr7g`.
- **Banner:** reads "Manager's declared receiving checkout (committed) · integrate/management-workstreams-20260911 · 21162588cd49", content `43b0917b20ea`, checkout = the main worktree.
- **Facilities page:** the committed version links back to the main dashboard through both the "CORBANU CONTROL" logo and the section menu. It has no source banner; that only appears on `index.html`, by design.
- **Evidence** is saved in `.codex-work/dashboard-clean-publish-20261003/`.

**Code change:** commit `21162588cd`, pushed fast-forward to the integration branch (`444f7f8678..21162588cd`).
- `export.py --committed` reads the dashboard's files straight from the commit, not the working copy. It fails if a file the dashboard needs isn't committed or isn't a regular file, and the banner shows "(committed)".
- `sync-source.sh` takes an optional fourth argument, `committed`; without it, behaviour is unchanged.
- I chose this over keeping a separate clean checkout. That checkout shares the branch, so after any new commit its files would quietly lag behind while still being labelled with the new commit.
- Added two tests and a README note.
- **Tests:** the new tests and the facilities page test pass. The full suite, run on a clean copy, has 54 failures and errors; none of the failing tests touch the export:
  - 50 in `test_owner_daemon`, `test_coordinator` and `test_slack_reply_poll`. Apart from the one below, it's the same set as on the previous commit `444f7f8678`.
  - 4 modules that couldn't load because `slack_sdk` isn't installed in this venv.
  - The extra failure, `test_coordinator…process_crash…`, only fails in full-suite runs (a subprocess can't import `coordinator`). It passes 3/3 alone on both commits.
- **Private wrapper** (not in Git): it now passes `committed`. Backup at `sync-source.sh.bak-20261003`. I didn't touch the owner/manager loop config.
- **Temporary checkouts:** both removed with `git worktree remove`; the wrapper doesn't need either.

**Still open:**
- `export.py` itself still runs from the main checkout, so it loads the edited `facilities.py` as code. Nothing from that file is published, but a broken edit there could crash the export.
- The owner-daemon test failures and the missing `slack_sdk` in the control venv were there before this change.
- I didn't add a dashboard round to `owner-manager-enabled-20261002.md`, and `coordinatorInstructions.md` doesn't mention the new `committed` behaviour yet.