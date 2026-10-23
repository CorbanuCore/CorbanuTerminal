Both fixes are merged with all CI checks green, and issues #535 and #536 are closed with evidence comments. Nothing is left open or uncommitted, and I've removed the worktrees and build targets.

- **#535 → PR #591, merged earlier** (`596ae2351b`): a writable root that's a link into another root's read-only path is no longer a grant, on both macOS (Seatbelt) and Linux (bwrap), at every level including Permissive. The read-only path stays read-only through the link, and a warning at launch explains why. The review asked for changes once, then approved.
- **#536 → PR #597, merged this run** (`a0dd613ca6`): at every level including Permissive, a command is refused, with a message naming the link and how to remove it, when:
  - a writable root now leads through a link made after the session started, or
  - a root held by another root leads outside every root that holds it.

  Each root's real location is recorded at session start and checked again before every command, and the sandbox only grants writes at that checked location. The review asked for changes twice, then approved twice; after that the branch only took merges from main.

**What this run did:** main had moved 22 commits, including #596 (#583) and #600 (#581), which change the same sandbox files. I merged main into #597 (no conflicts), re-ran every check on the new head, waited for all CI checks, then merged with the head-commit match.

**Checks on the final #597 head:**
- **Core `security::` tests:** 314/314 on macOS with both `TMPDIR=/tmp` and the default.
- **`pf_23_s01` and `pf_25_s01`:** 29/29.
- **glitch:** Linux sandbox, sandboxing and protocol tests pass.
- **clippy `-D warnings`:** clean on macOS and Linux.
- **Real-sandbox tests:** fail before the fix and pass after, on both `sandbox-exec` and bwrap.
- **Ordinary setups:** a `/tmp` alias, a worktree with its main checkout, and a folder created later still work at every level, including Permissive.
- **tmux check** (GLM 5.3 Flash, Permissive, disposable home): before the fix, a link created during the session let a write land outside the workspace. After it, the write was refused and nothing was written. Removing the link let commands run again.

Two local seatbelt tests (`create_seatbelt_args_for_cwd_as_git_repo` and `create_seatbelt_args_with_read_only_git_and_codex_subpaths`) fail only with the default `TMPDIR` on CorbanuDrive. They fail the same way on main without these changes, and pass with `TMPDIR=/tmp` and in CI.

**Not covered by #597** (stated in the PR):
- A root added mid-session (by a permission request or a new working folder) still gets the outside-the-holder check, but isn't compared with the session's start.
- Remote exec-server environments aren't covered.
- Windows is unchanged.

Follow-ups from the #591 review are filed as #605 and #606.

**Cleanup:** I removed both worktrees and their build targets on the Mac, plus the clone and its 55 GB build target on glitch (`~/corbanu-rtx/sr56`, `sr56-target`). The report is `.codex-work/workers-20261002/sbx-links2.result.md`.