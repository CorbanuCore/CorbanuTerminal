You are the manager-loop recovery worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/mgr-status1.result.md
and sec-common.md (binding). Scope: restore the existing manager/owner jobs to a healthy standby. Not granted: pushes to
main except docs PRs, Slack sends, credential-store reads, giving the manager new allocations, changing its mode.
1. Recreate worktrees/owner-loop-r7-20261003 detached at ec7918b368 so the live owner settings and their digest are
   unchanged; verify the digest; then recover the owner job (com.corbanu.initiative-owner) using the supported
   --recover <evidence> path described in the records, and confirm it leaves HOLD.
2. Diagnose the manager job's TimeoutExpired HOLD (com.corbanu.initiative-owner.manager) from its logs and records; if the
   cause is understood and benign (e.g. a long tick), recover it the supported way; if not, leave it held and report.
3. Confirm both jobs tick idle with no new holds over ~15 minutes. Do not seed work.
4. Record what you did under qa/initiative-control/ in a docs-only PR (merge when green with
   `gh pr merge N --merge --match-head-commit <sha>`).
Report concisely, including anything that needs Travis.
