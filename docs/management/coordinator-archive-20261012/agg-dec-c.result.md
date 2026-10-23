I've done all five decisions. Both PRs are merged with every check green: #521 for the home listing and #522 for the kill switch. Item 1 is a new issue with no code. The Anthropic content filter did not stop me.

**Decisions recorded:** each one is a comment on the issue I opened or touched: #505, #506, #507, #443 and #445.

**What shipped**
- **Item 1 → #505:** I opened "Aggressive: apply restart-only protections at confirmation". It covers what applying each one immediately would take: sandbox, approvals, network and web search, environment stripping, the model-key broker and the child-agent rules.
- **Item 2 → #521, fixes #506, merged as `695850a6d8`:** agent commands and file tools can no longer list the Corbanu home under Aggressive. This holds from the moment of confirmation and after a restart.
  - **macOS:** listing the folder is denied, but the folder itself and everything below it still work.
  - **Linux:** the folder is rebuilt as an empty, read-only, non-listable folder with its entries put back.
  - **What still works:** the home's runtime entries (helper launchers, shell snapshots, skills, plugins). Protected entries stay denied, and Corbanu itself is unaffected.
  - **Explanation:** the #464-style message ("Security level Aggressive blocked listing `$CORBANU_HOME`…") shows where it fails. It also covers `$CODEX_HOME`, `~`, `dir/..` and an `ls` run inside the home.
- **Items 3–5 → #522, fixes #507, merged as `eb92efdf82`:**
  - **Interrupt (4):** turning the switch on stops the turn in progress in every pane of this Corbanu Terminal, child agents included. A streaming reply is cancelled, a running command is killed, a command waiting for approval never runs, and background terminals end. The pane says "Stopped: the security kill switch was turned on…".
  - **Claude panes (3):** new Claude turns are refused with the same "Not sent: …" message, and running ones are interrupted. Tasks other agents send to Claude panes wait and are sent when the switch is turned off.
  - **Waiting messages (5):** unchanged for Codex panes. The kill-switch texts now say that messages from other agents wait.

**Checks**
- **Regression tests** fail before the change and pass after, on both PRs. That includes real Seatbelt and real bubblewrap tests, and core and TUI tests for the kill switch.
- **`just test` on the Mac:**
  - #521: 771 of 772 pass; 634 of 634 with developer-accounting.
  - #522: 687 of 687 pass, with and without developer-accounting.
  - The one #521 failure (`shell_command_snapshot_still_intercepts_apply_patch`) fails the same way on `origin/main` on this Mac.
- **Clippy `-D warnings`:** clean on the Mac and on glitch (`-j10`), plain and developer-accounting. All Linux sandbox tests pass on glitch.
- **Reviews (Opus 5.5 High):** every finding was fixed before merge.
  - #521: changes requested twice, then approved. The main fix: on Linux, a read-only mount could put the real home back over the hidden one, so it could be listed again.
  - #522: changes requested three times, then approved. The main fixes: Claude-pane tasks were dropped although the text promised they wait, and repeated timer fires could pile up tasks while the switch was on.
- **tmux before/after:** GLM 5.3 Flash in a disposable home. The Claude pane ran on Opus 5.5 through `claude-plan-test-token` with a disposable Claude config. Results are in `qa/security-levels/fixes/issue-506-20261011/` and `issue-507-20261011/`.
  - **Before:** the home is listed, the 45-second command finishes after the switch, and Claude keeps writing.
  - **After:** the listing is denied (also after a restart), and both stop with the message.
- **Videos:**
  - [home listing denied](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-506-issue506-home-listing-denied-143cabebfb01-2026-10-11.mp4)
  - [kill switch stops a running command](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-507-issue507-kill-switch-stops-running-work-bd25c93499a9-2026-10-11.mp4)
  - [kill switch stops a Claude pane](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-507-issue507-kill-switch-stops-claude-pane-bd25c93499a9-2026-10-11.mp4)
- **Leaks:** the demo tool's scan passed, and the evidence and run folders contain neither the Z.AI key nor the Claude token.

**Decisions for Travis**
1. **Claude panes when the switch is turned off:** each Claude pane keeps only the newest task sent while the switch was on. The older one is reported to its sender as superseded. That task starts a Claude turn as soon as the switch goes off. Codex panes start nothing and deliver waiting messages with their next turn. The alternative is to hold Claude tasks until that pane's next turn. My recommendation is to keep it as built, but it is your call because it differs from the Codex behaviour.
2. **Other Corbanu Terminals on the same home:** they only take the switch when they next start, and their running work is not stopped. The kill-switch text says so. Should a running process pick it up immediately?

**Limits and notes**
- **Windows:** the listing denial isn't enforced there; protected entries stay denied.
- **Listing from a parent folder:** `find ~` or `ls -R ~` is denied for the home but not explained.
- **Interrupted Claude turns:** they still say "interrupted by user" under the stop message.
- **Machine setup:** contained Claude panes need Claude Code 2.1.292 or later, and this Mac has 2.1.37 and 2.1.92. I installed 2.1.296 in a scratch folder for the runs and removed it afterwards.
- **Pre-existing CI flake:** `security_revocation_one_grant_ends_only_that_grant` failed twice on CI for #522, though it is not in this PR's code. Under `cargo test` threads, another test in the same file ended its grants. I made that file's tests run one at a time.
- **Overlaps:** agg-dec-a and #499, #503 and #508 changed `bwrap.rs`, `seatbelt.rs` and `protected_reads.rs`. I resolved the conflicts when merging `origin/main`.
- **`rm -f` slip:** I used it once on glitch, on a log marker file in my own folder that didn't exist, so nothing was removed. Everything else used `rm -r … < /dev/null`.

**Cleanup:** the worktrees, build targets, scratch binaries and run folders are removed on the Mac. The clone, target, logs and scripts are removed on glitch.