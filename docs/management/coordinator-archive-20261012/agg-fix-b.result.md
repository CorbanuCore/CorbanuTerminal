Five of the six issues are fixed and merged on main. The sixth, #444, is merged except its python3 noise part (AGG-36), which is in PR #463 and waits on Travis. The content filter never stopped me.

One thing to know first: IridiumMaster merged my first four PRs (#448–#451) before my independent reviews came back. One review found that #448 made Aggressive weaker: a password stored in a setting that only *ends* with `credential_store`'s name, nested under `shell_environment_policy.set`, no longer blocked Aggressive. That was on main briefly; #457 fixed it and is merged. The other review fixes went in through #460.

## What shipped
| Issue | PRs | State |
|---|---|---|
| #438 preflight false positive | #448, follow-up #457 | merged, closed |
| #436 restart now lost the env key | #449, follow-up #460 | merged, closed |
| #439 refusals not attributed | #450, follow-up #460 | merged, closed |
| #437 inspector problems | #451, follow-up #460 | merged, closed |
| #435 /spawn Troll failure | #451 (+#460) | merged, closed |
| #444 polish | #452 merged; #463 open | issue left open for #463 |

- **#438:** only the exact top-level settings with their allowed values are exempt (`cli_auth_credentials_store`, `mcp_oauth_credentials_store`, `model_auto_compact_token_limit_scope`). The same name under another table, or any other value, still blocks.
- **#436:** when the environment key has been handed to the credential broker, restart now is no longer offered. The screen says to quit and start again from the shell. The downgrade review now also lists nested agents and protected paths.
- **#439:**
  - Writes outside the project, protected reads and network failures now show a warning naming Aggressive and /security, and the model gets the same note.
  - Failing git/python3 commands are not blamed on the level.
  - The model is told web search is off.
  - The raw `CreateProcess { message: "Rejected(..." }` string is gone.
- **#437:**
  - A declined approval now appears under Recent denials.
  - "every row verified at launch" now only claims the rows marked "checked at launch".
  - After the first turn the TUI was rebuilding its sandbox profile from a lossy summary, which made the session rows look weaker. That is fixed; the actual policy never loosened.
- **#435:** a Troll spawned before any Nazgul now starts the crew instead of failing.
  - Child panes: approved commands stay sandboxed. In a Troll pane under Aggressive, an approved `touch /tmp/…` was denied and an approved `curl` had no network.
- **#444 (#452):**
  - The "Restarting…" line is erased after restart.
  - Approval prompts now say a letter in brackets only highlights its option and Enter confirms.
  - The startup boundary warning no longer says a file present at start was "not found".
  - New test: a remembered "don't ask again" command does not cover a chained command.

## Gate evidence
- **Regression tests:**
  - The tests that reproduce the milestone failures fail on the old code and pass now (e.g. the exact raw `Rejected(...)` string, "nothing refused" after a decline).
  - The behaviour-level tests (#439 suite test, #436 picker test) were added with the fix, so they were never run against the old code.
  - The focused tests were last run together on all fixes merged onto main (217 passed), both with and without developer-accounting.
  - The full codex-tui unit run has two failures that are not mine: an out-of-date command-popup snapshot on main and a terminal-dependent pets test.
- **Linux clippy -D warnings on glitch:** clean, plain and developer-accounting, on all fixes combined.
- **Independent reviews (Opus 5.5 High):**
  - First round: #449 approved; #448, #450, #451 and #452 got changes requested, and all of those were fixed.
  - Follow-up round: #457 and #460 approved. #452's only remaining point (a stale snapshot) was fixed and tested; I didn't re-run the review after that.
- **tmux runs:** GLM 5.3 Flash, disposable homes, keychain off. Each fix was reproduced with the milestone steps on main, then shown fixed on the patched build.
- **Videos:** published to the `demos` release, with specs and index rows committed:
  - `agg-438-store-mode-preflight`
  - `agg-436-restart-keeps-key`
  - `agg-439-refusals-named`
  - `agg-437-inspector-decline`
  - `agg-435-first-troll`
  - `agg-444-banner-hint`
  - `agg-444-python-clean` (for #463)

## Decisions for Travis
1. **#463 (sandbox change for the python3 noise).** It allows two harmless macOS lookups (temp-folder and user-membership) in the sandbox's base policy, so they apply at every level, Permissive with network off too. The spec says Permissive must not silently change, and the reviewer asked for your sign-off.
   - Even with it, the first python3 run under Aggressive can still be noisy until the macOS tool cache exists.
   - Options: approve and merge, scope it to Aggressive only (more work), or accept the noise.
   - My recommendation: approve. The two lookups give no network, file or credential access, and the stock Codex sandbox already allows them in another mode.
2. **#436, restart now under Aggressive.** I kept the credential broker one-way: it never hands keys back, so restart now is simply unavailable once an environment key was handed over.
   - The alternative is a one-time "hand keys back for restart" call in the broker. That would weaken a stated design rule of the broker.
   - My recommendation: keep it one-way.

## Also found (filed, not fixed)
- #466: a sandbox-denied command can stay shown as "Running".
- #467: a workspace opened through a symlinked path (e.g. /tmp on macOS) reports `.env` as not isolated. The same mismatch makes some security unit tests fail when TMPDIR is /tmp.
- #468: in /spawn, choosing Max reasoning for a Troll saves the main model instead of spawning.

I didn't change any behaviour for #443 or #445. Cleanup is done: my worktree, the 106 GB build target, the glitch worktree and target, and all temporary run folders are removed. PR #463's branch stays on origin.