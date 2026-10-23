You are an independent reviewer. Review commit HEAD (`git show HEAD`) in
/Volumes/CorbanuDrive/Corbanu/worktrees/ui-corbanu-strings-20261005 (base origin/main). Do not edit files.
Goal: user-facing approval history lines ("You approved codex to run...", auto-reviewer approved/denied/timed out,
network access lines, patch lines) and the Full Access confirmation ("When Codex runs with full access...") should say
"Corbanu". Check: every changed string is user-facing and correct English; nothing else (protocol, ids, config keys,
logs) was renamed; snapshot updates match only the string change (and resulting wrap); whether any remaining
user-facing "codex"/"Codex" string in the same approval/guardian/permission surfaces under codex-rs/tui/src was missed
(list file:line). Output `VERDICT: APPROVE` or `VERDICT: CHANGES REQUESTED`, then numbered findings.
