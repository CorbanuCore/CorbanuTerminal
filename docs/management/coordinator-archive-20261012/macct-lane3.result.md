PF-84-S04 is merged and set to `ready` (not archived). PR #430 went in at its fully green head `710fa7a562` (merge commit `c7e6e07cbc`). It was merged by IridiumMaster while my own merge command was queued. I stopped before S05 because my context is large. S05 is still an unallocated draft for a fresh worker.

**What S04 adds** (all behind `named_accounts` except the duplicate-row fix)
- **Duplicate-row bug fixed:** a configured provider now has exactly one row in onboarding, not a setup row plus a "use existing" row. Choosing it offers:
  - Use configured credentials
  - Add another account
  - Replace with … (a new key or sign-in)
- **`/providers`:** one row per provider and account. Each row shows the kind, a 12-character fingerprint and "this session" / "default for new sessions" markers. Actions:
  - **Add:** the value is masked, and adding never overwrites an existing account.
  - **Use for this session:** starts a new session on that account; a running thread never changes account.
  - **Make default:** writes `[provider_accounts]` in config.toml.
  - **Rename:** refused for the account this session is using; the default follows the rename.
  - **Remove:** deletes only that account. If the account is in use, you pick a replacement first.
- **Account shown:** `/status` and the spawn line name the account in use, e.g. "account fake".
- **S03 follow-up tests:** D3 after the security level is raised mid-session, a worker started by the TUI inheriting its parent's account, and a running-thread account mismatch.

**Gate evidence** (`qa/provider-auth/pf-84/s04-gate.md`)
- **Tests:** focused `just test` passes on core, app-server, protocol and TUI, with and without developer-accounting. The remaining TUI failures are environmental and were already recorded by earlier lanes: Unix socket paths too long on this drive, the wallet daemon not starting, and a popup snapshot that doesn't match main.
- **Real regression fixed:** one onboarding tmux test failed because focus left a provider's row once it was configured. It now passes.
- **Linux clippy `-D warnings`:** clean on glitch, with and without developer-accounting, on the final merged tree. All 38 PR checks passed; one flaky Windows sandbox probe passed on rerun (it also flaked on #422).
- **Review (Opus 5.5 High):** the first pass asked for changes with 2 blocking findings:
  - adding an account could overwrite an existing one;
  - a failed session switch still reported success, and removal went ahead.

  Both were fixed, along with 7 non-blocking findings. A scoped second pass approved, and I fixed its two notes. That second pass is the one review over budget, which I authorised as lane owner.
- **tmux runs on GLM 5.3 Flash with the real Z.AI key:**
  - The default account replies "pong". After adding account `fake` with a masked key, switching the session to it shows `/status … account fake` and gets a 401 naming `fake`.
  - Esc in the menus changes nothing.
  - After a restart, the default set in config still holds.
  - Rename and remove behaved as designed, and `corbanu account list` shows only that account's entry is gone.
  - Claude Plan took a second account.
  - Kimi (second provider) gave the same pong-then-401 result.
  - The spawn line read `(glm-5.3-flash max · account fake)`.
  - Leak scans found no fake key and no real Z.AI key in any file in the test homes.
- **Videos** (leak scan: no hits):
  - [no duplicate rows](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s04-pf84-no-duplicate-rows-7a1ab9bc9ddb-2026-10-10.mp4)
  - [add account](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s04-pf84-add-account-7a1ab9bc9ddb-2026-10-10.mp4)
  - [switch account](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s04-pf84-switch-account-7a1ab9bc9ddb-2026-10-10.mp4)

**Open items**
- **Next worker:** S05 needs a small docs PR first to allocate its worktree, branch and write scope.
- **Code-blind acceptance for S04** hasn't been run; it belongs to the independent acceptance step.
- **Known limits:**
  - A session can switch only one provider to a named account at a time; a second one is refused with an explanation.
  - "Use for this session" starts a new session rather than switching the current one.
  - Onboarding adds key and Claude-token accounts; a Claude Code login folder is added only in `/providers`.
  - `corbanu account add` (from S02) still overwrites an existing account.
  - The newer v2 multi-agent spawns don't show the account (v1, the default, does).
- **Possible side effect:** while stopping my own test run on 2026-10-10, I ran `pkill -f "nextest run"`. That may also have killed other workers' test runs that were going at the same moment, so they may need rerunning.
- **Clean-up:** my local and glitch worktrees and build folders are removed. The S01/S02 leftovers on the RTX box are still there because that box is down.

**For Travis:** nothing new. ChatGPT and AWS accounts are still refused, and your D4 decision has already moved them to PF-84-S06.