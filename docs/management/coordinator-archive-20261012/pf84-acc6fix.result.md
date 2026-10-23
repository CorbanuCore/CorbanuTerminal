I've merged both PRs with every check green, and closed all four issues with evidence. Each PR's tests fail on main's code and pass after the fix. The one exception: #572's new unit tests use functions the fix adds, so its before/after proof is the tmux check.

| PR | Fixes | Merge commit |
|---|---|---|
| #604 | #570, #571, #573 | `8ad5610319` |
| #610 | #572 | `1e8db9d09d` |

**#570 — a named-account session no longer deletes `auth.json`.**
- Cases 1, 5 and 7 now leave `auth.json` byte-for-byte unchanged, and case 7 is answered as `key` instead of being refused.
- The same holds for `resume`, `fork` and `exec resume` of a named thread, because their account is only known after startup.
- The ruled-out default login is still left in place, but that process never uses it. Switching to the default account, or starting an agent on it, is refused with the reason, and the login is not deleted.
- Default sessions behave as before: case 3 is still logged out.
- **Exception:** when the TUI reuses an already-running local app-server daemon, it still logs out the default login at startup as before. Fixing that needs your decision (#595, below).

**#571 — no more requests without credentials.** The cause was that a Z.AI session quietly sets an "API key only" sign-in rule for onboarding. The named ChatGPT account inherited that rule, so its sign-in was refused and the request went out unauthenticated.
- Named accounts now follow only a `forced_login_method` you actually set in config.
- A named account with no usable sign-in is refused with the reason instead of sending a bare request.
- While testing I found that OpenAI didn't appear in `/model` at all without some default OpenAI credential. The acceptance runs hid this by setting `OPENAI_API_KEY`. That is fixed too.

**#573:** `corbanu login --account <name> --device-auth` and *Add another account* in `/providers` offer ChatGPT sign-in whatever the configured provider. Only a configured `forced_login_method` narrows them, and the hint now names the account.

**#572:** while the vault holds a Bedrock AWS-profile account, `/model` offers Bedrock's models from another provider's session. Requests are signed by that account's profile (`AKIDWORKPROFILE` in the mock), and a "model metadata not found" warning that appeared after the switch is gone. Picking a Bedrock model without choosing an account uses Bedrock's default account; that's documented.

**Overlap with pf84-fix6:** its PR (#585) merged while I was working, and I merged it into both branches. The #571 root cause was separate from its switching work. The final review round checked how the two interact and approved.

**Gate evidence** (all with mock endpoints and fake tokens; Travis's login and real `auth.json` were never used):
- **Tests:** focused tests pass with and without developer-accounting on glitch, on the final merged heads of both PRs.
  - On macOS, `just test` passed 248 of 259. The 11 failures are not from these changes:
    - one stale TUI test that #585 has since fixed;
    - four that hit macOS's socket-path length limit in this checkout and pass on Linux;
    - six tmux onboarding tests that also fail on main.
- **Clippy:** Linux `-D warnings` clean both ways; macOS clean.
- **tmux, before and after** (disposable homes, `CORBANU_TEST_NO_NATIVE_KEYRING=1`): the acceptance's V1/V2/V3, cases 1/3/5/7, switching to the default account, resume/fork, and the #572 Bedrock path. Captures are in `qa/provider-auth/pf-84/s06/fix-570-573/` and `fix-572/`.
- **Opus 5.5 High review:** #604 needed 8 rounds, #610 needed 4; both ended at "approve". The findings were fixed or filed as issues.
- **CI:** one `pf_23_s01` security test failed on both PRs. It passes locally and on other PRs, and passed on a rerun.
- **Videos** (leak-scanned): [named ChatGPT account from Z.AI](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s06-pf84-named-chatgpt-from-zai-064dad64b873-2026-10-12.mp4) · [default login kept](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s06-pf84-named-session-keeps-default-login-064dad64b873-2026-10-12.mp4) · [Bedrock from Z.AI](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/pf-84-s06-pf84-bedrock-from-zai-ca46cf7ba80a-2026-10-12.mp4)

**Cleanup:**
- My glitch build folder ballooned to 211 GB and briefly filled glitch's disk; one build failed with "no space left". I deleted it, and glitch now has 461 GB free.
- Both worktrees, the build folders on the Mac and glitch, scratch homes, and my mock servers are removed.

**Still open:**
- `corbanu fork` of a named thread without `--account`, while a forced login rules out the default, still shows the default sign-in screen. Passing `--account <name>` works; this is documented.
- #609 collects small follow-ups:
  - how `/providers` labels and reports the session's named OpenAI account;
  - status with a remote app server;
  - the switch message after #585 when the default login is hidden.
- From the #572 review: opening `/providers` after switching to a Bedrock model can add a stray row. The cause already exists on main; I noted it in the PR rather than fixing it.

**Decision for Travis (#595):**
- **The gap:** a TUI that reuses a running local app-server daemon still logs out a ruled-out default login, because the daemon is a separate process.
- **Option 1 (recommended):** have the app server and daemon also leave the default login in place, and refuse to use it, whenever named accounts are on. This closes the gap everywhere. It also changes the standalone app server, which today never checks the default login and would use a ruled-out one.
- **Option 2:** keep the current documented limit.