Both fixes are merged with every CI check green, and both issues are closed with evidence comments.

| Issue | PR | Merge commit |
|---|---|---|
| #546 | #550 | `3a0a38bed2` |
| #545 | #552 | `e7be21a1a9` |

**#546 — the test was wrong, not the product**
- **Cause:** with developer-accounting, each usage report is written to the cost ledger before `response.completed` is passed on to the turn. So the question always reached the test first, and the test interrupted before the turn had recorded the usage. The token count then came back empty. The ledger still has the usage.
- **Fix (test only):** the test now waits until it has seen both the question and the completed response before interrupting. It also fails if a token count shows up while the question is pending.
- **CI coverage:** no CI job runs codex-core tests with developer-accounting. The only use of the feature is the packaging check, which builds `corbanu` with it and checks it can't be packaged. A full codex-core test job with the feature would cost roughly 20–30 runner-minutes per run, so I didn't add one.
- **Gate:**
  - Tests (glitch): failed 3 of 3 runs with developer-accounting before; after, 5 of 5 pass with it and 5 of 5 without.
  - Clippy `-D warnings`: clean on macOS and glitch, with and without developer-accounting.
  - Opus 5.5 High review: APPROVE.

**#545 — resume now uses the conversation's recorded account**
- **`corbanu resume`** (an id, `--last` or the picker) without `--account` now resumes on the account the conversation ran on, even with no default key. When onboarding would otherwise appear, the picker now comes first.
- **In-app `/resume`:** `/status` now shows the resumed conversation's account. Before, it showed the session's default account. `/new` goes back to the default.
- **Missing account:** the TUI names it with the same `corbanu account add zai work` recovery as the account line. At startup it exits; in `/resume` it shows the error and stays on the current conversation.
- **Explicit `--account`:** still wins.
- **Gate:**
  - **Regression tests:** a new tmux suite, `suite::resume_recorded_account`. Both tests fail on main's binary and pass after; they also run in Ubuntu tmux smoke. Unit tests cover the account check.
  - **Focused tests:** 166/166 pass on macOS. On glitch they pass with and without developer-accounting.
  - **Clippy `-D warnings`:** clean on macOS and glitch, both ways.
  - **tmux check:** GLM 5.3 Flash, disposable home, keyring isolation on. Before/after captures are committed under `qa/provider-auth/pf-84/fix-545-20261011/`.
  - **Video** (23 s, leak scan 0 hits): [pf84-resume-recorded-account](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-545-pf84-resume-recorded-account-218c0d3650bd-2026-10-11.mp4)
  - **Opus review:** round 1 asked for changes because my in-app `/resume` capture didn't actually prove the fix. I re-captured it, then moved the new code into its own module and added unit tests. Round 2 approved.

**Still open**
- **Filed as #555 (none caused by #552):**
  - Forks and side conversations run on the default account, but `/status` shows the parent's account.
  - Resuming a conversation from a provider other than the home's configured one still shows that provider's onboarding.
  - A missing account set in `[provider_accounts]` still refuses `corbanu resume`.
- **Not done:**
  - Inside the TUI, the missing-account message still says to use the `--account` flag rather than `/providers`.
  - No automated test covers in-app `/resume`, or Esc in the picker leading to onboarding. I checked both by hand.
  - The evidence has no record of the code-blind test design that repo policy requires before handing off to human testing.

Cleanup is done: worktrees, branches, build folders on the Mac and glitch, scratch homes and my glitch files are removed.