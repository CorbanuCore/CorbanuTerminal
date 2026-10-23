# Four receipts failed before one passed, and none of it was the code

**Fable, 2026-09-16.** The PF-83 security increment took five receiving attempts.
Every gate in it passed on the first try. All four failures were my harness, and
each one taught something different, so they are worth separating rather than
summarising as "flaky".

## 1. A shared Cargo target directory lies in both directions

The original problem: workers sharing a target directory with other worktrees.
One worker read another worktree's golden. Another ran its *default-feature*
accounting tests against a library compiled **with** the feature enabled, and
reported 70 failures — including, tellingly,
`accounting_anthropic_default_off_does_not_install...`. Re-running the same
command on the same commit in a clean target: **124 of 124**.

A shared target can turn a passing change into a failure, and can equally hide a
real one. Neither direction is safe.

## 2. A pristine target directory lies too, and it reads like a product bug

So I gave every lane a fresh target. That introduced the mirror error. Tests that
shell out to a built binary fail in **milliseconds** with "could not locate
binary", which looks exactly like a product failure in a summary line:

- `provider_convergence::tmux_permission_reload_...` needs `codex`.
- `thread_settings_confirmation_leaves_mcp_status_responses_unaffected` needs
  `rmcp-client`'s `test_stdio_server`.

Both now get built into the lane's target before the tests run. A 0.027-second
"failure" is a clue, not a verdict.

## 3. Freshness was never the requirement, and it cost 158 GB

Isolation from *other worktrees* was the actual requirement. I conflated it with
freshness, and a fresh target per lane costs roughly 20 GB. After nine receiving
lanes the volume filled and a receipt died with `cargo` exit **101** and
`No space left on device` — an error that reads nothing like a disk problem when
it surfaces as "could not compile codex-app-server".

Receiving now uses **one stable target directory**, reused across lanes and
receipts. It is isolated from every worker worktree, which was the point, and
incremental reuse makes later receipts several minutes faster. 152 GB reclaimed.

## 4. A receipt can fail with every gate green

The integrator requires a clean checkout *after* the tests, which is right — a
gate that leaves the tree dirty has done something it did not declare. The TUI
tests write tmux capture artifacts into `codex-rs/tui/target/`, which was not
gitignored. So a receipt with two exit-0 gates was marked `verification_failed`
for a reason that had nothing to do with either.

`codex-rs/.gitignore` now ignores `/tui/target/`, with the reason recorded at the
line rather than only here.

## What I take from this

Three of the four are the same mistake in different clothes: I fixed a real
problem and did not ask what the fix would cost. Contamination was real, and the
cure was disk exhaustion. Both were real, and the cure for *those* was a false
negative that looked like a broken product.

The part of my method that held up is the part I did not change: **never accept
"the failure is unrelated" without running the same gate against the same base.**
Every one of these was found that way. If I had accepted any worker's word, I
would have received on a contaminated gate or rejected a correct change.
