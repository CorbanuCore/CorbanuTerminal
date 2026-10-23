Both issues are fixed and merged in PR #381 (merge commit `a41ad381d6`), after every check passed on the PR head, with none pending. #365 and #366 are closed with evidence comments.

**What changed**

**#365 (OpenAI key):**
- The built-in `openai` provider now uses `OPENAI_API_KEY` in `corbanu exec`, the TUI and the app server.
- It uses the first credential it finds, in this order:
  1. `CODEX_API_KEY` (exec only, as before);
  2. a sign-in supplied for the session (external ChatGPT tokens or `CODEX_ACCESS_TOKEN`);
  3. a saved sign-in or key;
  4. `OPENAI_API_KEY`.
- A key exported in the shell never replaces a saved sign-in, so it can't change how a ChatGPT sign-in is billed.
- The key is only sent to OpenAI itself, never to local or custom endpoints. It is ignored when the config requires ChatGPT login (`forced_login_method = "chatgpt"`).
- A request with no credential now fails with a message that names `OPENAI_API_KEY`, `CODEX_API_KEY` and `corbanu login`.
- `docs/authentication.md` documents the order and the logout behaviour.

**#366 (Claude token):** I read the specs first. No security rule forbids this: the documented behaviour is that with no saved method, `CLAUDE_CODE_OAUTH_TOKEN` is used first, and exec and panes already did that. Only the TUI's status check disagreed.
- `/providers` now shows "Ready · CLAUDE_CODE_OAUTH_TOKEN from the environment", and Claude Plan models and panes run without saving the token.
- An explicitly chosen method still binds that exact token and fails closed if it changes. A lost selection still requires a choice.
- The Claude Plan pane now asks Claude Code for `claude-opus-5-5` by exact ID instead of the moving `opus` alias, and is labelled "Opus 5.5 Claude Plan". Saved panes with the old label reopen with the new one.

**Gate evidence**
- **Tests on macOS, with and without `developer-accounting`:**
  - codex-login 197/197, codex-model-provider 84/84, codex-core subset 809/809.
  - codex-tui lib: 4339 passed, 12 failed. The 12 failures come from this Mac (socket paths too long under `/Volumes`, a stale snapshot, one security-picker test), and all 12 pass on Linux.
- **Linux on the RTX box:** clippy `-D warnings` is clean for login, model-provider, core and tui, with and without `developer-accounting`. Tests passed: core subset 805/805, tui lib 4348/4348, login 196/196, model-provider 84/84, and the full tmux suite 71/71.
- **OpenAI functional check (Mac, fresh debug binary, disposable home):** `corbanu exec` on gpt-5.4 with only `OPENAI_API_KEY` set replied OK. The binary built before the fix reproduced the 401, and with no key the new error appears.
- **TUI checks (RTX box, tmux):**
  - With only `OPENAI_API_KEY`, a gpt-5.4 turn worked and `/providers` showed the key as coming from the environment.
  - With only `CLAUDE_CODE_OAUTH_TOKEN`, I opened an "Opus 5.5 Claude Plan" pane and one turn succeeded, served `claude-opus-5-5`.
  - I ran this on Linux so Claude Code, started by the debug binary, couldn't reach the Mac's login keychain.
- **Secret scans:** neither key was found in the evidence files, the exec homes or the Claude TUI home.
- **Review:** the installed `corbanu exec` (Opus 5.5 High, read-only) asked for changes twice, and approved on the third round.
  - **Round 1:** the env key could reach non-OpenAI providers, and the Claude doc described a step users can't reach. Both fixed.
  - **Round 2:** realtime conversations bypassed the filter. Fixed.

**Fixes made after CI failures**
- A PF-58 tmux test expects `/providers` to count `CODEX_API_KEY` as an environment credential. I had changed that, so I restored it and added a matching test for `OPENAI_API_KEY`.
- A test constant was Unix-only, which broke the Windows build. Fixed.
- The last two commits (a revert to existing behaviour plus test fixes) went in without another review round.

**Still open**
- **Product decision for Travis:** in a fresh home with `OPENAI_API_KEY` exported, the TUI now skips provider onboarding, as it already does for other providers' environment keys. After logout, requests quietly fall back to that key and are billed per use; this is documented. Options:
  - accept it as is (my recommendation);
  - add a visible notice;
  - make the variable opt-in.
- **#380 (filed, security):** with `RUST_LOG=trace`, the websocket library writes the `authorization: Bearer` header to `codex-tui.log`. This existed before and affects any OpenAI key. I deleted the RTX test home that contained it.
- **Not addressed:**
  - With only a Claude Code login and no saved method, the TUI still asks for a choice while exec just uses the login.
  - On local and custom providers, the cost-accounting metadata still sees an unfiltered OpenAI key (labels only; the key is never sent).
- **Process:** the reviewer argued that a new credential source could count as a product initiative rather than a bounded fix. I treated your assignment as the product decision and cited the spec heading in the PR.

**Cleanup:** the Mac build directory, the worktree, the RTX worktree, its build directory, the temporary Claude Code install and all disposable homes are removed. Small evidence captures (scanned clean) remain in `/tmp/fix365-evidence`. The local branch `fix/365-366-env-credentials` wasn't deleted because the local main isn't updated; it's harmless.