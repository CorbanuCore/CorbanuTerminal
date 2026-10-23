PR #232 (#218 part 2) is merged as `d0f554aeee` (head `568c001d04`). All CI checks passed, and the fourth review round approved it with nothing blocking.

**Round-1 findings, all fixed**
1. **Enter no longer allows by default.**
   - Requests queue in one popup that starts on Deny. Allow needs ← or →, a pause, then Enter.
   - All keys except Esc and Ctrl-C (both deny) are ignored for 600 ms. The timer starts when a request first shows, after any ignored key, after the last composer keystroke, and after choosing Allow. Held keys never confirm, and Tab, digits and paste never choose.
   - The popup goes back to Deny whenever it comes out from under another view.
2. **The popup shows everything.**
   - Every input field appears in full, wrapped and scrollable: Write content, Edit diffs, and hidden Bash fields such as `dangerouslyDisableSandbox`.
   - Line breaks show as `⏎`, and control, bidi, zero-width and private-use characters are escaped. Long runs of spaces are counted.
   - Allow only works once the last line has been on screen. Past 60,000 characters the popup only offers Deny.
3. **Things a pane or repo could plant are blocked**, through `--safe-mode`, an explicit `--tools` allowlist, deny rules for subagent/skill/slash-command tools, and `disableAllHooks`.
   - A real-binary regression (`claude_code_regression.py`, Claude Code 2.1.292 on Linux) seeds hooks, agents, skills, commands, `.claude.json` and `.mcp.json`. None of them load, and `/`, `!` and `#` prompts don't get around the prompts.
   - Contained launches refuse Claude Code older than 2.1.292. `claude` is found only in absolute PATH folders and is refused inside folders the pane can write. `--version` runs inside the sandbox.
4. **Wording is now accurate.** `ask` rules make every command, file edit and web request prompt, including `ls`, which ran without asking before. The `/security` row names `secretless_agent_launch`.
5. **Stale popups and hung turns are handled.** Popups close when the turn ends, is interrupted, or gets `control_cancel_request`. An unanswered request is denied after 15 minutes. A control request without an id, or a second result, ends the turn.
6. **Request ids are tracked.** A repeated id gets no popup and no answer, and `tool_use_id` is shown.
7. **Pane identity:** the popup shows the pane's short id and folder.
8. **README fixed:** the smoke and workflow commands are never contained.
9. **Redaction comment added.**

All the missing tests from the review were added.

**Gate evidence**
- **Tests:** `just test -p codex-tui` on `claude_panes`, `bottom_pane`, `security`, `app::` and `chatwidget` passed 2645 of 2648. The three failures (counting retries) are in two tests that don't touch this change: a slash-command list snapshot that has drifted, and a wallet test whose Unix socket path is too long in this checkout. I didn't run them on main to compare. Linux clippy (`-D warnings`) was clean on the RTX box.
- **tmux runs:** Z.AI had balance at run time, so the main run is real GLM 5.2. Under Aggressive, Right, pause, Enter created `approved.txt` and Esc denied `denied.txt`. A second run uses a stand-in for Claude Code, a shell script that sends two fixed Bash requests with no model. It shows an Enter pressed as the popup appears being ignored, and the escaped two-line command. Both runs passed again after each round of fixes. Notes are in `tmux-run/7-macos-review-fixes.txt`.
- **Videos:**
  - [approvals under Aggressive](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/claude-contained-claude-contained-approvals-e562055c0b97-2026-10-06.mp4)
  - [popup guard and escaping](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/claude-contained-claude-contained-approval-guard-e562055c0b97-2026-10-06.mp4)
- **Review:** I ran it with the installed `corbanu exec` (Opus 5.5 High, claude-plan, read-only), giving it the diff each round. Rounds 2 and 3 said "approve with fixes"; I fixed all of those findings, and round 4 approved. All rounds are saved in `qa/.../claude-contained/review/part2-review-1..4.md`.

**Cleanup:** I removed my worktree and target folder, the old review worktree, the demo run folders (including their disposable vault stores), and everything `sec-tui5*` on the RTX box. No test or demo run touched the real login keychain.

**Still open**
- Two limits are recorded in the README:
  - A `claude` in an absolute PATH folder that another contained pane can write isn't detected. It would run sandboxed but without asking.
  - Up to 7 spaces where a wrapped row breaks aren't shown; longer runs show as a count.
- The known bug where a pane's final answer appears twice, once while streaming and again with the result, is still there.
- The real-binary regression ran on Linux only, because on macOS a real `claude` would read the login keychain.