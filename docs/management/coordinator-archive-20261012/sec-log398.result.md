I fixed #398 and merged PR #410 (merge commit `85e9c43ac4`) once all 35 checks had passed, with 9 skipped and none pending. The issue is closed with an evidence comment. One gap: the RTX box went offline partway through, so the final Linux clippy came from CI rather than RTX, and my RTX files are still there.

**What changed**
- **Windows sandbox log:** everything written to the sandbox log now goes through the codex-log-guard redaction from #387. That covers the START/SUCCESS/FAILURE lines, notes, the setup helper's lines and `SBX_DEBUG` lines. The command is redacted one argument at a time before it's cut to 200 bytes, so a value with spaces in it is hidden whole.
- **Logs written before this fix:** they get masked once when the TUI or app-server starts, and `/feedback` redacts the sandbox log it attaches.
- **macOS and Linux:** their sandboxes write no command log of their own. Sandboxed commands only reach the general logs, which already redact, and the redaction now also catches:
  - password and token options, including in JSON argument lists
  - `curl -u`, `mysql -p`, `sshpass -p` and `docker login -p`
  - environment assignments such as `OPENAI_API_KEY=` and `PGPASSWORD=`
  - passwords in URLs, and more key formats (GitHub, Slack, Google, npm, Stripe, AWS)
- **Approvals:** a command carrying a credential is never saved to `rules/default.rules`. Approving it with "don't ask again" applies for the session only, and the user is told. Claude pane audit previews and Telegram approval messages are redacted too.
- **Left unredacted on purpose, as the user's own transcript:** rollouts, prompt history, rollout traces, Claude pane transcripts and the opt-in TUI session recorder. This is written up in `qa/security-levels/pf24-followups/issue-398/README.md`.
- **Follow-up from #387:** `clippy.toml` now forbids installing a tracing subscriber or `log` logger except through `codex_log_guard::guard(..).{init,try_init}()`. Bazel's clippy job uses the same file, so there's no source scan. The only exceptions are tracing-test's test modules and the `execpolicy-legacy` dev tool, each with a stated reason.

**Gate evidence**
- **Windows 11, real sandbox:** all 166 windows-sandbox tests pass. The new ones run commands containing fake credentials through the legacy, elevated and capture sandbox paths. With the redaction removed, the new tests fail. The real-sandbox test also caught a bug in my own fix: a bearer value swallowed the next `X-Api-Key` header. That's fixed.
- **Live turns on Windows:** I ran one GLM 5.3 Flash turn under each sandbox mode (unelevated and elevated). Each ran a command carrying four fake credentials. The sandbox logs show `REDACTED`. Scanning each throwaway home (about 5,500 files) found the fake credentials only in the rollout, and the Z.AI key in no file. The homes and key file are deleted.
- **Other tests:**
  - log-guard, state, feedback and telegram suites
  - exec-policy tests, including the new ones for credential commands
  - the core log-redaction tests, including a new sandboxed-command test (6/6 on macOS and on RTX Linux; it needs `RUST_MIN_STACK=16777216`)
  - TUI claude-pane tests
- **Clippy `-D warnings`:** clean on the whole macOS workspace and on every touched crate on Windows. On Linux, the whole workspace was clean on RTX at an earlier commit, and CI's Linux `cargo clippy` passed on the final head.
- **Review:** Opus 5.5 High, read-only, approved with fixes and found nothing blocking. Every finding is fixed or recorded in `review/disposition.md`.
- **Not done:** no tmux TUI run or demo video; this assignment didn't ask for them.

**Still open**
- **RTX cleanup:** I couldn't remove my files there because the box is unreachable:
  - `~/corbanu-rtx/l398`, a worktree of `~/corbanu-rtx/repo`
  - `~/corbanu-rtx/l398-target`
  - `~/corbanu-rtx/l398.sh`, `l398b.sh` and their `.log` files
  - tmux sessions `l398` and `l398b`
  
  When it's back, run `git -C ~/corbanu-rtx/repo worktree remove ~/corbanu-rtx/l398` and `rm -r ~/corbanu-rtx/l398-target`.
- **Branch:** the local branch `fix/398-sandbox-cmdlog-redact` remains because `git branch -d` refused (local main is behind); it is merged on origin.
- **Cleanup done:** Windows `C:\CorbanuQA\l398` is fully removed, and `C:\CorbanuQA\target` and the other worker's directories were left alone. The local worktree and build target are removed.
- **Known gaps:**
  - Env names ending in a plain `_KEY` (like `OPENAI_KEY`) aren't matched by name, because that would also hide cache keys and paths. Their values are still caught when they're in a known key format.
  - That every log writer also redacts is a review convention; the lint only enforces the guard.