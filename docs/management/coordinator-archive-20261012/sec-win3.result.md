Both defects are fixed and merged, and the real-Windows gate rerun passed in a normal (non-elevated) session. One product decision is open (#300 below).

**Issues and fixes**
- **D2, #294 (labelled security)** – fixed by **#298** (merged `b2ac6e6f`).
  - **Cause:** agent commands go through a launch path that never turned the contract's deny entries into Windows ACLs, so nothing protected `secrets\`. A protected folder that didn't exist yet was also skipped.
  - **Fix:** that path now applies the denies. Protected folders such as `secrets` and `wallet` are created and denied before first use.
  - **Hardening from review:** while the contract is on, every launch from that process keeps the vault denies, including ones the contract doesn't wrap.
- **D1, #295** – fixed by **#302** (merged `661b5c6a`).
  - **Cause:** Windows' secondary logon service needs three access rights on the calling process (measured on the gate machine). One of them amounts to full access, so it can't be granted back without undoing the hardening.
  - **Fix:** when the direct call is refused, Core starts a short-lived, unhardened launcher to make the call. It inherits only its two pipes, gets an almost empty environment, and its result is checked.
  - **Also affects elevated sessions:** with PF-27-S07 merged (#282/#284), the call failed from an elevated session too before this fix.
- **CI change (in #302):** `windows-security-probes` now runs both regression tests with the same restricted token Windows gives an admin in a normal session (medium integrity). That covers the `workspace-write` profile. It fails if that token isn't actually medium integrity, or if the tests are missing.
  - **Not covered:**
    - a separate non-admin account;
    - the sandbox's one-time admin setup from a normal session, which needs a UAC click. The step reuses a setup recorded just before.

**Gate evidence**
- **Tests on the real machine, normal session:** each regression test fails before its fix and passes after. They also pass elevated and through the CI script.
- **CI:** all checks green on #298 and #302.
- **Reviews (Opus 5.5 High):** #298 got REQUEST_CHANGES, then APPROVE. #302 got REQUEST_CHANGES twice, then APPROVE.
- **Clippy:**
  - Linux clippy `-D warnings` is clean on the RTX box for both PRs.
  - Windows clippy is clean for my code. `main` already fails Windows clippy on `core` (`model_broker_auth.rs`, `exec_policy.rs`) and on the `protected-state` crate, so I checked core with those lints allowed. Something to fix separately.
- **Rerun at `main` `661b5c6a48cd`, normal session:**
  - Both new tests pass.
  - `pf27s06-win-normal-session-works`: an agent's `whoami` runs as `CodexSandboxOffline`. The sandbox log confirms the launcher was used.
  - `pf27s06-win-vault-denied`: under `workspace-write` the vault, `auth.json`, config and state database are all denied.
  - Both videos are published and leak-scanned (the recorder's scan, plus a second scan on the Mac).
- **Docs:** the rerun is recorded in #312 (merged `8a566c88`). The sprint record is now `ready` for the P1 integration owner to receive and archive.

**Still open**
- **Decision for Travis (#300):** on the tool path, the unelevated Windows sandbox still doesn't enforce deny-read rules. I left that unchanged so users with `secretless_agent_launch` off see no difference. I recommend failing closed with a message pointing to `[windows] sandbox = "elevated"`, which is what the other exec path already does.
- **Follow-ups filed:**
  - #301: a second session on the same config directory with the flag off can remove the vault denies.
  - #304: deny ACLs are never actually removed, because the revoke call only removes allow entries. Side effect: user deny-read rules now applied on the elevated backend stay after the rule is removed.
  - #307: the launcher's pipe handles are briefly inheritable by other processes Core starts.
- **Cleanup:** removed the key file, the temporary local user, seed files with sandbox passwords, run folders, scheduled tasks, the RTX clone and the Mac worktrees.
  - I kept `C:\CorbanuQA\target` (a build of `main`) on the Windows host for the PF-27-S07 gate.
  - My notes are in `.codex-work/workers-20261002/sec-win3.result.md`.