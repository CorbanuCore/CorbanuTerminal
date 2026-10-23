You are the TUI lane worker (round 6). Read /Volumes/CorbanuDrive/Corbanu/.codex-work/workers-20261002/sec-common.md first
(binding; note the keychain rule and its clarification), then sec-tui5b.result.md and the independent review
rev-232.result.md in the same folder (verdict: request changes on PR #232, #218 part 2).

Address every finding in rev-232.result.md on PR #232's branch (worktree worktrees/sec-tui5-218-20261006 or a fresh
one off the #232 head): Enter must not allow by default (start on Deny, input guard after popup opens, test); show
exactly what is being approved (wrapping, escaped control/bidi chars, full command or explicit truncation that
defaults to deny, file edit content/diff, hidden Bash fields); cover subagents/skills/commands/.claude.json routes
with a real-binary regression or block them (disableAllHooks, disallow Task, version floor); fix the "asking before
each tool" wording or force prompts with permissions.ask; cancel/close stale popups, handle control_cancel_request,
turn timeout; track pending request IDs; pane IDs in the title; fix the README/smoke containment claim; add the
missing tests listed. Then run a fresh independent review with the INSTALLED `corbanu exec` (claude-plan, read-only,
giving it the diff), and merge #232 when CI is green and the review approves.
Note: the Z.AI (GLM 5.2) account is out of balance right now; if a GLM tmux run fails with 429 "Insufficient balance",
record that and use a fake/mock model for the TUI flow instead of blocking. Report as in sec-common.md.
