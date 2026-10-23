Postmerge-ci on main is green again. PR #471 is merged (`b3763ed0b2`), and the postmerge run on that merge (38149138407) passed, though two tests are still intermittent and could turn it red again (below).

**What shipped (PR #471, test-only, no product changes):**
- **PF-30-S03 vault test (red since #453):** split into two tests.
  - **Moderate:** keeps every original assertion. The vault command after untrusted content asks the human, no "don't ask again" rule is offered, and with approvals off it is refused and never runs.
  - **Aggressive (new test):** with approvals on-request and with approvals off, nobody is asked. The output carries "Security level Aggressive: agents cannot use the Corbanu vault." A stand-in `corbanu` script, called by absolute path, never runs.
- **App-server test `safety_check_downgrade::model_verification_emits_typed_notification_and_warning_v2` (red since #394 on 2026-10-10):** the test sets `OPENAI_API_KEY`, so the turn now also gets #394's one-time "Using OPENAI_API_KEY from your environment" notice. The test ignores exactly that notice; any other warning still fails it.
- **Exec test `trace_log_credentials::http_turn_keeps_the_key_out_of_trace_logs`:** diagnostics only. Its panic now names the file that vanished.

**Gate evidence (glitch, Linux):**
- **Focused tests:** core `pf_30_s03|issue_443` 30/30; app-server `safety_check_downgrade` 5/5; exec `trace_log_credentials` 2/2, plus 40/40 in a stress run.
- **Full `just test -p codex-core`:** 4067/4069. The two failures happen only on glitch, and both also fail on `c7e6e07cbc`, before today's Aggressive merges:
  - `pf_30_s01_source_envelopes…` is a real product bug, filed as **#473**: bubblewrap fails when a protected home path is missing under a read-only parent. In practice, `~/.claude` without `.credentials.json` makes every sandboxed command fail at Moderate and Aggressive. It also hit postmerge once, in run 38144209457.
  - `network_approval::ambiguous_unattributed…` times out on glitch waiting for the turn to abort after an interrupt. It isn't new today, and it isn't among the postmerge failures I saw, so I left it alone.
- **Linux clippy `-D warnings`** (core, app-server, exec): clean.
- **PR checks:** all 27 passed, none pending. Merged with `--match-head-commit c14733de23`.

**Other postmerge findings:** the inspection test was already fixed by #470. Every other failure in today's runs was one of the above.

**Open:**
- **#473 (product bug):** can also fail postmerge intermittently when another test creates `~/.claude` on the runner.
- **#477 (intermittent exec test, failing since #469):** the new message named the file: `$CODEX_HOME/.tmp/plugins-clone-*/.git/shallow.lock`. A background plugin-sync `git clone` keeps running after `codex-exec` exits. My recommendation, recorded on the issue, is to wait for or kill that clone when exec shuts down.

**Cleanup:**
- On glitch I removed my target dir, clone, scratch homes and the cargo-nextest binary I downloaded; small logs remain under `~/corbanu-rtx/cifix30-*`.
- I removed the Mac worktree (no Mac target dir was created).
- The local branch `ci/pf30-s03-aggressive-vault` is still there: `git branch -d` refused because local main hasn't fetched the merge, and I didn't force it.

No review, tmux run or video for this one, since it only changes tests.