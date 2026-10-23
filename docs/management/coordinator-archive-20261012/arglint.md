You are a CI repair worker for CorbanuCore/CorbanuTerminal. Clear main's argument-comment-lint backlog.

## Context
- `rust-ci / Argument comment lint` (Linux, macOS and Windows) fails on main. Earlier analysis counted about
  1,100 existing unannotated positional literal arguments, mostly in security-audit, core/security, vault,
  provider-auth and secret-broker tests.
- The convention is in codex-rs/AGENTS.md under "argument_comment_lint". Read it first and follow it exactly:
  `/*param_name*/` before opaque literal arguments (None, booleans, numeric literals) passed by position, using
  the exact callee parameter name, with the documented exemptions.
- Local runner: `just argument-comment-lint`. It uses Bazel, so the first run is slow. Read the justfile and
  the lint's own sources to find the fastest way to list every violation, ideally with machine-readable
  output and suggested fixes.

## Do
1. `git worktree add` a new worktree /Volumes/CorbanuDrive/Corbanu/worktrees/arglint-20261002 on a new
   branch `codex/arg-comment-backlog-20261002` from origin/main.
2. Collect every violation that CI reports on all three platforms. Check the failing jobs' logs on
   https://github.com/CorbanuCore/CorbanuTerminal/pull/147 (gh run view --log-failed) as well as your local
   run.
3. Fix them mechanically and exactly: comments only, plus no behavior change. Where a better API shape would
   be preferred, still add the comment; this PR is mechanical only.
4. Run `just fmt`, then rerun the lint until it is clean. Spot-check that the changed crates still compile
   (`cargo check --tests -p <crate>` for the touched crates).
5. Push the branch and open a PR to main titled "chore: annotate positional literal arguments for
   argument-comment-lint". The body says it is mechanical, gives the counts per crate, and says how it was
   verified.
6. Never force-push other branches, merge, or touch other PRs. Never print or read secrets.

## Report (your final message)
The PR URL, the violation count before and after, the counts per crate, the lint result, and anything you
couldn't fix mechanically, with the reason.
