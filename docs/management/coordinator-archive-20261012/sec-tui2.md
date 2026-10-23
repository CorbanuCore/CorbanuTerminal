You are the TUI lane worker (round 2). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first
(binding), then /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-picker.result.md (PF-24-S03 merged as #186,
archived #195, flag `security_levels`). The next TUI-lane sprint (PF-24-S02) is blocked on PF-23-S03 and PF-29-S02, so
this round does bounded follow-ups, each as its own small PR under the per-fix gate.

1. Provider key in the trace log. The picker's demo tool still found the provider API key in the private TUI log
   and log DB with trace logging on. #183 (issue #179) fixed the exec-env path. Reproduce on current main with a fake
   sentinel key; if it still leaks, find the path (request headers, provider config dump, client builder debug, etc.)
   and fix it with a regression test, plus a demo video showing the log is clean. File or reopen an issue with the
   security label.
2. Picker follow-ups that need no product decision:
   a. `/security` command description says "read only" when the flag is on; make it accurate.
   b. The Aggressive review screen should show the current value next to the Aggressive value for each row
      (code-blind case B-06).
   c. The launch check should load the full exec policy (not just check the vault rule file exists), so a broken
      `.rules` file can't silently drop the `corbanu vault` rule. If that needs `codex-execpolicy` as a TUI dependency,
      make the lockfile/Bazel edge changes together and say so in the PR.
3. Do NOT change the Aggressive sandbox design (`corbanu-aggressive` permission profile); that's waiting on Travis.
Gate per fix: focused `just test`, GLM 5.2 tmux run, one independent Opus 5.5 High review, short demo video. PR to
main; merge when green (stale CodeQL checks on PRs can be ignored). Report as in sec-common.md.
