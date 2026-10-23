You are a security bug-fix worker. Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first
(working rules and per-fix gate apply; this is a bug fix, not a sprint, so no sprint file is needed).

1. Fix GitHub issue #179 (CorbanuCore/CorbanuTerminal): a `!` shell command logs its full environment at trace
   level, including API keys, into codex-tui.log and the logs SQLite DB. Find every place that logs an exec
   environment or env map (user shell, unified exec, tool exec, sandbox spawn, MCP stdio launch, etc.) and stop
   logging values; names only, at most. Add a regression test with a sentinel secret in the env at trace level
   asserting it never reaches the log file or log DB. Check other trace/debug logging of request headers or
   config that could carry credentials while you're there, and fix what you find in the same PR if small.
2. Separately (second small PR): user-facing strings that still say "codex"/"Codex" in the approval history line
   ("You approved codex to run...") and the Full Access confirmation ("When Codex runs with full access...") should
   say "Corbanu". Update snapshots. Don't touch files under codex-rs/tui/src/bottom_pane/approval_overlay* or
   permission popups beyond the string change if another worker (sec-tui, issues #174-#176) has open edits there;
   merge origin/main first.
3. Gate per fix: focused `just test`, a short demo video via the SOP for #179 showing a `!env`-style command with a
   fake sentinel key and the log afterwards free of it, one independent Opus 5.5 High review. PR to main; merge when
   green; close #179 with the PR link.
Report as in sec-common.md.
