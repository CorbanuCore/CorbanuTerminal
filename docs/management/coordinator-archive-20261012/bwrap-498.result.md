Both fixes are merged with every check green and none pending, and #498 and #503 are closed with evidence comments. Travis's decision is recorded on #498.

**What shipped**
- **#508 → #498, merged `49c6eb66dc`.**
  - This path is used when the sandbox has full write access and network goes through the managed proxy. Writing to `/dev/null` there now works: the sandbox mounts a fresh `/dev` after the writable `/` bind, the same as #496.
  - Behaviour change for Permissive on this path: commands see a minimal `/dev` (14 entries instead of the host's 229), and `/dev/shm` and `/dev/pts` are private to the sandbox.
- **#511 → #503, merged `dbf278f833`.**
  - Writable roots are now ordered and compared by their real path, so a writable symlink to `/` is treated like `/`.
  - The reviewer found that this reordering alone could let protections be dropped: when two roots bind the same folder, or a link's target sits inside another root. Each pass now re-applies the read-only paths and denials that earlier passes set below it. For a link, that includes the folder the link points to.
  - It's stricter, never weaker: a writable link can no longer reopen a path another root made read-only, for example a link to a workspace's `.git`. Judging from the code (not tested), macOS still allows that. The PR says so.

**Gate evidence**
- **#498 before/after:** the new test `managed_proxy_full_write_keeps_dev_null_usable` (real bubblewrap, glitch) fails on the old code with `/dev/null: Permission denied` and passes with the fix.
- **#498 functional check:** fresh debug binary on glitch, disposable home, Permissive, GLM 5.3 Flash.
  - To reach this path I turned on the network proxy and used a profile with `":root" = "write"`.
  - Before, `echo hi > /dev/null; echo rc=$?` gave `rc=1` with Permission denied. After, it gave `rc=0`.
  - One early "before" run doesn't count: the model answered `rc=0` without running the command.
  - The Z.AI key file was deleted from glitch afterwards.
- **#503 before/after:**
  - On `origin/main`, three of the five new unit tests fail.
  - The real-bubblewrap test fails too, with `bwrap: Can't mkdir /.git`. Before this fix, a writable symlink to `/` made every command fail.
  - With each half of the fix removed in turn, the matching test fails. Without the re-apply, a read-only folder outside the workspace became writable.
  - With the full fix, everything passes.
- **Linux sandbox suites on glitch:** all pass. #508 had 102 unit and 33 integration tests; #511, merged with #508, had 107 unit and 34 integration tests.
- **Clippy `-D warnings`:** clean on Linux (glitch) and macOS for both PRs.
- **Reviews (Opus 5.5 High, read-only):**
  - #508: APPROVE in round 1. I then made its small suggestions (a doc comment, the PR's evidence section and product citation) and didn't run a second round.
  - #511: APPROVE in round 4. Rounds 1–3 each found a real gap, and each is now fixed and tested.

**Opened**
- **#525 needs your decision:** a writable root at `/` without full-write access still fails every command with `mkdir /.git`. It fails closed, so nothing is exposed. The fix would be to skip creating those folders at `/`, which matters if the sandbox runs as root in a container.

**Mistake:** I accidentally ran `rm -f /dev/null` on glitch. It ran as the normal user, failed and changed nothing; `/dev/null` is intact.

**Cleanup:** worktrees, build folders on both machines (including 31 GB on glitch), the glitch clone, the disposable homes and the binaries are removed. Only the glitch logs and scripts remain, 444 KB in `~/corbanu-rtx/fix498-logs`. The local branch `fix-503-bwrap-symlink-root` is still in the main checkout; deleting it would need a force-delete, since the local main isn't updated.

**Not done:** no demo videos; your brief didn't list them.

Files are in `.codex-work/workers-20261002`:
- `fix-498-503.result.md`
- `fix-498-review.result.md`
- `fix-503-review.result.md`