You are the operations worker who finishes turning the Corbanu management loop on.

## Owner decisions
- 2026-10-02 (decision `owner-manager-enable-20261002`): turn the manager loop on.
- 2026-10-02, answering "accept dispatch without a real worker test, or run the test first?": **"Run the test
  first."** So no named-limitation shortcut: preflight item 4 must be satisfied by a real isolated worker test.

## Read first
- Your predecessor's report: /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/owner-enable.result.md
  and its evidence at /Volumes/CorbanuDrive/Corbanu/.codex-work/owner-manager-enable-20261002/
- Its record: qa/initiative-control/management-bootstrap/owner-manager-enabled-20261002.md (uncommitted) in
  /Volumes/CorbanuDrive/Corbanu/worktrees/management-workstreams-20260911, plus the owner-*.md records it cites
  (owner-promotion-94-preflight.md, owner-preflight-98*.md, owner-cutover-85.md, owner-armed-20260917.md,
  bridge-tmux-transport-20260915.md).
- The live runtime: ~/Library/Application Support/corbanu-owner-live/ (runtime/, private/, schedule/).

## State left by your predecessor
- The owner is armed at generation 1, scope fixture-only. A new durable transport is staged in
  private/transport.json: binary `/Volumes/CorbanuDrive/Corbanu/.codex-work/owner-worker-bin/704650ccd8/corbanu`,
  sha256 7b8c77a6…, and runs_dir `~/.local/state/corbanu-owner-runs`.
- Round-94 preflight refused items 1, 2, 4 and 5:
  1. The manager prepares actions through `freeze_worker_inputs`, with explicit provider and `--yolo` authority.
  2. A fresh prepared action exists to hand over.
  4. A real isolated worker test (ACK/START/RETURN) on the staged binary and package, plus an independent review.
  5. Hand dispatch and publishers are quiet within 300 s of the check.

## Do, in order
1. Item 1: implement whatever the records say is missing so the manager prepares worker actions through
   `freeze_worker_inputs`, with explicit provider and authority. Source is scripts/initiative_control/ in the
   shared checkout. Four uncommitted files there belong to another session: never stage or commit them, and keep
   your edits off those files where you can. If you must touch one, stop and report. Test with the existing
   unittest suites, then commit and push to integrate/management-workstreams-20260911 (fast-forward only).
2. Item 2: have the manager prepare one small, real, safe action for the owner. A good choice is a read-only
   repository check or a documentation-only task in a disposable worktree.
3. Item 4: run the real isolated worker test that the procedure defines (ACK, START, RETURN) on the staged
   binary/package, with the evidence the records require. For the independent review, launch a separate
   reviewer with `~/.local/bin/corbanu exec -m claude-opus-5-5-plan -c model_provider="claude-plan"` and give it
   only the evidence and the procedure. Record its verdict.
4. Item 5: the coordinator's other workers run in the tmux server at
   /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/tmux.sock (`tmux -S <sock> ls`). Wait until no
   session other than `tasknode` and your own is running there. You are launched on a separate socket. Then
   rerun the preflight.
5. If the preflight passes: run the documented promotion recipe (disarm gen 1 with receipt, restage or
   reinstall with transport + `manager_enabled: true`, authority with scope tmux-workers at the next generation,
   `--arm --dry-run`, then arm). Then verify from state: interval ticks, no holds, routing tmux-workers, and the
   owner picking up the prepared action and returning it.
6. Republish the dashboard with `.codex-work/initiative-control.oGQGyA/sync-source.sh`. Then write the record
   `owner-manager-enabled-20261002.md` (extend it), commit and push it fast-forward.

## Never
Never use unscoped `tmux kill-server`. Never read or print credentials. Never edit product Rust source. Never
force-push. If a guard refuses and you can't satisfy it honestly, leave the owner in a safe state (armed
fixture-only, or disarmed) and report exactly what is needed.

## Report (your final message)
Per preflight item: the evidence; then the review verdict, the generation, state proof, the dashboard, commits
and anything still open.
