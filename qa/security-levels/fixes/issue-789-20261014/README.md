# #789: no Grant once on a child agent's prompt where it can't apply (AGG-R14)

Replay of the record's steps (AGG-R14, Q11/Q11b) in tmux with a disposable home and GLM 5.3 Flash on
zai (`qa/demos/specs/issue789-child-no-useless-grant.toml`): `/security` → Aggressive, restart, then
"Spawn a child agent and have it run exactly this command: sleep 60; echo finished > ks-child.txt".

- `before-child-prompt-without-789.txt`: a build without this change (PR #797's branch, from
  `origin/main` `aecd481368`). The child's prompt offers "4. Grant once: also let it write hooks and
  start-up files", as in the record (`milestones/aggressive-20261014-targeted/runs/Q11b/`).
- `after-child-prompt.txt`: PR #798's head `576ea39ea9`. No Grant once; the prompt's reason says
  "Grant once is not offered for this command: …".

Tests (fail before, pass after; macOS Seatbelt and Linux bubblewrap): `issue_789_no_grant_for_waits`
and `issue_789_child_agent_prompt_offers_no_useless_grant` (a real spawned child agent) in
`codex-rs/core/tests/suite/issue_736_740.rs`, and the unit cases in
`protected_reads_writes_tests.rs`.

Video: [mp4](https://github.com/CorbanuCore/CorbanuTerminal/releases/download/demos/issue-789-issue789-child-no-useless-grant-576ea39ea970-2026-10-14.mp4).
