# Worker brief: republish the initiative-control dashboard without the four uncommitted files

You are an Opus 5.5 High operations worker. Work autonomously; do not ask questions.

## Context
- Main integration worktree: `/Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911`
  (branch `integrate/management-workstreams-20260911`). It has four uncommitted files in
  `scripts/initiative_control/` (facilities.js, facilities.py, facility_control.py, test_control.py) that belong
  to another session (backup: `.codex-work/management-checkout-uncommitted-20261002.patch`). Never stage, commit,
  revert, stash or otherwise touch them.
- The dashboard publish (`scripts/initiative_control/sync-source.sh`, see `scripts/initiative_control/README.md`
  around "sync-source.sh", `coordinatorInstructions.md`, and the latest
  `qa/initiative-control/management-bootstrap/owner-manager-enabled-20261002.md`) has been skipped for several
  rounds because it exports from that dirty working copy.
- The owner/manager loop is armed (generation 11) and may hold the source-sync lock; respect the lock and the
  documented caller/wrapper — find how the publish is normally invoked (the "private wrapper", its task_root, the
  nonblocking lock) and use that path.

## Do
1. Work out exactly what the export reads (HEAD only, or working-copy files too) and whether a dirty tree blocks it.
2. Publish the current integration HEAD from a clean checkout: e.g. a separate worktree at the branch tip (use
   `git worktree add --force` if the export requires the branch name to be checked out), or whatever the documented
   wrapper supports for a declared receiving checkout. Do not modify the main worktree.
3. Run the publish through the normal wrapper/lock, then verify the server publication with the script's own
   `sync_check.py check` result and by fetching the published dashboard page(s) and confirming the banner shows the
   expected branch/commit/digest. Confirm the facilities page links back to the main dashboard; if it does not,
   report what the committed source does (do not fix it with the uncommitted files).
4. If a durable fix is warranted (e.g. the wrapper should always export from a clean receiving checkout), implement
   it narrowly with tests, run `scripts/initiative_control` tests, commit and push fast-forward to the integration
   branch (pull first; never stage the four files).
5. Leave the clean worktree in place only if the wrapper now depends on it; otherwise remove it with
   `git worktree remove`.

## Never
Never print secrets. `rm -f` is blocked. No force-push. Don't change the owner/manager loop config.

## Final report (your last message)
How the publish was run, the verification evidence (commit, digest, URLs checked), any code change with commits,
and what's still open.
